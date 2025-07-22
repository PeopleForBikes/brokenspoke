//! This crate defines the structures and functions which are shared between
//! the PFB projects.
pub mod aws;
pub mod bundle;
pub mod combine;
pub mod neon;
pub mod scorecard;
pub mod template;
pub mod versioning;

use std::{fmt, io};
use thiserror::Error;

/// Represent the PFB S3 storage base URL.
const PFB_S3_STORAGE_BASE_URL: &str =
    "https://s3.amazonaws.com/production-pfb-storage-us-east-1/results";
/// Represent the PFB S3 base URL for public documents.
const PFB_S3_PUBLIC_DOCUMENTS: &str = "https://s3.amazonaws.com/pfb-public-documents";

#[cfg(windows)]
/// Represent the maximum length for the command prompt on a Windows platform.
/// Ref: https://learn.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/command-line-string-limitation#more-information
pub const MAX_PROMPT_LENGTH: usize = 8191;

#[cfg(unix)]
/// There is no length limit for the command prompt on Unix platforms.
pub const MAX_PROMPT_LENGTH: usize = usize::MAX;

/// Errors that can happen when using bnacore.
#[derive(Error, Debug)]
pub enum Error {
    /// Error from an underlying system.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Error from the URL crate.
    #[error("URL error")]
    Url(#[from] url::ParseError),

    /// Error from the CSV crate.
    #[error("CSV error")]
    Csv(#[from] csv::Error),

    /// Line too long.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// I/O Error.
    #[error("I/O error")]
    IOError(#[from] io::Error),

    /// Error from the MiniJinja crate .
    #[error("MiniJinja error")]
    MiniJinja(#[from] minijinja::Error),

    /// Windows prompt too long.
    #[error("The length of the command prompt exceeds the maximum permitted by the platform (8191 characters).")]
    PromptTooLong,

    /// Error from the Zip crate.
    #[error("Zip error")]
    ZipError(#[from] zip::result::ZipError),

    /// Error from the Serde crate.
    #[error("Serde JSON error")]
    SerdeJSON(#[from] serde_json::Error),

    /// Error From the Reqwest crate.
    #[error("Reqwest error")]
    Reqwest(#[from] reqwest::Error),

    /// AWS error from the bnacore::aws module.
    #[error("AWS error")]
    BNAAWS(#[from] aws::AWSError),

    /// Environment variable error.
    #[error("Environment variable error")]
    VarError(#[from] std::env::VarError),
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
            _ => panic!("Cannot parse dataset name {item}"),
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

// /// Decribes all the objects to export to the Python bnacore module.
// #[pymodule]
// fn bnacore(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
//     m.add_class::<City>()?;
//     m.add_class::<CommunitySurvey>()?;
//     m.add_class::<ScoreCard>()?;
//     m.add_class::<BNA>()?;
//     Ok(())
// }

/// Build commands.
///
/// ```
/// use bnacore::build_cmd_args;
/// let cmds = build_cmd_args(
///   "inkscape",
///   &[
///       "--export-area-drawing".to_string(),
///       "--batch-process".to_string(),
///       "--export-type=pdf".to_string(),
///   ],
///   &[
///       "canada-on-toronto.svg".to_string(),
///       "united_states-co-boulder.svg".to_string(),
///       "united_states-tx-austin.svg".to_string(),
///       "united_states-tx-houston.svg".to_string(),
///   ],
///   112,
/// ).unwrap();
/// assert_eq!(
///   cmds,
///     vec![
///         vec![
///             "--export-area-drawing".to_string(),
///             "--batch-process".to_string(),
///             "--export-type=pdf".to_string(),
///             "canada-on-toronto.svg".to_string(),
///         ],
///         vec![
///             "--export-area-drawing".to_string(),
///             "--batch-process".to_string(),
///             "--export-type=pdf".to_string(),
///             "united_states-co-boulder.svg".to_string(),
///             "united_states-tx-austin.svg".to_string(),
///         ],
///         vec![
///             "--export-area-drawing".to_string(),
///             "--batch-process".to_string(),
///             "--export-type=pdf".to_string(),
///             "united_states-tx-houston.svg".to_string(),
///         ],
///     ]
/// );
/// ```
pub fn build_cmd_args(
    program: &str,
    flags: &[String],
    positionals: &[String],
    limit: usize,
) -> Result<Vec<Vec<String>>, Error> {
    let program_len = program.len();
    let flags_len: usize = flags.iter().map(|f| f.len()).sum();
    let base_len = program_len + flags_len + flags.len();

    let cmd_limit = limit - base_len;
    let positional_groups = word_chunks(positionals, cmd_limit)?;

    let mut cmds: Vec<Vec<String>> = Vec::new();

    for positional_group in positional_groups {
        let mut cmd: Vec<String> = Vec::new();
        cmd.extend(flags.iter().map(String::from));
        cmd.extend(positional_group);
        cmds.push(cmd);
    }

    Ok(cmds)
}

/// Group words into chunks of a certain size.
///
/// The size of the chunks account for the space between the words.
///
/// ```
/// use bnacore::word_chunks;
/// let chunks = word_chunks(
///   &[
///     "gastropub".to_string(),
///     "shaman".to_string(),
///     "skateboard".to_string(),
///     "succulents".to_string(),
///     "meditation".to_string(),
///     "street".to_string(),
///   ],
///   23,
/// ).unwrap();
/// assert_eq!(
///   chunks,
///   vec![
///     vec!["gastropub".to_string(), "shaman".to_string()],
///     vec!["skateboard".to_string(),"succulents".to_string(),"meditation".to_string()],
///     vec!["street".to_string()]
///   ]
/// );
/// ```
pub fn word_chunks(words: &[String], limit: usize) -> Result<Vec<Vec<String>>, Error> {
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut chunk: Vec<String> = Vec::new();
    let mut chunk_len: usize = 0;

    // Process the words.
    for word in words {
        // Validate the word size.
        let word_len = word.len();
        if word.len() > limit {
            return Err(Error::Internal(format!(
                "The length of the word (\"{word}\" ({word_len})) exceeds the limit defined({limit})."
            )));
        }
        if (chunk_len + word.len() + 1) < limit {
            chunk.push(word.clone());
            chunk_len += word.len() + 1;
        } else {
            chunks.push(chunk);
            chunk = vec![word.clone()];
            chunk_len = 0;
        }
    }
    chunks.push(chunk);

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "The length of the word (\\\"gastropub\\\" (9)) exceeds the limit defined(5)."
    )]
    fn test_word_chunks_too_long() {
        let _chunks = word_chunks(&["gastropub".to_string()], 5).unwrap();
    }
}
