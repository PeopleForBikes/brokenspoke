use aws_config::BehaviorVersion;
use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use std::path::{Path, PathBuf};
use time::{macros::format_description, OffsetDateTime};

use crate::Error;

/// Return the calver notation YY.0M for the UTC now date.
fn calver_utc_now() -> String {
    let now_utc = OffsetDateTime::now_utc();
    let format = format_description!("[year repr:last_two].[month padding:zero]");
    now_utc
        .format(&format)
        .expect("the description format must not be wrong")
}

/// Generate a base path following the PFB convention.
pub fn calver_base<P>(
    country: &str,
    city: &str,
    region: Option<&str>,
    date_override: Option<&str>,
    base_dir: Option<P>,
) -> PathBuf
where
    P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>,
{
    // Start with the base Path.
    let mut p = base_dir.map_or(PathBuf::new(), |v| Path::new(&v).to_path_buf());

    // Add the country.
    p.push(country.to_lowercase());

    // Add the region, falling back to the country name.
    if let Some(region) = region {
        p.push(region.to_lowercase());
    } else {
        p.push(country.to_lowercase());
    }

    // Add the city.
    p.push(city.to_lowercase());

    // Use the date override if any.
    if let Some(date_override) = date_override {
        p.push(date_override);
        return p;
    }

    // Otherwise use the appropriate calver.
    let calver = calver_utc_now();
    p.push(calver);
    p
}

/// Compute the next calver version for a series of paths.
///
/// This function does not validate the paths. Therefore different paths ending with a
/// valid Ubuntu-like calver format would still produce a result, even though it would
/// not make sense.
///
/// Paths are ignored if:
///   - they are not valid unicode
///   - they do not end with a number
pub fn calver_next(dirs: &[PathBuf]) -> u32 {
    let with_micro = dirs
        .iter()
        .filter_map(|d| d.file_name())
        .filter_map(|d| d.to_str())
        .filter(|d| d.chars().filter(|c| *c == '.').count() == 2)
        .filter_map(|d| {
            d.split('.')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .parse::<u32>()
                .ok()
        })
        .collect::<Vec<u32>>();

    // If there is no directory with a micro part, create the first one.
    // Otherwise get the highest micro and increment it.
    match with_micro.iter().max() {
        Some(micro) => micro + 1,
        None => 1,
    }
}

/// Create S3 directories in a sepecific bucket, following the PFB convention.
pub async fn create_calver_s3_directories(
    bucket_name: &str,
    country: &str,
    city: &str,
    region: Option<&str>,
) -> Result<PathBuf, crate::Error> {
    // Get the base path.
    let s3_dir = calver_base::<PathBuf>(country, city, region, None, None);
    let mut s3_dir_str = s3_dir.to_str().unwrap().to_string();

    // Configure the S3 client.
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_s3::Client::new(&config);

    // List the existing directory matching the base path.
    let mut response = client
        .list_objects_v2()
        .bucket(bucket_name.to_owned())
        .prefix(s3_dir_str.clone())
        .into_paginator()
        .send();

    // Filter the directories.
    let mut matches: Vec<String> = Vec::new();
    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                for object in output.contents() {
                    if let Some(key) = &object.key {
                        if key.ends_with('/') {
                            matches.push(key.to_string());
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("{err:?}")
            }
        }
    }

    // Get the next calver version if necessary.
    if !matches.is_empty() {
        let dirs = matches.iter().map(PathBuf::from).collect::<Vec<PathBuf>>();
        let revision = calver_next(&dirs);
        s3_dir_str.push('.');
        s3_dir_str.push_str(revision.to_string().as_str());
    }

    // Create the folder object.
    let res = client
        .put_object()
        .bucket(bucket_name)
        .key(format!("{s3_dir_str}/",))
        .body(ByteStream::new(SdkBody::from("")))
        .send()
        .await;
    match res {
        Ok(_) => Ok(PathBuf::from(s3_dir_str)),
        Err(e) => Err(Error::BNAAWS(super::AWSError::S3Error(e.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("country", Some("region"), "city", None, None, "country/region/city")]
    #[case("country", None, "city", None, None, "country/country/city")]
    #[case(
        "country",
        Some("region"),
        "city",
        Some("24.01"),
        None,
        "country/region/city"
    )]
    #[case(
        "country",
        Some("region"),
        "city",
        None,
        Some(PathBuf::from("base/path")),
        "base/path/country/region/city"
    )]
    fn test_calver_base(
        #[case] country: &str,
        #[case] region: Option<&str>,
        #[case] city: &str,
        #[case] date_override: Option<&str>,
        #[case] base_dir: Option<PathBuf>,
        #[case] expected: &str,
    ) {
        let actual = calver_base::<PathBuf>(country, city, region, date_override, base_dir);
        let date = date_override.map_or_else(|| calver_utc_now().to_string(), |v| v.to_string());
        let expected = PathBuf::from(format!("{expected}/{date}"));
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(vec![PathBuf::from("country/region/city/22.01")], 1)]
    #[case(vec![PathBuf::from("country/region/city/22.01.6")], 7)]
    #[case(vec![PathBuf::from("country/region/city/22.02"), PathBuf::from("country/region/city/22.02.1"), PathBuf::from("country/region/city/22.02.2")], 3)]
    fn test_calver_next(#[case] dirs: Vec<PathBuf>, #[case] expected: u32) {
        let actual = calver_next(&dirs);
        assert_eq!(actual, expected)
    }
}
