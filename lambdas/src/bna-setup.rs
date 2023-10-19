use aws_sdk_ecs::types::{
    AssignPublicIp, AwsVpcConfiguration, ContainerOverride, KeyValuePair, NetworkConfiguration,
    TaskOverride,
};
use aws_sdk_sqs::{self, types::Message};
use bnacore::neon::{
    model::{
        NeonBranch, NeonCreateBranchRequest, NeonCreateBranchResponse, NeonEndpoint,
        NeonEndpointType, NeonListBranchResponses,
    },
    NEON_PROJECTS_URL,
};
use bnalambdas::{get_aws_parameter, get_aws_secrets, AnalysisParameters};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::header::{self, HeaderValue};
use serde_json::Value;
use simple_error::SimpleError;
use tracing::info;
use url::Url;

const FARGATE_MAX_TASK: i32 = 1;
const NEON_MAX_BRANCHES: usize = 20;
const SQS_MAX_BATCH_SIZE: i32 = 1;

async fn function_handler(_event: LambdaEvent<Value>) -> Result<(), Error> {
    // Define variable bindings for the analysis.
    let analysis_parameters: AnalysisParameters;
    let message: Message;

    // Poll one analysis to be processed.
    let bna_sqs_queue = get_aws_parameter("BNA_SQS_QUEUE_URL").await?;
    let aws_config = aws_config::load_from_env().await;
    let sqs_client = aws_sdk_sqs::Client::new(&aws_config);
    let message_batch = sqs_client
        .receive_message()
        .queue_url(&bna_sqs_queue)
        .max_number_of_messages(SQS_MAX_BATCH_SIZE)
        .send()
        .await?;
    match message_batch.messages {
        Some(messages) => {
            // Ensure there is only one message.
            if messages.len() != 1 {
                return Err(Box::new(SimpleError::new(format!(
                    "{} messages were received instead of 1.",
                    messages.len()
                ))));
            }

            // Deserialize it.
            message = messages.first().unwrap().clone();
            match &message.body {
                Some(m) => {
                    analysis_parameters = serde_json::from_str(m.as_str()).unwrap();
                    info!("Message received: {:#?}", analysis_parameters);
                }
                None => {
                    // No message body.
                    return Err(Box::new(SimpleError::new(
                        "There is no message body to read from.",
                    )));
                }
            }
        }
        _ => {
            // Nothing to process
            info!("The queue is empty.");
            return Ok(());
        }
    }

    // Create the Neon HTTP client.
    let neon_api_key = get_aws_secrets("NEON_API_KEY").await?;
    let mut headers = header::HeaderMap::new();
    let mut auth_value = HeaderValue::from_str(format!("Bearer {}", neon_api_key).as_ref())?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);
    let neon_rest_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let neon_project_id = get_aws_parameter("NEON_BROKENSPOKE_ANALYZER_PROJECT").await?;
    let neon_branches_url = format!("{}/{}/branches", NEON_PROJECTS_URL, neon_project_id);

    // Query neon API and check whether we can create a branch or not.
    info!("Checking database branch capacity...");
    let branches = neon_rest_client
        .get(&neon_branches_url)
        .send()
        .await?
        .error_for_status()?
        .json::<NeonListBranchResponses>()
        .await?;

    // Not enough capacity to proceed. Back into the queue.
    if branches.branches.len() >= (NEON_MAX_BRANCHES + 1) {
        return Err(Box::new(SimpleError::new(format!(
            "Not enough capacity to proceed ({} branches). Back into the queue",
            branches.branches.len()
        ))));
    }
    info!(
        "There is(are) {} active branch(es).",
        branches.branches.len()
    );

    // Prepare new branch.
    let mut branch_name = format!(
        "{}-{}-",
        analysis_parameters.country, analysis_parameters.city,
    );
    match &analysis_parameters.region {
        Some(region) => branch_name.push_str(region),
        None => branch_name.push_str(&analysis_parameters.country),
    };
    branch_name = branch_name.replace(' ', "-");
    if branches
        .branches
        .into_iter()
        .filter_map(|b| b.branch.name)
        .any(|b| branch_name == b)
    {
        return Err(Box::new(SimpleError::new(
            "a branch with the same name already exists",
        )));
    }

    // Create the neon branch.
    info!("Creating branch {}...", branch_name);
    let create_branch_request = NeonCreateBranchRequest {
        endpoints: vec![NeonEndpoint {
            r#type: NeonEndpointType::ReadWrite,
            ..Default::default()
        }],
        branch: NeonBranch {
            name: Some(branch_name),
            ..Default::default()
        },
    };
    let create_branch_response = neon_rest_client
        .post(&neon_branches_url)
        .json(&create_branch_request)
        .send()
        .await?
        .error_for_status()?
        .json::<NeonCreateBranchResponse>()
        .await?;
    info!("{:#?}", create_branch_response);

    // Retrieve the secrets and parameters.
    info!("Retrieving secrets and parameters...");
    let main_db_url = get_aws_secrets("DATABASE_URL").await?;
    let ecs_cluster_arn = get_aws_parameter("BNA_CLUSTER_ARN").await?;
    let vpc_subnets = get_aws_parameter("PRIVATE_SUBNETS").await?;
    let vpc_security_groups = get_aws_parameter("BNA_TASK_SECURITY_GROUP").await?;
    let task_definition = get_aws_parameter("BNA_TASK_DEFINITION").await?;

    // Start the Fargate task.
    info!("Starting the Fargate task...");
    // Replace the main database host with the compute endpoint.
    let mut database_url = Url::parse(&main_db_url)?;
    database_url.set_host(
        create_branch_response
            .endpoints
            .first()
            .unwrap()
            .host
            .as_deref(),
    )?;

    let container_name = "brokenspoke-analyzer".to_string();
    let ecs_client = aws_sdk_ecs::Client::new(&aws_config);
    let mut container_command: Vec<String> = vec![
        "-vv".to_string(),
        "run".to_string(),
        "--with-export".to_string(),
        "s3".to_string(),
        analysis_parameters.country.clone(),
        analysis_parameters.city.clone(),
    ];
    if analysis_parameters.region.is_some() {
        container_command.push(analysis_parameters.region.clone().unwrap());
        container_command.push(analysis_parameters.fips_code.clone().unwrap());
    };
    let container_overrides = ContainerOverride::builder()
        .name(container_name)
        .set_command(Some(container_command))
        .environment(
            KeyValuePair::builder()
                .name("DATABASE_URL".to_string())
                .value(database_url)
                .build(),
        )
        .build();
    let task_overrides = TaskOverride::builder()
        .container_overrides(container_overrides)
        .build();
    let aws_vpc_configuration = AwsVpcConfiguration::builder()
        .subnets(vpc_subnets)
        .security_groups(vpc_security_groups)
        .assign_public_ip(AssignPublicIp::Enabled)
        .build();
    let network_configuration = NetworkConfiguration::builder()
        .awsvpc_configuration(aws_vpc_configuration)
        .build();
    let run_task_output = ecs_client
        .run_task()
        .cluster(ecs_cluster_arn)
        .count(FARGATE_MAX_TASK)
        .launch_type(aws_sdk_ecs::types::LaunchType::Fargate)
        .network_configuration(network_configuration)
        .overrides(task_overrides)
        .task_definition(task_definition)
        .send()
        .await?;
    info!("{:#?}", run_task_output);

    // Delete the message from the queue.
    let delete_output = sqs_client
        .delete_message()
        .queue_url(&bna_sqs_queue)
        .receipt_handle(message.receipt_handle.unwrap())
        .send()
        .await?;
    info!("{:#?}", delete_output);
    info!("The city is being processed: {:#?}", analysis_parameters);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
