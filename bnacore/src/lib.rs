//! This crate defines the structures and functions which are shared between
//! the PFB projects.
pub mod bundle;
pub mod combine;
pub mod scorecard;
pub mod template;

use crate::scorecard::{City, CommunitySurvey, ScoreCard, BNA};
use pyo3::{exceptions::PyOSError, prelude::*};
use std::{fmt, io};
use thiserror::Error;

/// Represent the PFB S3 storage base URL.
const PFB_S3_STORAGE_BASE_URL: &str =
    "https://s3.amazonaws.com/production-pfb-storage-us-east-1/results";
/// Represent the PFB S3 base URL for public documents.
const PFB_S3_PUBLIC_DOCUMENTS: &str = "https://s3.amazonaws.com/pfb-public-documents";

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
    /// MiniJinja Error.
    #[error("MiniJinja error")]
    MiniJinja {
        #[from]
        source: minijinja::Error,
    },
    /// Zip Error.
    #[error("Zip error")]
    ZipError {
        #[from]
        source: zip::result::ZipError,
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
    CensusBlock,
    ConnectedCensusBlock,
    DataDictionary,
    OverallScores,
    Ways,
}

impl From<&str> for Dataset {
    fn from(item: &str) -> Self {
        match item {
            Dataset::CENSUS_BLOCK => Dataset::CensusBlock,
            Dataset::CONNECTED_CENSUS_BLOCK => Dataset::CensusBlock,
            Dataset::DATA_DICTIONARY => Dataset::DataDictionary,
            Dataset::OVERALL_SCORES => Dataset::OverallScores,
            Dataset::WAYS => Dataset::Ways,
            _ => panic!("Cannot parse dataset name {}", item),
        }
    }
}

impl fmt::Display for Dataset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dataset::CensusBlock => write!(f, "{}", Dataset::CENSUS_BLOCK),
            Dataset::ConnectedCensusBlock => {
                write!(f, "{}", Dataset::CONNECTED_CENSUS_BLOCK)
            }
            Dataset::DataDictionary => write!(f, "{}", Dataset::DATA_DICTIONARY),
            Dataset::OverallScores => write!(f, "{}", Dataset::OVERALL_SCORES),
            Dataset::Ways => write!(f, "{}", Dataset::WAYS),
        }
    }
}

impl Dataset {
    /// Represent the census block dataset.
    const CENSUS_BLOCK: &'static str = "neighborhood_census_blocks";
    /// Represent the connected census block dataset.
    const CONNECTED_CENSUS_BLOCK: &'static str = "neighborhood_connected_census_blocks";
    /// Represent the BNA data set dictionary.
    const DATA_DICTIONARY: &'static str = "BNA.Data.Dictionary";
    /// Represent the name of the "neighborhood ways" dataset.
    const WAYS: &'static str = "neighborhood_ways";
    /// Represent the overall scores dataset.
    const OVERALL_SCORES: &'static str = "neighborhood_overall_scores";

    /// Return the file extension of a specific dataset.
    pub fn extension(&self) -> String {
        match self {
            Dataset::CensusBlock => String::from("zip"),
            Dataset::ConnectedCensusBlock => String::from("csv.zip"),
            Dataset::DataDictionary => String::from("xlsx"),
            Dataset::OverallScores => String::from("csv"),
            Dataset::Ways => String::from("zip"),
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
