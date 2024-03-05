//! Reads a CSV file containing scorecard data.
//!
//! Run with:
//! ```not_rust
//! cargo run -q --example csv_reader
//! ```
//!
use bnacore::scorecard::{scorecard24::ScoreCard24, ScorecardCsv};
use color_eyre::{eyre::Report, Result};

// The paths must be relative to the Cargo.toml file.
const CITY_RATINGS_CSV: &str =
    "../assets/city_ratings/city-ratings-all-historical-results-v24.1.csv";

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    let _scorecards = ScoreCard24::from_csv(CITY_RATINGS_CSV)?;
    Ok(())
}
