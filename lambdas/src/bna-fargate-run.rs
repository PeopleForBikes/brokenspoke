use aws_config::BehaviorVersion;
use aws_sdk_ecs::types::{
    AssignPublicIp, AwsVpcConfiguration, ContainerOverride, KeyValuePair, NetworkConfiguration,
    TaskOverride,
};
use bnacore::aws::get_aws_parameter_value;
use bnalambdas::{
    authenticate_service_account, update_pipeline, AnalysisParameters, BrokenspokePipeline,
    BrokenspokeState, Context,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
struct TaskInput {
    analysis_parameters: AnalysisParameters,
    context: Context,
}

#[derive(Serialize)]
struct TaskOutput {
    ecs_cluster_arn: String,
    task_arn: String,
    last_status: String,
    command: Vec<String>,
}

const FARGATE_MAX_TASK: i32 = 1;

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Retrieve API hostname.
    let api_hostname = get_aws_parameter_value("BNA_API_HOSTNAME").await?;

    // Prepare the API URL.
    let url = format!("{api_hostname}/bnas/analysis");

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Read the task inputs.
    let analysis_parameters = &event.payload.analysis_parameters;
    let state_machine_context = &event.payload.context;
    let (state_machine_id, _) = state_machine_context.execution.ids()?;

    // Update the pipeline status.
    let patch_url = format!("{url}/{state_machine_id}");
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        state: Some(BrokenspokeState::Analysis),
        ..Default::default()
    };
    update_pipeline(&patch_url, &auth, &pipeline)?;

    // Prepare the AWS client.
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let ecs_client = aws_sdk_ecs::Client::new(&aws_config);

    // Retrieve secrets and parameters.
    let ecs_cluster_arn = get_aws_parameter_value("BNA_CLUSTER_ARN").await?;
    let vpc_subnets = get_aws_parameter_value("PUBLIC_SUBNETS").await?;
    let vpc_security_groups = get_aws_parameter_value("BNA_TASK_SECURITY_GROUP").await?;
    let task_definition = get_aws_parameter_value("BNA_TASK_DEFINITION").await?;
    let s3_bucket = get_aws_parameter_value("BNA_BUCKET").await?;

    // Replace the main database host with the compute endpoint.
    let database_url = "postgresql://postgres:postgres@localhost:5432/postgres";

    // Prepare the command.
    let mut container_command: Vec<String> = vec![
        "-vv".to_string(),
        "run".to_string(),
        "--with-export".to_string(),
        "s3".to_string(),
        "--s3-bucket".to_string(),
        s3_bucket,
        analysis_parameters.country.clone(),
        analysis_parameters.city.clone(),
    ];
    if analysis_parameters.region.is_some() {
        container_command.push(analysis_parameters.region.clone().unwrap());
        container_command.push(analysis_parameters.fips_code.clone().unwrap());
    };

    // Prepare and run the task.
    let container_name = "brokenspoke-analyzer".to_string();
    let container_overrides = ContainerOverride::builder()
        .name(container_name)
        .set_command(Some(container_command.clone()))
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
        .build()?;
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

    // Prepare the output.
    let task = run_task_output.tasks().first().unwrap();
    let output = TaskOutput {
        ecs_cluster_arn: task.cluster_arn().unwrap().into(),
        task_arn: task.task_arn().unwrap().into(),
        last_status: task.last_status().unwrap().into(),
        command: container_command,
    };

    // Update the pipeline status.
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        fargate_task_arn: Some(task.task_arn().unwrap().into()),
        ..Default::default()
    };
    update_pipeline(&patch_url, &auth, &pipeline)?;

    Ok(output)
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

    run(service_fn(function_handler)).await.map_err(|e| {
        info!("{e}");
        e
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_deserialization() {
        let json_input = r#"
        {
          "analysis_parameters": {
            "country": "usa",
            "city": "santa rosa",
            "region": "new mexico",
            "fips_code": "3570670"
          },
          "receipt_handle": "AQEBMtAMiWSYxry6iA8NH0wHUYvOXNLS00piVRqNYWlI5Cs8RRhd21R+5L46DsJgQtbNyrnUATM6Dw70nQoKQ5nFaU3GjK+Aone90aWVAB7DPcYpnUt9uxKdRLdgeNUAAHvBT+K83cJgHwL2ek/fGHPEBCZGN8CV2ZXEDoY2GFfRB51el+4f61YqsIxOEOpgV0djb2D0B/WzS8i8BznanguRn3bT8iz0RXk60hZjp01PN9ljSqjpFwlXM0TLx3tI1RgVYconH2CGnII9qtWz0A4MciKW0vOnKyA70AfUgDPgFFmw6OTwuPeLedCt6lhpYc7fZUGuRAc/Ozz8uAkEI6eTm2yxh1p0OJzXDoqEEaoFgsHHaHOgulmL5QwhZw3z/lBEDii8g4MTZ6UqekkK9dcxew==",
          "context": {
            "Execution": {
              "Id": "arn:aws:states:us-west-2:123456789012:execution:brokenspoke-analyzer:73f24dfc-8978-4d93-a4f7-29d1b0263e4a",
              "Name": "73f24dfc-8978-4d93-a4f7-29d1b0263e4a",
              "RoleArn": "arn:aws:iam::123456789012:role/role",
              "StartTime": "+002024-02-13T00:22:50.787000000Z"
            },
            "State": {
              "EnteredTime": "+002024-02-13T00:22:51.019000000Z",
              "Name": "BNAContext"
            },
            "StateMachine": {
              "Id": "arn:aws:states:us-west-2:123456789012:stateMachine:brokenspoke-analyzer",
              "Name": "brokenspoke-analyzer"
            }
          }
        }"#;
        let _deserialized = serde_json::from_str::<TaskInput>(json_input).unwrap();
    }
}
