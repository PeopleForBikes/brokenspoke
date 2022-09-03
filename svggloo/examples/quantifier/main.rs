use color_eyre::{eyre::Report, Result};
use std::path::Path;
use svggloo::{
    setup,
    template::{render, Exporter},
};

// The paths must be relative to the Cargo.toml file.
const SVG_TEMPLATE_FILENAME: &'static str = "examples/quantifier/bike_lane_categories.svg";
const OUTPUT_DIR: &'static str = "examples/quantifier/output";

fn main() -> Result<(), Report> {
    // Setup the application.
    setup()?;

    // Load the template file.
    let svg_template = Path::new(SVG_TEMPLATE_FILENAME);

    // Specify the output directory.
    let output_dir = Path::new(OUTPUT_DIR);

    // Render the template.
    let fields = vec![
        String::from("country"),
        String::from("state"),
        String::from("city"),
    ];
    let _ = render(
        &svg_template,
        output_dir,
        Some(Exporter::Inkscape),
        Some(fields),
        None,
    )?;

    Ok(())
}
