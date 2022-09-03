//! Converts a city ratings CSV file to a ShortCode version of it.
//!
//! Run with:
//! ```not_rust
//! cargo run -q --example to_shortcodes
//! ```
//!
use bnacore::scorecard::{ScoreCard, ShortScoreCard};
use color_eyre::{eyre::Report, Result};
use std::fs;

// The paths must be relative to the Cargo.toml file.
const CITY_RATINGS_CSV: &'static str = "../assets/city_ratings/city_ratings_2021_v15.csv";
const OUTPUT_DIR: &'static str = "examples/output";
const SHORTCODES_CSV: &'static str = "examples/output/shortcodes.csv";

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    let scorecards = ScoreCard::from_csv(CITY_RATINGS_CSV)?;
    let short_scorecards = scorecards
        .iter()
        .map(ShortScoreCard::from)
        .collect::<Vec<ShortScoreCard>>();
    fs::create_dir_all(OUTPUT_DIR)?;
    Ok(ShortScoreCard::to_csv(SHORTCODES_CSV, &short_scorecards)?)
}
