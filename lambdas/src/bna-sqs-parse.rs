use aws_lambda_events::sqs::SqsApiMessageObj;
use bnalambdas::{
    authenticate_service_account, AnalysisParameters, BrokenspokePipeline, BrokenspokeState,
    Context,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize)]
struct TaskInput {
    #[serde(rename = "Messages")]
    messages: Vec<SqsApiMessageObj<AnalysisParameters>>,
    context: Context,
}
/// Response object returned by this function.
#[derive(Serialize)]
struct TaskOutput {
    analysis_parameters: AnalysisParameters,
    receipt_handle: String,
    context: Context,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Retrieve API URL.
    let url = "https://api.peopleforbikes.xyz/bnas/analysis";

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Parse the SQS message.
    info!("Parse the SQS message");
    let analysis_parameters = &event.payload.messages[0].body;
    let receipt_handle = &event.payload.messages[0].receipt_handle;
    let state_machine_context = &event.payload.context;
    let (state_machine_id, scheduled_trigger_id) = state_machine_context.execution.ids()?;

    // Create a new pipeline entry.
    info!(
        state_machine_id = state_machine_context.execution.name,
        "create a new Brokensspoke pipeline entry",
    );

    let pipeline = BrokenspokePipeline {
        state_machine_id,
        scheduled_trigger_id,
        state: Some(BrokenspokeState::SqsMessage),
        sqs_message: Some(serde_json::to_string(analysis_parameters)?),
        ..Default::default()
    };
    let _post = Client::new()
        .post(url)
        .bearer_auth(auth.access_token)
        .json(&pipeline)
        .send()?
        .error_for_status()?;

    // Return the task output.
    Ok(TaskOutput {
        analysis_parameters: analysis_parameters.clone(),
        receipt_handle: receipt_handle.clone().unwrap(),
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

#[test]
fn test_input_deserialization() {
    let raw_json = r#"{
    "Messages": [
      {
        "Body": "{\n  \"city\": \"santa rosa\",\n  \"country\": \"usa\",\n  \"fips_code\": \"3570670\",\n  \"region\": \"new mexico\"\n}",
        "Md5OfBody": "a7d2c2a0976976a725f06db3a9b90520",
        "MessageId": "b53c03f0-6993-4b4e-8815-a84e323a0e4c",
        "ReceiptHandle": "AQEBig5tn0SKv0mFFajmwh/50mLs1g2hFXGEcblkGGpa3pqmiposxJEsdgInINH3tHwWDQ6C1Xoly7abjNr3G6m88QYZYPYcFf3HnBM2s+zYXsAITrBAA92Z8CGXPvglx04NxgiYLFIegyqKRUmWNE2uwI/ubbpcAMdrCcjXRQ+LjGLHRYR567uW75TZHsAds8GPKJ937pJ9RiSU9hHSrLAjABZD/AWXgGeJ19619w9TOSRFFzKiZRxcWqhDEtasl4YN6mX3+/lY4Gx5/ATPzXmjlIKa33viTURtlMuAEKjJ4gmSFgdIaovSkrl7V+ZbJw85anWCzcQ8rQSqJB2p4aZgX57MBTrIKrgUKDaP7CvASuM27jj8Pou8Ka9bspOhfOHlwPSmxn3AfeYl3ruT1prGoA=="
      }
    ],
    "context": {
      "Execution": {
        "StartTime": "2024-02-12T16:45:38.655Z",
        "Id": "arn:aws:states:us-west-2:123456789012:execution:brokenspoke-analyzer:a0e708f8-3d9f-4749-b4de-20b2c2aab3d2",
        "RoleArn": "arn:aws:iam::123456789012:role/role",
        "Name": "a0e708f8-3d9f-4749-b4de-20b2c2aab3d2"
      },
      "State": {
        "EnteredTime": "2024-02-12T16:45:38.881Z",
        "Name": "BNAContext"
      },
      "StateMachine": {
        "Id": "arn:aws:states:us-west-2:123456789012:stateMachine:brokenspoke-analyzer",
        "Name": "brokenspoke-analyzer"
      }
    }
  }"#;
    let _deserialized = serde_json::from_str::<TaskInput>(&raw_json).unwrap();
}
