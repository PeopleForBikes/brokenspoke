use clap::Parser;
use color_eyre::{eyre::Report, Result};
use svggloo::cli::Opts;
use svggloo::setup;
use svggloo::template::render;

fn main() -> Result<(), Report> {
    // Setup the application.
    setup()?;

    // Setup the CLI.
    let opts: Opts = svggloo::cli::Opts::parse();
    dbg!(&opts);

    let _ = render(
        &opts.template,
        &opts.output_dir,
        opts.exporter,
        opts.field,
        Some(&opts.separator),
    );

    Ok(())
}
