use aws_lambda_events::event::sqs::SqsApiEventObj;
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
    event_obj: SqsApiEventObj<AnalysisParameters>,
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
    // Authenticate the service account.
    let auth_response = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Parse the SQS message.
    info!("Parse the SQS message");
    let analysis_parameters = &event.payload.event_obj.messages[0].body;
    let receipt_handle = &event.payload.event_obj.messages[0].receipt_handle;
    let state_machine_context = &event.payload.context;

    // Create a new pipeline entry.
    info!(
        state_machine_id = state_machine_context.execution.name,
        "create a new Brokensspoke pipeline entry",
    );
    let (state_machine_id, scheduled_trigger_id) = state_machine_context.execution.ids()?;
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        scheduled_trigger_id,
        state: Some(BrokenspokeState::SqsMessage),
        sqs_message: Some(serde_json::to_string(analysis_parameters)?),
        neon_branch_id: None,
        fargate_task_id: None,
        s3_bucket: None,
    };
    let _post = Client::new()
        .post("https://api.peopleforbikes.xyz/bnas/analysis")
        .bearer_auth(auth_response.access_token)
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
