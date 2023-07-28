use bnacore::{
    scorecard::{
        scorecard21::ScoreCard21, scorecard23::ScoreCard23, CsvExt, Format, ScoreCardVersion,
        ScorecardExt,
    },
    Dataset,
};
use clap::{Parser, ValueEnum, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::{convert::From, fs, path::PathBuf};
use trauma::{
    download::{Download, Status},
    downloader::DownloaderBuilder,
};

/// Describe all the available city datasets.
///
/// This enum must be in sync with [`bnacore::Dataset`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, ValueEnum)]
pub enum CliDataset {
    CensusBlock,
    ConnectedCensusBlock,
    DataDictionary,
    OverallScores,
    Ways,
}

impl From<Dataset> for CliDataset {
    fn from(dataset: Dataset) -> Self {
        match dataset {
            Dataset::CensusBlock => CliDataset::CensusBlock,
            Dataset::ConnectedCensusBlock => CliDataset::ConnectedCensusBlock,
            Dataset::DataDictionary => CliDataset::DataDictionary,
            Dataset::OverallScores => CliDataset::OverallScores,
            Dataset::Ways => CliDataset::Ways,
        }
    }
}

impl From<&CliDataset> for Dataset {
    fn from(dataset: &CliDataset) -> Self {
        match dataset {
            CliDataset::CensusBlock => Dataset::CensusBlock,
            CliDataset::ConnectedCensusBlock => Dataset::ConnectedCensusBlock,
            CliDataset::DataDictionary => Dataset::DataDictionary,
            CliDataset::OverallScores => Dataset::OverallScores,
            CliDataset::Ways => Dataset::Ways,
        }
    }
}

impl From<CliDataset> for Dataset {
    fn from(dataset: CliDataset) -> Self {
        Dataset::from(&dataset)
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CliFormat {
    V21,
    V23,
}

impl From<Format> for CliFormat {
    fn from(value: Format) -> Self {
        match value {
            Format::V21 => CliFormat::V21,
            Format::V23 => CliFormat::V23,
        }
    }
}

impl From<CliFormat> for Format {
    fn from(value: CliFormat) -> Self {
        match value {
            CliFormat::V21 => Format::V21,
            CliFormat::V23 => Format::V23,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opts {
    /// Number of files to download simultaneously
    #[clap(short, long, default_value_t = 32)]
    pub parallel_requests: u16,

    /// Number times to retry a failing download
    #[clap(short, long, default_value_t = 3)]
    pub retries: u16,

    /// Destination directory
    #[clap(short, long,value_parser, value_hint = ValueHint::DirPath, default_value = "output")]
    pub destination_folder: PathBuf,

    /// ScoreCard format to use
    #[clap(value_enum)]
    pub format: CliFormat,

    /// CSV file containing the list of city datasets to download
    #[clap()]
    pub from_csv: String,

    /// Dataset(s) to retrieve
    #[clap(value_enum)]
    pub datasets: Vec<CliDataset>,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Read the CLI arguments.
    let opts = Opts::parse();

    // Prepare the variable holding the list of cities to process.
    // let mut cities: Vec<ScoreCardVersion> = Vec::new();

    // Prepare the list of scorecards to retrieve from a CSV file.
    let scorecards: Vec<ScoreCardVersion> = match opts.format {
        CliFormat::V21 => ScoreCard21::from_csv(opts.from_csv)?
            .iter()
            .map(|e| ScoreCardVersion::V21(e.clone()))
            .collect(),
        CliFormat::V23 => ScoreCard23::from_csv(opts.from_csv)?
            .iter()
            .map(|e| ScoreCardVersion::V23(e.clone()))
            .collect(),
    };

    // Ensure the output folder exists.
    if !opts.destination_folder.exists() {
        fs::create_dir_all(&opts.destination_folder)?;
    }

    // Prepare the downloader.
    let downloader = DownloaderBuilder::new()
        .directory(opts.destination_folder)
        .build();

    // Prepare the downloads for each city.
    let mut downloads: Vec<Download> = Vec::new();
    for city in scorecards {
        // Prepare the dataset downloads for this city.
        for dataset in &opts.datasets {
            let ds: Dataset = dataset.into();
            let filename = format!("{}-{}.{}", &city.full_name(), &ds, &ds.extension());
            let d = Download::new(&city.url(&ds)?, &filename.replace(' ', "_"));
            downloads.push(d);
        }
    }

    // Start the download operations.
    let dl_result = downloader.download(&downloads).await;

    // Display information about the failures.
    dl_result
        .iter()
        .filter(|s| s.status() != &Status::Success)
        .for_each(|s| println!("{:?}", s.status()));

    Ok(())
}
