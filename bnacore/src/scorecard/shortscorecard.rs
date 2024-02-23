use super::{scorecard21::ScoreCard21, scorecard23::ScoreCard23, ScorecardCsv};
use pyo3::prelude::*;
use serde::Serialize;

/// Represent a ScoreCard to be passed to `svggloo`.
///
/// The fields must match all the fields from ScoreCard, and be represented by
/// their short forms.
#[pyclass]
#[derive(Debug, Serialize, Clone)]
pub struct ShortScoreCard {
    /// City
    #[pyo3(get, set)]
    pub ci: String,
    #[pyo3(get, set)]
    pub co: String,
    #[pyo3(get, set)]
    pub st: String,
    #[pyo3(get, set)]
    pub uuid: String,
    #[pyo3(get, set)]
    pub po: u32,
    #[pyo3(get, set)]
    pub ra: f64,
    #[pyo3(get, set)]
    pub rasc: u8,

    // Community Survey
    #[pyo3(get, set)]
    pub nw: u8,
    #[pyo3(get, set)]
    pub aw: u8,
    #[pyo3(get, set)]
    pub sf: u8,
    #[pyo3(get, set)]
    pub rs: u8,
    #[pyo3(get, set)]
    pub total: u8,
    #[pyo3(get, set)]
    pub cssc: u8,
    #[pyo3(get, set)]
    pub responses: u32,

    // BNA
    #[pyo3(get, set)]
    pub nh: u8,
    #[pyo3(get, set)]
    pub op: u8,
    #[pyo3(get, set)]
    pub es: u8,
    #[pyo3(get, set)]
    pub ret: u8,
    #[pyo3(get, set)]
    pub rec: u8,
    #[pyo3(get, set)]
    pub tr: u8,
    #[pyo3(get, set)]
    pub bnasc: u8,

    // Infrastructure
    #[pyo3(get, set)]
    pub lsm: u32,
    #[pyo3(get, set)]
    pub hsm: u32,
}

impl From<&ScoreCard21> for ShortScoreCard {
    fn from(sc: &ScoreCard21) -> Self {
        ShortScoreCard {
            ci: sc.city.name.clone(),
            co: sc.city.country.clone(),
            st: sc.city.state.clone(),
            uuid: sc.city.uuid.clone(),
            po: sc.city.population,
            ra: sc.city.ratings,
            rasc: sc.city.ratings_rounded,
            nw: sc.community_survey.network.round() as u8,
            aw: sc.community_survey.awareness.round() as u8,
            sf: sc.community_survey.safety.round() as u8,
            rs: sc.community_survey.ridership.round() as u8,
            total: sc.community_survey.total.round() as u8,
            cssc: sc.community_survey.total_rounded as u8,
            responses: sc.community_survey.responses,
            nh: sc.bna.neighborhoods.round() as u8,
            op: sc.bna.opportunity.round() as u8,
            es: sc.bna.essential_services.unwrap_or_default().round() as u8,
            ret: sc.bna.retail.round() as u8,
            rec: sc.bna.recreation.unwrap_or_default().round() as u8,
            tr: sc.bna.transit.round() as u8,
            bnasc: sc.bna.overall_score.round() as u8,
            lsm: sc
                .infrastructure
                .low_stress_miles
                .unwrap_or_default()
                .round() as u32,
            hsm: sc
                .infrastructure
                .high_stress_miles
                .unwrap_or_default()
                .round() as u32,
        }
    }
}

impl From<&ScoreCard23> for ShortScoreCard {
    fn from(sc: &ScoreCard23) -> Self {
        ShortScoreCard {
            ci: sc.city.city.clone(),
            co: sc.city.country.clone(),
            st: sc.city.state.clone(),
            uuid: sc.bna.bna_uuid.clone(),
            po: sc.bna.census_population,
            ra: sc.bna.bna_overall_score,
            rasc: sc.bna.bna_rounded_score,
            nw: 0,
            aw: 0,
            sf: 0,
            rs: 0,
            total: 0,
            cssc: 0,
            responses: 0,
            nh: sc.bna.bna_people.round() as u8,
            op: sc.bna.bna_opportunity.round() as u8,
            es: sc.bna.bna_core_services.round() as u8,
            ret: sc.bna.bna_retail.round() as u8,
            rec: sc.bna.bna_recreation.round() as u8,
            tr: sc.bna.bna_transit.round() as u8,
            bnasc: sc.bna.bna_overall_score.round() as u8,
            lsm: sc.bna.bna_total_low_stress_miles.round() as u32,
            hsm: sc.bna.bna_total_high_stress_miles.round() as u32,
        }
    }
}

impl ScorecardCsv for ShortScoreCard {}

/// Define Python compatible methods.
#[pymethods]
impl ShortScoreCard {
    /// Python wrapper for the [`ShortScoreCard::to_csv`] method.
    #[staticmethod]
    pub fn save_csv(path: &str, entries: Vec<ShortScoreCard>) -> PyResult<()> {
        Ok(ShortScoreCard::to_csv(path, &entries)?)
    }
}
