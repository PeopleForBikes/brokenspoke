use bnacore::{
    aws::{get_aws_parameter_value, get_aws_secrets_value},
    neon,
};
use bnalambdas::{
    authenticate_service_account, update_pipeline, AnalysisParameters, BrokenspokePipeline,
    BrokenspokeState,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
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
    // Retrieve API hostname.
    let api_hostname = get_aws_parameter_value("BNA_API_HOSTNAME").await?;

    // Prepare the API URL.
    let url = format!("{api_hostname}/bnas/analysis");

    // Authenticate the service account.
    info!("Authenticating service account...");
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Read the task inputs.
    info!("Reading input...");
    let analysis_parameters = &event.payload.analysis_parameters;
    let (state_machine_id, _) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());

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
    let api_key = get_aws_secrets_value("NEON_API_KEY", "NEON_API_KEY").await?;
    let project_id = get_aws_parameter_value("NEON_BROKENSPOKE_ANALYZER_PROJECT").await?;
    let neon = neon::Client::new(&api_key, &project_id)?;

    // Query neon API and check whether we can create a branch or not.
    info!("Checking database branch capacity...");
    let branches = neon.get_branches().await?;

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
    let create_branch_response = neon.create_branch(&branch_name).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_deserialization() {
        let json_input = r#"{
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
