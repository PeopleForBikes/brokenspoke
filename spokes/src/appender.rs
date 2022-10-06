use bnacore::combine::batch_append;
use clap::{crate_name, Parser, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::path::{Path, PathBuf};

// CLI options.
#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, value_parser)]
    pub verbose: u8,
    /// Specify the document to append to the other ones
    #[clap()]
    pub extra: PathBuf,
    /// Specify the files to append the extra document to
    #[clap( value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,
}

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup the CLI.
    let opts: Opts = Opts::parse();

    // Collect all the documents.
    let f = opts
        .files
        .iter()
        .map(|e| e.as_path())
        .collect::<Vec<&Path>>();

    // Combine the extra document to them all.
    Ok(batch_append(&f, &opts.extra)?)
}
