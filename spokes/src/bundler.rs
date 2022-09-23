use bnacore::bundle::{Bundle, GroupBy};
use clap::{crate_name, ArgEnum, Parser, ValueHint};
use color_eyre::{eyre::Report, Result};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum GroupByArg {
    Country,
    State,
}

// These 2 `From` Traits are implemented mainly to make sure that [`GroupBy`]
// and [`GroupByArg`] stay in sync.
impl From<GroupBy> for GroupByArg {
    fn from(group_by: GroupBy) -> Self {
        match group_by {
            GroupBy::Country => Self::Country,
            GroupBy::State => Self::State,
        }
    }
}
impl From<GroupByArg> for GroupBy {
    fn from(group_by_arg: GroupByArg) -> Self {
        match group_by_arg {
            GroupByArg::Country => Self::Country,
            GroupByArg::State => Self::State,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author, about, version)]
pub struct Opts {
    /// Sets the verbosity level
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    /// Silently ignore the files not matching the exact name format
    #[clap(short, long)]
    pub ignore: bool,
    /// Specify how to group the files,
    #[clap(arg_enum)]
    pub group_by: GroupByArg,
    /// Specify the directory containing the files to bundle.
    #[clap(parse(from_os_str), value_hint = ValueHint::DirPath)]
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
        ignore: opts.ignore,
    };

    // Zip'em.
    Ok(bundle.zip()?)
}
