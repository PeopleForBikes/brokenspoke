use bnacore::{
    aws::{get_aws_parameter, get_aws_secrets_value},
    neon::NEON_PROJECTS_URL,
};
use bnalambdas::{
    authenticate_service_account, update_pipeline, BrokenspokePipeline, BrokenspokeState, Context,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::header::{self, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct TaskInput {
    setup: Setup,
    context: Context,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Setup {
    neon: Neon,
}

#[derive(Debug, Serialize, Deserialize)]
struct Neon {
    branch_id: String,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<(), Error> {
    // Retrieve API URL.
    let url = "https://api.peopleforbikes.xyz/bnas/analysis";

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Read the task inputs.
    let neon_branch_id = &event.payload.setup.neon.branch_id;
    let state_machine_context = &event.payload.context;
    let (state_machine_id, _) = state_machine_context.execution.ids()?;

    // Update the pipeline status.
    let pipeline = BrokenspokePipeline {
        state_machine_id,
        state: Some(BrokenspokeState::Export),
        ..Default::default()
    };
    update_pipeline(url, &auth, &pipeline)?;

    // Create the Neon HTTP client.
    let neon_api_key = get_aws_secrets_value("NEON_API_KEY", "NEON_API_KEY").await?;
    let mut headers = header::HeaderMap::new();
    let mut auth_value = HeaderValue::from_str(format!("Bearer {}", neon_api_key).as_ref())?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);
    let neon = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let neon_project_id = get_aws_parameter("NEON_BROKENSPOKE_ANALYZER_PROJECT").await?;
    let neon_branches_url = format!(
        "{}/{}/branches/{}",
        NEON_PROJECTS_URL, neon_project_id, &neon_branch_id
    );

    // Delete neon branch.
    let delete_branch_response = neon
        .delete(&neon_branches_url)
        .send()
        .await?
        .error_for_status()?;
    info!("{:#?}", delete_branch_response);

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

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use bnalambdas::{Execution, State, StateMachine};
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_handler() {
        let id = "ID";

        let mut context = Context::default();
        context.request_id = id.to_string();

        let payload = TaskInput {
            setup: Setup {
                neon: Neon {
                    branch_id: "br-bold-mode-48632613".to_string(),
                },
            },
            context: bnalambdas::Context {
                execution: Execution {
                    id: "id".to_string(),
                    name: "name".to_string(),
                    role_arn: "arn".to_string(),
                    start_time: time::OffsetDateTime::now_utc(),
                },
                state: State {
                    entered_time: time::OffsetDateTime::now_utc(),
                    name: "name".to_string(),
                },
                state_machine: StateMachine {
                    id: "id".to_string(),
                    name: "name".to_string(),
                },
            },
        };
        let _event = LambdaEvent { payload, context };

        // let _result = function_handler(event).await.unwrap();

        // assert_eq!(result.msg, "Command X executed.");
        // assert_eq!(result.req_id, id.to_string());
    }

    #[test]
    fn test_deserialize_input() {
        let json_input = r#"{
            "analysis_parameters": {
              "country": "usa",
              "city": "provincetown",
              "region": "massachusetts",
              "fips_code": "555535"
            },
            "receipt_handle": "AQEB1tiDaN1qwFbZXhWBUwQmTRsUx06pGNOhVdZe86LABsb95D8oLIbFFcOTWQzc27SbKQ4xWtomueKwT8LjTv60SnjoTIm+bhM52w0LYRhadhdyRzQUNyOBEU18QLM8W2psRUm1bhyfRkPNPCl65uhrdJs1ta62d3n2rVOcLvNHp+EEGNnCenze8Cc9qvptMFohe9p56YBxKubA3f3Btv70FLpTZOSPHIa4aDBADLm9eZ16jN1Jc9GU4JMxeNBp3QAunPVFm94vrLCrprffJj4D83IfcQYIf1T7eYlH/LVQcp+Ihaxtas7qnjxa1W756olM3ppxq6ZjRcbVeAtQtrT/+M6YsAqXrBSS+TTOLqNS8Zn0R8/YqSdE31AUFUPeI6LIaF654LabYh/54hju6xRcyQ==",
            "setup": {
              "neon": {
                "branch_id": "br-bold-mode-48632613",
                "host": "ep-sweet-recipe-68291618.us-west-2.aws.neon.tech"
              }
            },
            "context": {
              "Execution": {
                "StartTime": "2024-02-12T16:45:38.655Z",
                "Id": "arn:aws:states:us-west-2:123456789012:execution:brokenspoke-analyzer:a0e708f8-3d9f-4749-b4de-20b2c2aab3d2",
                "RoleArn": "arn:aws:iam::123456789012:role/role",
                "Name": "a0e708f8-3d9f-4749-b4de-20b2c2aab3d2"
              },
              "State": {
                "EnteredTime": "2024-02-12T16:45:38.881Z",
                "Name": "BNAContext"
              },
              "StateMachine": {
                "Id": "arn:aws:states:us-west-2:123456789012:stateMachine:brokenspoke-analyzer",
                "Name": "brokenspoke-analyzer"
              }
            }
          }"#;
        let deserialized = serde_json::from_str::<TaskInput>(&json_input).unwrap();
        assert_eq!(deserialized.setup.neon.branch_id, "br-bold-mode-48632613");
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }
}
