use bnacore::template::{render, Exporter};
use clap::Parser;
use clap::{crate_name, ArgAction, ValueEnum, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::path::PathBuf;

/// Define the SVG exporters.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ExporterArg {
    Inkscape,
    CairoSVG,
    SVG2PDF,
}

// These 2 `From` Traits are implemented mainly to make sure that [`Exporter`]
// and [`ExporterArg`] stay in sync.
impl From<Exporter> for ExporterArg {
    fn from(exporter: Exporter) -> Self {
        match exporter {
            Exporter::CairoSVG => Self::CairoSVG,
            Exporter::Inkscape => Self::Inkscape,
            Exporter::SVG2PDF => Self::SVG2PDF,
        }
    }
}
impl From<ExporterArg> for Exporter {
    fn from(exporter_arg: ExporterArg) -> Self {
        match exporter_arg {
            ExporterArg::CairoSVG => Self::CairoSVG,
            ExporterArg::Inkscape => Self::Inkscape,
            ExporterArg::SVG2PDF => Self::SVG2PDF,
        }
    }
}

// CLI options.
#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    /// Specify the data fields to use to generate the rendered template name
    // Due to a bug in clap parser, we cannot use a `Option<Vec<String>>` with
    // multiple values. Therefore we are allowing multiple occurences with one
    // single value.
    // Ref: https://github.com/clap-rs/clap/issues/1772
    // Ref: https://github.com/clap-rs/clap/issues/3066
    #[clap(long, action = ArgAction::Append, number_of_values = 1)]
    pub field: Option<Vec<String>>,
    /// Specify the template
    #[clap(value_parser, value_hint = ValueHint::FilePath)]
    pub template: PathBuf,
    /// Specify the output directory
    #[clap(value_parser, value_hint = ValueHint::DirPath, default_value = "output")]
    pub output_dir: PathBuf,
    /// Specify the separator
    #[clap(short, long, default_value = "-")]
    pub separator: String,
    /// Export the rendered template as PDF
    #[clap(short, long, value_enum)]
    pub exporter: Option<ExporterArg>,
}

// Perform a data-merge operation, and export SVGs to PDFs.
fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup the CLI.
    let opts: Opts = Opts::parse();

    // Convert the exporter.
    let exporter: Option<Exporter> = opts.exporter.map(|e| e.into());

    let _ = render(
        &opts.template,
        &opts.output_dir,
        exporter,
        opts.field,
        Some(&opts.separator),
    );

    Ok(())
}
