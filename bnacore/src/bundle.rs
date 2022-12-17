use crate::Error;
use libflate::gzip::{EncodeOptions, Encoder};
use regex::Regex;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, ZipWriter};

/// Define a structure to handle brochure bundles.
pub struct Bundle {
    pub input_dir: PathBuf,
    pub group_by: GroupBy,
    pub strict: bool,
    pub filetype: FileType,
}

/// Define the different ways to groups city rating brochures.
pub enum GroupBy {
    City,
    Country,
    State,
}

pub enum FileType {
    All,
    Pdf,
}

impl Bundle {
    /// Group file names by [`GroupBy`], usually country or state.
    ///
    /// The file names are expected to be in the following format:
    /// `<country>-<state>-<city>[-<filename>].<extension>`.
    /// The function will panic if a file does not match this format.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// use bnacore::bundle::{Bundle, FileType, GroupBy};
    ///
    /// let files = vec![
    ///     String::from("australia-nt-alice_springs.pdf"),
    ///     String::from("england-eng-london.pdf"),
    ///     String::from("france-idf-paris.pdf"),
    ///     String::from("united_states-ca-arcata.pdf"),
    ///     String::from("united_states-fl-altamonte_springs.pdf"),
    /// ];
    /// let mut country_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    /// country_groups.insert(String::from("australia"), vec![PathBuf::from("australia-nt-alice_springs.pdf")]);
    /// country_groups.insert(String::from("england"), vec![PathBuf::from("england-eng-london.pdf")]);
    /// country_groups.insert(String::from("france"), vec![PathBuf::from("france-idf-paris.pdf")]);
    /// country_groups.insert(String::from("united_states"), vec![PathBuf::from("united_states-ca-arcata.pdf"), PathBuf::from("united_states-fl-altamonte_springs.pdf")]);
    /// let bundle = Bundle {input_dir: PathBuf::from("."), group_by: GroupBy::Country, strict: true, filetype: FileType::Pdf};
    /// let groups = bundle.group(&files);
    /// assert_eq!(country_groups, groups);
    /// ````
    pub fn group(&self, filenames: &[String]) -> HashMap<String, Vec<PathBuf>> {
        let paths = filenames
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>();
        self.group_files(&paths)
    }

    /// Group file names by [`GroupBy`], usually country or state.
    ///
    /// The file names are expected to be in the following format:
    /// `<country>-<state>-<city>[-<filename>].<extension>`.
    /// The function will panic if a file does not match this format.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// use bnacore::bundle::{Bundle, FileType, GroupBy};
    ///
    /// let files = vec![
    ///     PathBuf::from("australia-nt-alice_springs.pdf"),
    ///     PathBuf::from("england-eng-london.pdf"),
    ///     PathBuf::from("france-idf-paris.pdf"),
    ///     PathBuf::from("united_states-ca-arcata.pdf"),
    ///     PathBuf::from("united_states-fl-altamonte_springs.pdf"),
    /// ];
    /// let mut country_groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    /// country_groups.insert(String::from("australia"), vec![PathBuf::from("australia-nt-alice_springs.pdf")]);
    /// country_groups.insert(String::from("england"), vec![PathBuf::from("england-eng-london.pdf")]);
    /// country_groups.insert(String::from("france"), vec![PathBuf::from("france-idf-paris.pdf")]);
    /// country_groups.insert(String::from("united_states"), vec![PathBuf::from("united_states-ca-arcata.pdf"), PathBuf::from("united_states-fl-altamonte_springs.pdf")]);
    /// let bundle = Bundle {input_dir: PathBuf::from("."), group_by: GroupBy::Country, strict: true, filetype: FileType::Pdf};
    /// let groups = bundle.group_files(&files);
    /// assert_eq!(country_groups, groups);
    /// ````
    pub fn group_files(&self, paths: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
        let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let re = Regex::new(
            r"(?x)
        (?P<country>([^-]*))# country
        -
        (?P<state>([^-]*))  # state (or country)
        -
        (?P<city>([^-]*))   # city
        -?                  # optional separator
        (.*)?               # optional file name
        \.
        (.*)                # file extension
        $
      ",
        )
        .unwrap();

        for path in paths {
            let filename = path.file_name().map(|f| f.to_str()).unwrap().unwrap();
            let caps = match re.captures(filename) {
                None => {
                    if self.strict {
                        panic!("The file `{filename}` does not have the right format. `<country>-<state>-<city>[-<filename>].<extension>` was expected.")
                    } else {
                        continue;
                    }
                }
                Some(c) => c,
            };

            let key = match self.group_by {
                GroupBy::City => format!(
                    "{}-{}",
                    caps.name("city").unwrap().as_str(),
                    caps.name("state").unwrap().as_str()
                ),
                GroupBy::Country => caps.name("country").unwrap().as_str().to_string(),
                GroupBy::State => caps.name("state").unwrap().as_str().to_string(),
            };

            groups
                .entry(key)
                .and_modify(|g| g.push(path.to_path_buf()))
                .or_insert_with(|| vec![path.to_path_buf()]);
        }
        groups
    }

