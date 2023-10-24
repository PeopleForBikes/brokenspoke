use aws_lambda_events::event::sqs::SqsApiEventObj;
use bnalambdas::AnalysisParameters;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Serialize;

/// Response object returned by this function.
#[derive(Serialize)]
struct TaskOutput {
    analysis_parameters: AnalysisParameters,
    receipt_handle: String,
}

async fn function_handler(
    event: LambdaEvent<SqsApiEventObj<AnalysisParameters>>,
) -> Result<TaskOutput, Error> {
    let analysis_parameters = &event.payload.messages[0].body;
    let receipt_handle = &event.payload.messages[0].receipt_handle;

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
