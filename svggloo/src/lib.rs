#![doc = include_str!("../README.md")]
pub mod cli;
pub mod template;

use color_eyre::{eyre::Report, Result};

/// Setup the application.
///
/// Set up the `color_eyre` hooks.
pub fn setup() -> Result<(), Report> {
    color_eyre::install()?;

    Ok(())
}
