use bnacore::aws::s3::create_calver_s3_directories;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        .init();

    let dir = create_calver_s3_directories(
        "brokenspoke-analyzer",
        "testland",
        "testville",
        Some("testregion"),
    )
    .await?;
    dbg!(dir);

    Ok(())
}
