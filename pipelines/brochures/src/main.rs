use color_eyre::{
    eyre::{eyre, Report},
    Result,
};
use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Output},
};
use tracing::{debug, info};
use walkdir::WalkDir;

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
    let output_dir = top_dir.join("pipelines/brochures/output");
    let brochure_template = asset_dir.join("brochures/brochure.svg").canonicalize()?;
    let city_ratings_15 = asset_dir
        .join("city_ratings/city_ratings_2021_v15.csv")
        .canonicalize()?;
    let brochure_template_copy = output_dir.join("brochure.svg");

    // Create the output directory.
    info!("📁 Creating the output directory...");
    fs::create_dir_all(&output_dir)?;
    // dbg!(&output_dir);

    // Copy the brochure template from the asset directory.
    info!("⚙️  Copying the brochure template...");
    fs::copy(&brochure_template, &brochure_template_copy)?;

    // Convert the City Ratings file to a Shortcode file.
    info!("🔄 Converting the City Ratings file to a Shortcode file...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("shortcodes")
        .arg(&city_ratings_15)
        .arg(&output_dir.join("brochure.csv"))
        .output()?;
    process_output(&output)?;
    // dbg!(&_output);

    //  Generate SVG files.
    info!("📄 Generating SVG files...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("svggloo")
        .arg("--")
        .arg("--field")
        .arg("co")
        .arg("--field")
        .arg("st")
        .arg("--field")
        .arg("ci")
        .arg(&brochure_template_copy)
        .arg(&output_dir)
        .output()?;
    process_output(&output)?;
    // dbg!(&output);

    // Collect all the SVGs.
    debug!("🗄️  Collecting the generated SVG files...");
    let mut svg_files = Vec::new();
    for entry in WalkDir::new(&output_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.into_path();
        if let Some(ext) = path.extension() {
            if ext == OsStr::new("svg") {
                let filename = path.file_name().unwrap();
                let filename_str = filename.to_str().unwrap();
                svg_files.push(filename_str.to_string())
            }
        }
    }
    // dbg!(&svg_files);

    // Generate the PDF files.
    info!("📃 Generating PDF files...");
    let mut cmd = Command::new("inkscape");
    cmd.arg("--export-area-drawing")
        .arg("--batch-process")
        .arg("--export-type=pdf");
    cmd.args(svg_files);
    cmd.current_dir(&output_dir);
    let output = cmd.output()?;
    process_output_with_command(&output, &cmd)?;
    // dbg!(&output);

    // Bundle the brochures.
    info!("📦 Bundling the brochures...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("bundler")
        .arg("--")
        .arg("--ignore")
        .arg("country")
        .arg(&output_dir.canonicalize()?)
        .output()?;
    process_output_with_command(&output, &cmd)?;
    // dbg!(&output);

    info!("✅ Done");
    Ok(())
}

fn process_output_with_command(output: &Output, cmd: &Command) -> Result<(), Report> {
    if output.status.success() {
        return Ok(());
    }

    Err(eyre!(
        "The command {:?} failed with status code {:?} and the following error: {:?}.\n The following arguments were passed to the command:\n {:?}",
        cmd.get_program(),
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        cmd.get_args().collect::<Vec<&OsStr>>()
    ))
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
