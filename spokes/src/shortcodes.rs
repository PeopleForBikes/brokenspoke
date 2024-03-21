use bnacore::scorecard::{
    scorecard21::ScoreCard21, scorecard23::ScoreCard23, scorecard24::ScoreCard24,
    shortscorecard::ShortScoreCard, Format, ScoreCardVersion, ScorecardCsv,
};
use clap::{crate_name, ArgAction, Parser, ValueEnum, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, ValueEnum)]
pub enum CliFormat {
    V21,
    V23,
    V24,
}

impl From<Format> for CliFormat {
    fn from(value: Format) -> Self {
        match value {
            Format::V21 => CliFormat::V21,
            Format::V23 => CliFormat::V23,
            Format::V24 => CliFormat::V24,
        }
    }
}

impl From<CliFormat> for Format {
    fn from(value: CliFormat) -> Self {
        match value {
            CliFormat::V21 => Format::V21,
            CliFormat::V23 => Format::V23,
            CliFormat::V24 => Format::V24,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    /// ScoreCard format to use
    #[clap(value_enum)]
    pub format: CliFormat,
    /// Specify the template
    #[clap(value_parser, value_hint = ValueHint::FilePath)]
    pub city_ratings: PathBuf,
    /// Specify the output directory
    #[clap(value_parser, value_hint = ValueHint::FilePath, default_value = "brochure.csv")]
    pub output_file: PathBuf,
}

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup the CLI.
    let opts: Opts = Opts::parse();

    // Convert to shortcode.
    let scorecards: Vec<ScoreCardVersion> = match opts.format {
        CliFormat::V21 => ScoreCard21::from_csv(opts.city_ratings)?
            .iter()
            .map(|e| ScoreCardVersion::V21(e.clone()))
            .collect(),
        CliFormat::V23 => ScoreCard23::from_csv(opts.city_ratings)?
            .iter()
            .map(|e| ScoreCardVersion::V23(e.clone()))
            .collect(),
        CliFormat::V24 => ScoreCard24::from_csv(opts.city_ratings)?
            .iter()
            .map(|e| ScoreCardVersion::V24(e.clone()))
            .collect(),
    };
    let short_scorecards = scorecards
        .iter()
        .map(ShortScoreCard::from)
        .collect::<Vec<ShortScoreCard>>();
    if let Some(dir) = &opts.output_file.parent() {
        fs::create_dir_all(dir)?;
    }
    Ok(ShortScoreCard::to_csv(opts.output_file, &short_scorecards)?)
}
