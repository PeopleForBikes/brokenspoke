use bnacore::scorecard::{ScoreCard, ShortScoreCard};
use clap::{crate_name, Parser, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Specify the template
    #[clap(parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub city_ratings: PathBuf,
    /// Specify the output directory
    #[clap(parse(from_os_str), value_hint = ValueHint::FilePath, default_value = "brochure.csv")]
    pub output_file: PathBuf,
}

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup the CLI.
    let opts: Opts = Opts::parse();

    // Convert to shortcode.
    let scorecards = ScoreCard::from_csv(opts.city_ratings)?;
    let short_scorecards = scorecards
        .iter()
        .map(ShortScoreCard::from)
        .collect::<Vec<ShortScoreCard>>();
    if let Some(dir) = &opts.output_file.parent() {
        fs::create_dir_all(dir)?;
    }
    Ok(ShortScoreCard::to_csv(opts.output_file, &short_scorecards)?)
}
