use axum::{
    extract::Path,
    http::{HeaderValue, Method, StatusCode},
    response::Html,
    routing::{get, get_service, post},
    Json, Router, Server,
};
use bnacore::{scorecard::ShortScoreCard, template::render_record};
use color_eyre::{eyre::eyre, eyre::Report, Result};
use dotenv::dotenv;
use entity::{bna, city, community_survey, infrastructure};
use nats::asynk::Connection as NatsConnection;
use once_cell::sync::OnceCell;
use sea_orm::{
    error::DbErr, prelude::Uuid, ColumnTrait, Database, DatabaseBackend, DatabaseConnection,
    EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Statement,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};

static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::new();
static NATS_CONNECTION: OnceCell<NatsConnection> = OnceCell::new();

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CityRating {
    // city
    pub id: Uuid,
    pub name: String,
    pub country: String,
    pub state: String,
    pub uuid: Uuid,
    pub population: i32,
    pub ratings: f64,
    // bna
    pub neighborhoods: f64,
    pub opportunity: f64,
    pub essential_services: Option<f64>,
    pub retail: f64,
    pub recreation: Option<f64>,
    pub transit: f64,
    pub overall_score: f64,
    // cs
    pub network: f64,
    pub awareness: f64,
    pub safety: f64,
    pub ridership: f64,
    pub total: f64,
    pub responses: i32,
    // infra
    pub low_stress_miles: Option<f64>,
    pub high_stress_miles: Option<f64>,
}

impl CityRating {
    pub fn new(
        city: city::Model,
        bna: bna::Model,
        infra: infrastructure::Model,
        cs: community_survey::Model,
    ) -> Self {
        CityRating {
            id: city.id,
            name: city.name,
            country: city.country,
            state: city.state,
            uuid: city.uuid,
            population: city.population,
            ratings: city.ratings,
            neighborhoods: bna.neighborhoods,
            opportunity: bna.opportunity,
            essential_services: bna.essential_services,
            retail: bna.retail,
            recreation: bna.recreation,
            transit: bna.transit,
            overall_score: bna.overall_score,
            network: cs.network,
            awareness: cs.awareness,
            safety: cs.safety,
            ridership: cs.ridership,
            total: cs.total,
            responses: cs.responses,
            low_stress_miles: infra.low_stress_miles,
            high_stress_miles: infra.high_stress_miles,
        }
    }

    pub fn from_schortscorecard(sc: ShortScoreCard) -> Self {
        CityRating {
            id: Uuid::parse_str(&sc.uuid).unwrap(),
            name: sc.ci,
            country: sc.co,
            state: sc.st,
            uuid: Uuid::parse_str(&sc.uuid).unwrap(),
            population: sc.po.try_into().unwrap(),
            ratings: sc.ra,
            neighborhoods: sc.nh.into(),
            opportunity: sc.op.try_into().unwrap(),
            essential_services: Some(sc.es.try_into().unwrap()),
            retail: sc.ret.try_into().unwrap(),
            recreation: Some(sc.rec.try_into().unwrap()),
            transit: sc.tr.try_into().unwrap(),
            overall_score: sc.bnasc.try_into().unwrap(),
            network: sc.nw.try_into().unwrap(),
            awareness: sc.aw.try_into().unwrap(),
            safety: sc.sf.try_into().unwrap(),
            ridership: sc.rs.try_into().unwrap(),
            total: sc.cssc.try_into().unwrap(),
            responses: sc.responses.try_into().unwrap(),
            low_stress_miles: Some(sc.lsm.try_into().unwrap()),
            high_stress_miles: Some(sc.hsm.try_into().unwrap()),
        }
    }

