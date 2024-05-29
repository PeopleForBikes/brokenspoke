use aws_lambda_events::sqs::SqsApiMessageObj;
use bnalambdas::{AnalysisParameters, Context};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use slug::slugify;
use tracing::info;

const SLUG_LENGTH: usize = 71;
const SHORT_UUID_LENGTH: usize = 8;
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
    slug: String,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Parse the SQS message.
    info!("Parse the SQS message");
    let analysis_parameters = &event.payload.messages[0].body;
    let receipt_handle = &event.payload.messages[0].receipt_handle;
    let state_machine_context = &event.payload.context;

    // Valiadate the parameters.
    let params = analysis_parameters.sanitized();

    // Generate a slug.
    let mut slug = slugify(format!(
        "{}-{}-{}",
        params.country,
        params.region.clone().unwrap(),
        params.city
    ));
    slug.truncate(SLUG_LENGTH);
    let mut context_id = state_machine_context.id.to_string();
    context_id.truncate(SHORT_UUID_LENGTH);

    // Return the task output.
    Ok(TaskOutput {
        analysis_parameters: params,
        receipt_handle: receipt_handle.clone().unwrap(),
        context: state_machine_context.clone(),
        slug: format!("{slug}-{context_id}"),
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
    let json_input = r#"{
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
      },
      "Id": "9ff90cac-0cf5-4923-897f-4416df5e7328"
    }
  }"#;
    let _deserialized = serde_json::from_str::<TaskInput>(json_input).unwrap();
}
