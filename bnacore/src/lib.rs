//! This crate defines the structures and functions which are shared between
//! the PFB projects.
pub mod scorecard;

use crate::scorecard::{City, CommunitySurvey, ScoreCard, BNA};
use pyo3::{exceptions::PyOSError, prelude::*};
use std::{fmt, io};
use thiserror::Error;

/// Represent the PFB S3 storage base URL.
const PFB_S3_STORAGE_BASE_URL: &str =
    "https://s3.amazonaws.com/production-pfb-storage-us-east-1/results";

/// Represent the name of the "neighborhood ways" dataset.
const DS_NEIGHBORHOOD_WAYS: &str = "neighborhood_ways";
const DS_NEIGHBORHOOD_OVERALL_SCORES: &str = "neighborhood_overall_scores";

/// Errors that can happen when using pfbcore.
#[derive(Error, Debug)]
pub enum Error {
    /// Error from an underlying system.
    #[error("Internal error: {0}")]
    Internal(String),
    /// Error from the URL crate.
    #[error("URL error")]
    Url {
        #[from]
        source: url::ParseError,
    },
    /// Error from the CSV crate.
    #[error("CSV error")]
    Csv {
        #[from]
        source: csv::Error,
    },
    /// I/O Error.
    #[error("I/O error")]
    IOError {
        #[from]
        source: io::Error,
    },
}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

/// Describe all the available city datasets.
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum Dataset {
    NeighborhoodWays,
    NeighborhoodOverallScores,
}

impl From<&str> for Dataset {
    fn from(item: &str) -> Self {
        match item {
            DS_NEIGHBORHOOD_WAYS => Dataset::NeighborhoodWays,
            DS_NEIGHBORHOOD_OVERALL_SCORES => Dataset::NeighborhoodOverallScores,
            _ => panic!("Cannot parse dataset name {}", item),
        }
    }
}

impl fmt::Display for Dataset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dataset::NeighborhoodWays => write!(f, "{}", DS_NEIGHBORHOOD_WAYS),
            Dataset::NeighborhoodOverallScores => write!(f, "{}", DS_NEIGHBORHOOD_OVERALL_SCORES),
        }
    }
}

impl Dataset {
    /// Return the file extension of a specific dataset.
    pub fn extension(&self) -> String {
        match self {
            Dataset::NeighborhoodWays => String::from("zip"),
            Dataset::NeighborhoodOverallScores => String::from("csv"),
        }
    }
}

/// Decribes all the objects to export to the Python bnacore module.
#[pymodule]
fn bnacore(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<City>()?;
    m.add_class::<CommunitySurvey>()?;
    m.add_class::<ScoreCard>()?;
    m.add_class::<BNA>()?;
    Ok(())
}
