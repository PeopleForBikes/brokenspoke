use bnacore::{
    aws::{get_aws_parameter, get_aws_secrets_value},
    neon::{
        model::{
            NeonBranch, NeonCreateBranchRequest, NeonCreateBranchResponse, NeonEndpoint,
            NeonEndpointType, NeonListBranchResponses,
        },
        NEON_PROJECTS_URL,
    },
};
use bnalambdas::{
    authenticate_service_account, update_pipeline, AnalysisParameters, BrokenspokePipeline,
    BrokenspokeState, Context,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::header::{self, HeaderValue};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use tracing::info;

const NEON_MAX_BRANCHES: usize = 20;

#[derive(Deserialize)]
struct TaskInput {
    analysis_parameters: AnalysisParameters,
    context: Context,
}

#[derive(Serialize)]
struct TaskOutput {
    neon: Neon,
    context: Context,
}

#[derive(Serialize)]
struct Neon {
    branch_id: String,
    host: String,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Retrieve API URL.
    let url = "https://api.peopleforbikes.xyz/bnas/analysis";

    // Authenticate the service account.
    info!("Authenticating service account...");
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Read the task inputs.
    info!("Reading input...");
    let analysis_parameters = &event.payload.analysis_parameters;
    let state_machine_context = &event.payload.context;
    let (state_machine_id, _) = state_machine_context.execution.ids()?;

    // Update the pipeline status.
    info!("updating pipeline...");
    let patch_url = format!("{url}/{state_machine_id}");
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        state: Some(BrokenspokeState::Setup),
        ..Default::default()
    };
    update_pipeline(&patch_url, &auth, &pipeline)?;

    // Create the Neon HTTP client.
    info!("Creating Neon client...");
    let neon_api_key = get_aws_secrets_value("NEON_API_KEY", "NEON_API_KEY").await?;
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

    // Update the pipeline status.
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        neon_branch_id: Some(neon_branch_id.clone()),
        ..Default::default()
    };
    update_pipeline(&patch_url, &auth, &pipeline)?;

    // Return the ID of the created database branch.
    Ok(TaskOutput {
        neon: Neon {
            branch_id: neon_branch_id,
            host: neon_host,
        },
        context: state_machine_context.clone(),
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

    run(service_fn(function_handler)).await.map_err(|e| {
        info!("{e}");
        e
    })
}
