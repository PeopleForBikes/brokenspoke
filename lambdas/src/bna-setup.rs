use bnacore::{
    aws::{get_aws_parameter, get_aws_secrets},
    neon::{
        model::{
            NeonBranch, NeonCreateBranchRequest, NeonCreateBranchResponse, NeonEndpoint,
            NeonEndpointType, NeonListBranchResponses,
        },
        NEON_PROJECTS_URL,
    },
};
use bnalambdas::AnalysisParameters;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::header::{self, HeaderValue};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use tracing::info;

const NEON_MAX_BRANCHES: usize = 20;

#[derive(Deserialize)]
struct TaskInput {
    analysis_parameters: AnalysisParameters,
}

#[derive(Serialize)]
struct TaskOutput {
    neon: Neon,
}

#[derive(Serialize)]
struct Neon {
    branch_id: String,
    host: String,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Read the task inputs.
    let analysis_parameters = &event.payload.analysis_parameters;

    // Create the Neon HTTP client.
    let neon_api_key = get_aws_secrets("NEON_API_KEY").await?;
    let mut headers = header::HeaderMap::new();
    let mut auth_value = HeaderValue::from_str(format!("Bearer {}", neon_api_key).as_ref())?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);
    let neon = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let neon_project_id = get_aws_parameter("NEON_BROKENSPOKE_ANALYZER_PROJECT").await?;
    let neon_branches_url = format!("{}/{}/branches", NEON_PROJECTS_URL, neon_project_id);

    // Query neon API and check whether we can create a branch or not.
    info!("Checking database branch capacity...");
    let branches = neon
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
    let create_branch_response = neon
        .post(&neon_branches_url)
        .json(&create_branch_request)
        .send()
        .await?
        .error_for_status()?
        .json::<NeonCreateBranchResponse>()
        .await?;
    info!("{:#?}", create_branch_response);

    let neon_branch_id = create_branch_response.branch.id.unwrap();
    let neon_host = create_branch_response
        .endpoints
        .first()
        .unwrap()
        .host
        .clone()
        .unwrap();

    // Return the ID of the created database branch.
    Ok(TaskOutput {
        neon: Neon {
            branch_id: neon_branch_id,
            host: neon_host,
        },
    })
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
