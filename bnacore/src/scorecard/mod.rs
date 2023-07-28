pub mod scorecard21;
pub mod scorecard23;
pub mod shortscorecard;

use crate::{Dataset, Error};
use csv::Reader;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use url::Url;

use self::{scorecard21::ScoreCard21, scorecard23::ScoreCard23, shortscorecard::ShortScoreCard};

pub trait CsvExt {
    /// Read a CSV file and populate a Vector of Self.
    fn from_csv<P>(path: P) -> Result<Vec<Self>, Error>
    where
        P: AsRef<Path>,
        Self: Sized + DeserializeOwned,
    {
        let mut csv_reader = Reader::from_path(path)?;
        let mut scorecards: Vec<Self> = vec![];
        for record in csv_reader.deserialize() {
            scorecards.push(record?);
        }

        Ok(scorecards)
    }

    /// Saves a slice of Ts to a CSV file.
    fn to_csv<P, T>(path: P, entries: &[T]) -> Result<(), Error>
    where
        P: AsRef<Path>,
        T: Serialize,
    {
        let mut w = csv::Writer::from_path(path)?;
        for entry in entries {
            w.serialize(entry)?;
        }
        Ok(w.flush()?)
    }
}

pub trait ScorecardExt {
    fn full_name(&self) -> String;
    fn url(&self, dataset: &Dataset) -> Result<Url, Error>;
}

#[derive(Debug, Clone)]
pub enum Format {
    V21,
    V23,
}

#[derive(Debug, Clone)]
pub enum ScoreCardVersion {
    V21(ScoreCard21),
    V23(ScoreCard23),
}

impl ScorecardExt for ScoreCardVersion {
    fn full_name(&self) -> String {
        match self {
            ScoreCardVersion::V21(s) => s.full_name(),
            ScoreCardVersion::V23(s) => s.full_name(),
        }
    }

    fn url(&self, dataset: &Dataset) -> Result<Url, Error> {
        match self {
            ScoreCardVersion::V21(s) => s.url(dataset),
            ScoreCardVersion::V23(s) => s.url(dataset),
        }
    }
}

impl From<&ScoreCardVersion> for ShortScoreCard {
    fn from(value: &ScoreCardVersion) -> Self {
        match value {
            ScoreCardVersion::V21(s) => ShortScoreCard::from(s),
            ScoreCardVersion::V23(s) => ShortScoreCard::from(s),
        }
    }
}
