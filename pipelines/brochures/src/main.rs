use bnacore::build_cmd_args;
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
    let brochure_template = asset_dir
        .join("visuals/template-scorecard-pg1-v23.1.svg")
        .canonicalize()?;
    let brochure_information_page = asset_dir.join("visuals/template-scorecard-pg2-v23.1.pdf");
    let city_ratings_15 = asset_dir
        .join("city_ratings/city_ratings_2022_v7.csv")
        .canonicalize()?;
    let brochure_template_copy = output_dir.join("scorecard.svg");
    let shortcodes = output_dir.join("scorecard.csv");

    // Create the output directory.
    info!("📁 Creating the output directory...");
    fs::create_dir_all(&output_dir)?;

    // Copy the brochure template from the asset directory.
    info!("⚙️  Copying the brochure template...");
    fs::copy(brochure_template, &brochure_template_copy)?;

    // Convert the City Ratings file to a Shortcode file.
    info!("🔄 Converting the City Ratings file to a Shortcode file...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("shortcodes")
        .arg(&city_ratings_15)
        .arg(&shortcodes)
        .output()?;
    process_output(&output)?;

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

    // Collect all the SVGs.
    debug!("🗄️  Collecting the generated SVG files...");
    let mut svg_files = Vec::new();
    for entry in WalkDir::new(&output_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.into_path();

        // Ignore the template itself.
        if path.file_name() == brochure_template_copy.file_name() {
            continue;
        }

        // Otherwise ensure the file is a .svg and add it to the list.
        if let Some(ext) = path.extension() {
            if ext == OsStr::new("svg") {
                let filename = path.file_name().unwrap();
                let filename_str = filename.to_str().unwrap();
                svg_files.push(filename_str.to_string())
            }
        }
    }

    // Generate the PDF files.
    info!("📃 Generating PDF files...");
    // generate_pdf(&svg_files, &output_dir)?;
    let cmd_args_groups = build_cmd_args(
        "inkscape",
        &[
            "--export-area-drawing".to_string(),
            "--batch-process".to_string(),
            "--export-type=pdf".to_string(),
        ],
        &svg_files,
        bnacore::MAX_PROMPT_LENGTH,
    )?;
    for cmd_args in cmd_args_groups {
        let mut cmd = Command::new("inkscape");
        cmd.args(cmd_args).current_dir(&output_dir);
        let output = cmd.output().map_err(Report::new)?;
        process_output_with_command(&output, &cmd)?
    }

    // Append information page.
    info!("📎 Append information page");
    let pdf_files = svg_files
        .iter()
        .map(|f| output_dir.join(f))
        .map(|f| f.with_extension("pdf"))
        .map(|f| f.to_str().unwrap().to_string())
        .collect::<Vec<String>>();
    let cmd_args_groups = build_cmd_args(
        "cargo",
        &[
            "run".to_string(),
            "-p".to_string(),
            "spokes".to_string(),
            "--bin".to_string(),
            "appender".to_string(),
            brochure_information_page.to_str().unwrap().to_string(),
        ],
        &pdf_files,
        bnacore::MAX_PROMPT_LENGTH,
    )?;
    for cmd_args in cmd_args_groups {
        let mut cmd = Command::new("cargo");
        cmd.args(cmd_args).current_dir(&output_dir);
        let output = cmd.output().map_err(Report::new)?;
        process_output_with_command(&output, &cmd)?
    }

    // Bundle the brochures.
    info!("📦 Bundling the brochures...");
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("bundler")
        .arg("--")
        .arg("pdf")
        .arg("country")
        .arg(&output_dir.canonicalize()?)
        .output()?;
    process_output(&output)?;

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
