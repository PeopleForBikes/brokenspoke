use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct TaskInput {
    fargate: Fargate,
}

#[derive(Deserialize, Serialize)]
struct Fargate {
    ecs_cluster_arn: String,
    task_arn: String,
    last_status: String,
}

#[derive(Deserialize, Serialize)]
struct TaskOutput {
    ecs_cluster_arn: String,
    task_arn: String,
    last_status: String,
}

async fn function_handler(event: LambdaEvent<TaskInput>) -> Result<TaskOutput, Error> {
    // Read the task inputs.
    let ecs_cluster_arn = &event.payload.fargate.ecs_cluster_arn;
    let task_arn = &event.payload.fargate.task_arn;

    // Prepare the AWS client.
    let aws_config = aws_config::load_from_env().await;
    let ecs_client = aws_sdk_ecs::Client::new(&aws_config);

    // Describe the task.
    let task = ecs_client
        .describe_tasks()
        .cluster(ecs_cluster_arn)
        .tasks(task_arn)
        .send()
        .await?;
    let last_status = task
        .tasks()
        .unwrap()
        .first()
        .unwrap()
        .last_status()
        .unwrap()
        .to_owned();

    Ok(TaskOutput {
        ecs_cluster_arn: ecs_cluster_arn.into(),
        task_arn: task_arn.into(),
        last_status,
    })
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
            fargate: Fargate {
                ecs_cluster_arn: "arn:aws:ecs:us-west-2:960791345675:cluster/bna".to_string(),
                task_arn:
                    "arn:aws:ecs:us-west-2:960791345675:task/bna/e8729e4e8bac43c795d8fde735a86ef0"
                        .to_string(),
                last_status: "PROVISIONING".to_string(),
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
          "receipt_handle": "AQEBpbFnr1X4IKFuToT4mthKhHWr1+ZclCgwcuqnJQMqPj4szg2ZuKWQM/tkrApB2LMR+AJA6iZMqXY8AxxNCSRlE/h7odRrJHUOtO1sD499lWG5H0Hjcm4dbkXRHR/TjvEISyp6tKHpvbpr65GYNSk0gWIBHKxuvmUEY3eteKp/omGrPsVc6P1sVI3vwV6ZJzy/zw0WBJJG4h7jKmEqSvQdO5Dfh9SU4lQTExRaPsgIJDpa7YmyM4xgG9mezGFmsG3Pl6vS5lZhAEuNTnTiNHCbQKOXXw2Q1t5G8sMrSFQkRAtMVVT+LQNXcCd/PnVL7pDQ3FtP/ZlsFzKbTgr4C/4H546+exKrgm3U3tHM/BEmgdnSbNG0LBNtZLKpjKcC/v6JxRTw5VFGY4f9Hpa2lRoMCA==",
          "setup": {
            "neon": {
              "branch_id": "br-polished-brook-64925552",
              "host": "ep-blue-butterfly-84466944.us-west-2.aws.neon.tech"
            }
          },
          "fargate": {
            "ecs_cluster_arn": "arn:aws:ecs:us-west-2:960791345675:cluster/bna",
            "task_arn": "arn:aws:ecs:us-west-2:960791345675:task/bna/e8729e4e8bac43c795d8fde735a86ef0",
            "last_status": "PROVISIONING"
          }
        }"#;
        let deserialized = serde_json::from_str::<TaskInput>(&json_input).unwrap();
        assert_eq!(deserialized.fargate.last_status, "PROVISIONING");
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }
}