    fn to_schortscorecard(&self) -> ShortScoreCard {
        ShortScoreCard {
            ci: self.name.clone(),
            co: self.country.clone(),
            st: self.state.clone(),
            uuid: self.uuid.to_string(),
            po: self.population.try_into().unwrap(),
            ra: self.ratings,
            rasc: self.ratings as u8,
            nw: self.network as u8,
            aw: self.awareness as u8,
            sf: self.safety as u8,
            rs: self.ridership as u8,
            total: self.total as u8,
            cssc: self.total as u8,
            responses: self.responses.try_into().unwrap(),
            nh: self.neighborhoods as u8,
            op: self.opportunity as u8,
            es: self.essential_services.unwrap_or_default() as u8,
            ret: self.retail as u8,
            rec: self.recreation.unwrap_or_default() as u8,
            tr: self.transit as u8,
            bnasc: self.overall_score as u8,
            lsm: self.low_stress_miles.unwrap_or_default() as u32,
            hsm: self.high_stress_miles.unwrap_or_default() as u32,
        }
    }
}

impl From<ShortScoreCard> for CityRating {
    fn from(sc: ShortScoreCard) -> Self {
        CityRating {
            id: Uuid::parse_str(&sc.uuid).unwrap(),
            name: sc.ci,
            country: sc.co,
            state: sc.st,
            uuid: Uuid::parse_str(&sc.uuid).unwrap(),
            population: sc.po.try_into().unwrap(),
            ratings: sc.ra,
            neighborhoods: sc.nh.into(),
            opportunity: sc.op.try_into().unwrap(),
            essential_services: Some(sc.es.try_into().unwrap()),
            retail: sc.ret.try_into().unwrap(),
            recreation: Some(sc.rec.try_into().unwrap()),
            transit: sc.tr.try_into().unwrap(),
            overall_score: sc.bnasc.try_into().unwrap(),
            network: sc.nw.try_into().unwrap(),
            awareness: sc.aw.try_into().unwrap(),
            safety: sc.sf.try_into().unwrap(),
            ridership: sc.rs.try_into().unwrap(),
            total: sc.cssc.try_into().unwrap(),
            responses: sc.responses.try_into().unwrap(),
            low_stress_miles: Some(sc.lsm.try_into().unwrap()),
            high_stress_miles: Some(sc.hsm.try_into().unwrap()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup the application.
    color_eyre::install()?;

    // Setup logging.
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Read the .env file if any.
    dotenv().ok();

    // Set the database connection.
    let db_connection_string = dotenv::var("DATABASE_URL")?;
    // let db_url = Url::parse(&db_connection_string)?;
    let db = Database::connect(db_connection_string).await?;
    DATABASE_CONNECTION.set(db).unwrap();
    // let db = DATABASE_CONNECTION.get().unwrap();

    // Set the messaging connection.
    let nc_connection_string = dotenv::var("NATS_URL")?;
    let nc = nats::asynk::connect(nc_connection_string).await?;
    NATS_CONNECTION.set(nc).unwrap();

    // Create the routes.
    let app = Router::new()
        .route("/cities", get(get_cities))
        .route("/city/:city_id", get(get_city))
        .route("/ratings", get(get_city_ratings))
        .route("/rating/:city_id", get(get_city_rating))
        .route("/fastratings", get(get_fast_city_ratings))
        .route("/brochure/:city_id", get(get_brochure))
        .route("/task/analysis", post(schedule_task))
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            // .handle_error(|error: std::io::Error| async move {
            //     (
            //         StatusCode::INTERNAL_SERVER_ERROR,
            //         format!("Unhandled internal error: {}", error),
            //     )
            // }),
        )
        .layer(ServiceBuilder::new().layer(CookieManagerLayer::new()))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods(vec![Method::GET, Method::POST]),
        );

    // Serve the route.
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

pub async fn get_cities() -> (StatusCode, Json<Value>) {
    let db = DATABASE_CONNECTION.get().unwrap();
    match city::Entity::find().all(db).await {
        Ok(city_models) => (StatusCode::ACCEPTED, Json(json!(city_models))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(vec![error.to_string()])),
        ),
    }
}

pub async fn get_city(Path(city_id): Path<Uuid>) -> (StatusCode, Json<Value>) {
    let db = DATABASE_CONNECTION.get().unwrap();
    let result = city::Entity::find_by_id(city_id).one(db).await;

    match result {
        Ok(city) => {
            if let Some(c) = city {
                (StatusCode::ACCEPTED, Json(json!(c)))
            } else {
                (StatusCode::ACCEPTED, Json(json!("{}")))
            }
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(vec![error.to_string()])),
        ),
    }
}

