use super::{
    scorecard21::ScoreCard21, scorecard23::ScoreCard23, scorecard24::ScoreCard24, ScorecardCsv,
};
use serde::Serialize;

/// Represent a ScoreCard to be passed to `svggloo`.
///
/// The fields must match all the fields from ScoreCard, and be represented by
/// their short forms.
#[derive(Debug, Serialize, Clone)]
pub struct ShortScoreCard {
    /// City
    pub ci: String,
    pub co: String,
    pub st: String,
    pub uuid: String,
    pub po: u32,
    pub ra: f64,
    pub rasc: u8,

    // Community Survey
    pub nw: u8,
    pub aw: u8,
    pub sf: u8,
    pub rs: u8,
    pub total: u8,
    pub cssc: u8,
    pub responses: u32,

    // BNA
    pub nh: u8,
    pub op: u8,
    pub es: u8,
    pub ret: u8,
    pub rec: u8,
    pub tr: u8,
    pub bnasc: u8,

    // Infrastructure
    pub lsm: u32,
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

impl From<&ScoreCard24> for ShortScoreCard {
    fn from(sc: &ScoreCard24) -> Self {
        ShortScoreCard {
            ci: sc.city.clone(),
            co: sc.country.clone(),
            // st: sc.state.expect("there must be a state").clone(),
            st: <std::option::Option<std::string::String> as Clone>::clone(&sc.state)
                .unwrap_or_default()
                .clone(),
            uuid: sc.bna_uuid.clone(),
            po: sc.census_population,
            ra: sc.bna_overall_score.unwrap_or_default(),
            rasc: sc.bna_rounded_score,
            nw: 0,
            aw: 0,
            sf: 0,
            rs: 0,
            total: 0,
            cssc: 0,
            responses: 0,
            nh: sc.bna_people.unwrap_or_default().round() as u8,
            op: sc.bna_opportunity.unwrap_or_default().round() as u8,
            es: sc.bna_core_services.unwrap_or_default().round() as u8,
            ret: sc.bna_retail.unwrap_or_default().round() as u8,
            rec: sc.bna_recreation.unwrap_or_default().round() as u8,
            tr: sc.bna_transit.unwrap_or_default().round() as u8,
            bnasc: sc.bna_overall_score.unwrap_or_default().round() as u8,
            lsm: sc.bna_total_low_stress_miles.unwrap_or_default().round() as u32,
            hsm: sc.bna_total_high_stress_miles.unwrap_or_default().round() as u32,
        }
    }
}

impl ScorecardCsv for ShortScoreCard {}
