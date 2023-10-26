use bnalambdas::{get_aws_parameter, get_aws_secrets, neon::NEON_PROJECTS_URL};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest::header::{self, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct TaskInput {
    setup: Setup,
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
    // Read the task inputs.
    let neon_branch_id = &event.payload.setup.neon.branch_id;

    // Create the Neon HTTP client.
    let neon_api_key = get_aws_secrets("NEON_API_KEY").await?;
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
            }
          }"#;
        let deserialized = serde_json::from_str::<TaskInput>(&json_input).unwrap();
        assert_eq!(deserialized.setup.neon.branch_id, "br-bold-mode-48632613");
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }
}
