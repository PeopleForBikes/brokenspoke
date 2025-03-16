use color_eyre::{
    eyre::{eyre, Report},
    Result,
};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};
use tracing::info;

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup logging.
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Get the paths.
    let top_dir = PathBuf::from("../../").canonicalize()?;
    let asset_dir = top_dir.join("assets");
    let output_dir = top_dir.join("pipelines/retrieve/output");
    let city_ratings_15 = asset_dir
        .join("city-ratings/city-ratings-v25.09.csv")
        .canonicalize()?;

    // Create the output directory.
    info!("📁 Creating the output directory...");
    fs::create_dir_all(&output_dir)?;

    // Retrieve the datasets.
    info!("📡 Downloading datasets...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("retriever")
        .arg("--")
        .arg("v24")
        .arg(&city_ratings_15)
        .arg("census-block")
        .arg("connected-census-block")
        .arg("data-dictionary")
        .arg("overall-scores")
        .arg("ways")
        .output()?;
    process_output(&output)?;

    // Bundle the datasets.
    info!("📦 Bundling datasets...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("bundler")
        .arg("--")
        .arg("all")
        .arg("city")
        .arg(&output_dir.canonicalize()?)
        .output()?;
    process_output(&output)?;

    info!("✅ Done");
    Ok(())
}

fn process_output(output: &Output) -> Result<(), Report> {
    if output.status.success() {
        return Ok(());
    }

    Err(eyre!(
        "The command  failed with status code {:?} and the following error: {:?}.",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    ))
}
