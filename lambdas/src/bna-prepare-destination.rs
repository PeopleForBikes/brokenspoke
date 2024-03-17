use bnacore::aws::s3::create_calver_s3_directories;
use bnalambdas::{AnalysisParameters, Context, BROKENSPOKE_ANALYZER_BUCKET};
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
    analysis_parameters: AnalysisParameters,
    s3_destination: String,
    context: Context,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Read the task inputs.
    info!("Reading input...");
    let analysis_parameters = &event.payload.analysis_parameters;
    let state_machine_context = &event.payload.context;

    // Read the task inputs.
    info!("Creating S3 directory...");
    let dir = create_calver_s3_directories(
        BROKENSPOKE_ANALYZER_BUCKET,
        analysis_parameters.country.as_str(),
        analysis_parameters.city.as_str(),
        analysis_parameters.region.as_deref(),
    )
    .await?;

    // Update the output with the S3 folder that was created.
    Ok(TaskOutput {
        analysis_parameters: analysis_parameters.clone(),
        s3_destination: dir.to_str().unwrap().to_string(),
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