async fn get_brochure(Path(city_id): Path<Uuid>) -> (StatusCode, Html<String>) {
    // Lookup for the requested entry.
    let rating = match find_city_rating(city_id).await {
        Ok(r) => r,
        Err(error) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(error.to_string())),
    };
    let shortsc: ShortScoreCard = rating.to_schortscorecard();

    match render_record(
        std::path::Path::new("assets/brochure.svg")
            .as_os_str()
            .to_str()
            .unwrap(),
        shortsc,
    ) {
        Ok(r) => (StatusCode::OK, Html(r)),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Html(format!("cannot render the brochure: {}", e)),
        ),
    }
}

#[derive(FromQueryResult, Debug, Serialize, Deserialize)]
struct CityId {
    id: Uuid,
}

async fn get_city_ratings() -> (StatusCode, Json<Value>) {
    let db = DATABASE_CONNECTION.get().unwrap();
    let city_id_results = city::Entity::find()
        .select_only()
        .column(city::Column::Id)
        .into_model::<CityId>()
        .all(db)
        .await;

    let city_ids = match city_id_results {
        Ok(c) => c,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(vec![error.to_string()])),
            )
        }
    };

    let mut ratings: Vec<CityRating> = Vec::new();
    for city_id in city_ids {
        match find_city_rating(city_id.id).await {
            Ok(rating) => ratings.push(rating),
            Err(error) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!(vec![error.to_string()])),
                )
            }
        }
    }

    (StatusCode::ACCEPTED, Json(json!(ratings)))
}

async fn get_city_rating(Path(city_id): Path<Uuid>) -> (StatusCode, Json<Value>) {
    match find_city_rating(city_id).await {
        Ok(cr) => (StatusCode::ACCEPTED, Json(json!(cr))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(vec![error.to_string()])),
        ),
    }
}

async fn find_city_rating(city_id: Uuid) -> Result<CityRating, Report> {
    let city = match find_city_by_id(city_id).await? {
        Some(model) => model,
        None => return Err(eyre!("cannot find a city with the id \"{city_id}\"")),
    };

    let bna = match find_latest_bna_by_city_id(city_id).await? {
        Some(model) => model,
        None => {
            return Err(eyre!(
                "cannot find BNA results for a city with the id \"{city_id}\""
            ))
        }
    };

    let cs = match find_latest_community_survey_by_city_id(city_id).await? {
        Some(model) => model,
        None => {
            return Err(eyre!(
                "cannot find community survey results for a city with the id \"{city_id}\""
            ))
        }
    };

    let infra = match find_latest_infrastructure_by_city_id(city_id).await? {
        Some(model) => model,
        None => {
            return Err(eyre!(
                "cannot find community survey results for a city with the id \"{city_id}\""
            ))
        }
    };

    Ok(CityRating::new(city, bna, infra, cs))
}

async fn find_city_by_id(city_id: Uuid) -> Result<Option<city::Model>, DbErr> {
    let db = DATABASE_CONNECTION.get().unwrap();
    city::Entity::find_by_id(city_id).one(db).await
}

async fn find_latest_bna_by_city_id(city_id: Uuid) -> Result<Option<bna::Model>, DbErr> {
    let db = DATABASE_CONNECTION.get().unwrap();
    bna::Entity::find()
        .filter(bna::Column::CityId.eq(city_id))
        .order_by_desc(bna::Column::CreatedAt)
        .one(db)
        .await
}

async fn find_latest_community_survey_by_city_id(
    city_id: Uuid,
) -> Result<Option<community_survey::Model>, DbErr> {
    let db = DATABASE_CONNECTION.get().unwrap();
    community_survey::Entity::find()
        .filter(community_survey::Column::CityId.eq(city_id))
        .order_by_desc(community_survey::Column::CreatedAt)
        .one(db)
        .await
}

async fn find_latest_infrastructure_by_city_id(
    city_id: Uuid,
) -> Result<Option<infrastructure::Model>, DbErr> {
    let db = DATABASE_CONNECTION.get().unwrap();
    infrastructure::Entity::find()
        .filter(infrastructure::Column::CityId.eq(city_id))
        .order_by_desc(infrastructure::Column::CreatedAt)
        .one(db)
        .await
}

