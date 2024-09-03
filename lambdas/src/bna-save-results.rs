use aws_config::BehaviorVersion;
use aws_smithy_types_convert::date_time::DateTimeExt;
use bnacore::aws::get_aws_parameter_value;
use bnalambdas::{
    authenticate_service_account, update_pipeline, AnalysisParameters, BNAPipeline, Context,
    Fargate, AWSS3,
};
use csv::ReaderBuilder;
use heck::ToTitleCase;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::{collections::HashMap, io::Write};
use time::OffsetDateTime;
use tracing::info;
use uuid::Uuid;

const OVERALL_SCORES_COUNT: usize = 23;

#[derive(Deserialize)]
struct TaskInput {
    analysis_parameters: AnalysisParameters,
    aws_s3: AWSS3,
    context: Context,
    fargate: Fargate,
}

#[derive(Deserialize, Clone)]
struct OverallScore {
    pub score_id: String,
    pub score_normalized: Option<f64>,
}

#[derive(Deserialize)]
struct OverallScores(HashMap<String, OverallScore>);

impl OverallScores {
    /// Create an empty OverallScores.
    pub fn new() -> Self {
        OverallScores(HashMap::with_capacity(OVERALL_SCORES_COUNT))
    }

    /// Retrieve an OverallScore item by id.
    fn get_overall_score(&self, score_id: &str) -> Option<OverallScore> {
        self.0.get(score_id).cloned()
    }

