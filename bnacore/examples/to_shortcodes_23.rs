//! Converts a city ratings CSV file to a ShortCode version of it.
//!
//! Run with:
//! ```not_rust
//! cargo run -q --example to_shortcodes_23
//! ```
//!
use bnacore::scorecard::{scorecard23::ScoreCard23, shortscorecard::ShortScoreCard, CsvExt};
use color_eyre::{eyre::Report, Result};
use std::fs;

// The paths must be relative to the Cargo.toml file.
const CITY_RATINGS_CSV: &'static str = "../assets/city_ratings/city-ratings-v23.2.csv";
const OUTPUT_DIR: &'static str = "examples/output";
const SHORTCODES_CSV: &'static str = "examples/output/shortcodes.csv";

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    let scorecards = ScoreCard23::from_csv(CITY_RATINGS_CSV)?;
    let short_scorecards = scorecards
        .iter()
        .map(ShortScoreCard::from)
        .collect::<Vec<ShortScoreCard>>();
    fs::create_dir_all(OUTPUT_DIR)?;
    Ok(ShortScoreCard::to_csv(SHORTCODES_CSV, &short_scorecards)?)
}
