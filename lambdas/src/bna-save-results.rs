use std::{collections::HashMap, io::Write};

use aws_config::BehaviorVersion;
use bnacore::aws::get_aws_parameter_value;
use bnalambdas::{authenticate_service_account, Context};
use csv::Reader;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

const OVERALL_SCORES_COUNT: usize = 23;

#[derive(Deserialize)]
struct TaskInput {
    aws_s3: AWSS3,
    context: Context,
}

#[derive(Deserialize, Serialize, Clone)]
struct AWSS3 {
    destination: String,
}

impl AWSS3 {
    fn get_version(&self) -> String {
        self.destination
            .clone()
            .split_terminator('/')
            .last()
            .unwrap()
            .to_owned()
    }
}

#[derive(Deserialize, Clone)]
struct OverallScore {
    score_id: String,
    score_normalized: f64,
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
        self.get_overall_score(score_id).map(|s| s.score_normalized)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNASummary {
    pub bna_uuid: Uuid,
    pub version: String,
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAOpportunity {
    pub employment: Option<f64>,
    pub higher_education: Option<f64>,
    pub k12_education: Option<f64>,
    pub technical_vocational_college: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNACoreServices {
    pub dentists: Option<f64>,
    pub doctors: Option<f64>,
    pub grocery: Option<f64>,
    pub hospitals: Option<f64>,
    pub pharmacies: Option<f64>,
    pub social_services: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAFeatures {
    pub people: Option<f64>,
    pub retail: Option<f64>,
    pub transit: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BNAPost {
    pub core_services: BNACoreServices,
    pub features: BNAFeatures,
    pub infrastructure: BNAInfrastructure,
    pub opportunity: BNAOpportunity,
    pub recreation: BNARecreation,
    pub summary: BNASummary,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<(), Error> {
    // Read the task inputs.
    info!("Reading input...");
    let aws_s3 = &event.payload.aws_s3;
    let state_machine_context = &event.payload.context;
    let (_state_machine_id, _) = state_machine_context.execution.ids()?;

    info!("Retrieve secrets and parameters...");
    // Retrieve API hostname.
    let api_hostname = get_aws_parameter_value("BNA_API_HOSTNAME").await?;

    // Retrieve bna_bucket name.
    let bna_bucket = get_aws_parameter_value("BNA_BUCKET").await?;

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Configure the S3 client.
    info!("Configure the S3 client...");
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_s3::Client::new(&config);

    // Download the CSV file with the results.
    info!("Download the CSV file with the results...");
    let mut object = client
        .get_object()
        .bucket(bna_bucket)
        .key(aws_s3.destination.clone())
        .send()
        .await?;
    let mut buffer: Vec<u8> = Vec::new();
    while let Some(bytes) = object.body.try_next().await? {
        buffer.write_all(&bytes)?;
    }

    // Parse the results.
    info!("Parse the results...");
    let mut overall_scores = OverallScores::new();
    let mut rdr = Reader::from_reader(buffer.as_slice());
    for result in rdr.deserialize() {
        let score: OverallScore = result?;
        overall_scores.0.insert(score.score_id.clone(), score);
    }

    // Convert the overall scores to a BNAPost struct.
    let version = aws_s3.get_version();
    let bna_post = BNAPost {
        core_services: BNACoreServices {
            dentists: overall_scores.get_normalized_score("core_services_dentists"),
            doctors: overall_scores.get_normalized_score("core_services_doctors"),
            grocery: overall_scores.get_normalized_score("core_services_grocery"),
            hospitals: overall_scores.get_normalized_score("core_services_hospitals"),
            pharmacies: overall_scores.get_normalized_score("core_services_pharmacies"),
            social_services: overall_scores.get_normalized_score("core_services_social_services"),
        },
        features: BNAFeatures {
            people: overall_scores.get_normalized_score("people"),
            retail: overall_scores.get_normalized_score("retail"),
            transit: overall_scores.get_normalized_score("transit"),
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
        },
        recreation: BNARecreation {
            community_centers: overall_scores.get_normalized_score("recreation_community_centers"),
            parks: overall_scores.get_normalized_score("recreation_parks"),
            recreation_trails: overall_scores.get_normalized_score("recreation_trails"),
        },
        summary: BNASummary {
            bna_uuid: Uuid::new_v4(),
            version,
        },
    };

    // Prepare API URLs.
    let bnas_url = format!("{api_hostname}/bnas");

    // Post a new entry via the API.
    info!("Post a new entry via the API...");
    Client::new()
        .post(bnas_url)
        .bearer_auth(auth.access_token.clone())
        .json(&bna_post)
        .send()?
        .error_for_status()?;

    // TODO(rgreinho): Update the pipeline status when the new state will be available.
    // Update the pipeline status.
    // info!("updating pipeline...");
    // let patch_url = format!("{bnas_url}/analysis/{state_machine_id}");
    // let pipeline = BrokenspokePipeline {
    //     state_machine_id,
    //     state: Some(BrokenspokeState::Setup),
    //     ..Default::default()
    // };
    // update_pipeline(&patch_url, &auth, &pipeline)?;

    Ok(())
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
            }
          },
          "aws_s3": {
            "destination": "usa/new mexico/santa rosa/24.04.4"
          },
          "fargate": {
            "ecs_cluster_arn": "arn:aws:ecs:us-west-2:863246263227:cluster/bna",
            "task_arn": "arn:aws:ecs:us-west-2:863246263227:task/bna/681690ef8bbb446a93e1324f113e75f0",
            "last_status": "STOPPED"
          }
        }"#;
        let _deserialized = serde_json::from_str::<TaskInput>(json_input).unwrap();
    }
}