async fn schedule_task(Json(city): Json<city::Model>) -> (StatusCode, String) {
    dbg!(&city);
    let nc = NATS_CONNECTION.get().unwrap();
    let city_name = city.clone().name;
    let message = match serde_json::to_string(&city) {
        Ok(city_json) => city_json,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Cannot enqueue {city_name}: {e}"),
            )
        }
    };
    dbg!(&message);
    match nc.publish("analyze.city", &message).await {
        Ok(_r) => (
            StatusCode::OK,
            format!("{{\"action\": \"Enqueuing {city_name} for analysis...\"}}"),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Cannot enqueue {city_name}: {e}"),
        ),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromQueryResult)]
pub struct CityRatingSimplified {
    // city
    pub name: String,
    pub country: String,
    pub state: String,
    pub population: i32,
    // bna
    pub neighborhoods: f64,
    pub opportunity: f64,
    pub essential_services: Option<f64>,
    pub retail: f64,
    pub recreation: Option<f64>,
    pub transit: f64,
    pub overall_score: f64,
    // cs
    pub network: f64,
    pub awareness: f64,
    pub safety: f64,
    pub ridership: f64,
    pub total: f64,
    pub responses: i32,
    // infra
    pub low_stress_miles: Option<f64>,
    pub high_stress_miles: Option<f64>,
}

async fn find_city_ratings() -> Result<Vec<CityRatingSimplified>, DbErr> {
    let query = r###"
    -- Select the latest bna analysis per city, with details.
    WITH latest_bna AS (
        SELECT bna.*
        FROM
            (SELECT
                city_id,
                max(created_at) AS created_at
                FROM bna
                GROUP BY city_id
            ) AS newest_bna
        INNER JOIN
            bna
            ON
                bna.city_id = newest_bna.city_id
                AND bna.created_at = newest_bna.created_at
    ),
    latest_infrastructure AS (
        SELECT infrastructure.*
        FROM
            (SELECT
                city_id,
                max(created_at) AS created_at
                FROM infrastructure
                GROUP BY city_id
            ) AS newest_infrastructure
        INNER JOIN
            infrastructure
            ON
                infrastructure.city_id = newest_infrastructure.city_id
                AND infrastructure.created_at = newest_infrastructure.created_at
    ),
    latest_community_survey AS (
        SELECT community_survey.*
        FROM
            (SELECT
                city_id,
                max(created_at) AS created_at
                FROM community_survey
                GROUP BY city_id
            ) AS newest_community_survey
        INNER JOIN
            community_survey
            ON
                community_survey.city_id = newest_community_survey.city_id
                AND community_survey.created_at = community_survey.created_at
    )
    SELECT
        city.name,
        city.state,
        city.country,
        city.population,
        latest_bna.neighborhoods,
        latest_bna.opportunity,
        latest_bna.essential_services,
        latest_bna.retail,
        latest_bna.recreation,
        latest_bna.transit,
        latest_bna.overall_score,
        latest_infrastructure.low_stress_miles,
        latest_infrastructure.high_stress_miles,
        latest_community_survey.network,
        latest_community_survey.awareness,
        latest_community_survey.safety,
        latest_community_survey.ridership,
        latest_community_survey.total,
        latest_community_survey.responses
    FROM city
    -- Join with BNA analysis
    INNER JOIN latest_bna
        ON city.id = latest_bna.city_id
    -- join with infrastructure analysis
    INNER JOIN latest_infrastructure
        ON city.id = latest_infrastructure.city_id
    -- join with community survey
    INNER JOIN latest_community_survey
        ON city.id = latest_community_survey.city_id;
  "###;
    let db = DATABASE_CONNECTION.get().unwrap();

    CityRatingSimplified::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        query,
        vec![],
    ))
    .all(db)
    .await
}

async fn get_fast_city_ratings() -> (StatusCode, Json<Value>) {
    match find_city_ratings().await {
        Ok(r) => (StatusCode::ACCEPTED, Json(json!(r))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(vec![error.to_string()])),
        ),
    }
}