    /// Retrieve the normalized score of an OverallScore item by id.
    fn get_normalized_score(&self, score_id: &str) -> Option<f64> {
        self.get_overall_score(score_id)
            .and_then(|s| s.score_normalized)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNASummary {
    pub bna_uuid: Uuid,
    pub version: String,
    pub city_id: Uuid,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAInfrastructure {
    pub low_stress_miles: Option<f64>,
    pub high_stress_miles: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNARecreation {
    pub community_centers: Option<f64>,
    pub parks: Option<f64>,
    pub recreation_trails: Option<f64>,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAOpportunity {
    pub employment: Option<f64>,
    pub higher_education: Option<f64>,
    pub k12_education: Option<f64>,
    pub technical_vocational_college: Option<f64>,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNACoreServices {
    pub dentists: Option<f64>,
    pub doctors: Option<f64>,
    pub grocery: Option<f64>,
    pub hospitals: Option<f64>,
    pub pharmacies: Option<f64>,
    pub social_services: Option<f64>,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAPeople {
    pub score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNARetail {
    pub score: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNATransit {
    pub score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAPost {
    pub core_services: BNACoreServices,
    pub people: BNAPeople,
    pub retail: BNARetail,
    pub transit: BNATransit,
    pub infrastructure: BNAInfrastructure,
    pub opportunity: BNAOpportunity,
    pub recreation: BNARecreation,
    pub summary: BNASummary,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct City {
    pub id: Option<Uuid>,
    pub country: String,
    pub state: String,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub region: Option<String>,
    pub state_abbrev: Option<String>,
    pub speed_limit: Option<i32>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<(), Error> {
    // Read the task inputs.
    info!("Reading input...");
    let analysis_parameters = &event.payload.analysis_parameters;
    let aws_s3 = &event.payload.aws_s3;
    let state_machine_context = &event.payload.context;
    let state_machine_id = state_machine_context.id;
    let fargate = &event.payload.fargate;

    info!("Retrieve secrets and parameters...");
    // Retrieve API hostname.
    let api_hostname = get_aws_parameter_value("BNA_API_HOSTNAME").await?;

    // Retrieve bna_bucket name.
    let bna_bucket = get_aws_parameter_value("BNA_BUCKET").await?;

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Prepare the AWS configuration.
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    // Configure the S3 client.
    info!("Configure the S3 client...");
    let s3_client = aws_sdk_s3::Client::new(&config);

    // Configure the ECS client.
    info!("Configure the ECS client...");
    let ecs_client = aws_sdk_ecs::Client::new(&config);

    // Download the CSV file with the results.
    let scores_csv = format!(
        "{}/neighborhood_overall_scores.csv",
        aws_s3.destination.clone()
    );
    info!(
        "Download the CSV file with the results from {}...",
        scores_csv
    );
    let buffer = fetch_s3_object_as_bytes(&s3_client, &bna_bucket, &scores_csv).await?;

    // Parse the results.
    info!("Parse the results...");
    let overall_scores = parse_overall_scores(buffer.as_slice())?;

    // Query city.
    info!("Check for existing city...");
    let country = &analysis_parameters.country;
    let region = match &analysis_parameters.region {
        Some(region) => region.to_owned(),
        None => country.to_owned(),
    };
    let name = &analysis_parameters.city;
    let cities_url = format!("{api_hostname}/cities");
    let get_cities_url = format!(
        "{cities_url}/{}/{}/{}",
        country.to_title_case(),
        region.to_title_case(),
        name.to_title_case()
    );
    info!("endpoint: {}", get_cities_url);
    let client = Client::new();
    let r = client.get(&get_cities_url).send()?;
    let city: Option<City> = match r.status().as_u16() {
        x if x < 400 => Some(r.json::<City>()?),
        404 => None,
        _ => {
            return Err(Box::new(SimpleError::new(format!(
                "cannot retrieve city at {get_cities_url}: {} {}",
                r.status(),
                r.status().as_str()
            ))))
        }
    };
    info!("City: {:#?}", city);

    // Create city if it does not exist and save the city_id.
    // Otherwise save the city_id and update the population..
    let city_id: Uuid;
    if let Some(city) = city {
        info!("The city exists, update the population...");
        city_id = city.id.unwrap();
        // TODO: Patch city census.
    } else {
        info!("Create a new city...");
        // Create the city.
        let c = City {
            country: country.clone().to_title_case(),
            state: region.clone().to_title_case(),
            name: name.clone().to_title_case(),
            ..Default::default()
        };
        let city = client
            .post(cities_url)
            .bearer_auth(auth.access_token.clone())
            .json(&c)
            .send()?
            .error_for_status()?
            .json::<City>()?;
        city_id = city.id.unwrap();
    }

    // Convert the overall scores to a BNAPost struct.
    let version = aws_s3.get_version();
    let bna_post = scores_to_bnapost(overall_scores, version, city_id);

    // Prepare API URLs.
    let bnas_url = format!("{api_hostname}/ratings");

    // Post a new entry via the API.
    info!("Post a new BNA entry via the API...");
    info!("New entry: {:?}", &bna_post);
    Client::new()
        .post(&bnas_url)
        .bearer_auth(auth.access_token.clone())
        .json(&bna_post)
        .send()?
        .error_for_status()?;

    // Compute the time it took to run the fargate task.
    let describe_tasks = ecs_client
        .describe_tasks()
        .tasks(fargate.task_arn.clone())
        .send()
        .await?;
    let task_info = describe_tasks.tasks().first().unwrap();
    let started_at = task_info
        .started_at()
        .expect("the task must have started at this point");
    let stopped_at = task_info
        .started_at()
        .expect("the task must have stopped at this point");
    let started_secs = started_at.secs();
    let stopped_secs = stopped_at.secs();
    let elapsed = (stopped_secs - started_secs) / 1000;

    // Compute the price.
    const FARGATE_COST_PER_SEC: Decimal = dec!(0.00228333333333);
    let elapsed_decimal: Decimal = elapsed.into();
    let cost = elapsed_decimal.checked_mul(FARGATE_COST_PER_SEC);

    // TODO(rgreinho): Update the pipeline status when the new state will be available.
    // Update the pipeline status.
    info!("updating pipeline...");
    let patch_url = format!("{bnas_url}/analysis/{state_machine_id}");
    let start_time = started_at
        .to_time()
        .expect("a valid start time is expected");
    let end_time = stopped_at.to_time().ok();
    let pipeline = BNAPipeline {
        cost,
        end_time,
        start_time,
        state_machine_id,
        step: Some("Setup".to_string()),
        ..Default::default()
    };
    update_pipeline(&patch_url, &auth, &pipeline)?;

    Ok(())
}

fn parse_overall_scores(data: &[u8]) -> Result<OverallScores, Error> {
    let mut overall_scores = OverallScores::new();
    let mut rdr = ReaderBuilder::new().flexible(true).from_reader(data);
    for result in rdr.deserialize() {
        let score: OverallScore = result?;
        overall_scores.0.insert(score.score_id.clone(), score);
    }
    Ok(overall_scores)
}

fn scores_to_bnapost(overall_scores: OverallScores, version: String, city_id: Uuid) -> BNAPost {
    BNAPost {
        core_services: BNACoreServices {
            dentists: overall_scores.get_normalized_score("core_services_dentists"),
            doctors: overall_scores.get_normalized_score("core_services_doctors"),
            grocery: overall_scores.get_normalized_score("core_services_grocery"),
            hospitals: overall_scores.get_normalized_score("core_services_hospitals"),
            pharmacies: overall_scores.get_normalized_score("core_services_pharmacies"),
            social_services: overall_scores.get_normalized_score("core_services_social_services"),
            score: overall_scores
                .get_normalized_score("core_services")
                .unwrap_or_default(),
        },
        people: BNAPeople {
            score: overall_scores.get_normalized_score("people"),
        },
        retail: BNARetail {
            score: overall_scores.get_normalized_score("retail"),
        },
        transit: BNATransit {
            score: overall_scores.get_normalized_score("transit"),
        },
        infrastructure: BNAInfrastructure {
            low_stress_miles: overall_scores.get_normalized_score("total_miles_low_stress"),
            high_stress_miles: overall_scores.get_normalized_score("total_miles_high_stress"),
        },
        opportunity: BNAOpportunity {
            employment: overall_scores.get_normalized_score("opportunity_employment"),
            higher_education: overall_scores.get_normalized_score("opportunity_higher_education"),
            k12_education: overall_scores.get_normalized_score("opportunity_k12_education"),
            technical_vocational_college: overall_scores
                .get_normalized_score("opportunity_technical_vocational_college"),
            score: overall_scores
                .get_normalized_score("opportunity")
                .unwrap_or_default(),
        },
        recreation: BNARecreation {
            community_centers: overall_scores.get_normalized_score("recreation_community_centers"),
            parks: overall_scores.get_normalized_score("recreation_parks"),
            recreation_trails: overall_scores.get_normalized_score("recreation_trails"),
            score: overall_scores
                .get_normalized_score("recreation_trails")
                .unwrap_or_default(),
        },
        summary: BNASummary {
            bna_uuid: Uuid::new_v4(),
            version,
            city_id,
            score: 0.0,
        },
    }
}

async fn fetch_s3_object_as_bytes(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> std::io::Result<Vec<u8>> {
    let mut object = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("AWS S3 operation error: {e}"),
            )
        })?;
    let mut buffer: Vec<u8> = Vec::new();
    while let Some(bytes) = object.body.try_next().await? {
        buffer.write_all(&bytes)?;
    }
    Ok(buffer)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await.map_err(|e| {
        info!("{e}");
        e
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    // use bnalambdas::AuthResponse;

    #[test]
    fn test_input_deserialization() {
        let json_input = r#"{
          "analysis_parameters": {
            "country": "usa",
            "city": "santa rosa",
            "region": "new mexico",
            "fips_code": "3570670"
          },
          "receipt_handle": "AQEBFo+wTTIZdCvaF2KtZN4ZolAGKeKVGSAhQ7BTA9MUirBT/8mprrHIOg8LuWi3LK9Lu1oFDd5GqVmzExGeHlVbmRA3HWd+vy11b1N4qVeHywvUJJT5/G/GVG2jimkHDa31893N0k2HIm2USSsN6Bqw0JI57ac0ymUWJxzkN9/yJQQXg2dmnNn3YuouzQTGpOJnMjv9UnZaHGVjZXV30IWjs9VzUZd9Wnl721B99pF9t1FUeYnAxShtNUZKzbfbNmSmwtKoE+SwohFL0k84cYkJUjgdXw9yEoT2+zEqeGWtU/oSGmbLorPWIiVYubPcwni1Q9KZROUDvBX7sPDwUeYxxhw9SBxz3y4Tg5hH7X99D4tDXbnRJR1v/0aBAs9h/ohfcEjoYmHdYqRL9r2t33SwYg==",
          "context": {
            "Execution": {
              "Id": "arn:aws:states:us-west-2:863246263227:execution:brokenspoke-analyzer:fd34f1d1-8009-44f1-9111-d3a2daf8a8fe",
              "Name": "fd34f1d1-8009-44f1-9111-d3a2daf8a8fe",
              "RoleArn": "arn:aws:iam::863246263227:role/BNAPipelineLambdaExecution",
              "StartTime": "+002024-04-11T03:05:31.843000000Z"
            },
            "State": {
              "EnteredTime": "+002024-04-11T03:05:32.059000000Z",
              "Name": "BNAContext"
            },
            "StateMachine": {
              "Id": "arn:aws:states:us-west-2:863246263227:stateMachine:brokenspoke-analyzer",
              "Name": "brokenspoke-analyzer"
            },
            "Id": "9ff90cac-0cf5-4923-897f-4416df5e7328"
          },
          "aws_s3": {
            "destination": "usa/new mexico/santa rosa/23.12.4"
          },
          "fargate": {
            "ecs_cluster_arn": "arn:aws:ecs:us-west-2:863246263227:cluster/bna",
            "task_arn": "arn:aws:ecs:us-west-2:863246263227:task/bna/681690ef8bbb446a93e1324f113e75f0",
            "last_status": "STOPPED"
          }
        }"#;
        let _deserialized = serde_json::from_str::<TaskInput>(json_input).unwrap();
    }

    #[test]
    fn test_parse_overallscores() {
        let data = r#"id,score_id,score_original,score_normalized,human_explanation
1,people,0.1917,19.1700,"On average, census blocks in the neighborhood received this population score."
2,opportunity_employment,0.0826,8.2600,"On average, census blocks in the neighborhood received this employment score."
3,opportunity_k12_education,0.0831,8.3100,"On average, census blocks in the neighborhood received this K12 schools score."
4,opportunity_technical_vocational_college,0.0000,0.0000,"On average, census blocks in the neighborhood received this tech/vocational colleges score."
5,opportunity_higher_education,0.0000,0.0000,"On average, census blocks in the neighborhood received this universities score."
6,opportunity,0.0829,8.2900,
7,core_services_doctors,0.0000,0.0000,"On average, census blocks in the neighborhood received this doctors score."
8,core_services_dentists,0.0000,0.0000,"On average, census blocks in the neighborhood received this dentists score."
9,core_services_hospitals,0.0518,5.1800,"On average, census blocks in the neighborhood received this hospital score."
10,core_services_pharmacies,0.0000,0.0000,"On average, census blocks in the neighborhood received this pharmacies score."
11,core_services_grocery,0.0169,1.6900,"On average, census blocks in the neighborhood received this grocery score."
12,core_services_social_services,0.0000,0.0000,"On average, census blocks in the neighborhood received this social services score."
13,core_services,0.0324,3.2400,
14,retail,0.0000,0.0000,"On average, census blocks in the neighborhood received this retail score."
15,recreation_parks,0.0713,7.1300,"On average, census blocks in the neighborhood received this parks score."
16,recreation_trails,0.0000,0.0000,"On average, census blocks in the neighborhood received this trails score."
17,recreation_community_centers,0.0000,0.0000,"On average, census blocks in the neighborhood received this community centers score."
18,recreation,0.0713,7.1300,
19,transit,0.0000,0.0000,"On average, census blocks in the neighborhood received this transit score."
20,overall_score,0.0893,8.9300,
21,population_total,2960.0000,,Total population of boundary
22,total_miles_low_stress,9.3090,9.3000,Total low-stress miles
23,total_miles_high_stress,64.5092,64.5000,Total high-stress miles"#;
        let _scores = parse_overall_scores(data.as_bytes()).unwrap();
    }

    // #[test]
    // fn test_post() {
    //     let data = r#"id,score_id,score_original,score_normalized,human_explanation
    // 1,people,0.1917,19.1700,"On average, census blocks in the neighborhood received this population score."
    // 2,opportunity_employment,0.0826,8.2600,"On average, census blocks in the neighborhood received this employment score."
    // 3,opportunity_k12_education,0.0831,8.3100,"On average, census blocks in the neighborhood received this K12 schools score."
    // 4,opportunity_technical_vocational_college,0.0000,0.0000,"On average, census blocks in the neighborhood received this tech/vocational colleges score."
    // 5,opportunity_higher_education,0.0000,0.0000,"On average, census blocks in the neighborhood received this universities score."
    // 6,opportunity,0.0829,8.2900,
    // 7,core_services_doctors,0.0000,0.0000,"On average, census blocks in the neighborhood received this doctors score."
    // 8,core_services_dentists,0.0000,0.0000,"On average, census blocks in the neighborhood received this dentists score."
    // 9,core_services_hospitals,0.0518,5.1800,"On average, census blocks in the neighborhood received this hospital score."
    // 10,core_services_pharmacies,0.0000,0.0000,"On average, census blocks in the neighborhood received this pharmacies score."
    // 11,core_services_grocery,0.0169,1.6900,"On average, census blocks in the neighborhood received this grocery score."
    // 12,core_services_social_services,0.0000,0.0000,"On average, census blocks in the neighborhood received this social services score."
    // 13,core_services,0.0324,3.2400,
    // 14,retail,0.0000,0.0000,"On average, census blocks in the neighborhood received this retail score."
    // 15,recreation_parks,0.0713,7.1300,"On average, census blocks in the neighborhood received this parks score."
    // 16,recreation_trails,0.0000,0.0000,"On average, census blocks in the neighborhood received this trails score."
    // 17,recreation_community_centers,0.0000,0.0000,"On average, census blocks in the neighborhood received this community centers score."
    // 18,recreation,0.0713,7.1300,
    // 19,transit,0.0000,0.0000,"On average, census blocks in the neighborhood received this transit score."
    // 20,overall_score,0.0893,8.9300,
    // 21,population_total,2960.0000,,Total population of boundary
    // 22,total_miles_low_stress,9.3090,9.3000,Total low-stress miles
    // 23,total_miles_high_stress,64.5092,64.5000,Total high-stress miles"#;
    //     let overall_scores = parse_overall_scores(data.as_bytes()).unwrap();

    //     // Convert the overall scores to a BNAPost struct.
    //     let version = String::from("24.05");
    //     let city_id = Uuid::new_v4();
    //     let bna_post = scores_to_bnapost(overall_scores, version, city_id);
    //     dbg!(&bna_post);
    //     let s = serde_json::to_string(&bna_post);
    //     dbg!(s);

    // info!("Retrieve secrets and parameters...");
    // // Retrieve API hostname.
    // let api_hostname = String::from("https://api.peopleforbikes.xyz");

    // let auth = AuthResponse {
    //     access_token: String::from("")
    //     expires_in: 3600,
    //     token_type: String::from("Bearer"),
    // };

    // // Prepare API URLs.
    // let bnas_url = format!("{api_hostname}/bnas");

    // // Post a new entry via the API.
    // info!("Post a new entry via the API...");
    // Client::new()
    //     .post(bnas_url)
    //     .bearer_auth(auth.access_token.clone())
    //     .json(&bna_post)
    //     .send()
    //     .unwrap()
    //     .error_for_status()
    //     .unwrap();
    // }

    // #[tokio::test]
    // async fn test_fetch_s3_object() {
    //     let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    //     let client = aws_sdk_s3::Client::new(&config);
    //     let buffer = fetch_s3_object_as_bytes(
    //         client,
    //         "brokenspoke-analyzer",
    //         "spain/valencia/valencia/24.4/neighborhood_overall_scores.csv",
    //     )
    //     .await
    //     .unwrap();
    //     assert!(buffer.len() > 0)
    // }

    // #[test]
    // fn test_post_cities() {
    //     let country = String::from("United States");
    //     let region = String::from("New Mexico");
    //     let name = String::from("Santa Rosa");

    //     let api_hostname = "https://api.peopleforbikes.xyz";
    //     let cities_url = format!("{api_hostname}/cities");
    //     let get_cities_url = format!("{cities_url}/{country}/{region}/{name}");

    //     let auth = AuthResponse {
    //         access_token: String::from("eyJraWQiOiJJa2REUDBMVkVBckdqM25udDZoQmJ3T3VhdVdhSUI2K0tweXJLYVRaaEVZPSIsImFsZyI6IlJTMjU2In0.eyJzdWIiOiJlNmMwYjEyYS1iNjg1LTQ3YmMtYThmMi1kOGU0YTZjYTMyODAiLCJpc3MiOiJodHRwczpcL1wvY29nbml0by1pZHAudXMtd2VzdC0yLmFtYXpvbmF3cy5jb21cL3VzLXdlc3QtMl9OUjFXaGVPdFciLCJjbGllbnRfaWQiOiI0amtsbGk3MDEwZHFla2NtNTJ1bjVmNXFxNyIsIm9yaWdpbl9qdGkiOiI5NmMwMWI1Ny01YWQ0LTQ4ODMtOTIwYy1kZDc5ZjY2N2RlMzYiLCJldmVudF9pZCI6IjBmMmU5NDA3LTRlYmQtNGQ1Zi1hN2YwLTBhMzM1OTNkNzg5ZSIsInRva2VuX3VzZSI6ImFjY2VzcyIsInNjb3BlIjoiYXdzLmNvZ25pdG8uc2lnbmluLnVzZXIuYWRtaW4iLCJhdXRoX3RpbWUiOjE3MTYzMjkzMDMsImV4cCI6MTcxNjMzMjkwMywiaWF0IjoxNzE2MzI5MzAzLCJqdGkiOiJkYzA2ZThmOS1mYjM5LTRiM2YtYTE0My03ZWY0YmEwYmNmMzUiLCJ1c2VybmFtZSI6InJlbXkuZ3JlaW5ob2ZlciJ9.k39pNR6NOtE72dysEraDr52M4pnMzRd0PW2OmbL4oB4Hn3yR89tOdIB2CKo_hT9U0gR90UPxAeQsVkfZ98LTiK5Ysf1dkQl-0gJlYayZc-Rva3_5o7eH40PsdJvAMkaIYCdTI8NoTa8WsCwqVSOx1irotzBXrSlSQ_aS7k6SJO0cYFTJPk7VIRUiHwXwARhO5N38EUTRHCbHzKmkY6MMv04yCJz1FDoYFkTzeShB6pEaUDAwnjDY3KJGhxpPFLfsGToxNB0xKfw0cklTXHnP8ZgVT2LpvP3t068rrz-3JqdqeDPshcKUkVgCEJZ1m1TlPYndXhuZoTCpR07wJhqDZQ"),
    //         expires_in: 3600,
    //         token_type: String::from("Bearer"),
    //     };
    //     // Create the city.
    //     let c = City {
    //         country: country.clone(),
    //         state: region.clone(),
    //         name: name.clone(),
    //         ..Default::default()
    //     };
    //     let client = Client::new();
    //     let r = client
    //         .post(cities_url)
    //         .bearer_auth(auth.access_token.clone())
    //         .json(&c)
    //         .send()
    //         .unwrap()
    //         .error_for_status()
    //         .unwrap();
    //     dbg!(r);
    // }
}
