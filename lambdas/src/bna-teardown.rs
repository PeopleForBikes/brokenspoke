use bnacore::neon::NEON_PROJECTS_URL;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

async fn function_handler(_event: LambdaEvent<Value>) -> Result<(), Error> {
    // Delete neon branch.
    let delete_branch_response = reqwest::Client::new()
        .get(NEON_PROJECTS_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<HashMap<String, String>>()
        .await?;
    info!("{:#?}", delete_branch_response);

    Ok(())
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
