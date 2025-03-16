use super::{Scorecard, ScorecardCsv, Size};
use crate::{Dataset, Error, PFB_S3_PUBLIC_DOCUMENTS, PFB_S3_STORAGE_BASE_URL};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScoreCard24 {
    pub city: String,
    pub state: Option<String>,
    pub state_full: String,
    pub country: String,
    pub region: String,
    pub year: u32,
    pub census_fips_code: Option<u32>,
    pub census_population: u32,
    pub census_latitude: f64,
    pub census_longitude: f64,
    pub residential_speed_limit: Option<u8>,
    pub bna_id: String,
    pub bna_uuid: String,
    pub bna_people: Option<f64>,
    pub bna_opportunity_employment: Option<f64>,
    pub bna_opportunity_k12_education: Option<f64>,
    pub bna_opportunity_technical_vocational_college: Option<f64>,
    pub bna_opportunity_higher_education: Option<f64>,
    pub bna_opportunity: Option<f64>,
    pub bna_core_services_doctors: Option<f64>,
    pub bna_core_services_dentists: Option<f64>,
    pub bna_core_services_hospitals: Option<f64>,
    pub bna_core_services_pharmacies: Option<f64>,
    pub bna_core_services_grocery: Option<f64>,
    pub bna_core_services_social_services: Option<f64>,
    pub bna_core_services: Option<f64>,
    pub bna_recreation_community_centers: Option<f64>,
    pub bna_recreation_parks: Option<f64>,
    pub bna_recreation_trails: Option<f64>,
    pub bna_recreation: Option<f64>,
    pub bna_retail: Option<f64>,
    pub bna_transit: Option<f64>,
    pub bna_overall_score: Option<f64>,
    pub bna_rounded_score: u8,
    pub bna_total_low_stress_miles: Option<f64>,
    pub bna_total_high_stress_miles: Option<f64>,
    pub pop_size: Option<Size>,
    #[serde(with = "time::serde::iso8601")]
    pub creation_date: OffsetDateTime,
    pub filename: String,
}

impl ScorecardCsv for ScoreCard24 {}

impl Scorecard for ScoreCard24 {
    fn full_name(&self) -> String {
        format!("{}-{}-{}", self.country, self.state_full, self.city)
    }

    fn url(&self, dataset: &Dataset) -> Result<Url, Error> {
        let mut dataset_url: String = String::new();
        if *dataset == Dataset::DataDictionary {
            dataset_url.push_str(PFB_S3_PUBLIC_DOCUMENTS);
        } else {
            dataset_url.push_str(PFB_S3_STORAGE_BASE_URL);
            dataset_url.push('/');
            dataset_url.push_str(&self.bna_uuid);
        }
        dataset_url.push('/');
        dataset_url.push_str(&dataset.to_string());
        dataset_url.push('.');
        dataset_url.push_str(&dataset.extension());
        Ok(Url::parse(&dataset_url)?)
    }

    fn version(&self) -> String {
        extract_version_from_filename(&self.filename)
    }
}

impl ScoreCard24 {}

/// Extract the version number from the scorecard filename.
fn extract_version_from_filename(filename: &str) -> String {
    let mut parts = filename.split('_');
    let version_part = parts.next_back().unwrap();
    let v_version = version_part.replace(".csv", "");
    let version = v_version.replace('v', "");
    let version_parts = version
        .split('.')
        .map(|p| p.parse::<i32>().unwrap())
        .collect::<Vec<i32>>();
    let version_major = version_parts[0];
    let version_minor = version_parts[1];
    format!("{version_major}.{:0>2}", version_minor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_extract_version_from_filename() {
        assert_eq!(
            extract_version_from_filename("_Christchurch_v23.1.csv"),
            "23.01"
        );
    }

    #[test]
    fn test_scorecard24() {
        let sc = ScoreCard24 {
            city: "Yarra LGA".to_string(),
            state: Some("VIC".to_string()),
            state_full: "Victoria".to_string(),
            country: "Australia".to_string(),
            region: "Australia".to_string(),
            year: 2023,
            census_fips_code: Some(9900038),
            census_population: 90114,
            census_latitude: -37.8031,
            census_longitude: 144.9852,
            residential_speed_limit: Some(25),
            bna_id: "2b503949-a5e0-4238-95ea-15f022f68f71".to_string(),
            bna_uuid: "c18cf23b-af10-4f4a-81bc-76780bb13425".to_string(),
            bna_people: Some(35.84),
            bna_opportunity_employment: Some(0.0),
            bna_opportunity_k12_education: Some(67.73),
            bna_opportunity_technical_vocational_college: Some(48.29),
            bna_opportunity_higher_education: Some(43.82),
            bna_opportunity: Some(57.38),
            bna_core_services_doctors: Some(58.71),
            bna_core_services_dentists: Some(51.29),
            bna_core_services_hospitals: Some(57.92),
            bna_core_services_pharmacies: Some(63.84),
            bna_core_services_grocery: Some(74.77),
            bna_core_services_social_services: Some(51.21),
            bna_core_services: Some(61.21),
            bna_recreation: Some(71.1),
            bna_retail: None,
            bna_transit: Some(61.32),
            bna_overall_score: Some(59.91),
            bna_rounded_score: 60,
            bna_total_low_stress_miles: Some(292.9),
            bna_total_high_stress_miles: Some(95.0),
            pop_size: Some(Size::Medium),
            creation_date: datetime!(2023-04-14 14:26:00 UTC),
            filename: "VIC_Yarra LGA_v23.1.csv".to_string(),
            bna_recreation_community_centers: None,
            bna_recreation_parks: None,
            bna_recreation_trails: None,
        };

        assert_eq!(sc.state, Some("VIC".to_string()));
    }

    #[test]
    fn test_deserialize() {
        let raw_json = r#"
          {
            "city": "Christchurch",
            "state": "CAN",
            "state_full": "Canterbury",
            "country": "New Zealand",
            "region": "New Zealand",
            "year": 2023,
            "census_fips_code": 9900246,
            "census_population": 389300,
            "census_latitude": -43.532,
            "census_longitude": 172.6306,
            "residential_speed_limit": 25,
            "bna_id": "ae2250a5-9c90-4132-929d-63640c23d1c5",
            "bna_uuid": "9ac2465b-04f7-48a0-adc6-92502243b6e3",
            "bna_people": 48.26,
            "bna_opportunity_employment": 0,
            "bna_opportunity_k12_education": 68.7,
            "bna_opportunity_technical_vocational_college": 30.96,
            "bna_opportunity_higher_education": 43.18,
            "bna_opportunity": 55.04,
            "bna_core_services_doctors": 53.97,
            "bna_core_services_dentists": 46.27,
            "bna_core_services_hospitals": 47.93,
            "bna_core_services_pharmacies": 52.67,
            "bna_core_services_grocery": 64.39,
            "bna_core_services_social_services": 60.03,
            "bna_core_services": 55.38,
            "bna_recreation": 59.48,
            "bna_transit": 31.21,
            "bna_overall_score": 52.97,
            "bna_rounded_score": 53,
            "bna_total_low_stress_miles": 1452.4,
            "bna_total_high_stress_miles": 1929.7,
            "pop_size": "large",
            "creation_date": "2023-04-17T16:31:00Z",
            "filename": "_Christchurch_v23.1.csv"
          }"#;
        let deserialized = serde_json::from_str::<ScoreCard24>(raw_json).unwrap();
        assert_eq!(deserialized.state, Some("CAN".to_string()));
    }
}
