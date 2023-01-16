//! Utility to seed the database using the City Ratings CSV file provided in the
//! assets directory.
//!
use bnacore::scorecard::ScoreCard;
use chrono::prelude::*;
use color_eyre::{eyre::Report, Result};
use dotenv::dotenv;
use entity::{bna, city, community_survey, infrastructure};
use once_cell::sync::OnceCell;
use sea_orm::{ActiveValue::Set, Database, DatabaseConnection, EntityTrait};

static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenv().ok();

    // Set the database connection.
    let database_url = dotenv::var("DATABASE_URL")?;
    let db = Database::connect(database_url).await?;
    DATABASE_CONNECTION.set(db).unwrap();
    let db = DATABASE_CONNECTION.get().unwrap();

    // Load the scorecards.
    let scorecards = ScoreCard::from_csv("assets/city_ratings_2021_v15.csv")?;

    // Populate entities.
    let mut bnas: Vec<bna::ActiveModel> = Vec::new();
    let mut cities: Vec<city::ActiveModel> = Vec::new();
    let mut community_surveys: Vec<community_survey::ActiveModel> = Vec::new();
    let mut infrastructures: Vec<infrastructure::ActiveModel> = Vec::new();
    for scorecard in scorecards {
        let entry_id = sea_orm::prelude::Uuid::parse_str(&scorecard.city.uuid)?;
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());

        let city_entry = city::ActiveModel {
            id: Set(entry_id),
            name: Set(scorecard.city.name),
            country: Set(scorecard.city.country),
            state: Set(scorecard.city.state),
            uuid: Set(entry_id),
            population: Set(scorecard.city.population.try_into().unwrap_or_default()),
            ratings: Set(scorecard.city.ratings),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let bna_entry = bna::ActiveModel {
            id: Set(entry_id),
            neighborhoods: Set(scorecard.bna.neighborhoods),
            opportunity: Set(scorecard.bna.opportunity),
            essential_services: Set(scorecard.bna.essential_services),
            retail: Set(scorecard.bna.retail),
            recreation: Set(scorecard.bna.recreation),
            transit: Set(scorecard.bna.transit),
            overall_score: Set(scorecard.bna.overall_score),
            city_id: Set(Some(entry_id)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let cs_entry = community_survey::ActiveModel {
            id: Set(entry_id),
            network: Set(scorecard.community_survey.network),
            awareness: Set(scorecard.community_survey.awareness),
            safety: Set(scorecard.community_survey.safety),
            ridership: Set(scorecard.community_survey.ridership),
            total: Set(scorecard.community_survey.total),
            responses: Set(scorecard
                .community_survey
                .responses
                .try_into()
                .unwrap_or_default()),
            city_id: Set(Some(entry_id)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let infra_entry = infrastructure::ActiveModel {
            id: Set(entry_id),
            low_stress_miles: Set(scorecard.infrastructure.low_stress_miles),
            high_stress_miles: Set(scorecard.infrastructure.high_stress_miles),
            city_id: Set(Some(entry_id)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        bnas.push(bna_entry);
        cities.push(city_entry);
        community_surveys.push(cs_entry);
        infrastructures.push(infra_entry);
    }

    // Insert the entries.
    entity::city::Entity::insert_many(cities).exec(db).await?;
    entity::bna::Entity::insert_many(bnas).exec(db).await?;
    entity::community_survey::Entity::insert_many(community_surveys)
        .exec(db)
        .await?;
    entity::infrastructure::Entity::insert_many(infrastructures)
        .exec(db)
        .await?;

    Ok(())
}
