use color_eyre::{eyre::Report, Result};
use std::{ffi::OsStr, fs, path::PathBuf, process::Command};
use walkdir::WalkDir;

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Get the paths.
    let top_dir = PathBuf::from("../../");
    let asset_dir = top_dir.join("assets");
    let output_dir = top_dir.join("pipelines/brochures/output");
    let brochure_template = asset_dir.join("brochures/brochure.svg").canonicalize()?;
    let city_ratings_15 = asset_dir
        .join("city_ratings/city_ratings_2021_v15.csv")
        .canonicalize()?;
    let brochure_template_copy = output_dir.join("brochure.svg");

    // Create the output directory.
    fs::create_dir_all(&output_dir)?;

    // Copy the brochure template frmo the asset directory.
    fs::copy(&brochure_template, &brochure_template_copy)?;

    // Convert the City Ratings file to a Shortcode file.
    let _output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("spokes")
        .arg("--bin")
        .arg("shortcodes")
        .arg(&city_ratings_15)
        .arg(&output_dir.join("brochure.csv"))
        .output()?;
    dbg!(&_output);

    //  Generate SVG files.
    let _output = Command::new("cargo")
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
        .arg(&brochure_template_copy.canonicalize()?)
        .arg(&output_dir.canonicalize()?)
        .output()?;
    dbg!(&_output);

    // Collect all the SVGs.
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

    // Generate the PDF files.
    let mut cmd = Command::new("inkscape");
    cmd.arg("--export-area-drawing")
        .arg("--batch-process")
        .arg("--export-type=pdf");
    cmd.args(svg_files);
    cmd.current_dir(fs::canonicalize(&output_dir)?);
    let _output = cmd.output()?;
    dbg!(&_output);

    // Bundle the brochures.
    let _output = Command::new("cargo")
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
    dbg!(&_output);

    Ok(())
}
