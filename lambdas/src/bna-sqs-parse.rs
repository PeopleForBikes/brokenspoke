use aws_lambda_events::event::sqs::SqsApiEventObj;
use bnacore::aws::get_aws_secrets_value;
use bnalambdas::{AnalysisParameters, BrokenspokePipeline, BrokenspokeState};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

/// Response object returned by this function.
#[derive(Serialize)]
struct TaskOutput {
    analysis_parameters: AnalysisParameters,
    receipt_handle: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
}

async fn function_handler(
    event: LambdaEvent<SqsApiEventObj<AnalysisParameters>>,
) -> Result<TaskOutput, Error> {
    // Retrieve service account credentials.
    const SERVICE_ACCOUNT_CREDENTIALS: &str = "BROKENSPOKE_ANALYZER_SERVICE_ACCOUNT_CREDENTIALS";
    let client_id = get_aws_secrets_value(SERVICE_ACCOUNT_CREDENTIALS, "client_id").await?;
    let client_secret = get_aws_secrets_value(SERVICE_ACCOUNT_CREDENTIALS, "client_secret").await?;

    // Authenticate.
    let client = Client::new();
    let auth_response = client
        .post("https://peopleforbikes.auth.us-west-2.amazoncognito.com/oauth2/token")
        .form(&[
            ("grant_type", "client_credentials"),
            ("scope", "service_account/write"),
        ])
        .basic_auth(client_id, Some(client_secret))
        .send()?
        .error_for_status()?
        .json::<AuthResponse>()?;

    // Parse the SQS message.
    let analysis_parameters = &event.payload.messages[0].body;
    let receipt_handle = &event.payload.messages[0].receipt_handle;

    // Post a new analysis with the pipeline ID.
    let pipeline = BrokenspokePipeline {
        state: Some(BrokenspokeState::SqsMessage),
        state_machine_id: Some(String::from("")),
        sqs_message: Some(serde_json::to_string(analysis_parameters)?),
        neon_branch_id: None,
        fargate_task_id: None,
        s3_bucket: None,
    };
    let _post = client
        .post("https://api.peopleforbikes.xyz/bna/analysis")
        .bearer_auth(auth_response.access_token)
        .json(&pipeline)
        .send()?
        .error_for_status()?;

    // Return the task output.
    Ok(TaskOutput {
        analysis_parameters: analysis_parameters.clone(),
        receipt_handle: receipt_handle.clone().unwrap(),
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
