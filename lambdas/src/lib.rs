use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalysisParameters {
    pub country: String,
    pub city: String,
    pub region: Option<String>,
    pub fips_code: Option<String>,
}

impl AnalysisParameters {
    /// Create a new AnalysisParameter object.
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
            fips_code: fips_code.or(Some("0".to_string())),
        }
    }

    /// Create a new simple AnalysisParameter object with only a city and a country.
    pub fn simple(country: String, city: String) -> Self {
        Self::new(country, city, None, None)
    }

    /// Create a new AnalysisParameter object with a city, a country, and a region.
    pub fn with_region(country: String, city: String, region: String) -> Self {
        Self::new(country, city, Some(region), None)
    }

    /// Create a new AnalysisParameter object with a city, a country, a region and a FIPS code.
    pub fn with_fips_code(
        country: String,
        city: String,
        region: String,
        fips_code: String,
    ) -> Self {
        Self::new(country, city, Some(region), Some(fips_code))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokenspokeState {
    Analysis,
    Export,
    Pipeline,
    Setup,
    SqsMessage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokenspokePipeline {
    pub state: Option<BrokenspokeState>,
    pub state_machine_id: Option<String>,
    pub sqs_message: Option<String>,
    pub neon_branch_id: Option<String>,
    pub fargate_task_id: Option<Uuid>,
    pub s3_bucket: Option<String>,
}
