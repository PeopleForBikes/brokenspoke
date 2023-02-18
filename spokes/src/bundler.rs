use bnacore::bundle::{Bundle, FileType, GroupBy};
use clap::{crate_name, ArgAction, Parser, ValueEnum, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GroupByArg {
    City,
    Country,
    State,
}

// These 2 `From` Traits are implemented mainly to make sure that [`GroupBy`]
// and [`GroupByArg`] stay in sync.
impl From<GroupBy> for GroupByArg {
    fn from(group_by: GroupBy) -> Self {
        match group_by {
            GroupBy::City => Self::City,
            GroupBy::Country => Self::Country,
            GroupBy::State => Self::State,
        }
    }
}
impl From<GroupByArg> for GroupBy {
    fn from(group_by_arg: GroupByArg) -> Self {
        match group_by_arg {
            GroupByArg::City => Self::City,
            GroupByArg::Country => Self::Country,
            GroupByArg::State => Self::State,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FiletypeArg {
    All,
    Pdf,
}

// These 2 `From` Traits are implemented mainly to make sure that [`FileType`]
// and [`FiletypeArg`] stay in sync.
impl From<FileType> for FiletypeArg {
    fn from(group_by: FileType) -> Self {
        match group_by {
            FileType::All => Self::All,
            FileType::Pdf => Self::Pdf,
        }
    }
}
impl From<FiletypeArg> for FileType {
    fn from(group_by_arg: FiletypeArg) -> Self {
        match group_by_arg {
            FiletypeArg::All => Self::All,
            FiletypeArg::Pdf => Self::Pdf,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Set the verbosity level
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    /// Fail if the files do not match the exact name format
    #[clap(short, long)]
    pub strict: bool,
    /// Create an archive containig all the entries
    #[clap(short, long)]
    pub all: bool,
    /// Specify which files to look for.
    #[clap(value_enum)]
    pub filetype: FiletypeArg,
    /// Specify how to group the files,
    #[clap(value_enum)]
    pub group_by: GroupByArg,
    /// Specify the directory containing the files to bundle.
    #[clap(value_parser, value_hint = ValueHint::DirPath)]
    pub input_dir: PathBuf,
}

fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup the CLI.
    let opts: Opts = Opts::parse();

    // Bundle the brochures.
    let bundle = Bundle {
        input_dir: opts.input_dir,
        group_by: opts.group_by.into(),
        strict: opts.strict,
        filetype: opts.filetype.into(),
    };

    // Zip'em.
    Ok(bundle.zip(false)?)
}