    /// Creates a zip file for each group, as well as a zip file for all the files.
    pub fn zip(&self, bundle_all: bool) -> Result<(), Error> {
        // Collect the files.
        let collected_files = match self.filetype {
            FileType::All => self.gather_all_files(),
            FileType::Pdf => self.gather_pdf_files(),
        };

        // Group the files.
        let groups = self.group_files(&collected_files);

        // Create a "bundles" directory to store the bundles.
        let bundle_dir = self.input_dir.join("bundles");
        fs::create_dir_all(&bundle_dir)?;

        // Zip them all.
        let all_zip_path = bundle_dir.join("all.zip");
        if bundle_all {
            let all_zip_file = File::create(&all_zip_path).unwrap();
            ZipWriter::new(all_zip_file);
        }
        // Zip each group.
        for (group_name, files) in groups.iter() {
            // Zip the group.
            let group_name = format!("{group_name}.zip");
            let group_path = bundle_dir.join(group_name);
            let group_file = std::fs::File::create(group_path).unwrap();
            let mut group_zip = zip::ZipWriter::new(group_file);

            // Add each file from the group.
            for file in files {
                // Read the input file.
                let mut f = File::open(file)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;

                // Add the file object to the archive.
                let file_name = file.file_name().map(|f| f.to_str()).unwrap().unwrap();
                group_zip.start_file(file_name, FileOptions::default())?;
                group_zip.write_all(&buffer)?;

                // Add the file to the "all" archive.
                if bundle_all {
                    let mut all_zip = ZipWriter::new(File::open(&all_zip_path)?);
                    all_zip.start_file(file_name, FileOptions::default())?;
                    all_zip.write_all(&buffer)?;
                }
            }
        }

        Ok(())
    }

    /// Creates a gzip file for each group.
    pub fn gzip(&self) -> Result<(), Error> {
        // Collect the files.
        let collected_files = match self.filetype {
            FileType::All => self.gather_all_files(),
            FileType::Pdf => self.gather_pdf_files(),
        };

        // Group the files.
        let groups = self.group_files(&collected_files);

        // Create a "bundles" directory to store the bundles.
        let bundle_dir = self.input_dir.join("bundles");
        fs::create_dir_all(&bundle_dir)?;

        // Zip each group.
        for (group_name, files) in groups.iter() {
            // Zip the group.
            let group_name = format!("{group_name}.gz");
            let group_path = bundle_dir.join(group_name);
            let group_file = std::fs::File::create(group_path).unwrap();
            let options = EncodeOptions::new().no_compression();
            let mut archive = Encoder::with_options(group_file, options).unwrap();

            // Add each file from the group.
            for file in files {
                // Read the input file.
                let mut f = File::open(file)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;

                // Add the file object to the archive.
                archive.write_all(&buffer)?;
            }
            archive.finish().into_result()?;
        }

        Ok(())
    }

    /// Gather the files satisfying a filter predicate.
    pub fn gather_files<F>(&self, filter: F) -> Vec<PathBuf>
    where
        F: FnOnce(&DirEntry) -> bool + Copy,
    {
        let mut files: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(&self.input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !filter(&entry) {
                continue;
            }
            let f = entry.path();
            files.push(PathBuf::from(f));
        }
        files
    }

    /// Gather PDF files.
    pub fn gather_pdf_files(&self) -> Vec<PathBuf> {
        self.gather_files(filter_pdf_files)
    }

    /// Gather any file.
    pub fn gather_all_files(&self) -> Vec<PathBuf> {
        self.gather_files(filter_files)
    }
}

/// Define the conditions to select files.
pub fn filter_files(entry: &DirEntry) -> bool {
    entry.metadata().unwrap().is_file()
}

/// Define the conditions to select a PDF file.
pub fn filter_pdf_files(entry: &DirEntry) -> bool {
    entry.metadata().unwrap().is_file() && entry.path().extension() == Some(OsStr::new("pdf"))
}
