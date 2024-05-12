use crate::Error;
use libflate::gzip::{EncodeOptions, Encoder};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};
use zip::{write::SimpleFileOptions, ZipWriter};

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
    /// let groups = bundle.group(&files).unwrap();
    /// assert_eq!(country_groups, groups);
    /// ````
    pub fn group(&self, filenames: &[String]) -> Result<HashMap<String, Vec<PathBuf>>, Error> {
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
    /// let groups = bundle.group_files(&files).unwrap();
    /// assert_eq!(country_groups, groups);
    /// ````
    pub fn group_files(&self, paths: &[PathBuf]) -> Result<HashMap<String, Vec<PathBuf>>, Error> {
        let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for path in paths {
            let filename = path.file_name().map(|f| f.to_str()).unwrap().unwrap();
            let bna_filename = BNAFilename::parse(filename)?;
            let key = match self.group_by {
                GroupBy::City => format!("{}-{}", bna_filename.city, bna_filename.state,),
                GroupBy::Country => bna_filename.country,
                GroupBy::State => bna_filename.state,
            };

            groups
                .entry(key)
                .and_modify(|g| g.push(path.to_path_buf()))
                .or_insert_with(|| vec![path.to_path_buf()]);
        }
        Ok(groups)
    }

    /// Creates a zip file for each group, as well as a zip file for all the files.
    pub fn zip(&self, bundle_all: bool) -> Result<(), Error> {
        // Collect the files.
        let collected_files = match self.filetype {
            FileType::All => self.gather_all_files(),
            FileType::Pdf => self.gather_pdf_files(),
        };

        // Group the files.
        let groups = self.group_files(&collected_files)?;

        // Create a "bundles" directory to store the bundles.
        let bundle_dir = self.input_dir.join("bundles");
        fs::create_dir_all(&bundle_dir)?;

        // Define the compression options.
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

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
                group_zip.start_file(file_name, options)?;
                group_zip.write_all(&buffer)?;

                // Add the file to the "all" archive.
                if bundle_all {
                    let mut all_zip = ZipWriter::new(File::open(&all_zip_path)?);
                    all_zip.start_file(file_name, options)?;
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
        let groups = self.group_files(&collected_files)?;

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

pub struct BNAFilename {
    pub country: String,
    pub state: String,
    pub city: String,
    pub description: Option<String>,
    pub extension: String,
}

impl BNAFilename {
    pub fn parse(i: &str) -> Result<Self, Error> {
        let file = Path::new(i);

        // Extract the extension.
        let extension = match file.extension() {
            Some(extension) => match extension.to_str() {
                Some(extension) => extension.to_string(),
                None => {
                    return Err(Error::IOError(io::Error::new(
                        ErrorKind::InvalidInput,
                        format!("this extension is not valid UTF-8: {i}"),
                    )))
                }
            },
            None => {
                return Err(Error::IOError(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("this file has no extension: {i}"),
                )))
            }
        };

        // Extract the stem.
        let stem = match file.file_stem() {
            Some(stem) => match stem.to_str() {
                Some(stem) => stem.to_string(),
                None => {
                    return Err(Error::IOError(io::Error::new(
                        ErrorKind::InvalidInput,
                        format!("this filename is not valid UTF-8: {i}"),
                    )))
                }
            },
            None => {
                return Err(Error::IOError(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("this file has no stem: {i}"),
                )))
            }
        };

        // Process the filename parts.
        let split_stem = stem.split('-').collect::<Vec<&str>>();
        let country = match split_stem.first() {
            Some(country) => country.to_string(),
            None => {
                return Err(Error::IOError(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "the first part of the file name is expected to be the country name: {i}"
                    ),
                )))
            }
        };
        let state = match split_stem.get(1) {
            Some(state) => state.to_string(),
            None => {
                return Err(Error::IOError(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "the second part of the file name is expected to be the state name: {i}"
                    ),
                )))
            }
        };
        let city = match split_stem.get(2) {
            Some(city) => city.to_string(),
            None => {
                return Err(Error::IOError(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("the third part of the file name is expected to be the city name: {i}"),
                )))
            }
        };
        let description = split_stem.get(4).map(|&part| String::from(part));

        Ok(BNAFilename {
            country,
            state,
            city,
            description,
            extension,
        })
    }
}
