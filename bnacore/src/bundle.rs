use crate::Error;
use regex::Regex;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};
use zip::write::FileOptions;

/// Define a structure to handle brochure bundles.
pub struct Bundle {
    pub input_dir: PathBuf,
    pub group_by: GroupBy,
    pub ignore: bool,
}

/// Define the different ways to groups city rating brochures.
pub enum GroupBy {
    Country,
    State,
}

impl Bundle {
    /// Group file names by [`GroupBy`], usually country or state.
    ///
    /// The file names are expected to be in the following format:
    /// `<country>-<state>-<city>.pdf`.
    /// The function will panic if a file does not match this format.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// use bnacore::bundle::{Bundle, GroupBy};
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
    /// let bundle = Bundle {input_dir: PathBuf::from("."), group_by: GroupBy::Country, ignore: false};
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
    /// `<country>-<state>-<city>.pdf`.
    /// The function will panic if a file does not match this format.
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// use bnacore::bundle::{Bundle, GroupBy};
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
    /// let bundle = Bundle {input_dir: PathBuf::from("."), group_by: GroupBy::Country, ignore: false};
    /// let groups = bundle.group_files(&files);
    /// assert_eq!(country_groups, groups);
    /// ````
    pub fn group_files(&self, paths: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
        let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let re = Regex::new(
            r"(?x)
        (?P<country>(.*)) # country
        -
        (?P<state>(.*))   # state (or country)
        -
        (?P<city>(.*))   # city
        .pdf              # file extension
        $
      ",
        )
        .unwrap();

        for path in paths {
            let filename = path.file_name().map(|f| f.to_str()).unwrap().unwrap();
            let caps = match re.captures(filename) {
                None => {
                    if self.ignore {
                        continue;
                    } else {
                        panic!("The file `{filename}` does not have the right format. `<country>-<state>-<city>.pdf` was expected.")
                    }
                }
                Some(c) => c,
            };

            let key = match self.group_by {
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
    pub fn zip(&self) -> Result<(), Error> {
        // Collect the PDF files.
        let pdf_files = self.gather_pdf_files();

        // Group the files.
        let groups = self.group_files(&pdf_files);

        // Zip them all.
        let all_path = self.input_dir.join("all.zip");
        let all_file = std::fs::File::create(&all_path).unwrap();
        let mut all_zip = zip::ZipWriter::new(all_file);

        // Zip each group.
        for (group_name, files) in groups.iter() {
            // Zip the group.
            let group_name = format!("{group_name}.zip");
            let group_path = self.input_dir.join(&group_name);
            let group_file = std::fs::File::create(&group_path).unwrap();
            let mut group_zip = zip::ZipWriter::new(group_file);

            // Add each file from the group.
            for file in files {
                // Read the input file.
                let mut f = File::open(file)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;

                // Add a file object to the archive.
                let file_name = file.file_name().map(|f| f.to_str()).unwrap().unwrap();
                group_zip.start_file(file_name, FileOptions::default())?;
                all_zip.start_file(file_name, FileOptions::default())?;

                // Write the content of the file into each zip files.
                group_zip.write_all(&buffer)?;
                all_zip.write_all(&buffer)?;
            }
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
}

/// Define the conditions to select a PDF file.
pub fn filter_pdf_files(entry: &DirEntry) -> bool {
    entry.metadata().unwrap().is_file() && entry.path().extension() == Some(OsStr::new("pdf"))
}
