use bnacore::aws::get_aws_parameter_value;
use bnalambdas::{
    authenticate_service_account, update_pipeline, AnalysisParameters, BNAPipeline, Context,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct TaskInput {
    analysis_parameters: AnalysisParameters,
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
    // Retrieve API hostname.
    let api_hostname = get_aws_parameter_value("BNA_API_HOSTNAME").await?;

    // Prepare the API URL.
    let url = format!("{api_hostname}/ratings/analysis");

    // Authenticate the service account.
    let auth = authenticate_service_account()
        .await
        .map_err(|e| format!("cannot authenticate service account: {e}"))?;

    // Read the task inputs.
    let state_machine_context = &event.payload.context;
    let state_machine_id = state_machine_context.id;

    // Update the pipeline status.
    let patch_url = format!("{url}/{state_machine_id}");
    let pipeline = BNAPipeline {
        state_machine_id,
        step: Some("Cleanup".to_string()),
        ..Default::default()
    };
    update_pipeline(&patch_url, &auth, &pipeline)?;

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
    use bnalambdas::{Execution, State, StateMachine};
    use lambda_runtime::{Context, LambdaEvent};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_handler() {
        let id = "ID";

        let mut context = Context::default();
        context.request_id = id.to_string();

        let payload = TaskInput {
            analysis_parameters: AnalysisParameters::simple(
                "Malta".to_string(),
                "Valetta".to_string(),
            ),
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
                id: Uuid::new_v4(),
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
              },
              "Id": "8a4d7089-fe9c-42be-b9f5-0a3b0c31edef"
            }
          }"#;
        let deserialized = serde_json::from_str::<TaskInput>(json_input).unwrap();
        assert_eq!(deserialized.analysis_parameters.city, "provincetown");
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }
}
