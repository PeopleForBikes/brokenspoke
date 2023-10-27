use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalysisParameters {
    pub country: String,
    pub city: String,
    pub region: Option<String>,
    pub fips_code: Option<String>,
}

impl AnalysisParameters {
    pub fn new(
        country: String,
        city: String,
        region: Option<String>,
        fips_code: Option<String>,
    ) -> Self {
        Self {
            country,
            city,
            region,
            fips_code: match fips_code {
                Some(fips_code) => Some(fips_code),
                None => Some("0".to_string()),
            },
        }
    }
}
