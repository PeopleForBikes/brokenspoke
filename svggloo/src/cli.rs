use crate::template::Exporter;
use clap::{crate_name, Parser, ValueHint};
use std::path::PathBuf;

// Main options.
#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Specify the data fields to use to generate the rendered template name
    // Due to a bug in clap parser, we cannot use a `Option<Vec<String>>` with
    // multiple values. Therefore we are allowing multiple occurences with one
    // single value.
    // Ref: https://github.com/clap-rs/clap/issues/1772
    // Ref: https://github.com/clap-rs/clap/issues/3066
    #[clap(long, multiple_occurrences(true), number_of_values = 1)]
    pub field: Option<Vec<String>>,
    /// Specify the template
    #[clap(parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub template: PathBuf,
    /// Specify the output directory
    #[clap(parse(from_os_str), value_hint = ValueHint::DirPath, default_value = "output")]
    pub output_dir: PathBuf,
    /// Specify the separator
    #[clap(short, long, default_value = "-")]
    pub separator: String,
    /// Export the rendered template as PDF
    #[clap(short, long, arg_enum)]
    pub exporter: Option<Exporter>,
}
