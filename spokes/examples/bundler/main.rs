//! Bundle all the brochures from the same country in a zip file.
//!
//! ```
//! cargo run --example bundler
//! ```
use bnacore::bundle::{Bundle, GroupBy};
use color_eyre::{eyre::Report, Result};
use std::path::PathBuf;

fn main() -> Result<(), Report> {
    // Bundle the brochures.
    let bundle = Bundle {
        input_dir: PathBuf::from("examples/brochures/output"),
        group_by: GroupBy::Country,
        ignore: false,
    };

    // Zip'em.
    Ok(bundle.zip()?)
}
