//! Perform a data-merge operation for a subset of city ratings entries.
//!
//! ```
//! cargo run --example brochures
//! ```
//!
//! The sample CSV file is being generated from the city ratings csv file, by
//! converting it to shortcodes (run the commands from the `examples/brochure`
//! directory):
//! ```
//! cargo run --bin shortcodes -- ../../../assets/city_ratings/city_ratings_2021_v15.csv shortcodes-2021-v15.csv
//! ```
//!
//! Then sampling it with the `xsv` tool.
//! ```
//!  xsv sample 10 shortcodes-2021-v15.csv > brochure.csv
//! ```
//!
use bnacore::template::{render, Exporter};
use color_eyre::{eyre::Report, Result};
use std::path::PathBuf;

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Get the paths.
    let example_dir = PathBuf::from("examples/brochures").canonicalize()?;
    let brochure_template = example_dir.join("brochure.svg").canonicalize()?;
    let output_dir = example_dir.join("output");

    // Render the template.
    let fields = vec![String::from("co"), String::from("st"), String::from("ci")];
    render(
        &brochure_template,
        &output_dir,
        Some(Exporter::Inkscape),
        Some(fields),
        None,
    )?;

    Ok(())
}
