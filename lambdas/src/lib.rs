use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use time::OffsetDateTime;

/// Represent the contents of the encrypted fields SecretString or SecretBinary
/// from the specified version of a secret, whichever contains content.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SecretValue {
    /// Amazon Resource Name of the secret.
    #[serde(rename = "ARN")]
    pub arn: String,
    /// Creation date.
    pub created_date: String,
    /// The friendly name of the secret.
    pub name: String,
    /// The decrypted secret value, if the secret value was originally provided
    /// as binary data in the form of a byte array. The response parameter
    /// represents the binary data as a base64-encoded string.
    ///
    /// If the secret was created by using the Secrets Manager console, or if
    /// the secret value was originally provided as a string, then this field
    /// is omitted. The secret value appears in SecretString instead.
    pub secret_binary: Option<String>,
    /// The decrypted secret value, if the secret value was originally provided
    /// as a string or through the Secrets Manager console.
    /// If this secret was created by using the console, then Secrets Manager
    /// stores the information as a JSON structure of key/value pairs.
    pub secret_string: String,
    /// Unique identifier of the version of the secret.
    pub version_id: String,
    /// A list of all of the staging labels currently attached to this version
    /// of the secret.
    pub version_stages: Vec<String>,
    /// Metadata.
    pub result_metadata: HashMap<String, String>,
}

impl SecretValue {
    pub fn parse_string(&self) -> serde_json::Result<HashMap<String, String>> {
        serde_json::from_str::<HashMap<String, String>>(&self.secret_string)
    }

    pub fn extract_secret(&self, secret_name: &str) -> serde_json::Result<String> {
        let secrets = self.parse_string()?;
        Ok(secrets.get(secret_name).unwrap().clone())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ParameterType {
    SecureString,
    String,
    StringList,
}

/// Represent a single parameter from the store.
///
/// Ref: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/ssm/client/get_parameter.html
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    /// The Amazon Resource Name (ARN) of the parameter.
    #[serde(rename = "ARN")]
    pub arn: String,
    /// The data type of the parameter, such as text or aws:ec2:image. The default is text.
    pub data_type: String,
    /// Date the parameter was last changed or updated and the parameter version was created.
    #[serde(with = "time::serde::iso8601")]
    pub last_modified_date: OffsetDateTime,
    /// The name of the parameter.
    pub name: String,
    /// The type of parameter.
    pub r#type: ParameterType,
    /// Either the version number or the label used to retrieve the parameter value.
    /// Specify selectors by using one of the following formats:
    ///   parameter_name:version
    ///   parameter_name:label
    pub selector: Option<String>,
    /// Applies to parameters that reference information in other Amazon Web Services services.
    /// SourceResult is the raw result or response from the source.
    pub source_result: Option<String>,
    /// The parameter value.
    /// > **If type is StringList, the system returns a comma-separated string
    /// > with no spaces between commas in the Value field.**
    pub value: String,
    /// The parameter version.
    pub version: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResultMetadata {}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SSMParameter {
    pub parameter: Parameter,
    pub result_metadata: ResultMetadata,
}

/// Retrieves a secret from the AWS Secrets Manager using the Lambda caching layer.
///
/// Ref: <https://docs.aws.amazon.com/secretsmanager/latest/userguide/retrieving-secrets_lambda.html>
pub async fn get_aws_secrets(secret_id: &str) -> Result<String, String> {
    let aws_session_token =
        env::var("AWS_SESSION_TOKEN").map_err(|e| format!("Cannot find AWS session token: {e}"))?;
    let secret = reqwest::Client::new()
        .get(format!(
            "http://localhost:2773/secretsmanager/get?secretId={secret_id}"
        ))
        .header("X-Aws-Parameters-Secrets-Token", aws_session_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<SecretValue>()
        .await
        .map_err(|e| e.to_string())?;
    secret.extract_secret(secret_id).map_err(|e| e.to_string())
}

/// Ref: https://docs.aws.amazon.com/systems-manager/latest/userguide/ps-integration-lambda-extensions.html
pub async fn get_aws_parameter(name: &str) -> Result<String, String> {
    let aws_session_token =
        env::var("AWS_SESSION_TOKEN").map_err(|e| format!("Cannot find AWS session token: {e}"))?;
    let param = reqwest::Client::new()
        .get(format!(
            "http://localhost:2773/systemsmanager/parameters/get/?name={name}"
        ))
        .header("X-Aws-Parameters-Secrets-Token", aws_session_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<SSMParameter>()
        .await
        .map_err(|e| e.to_string())?;
    Ok(param.parameter.value)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisParameters {
    pub country: String,
    pub city: String,
    pub region: Option<String>,
    pub fips_code: Option<String>,
}

impl AnalysisParameters {
    pub fn new(
        country: String,
        city: String,
        region: Option<String>,
        fips_code: Option<String>,
    ) -> Self {
        Self {
            country,
            city,
            region,
            fips_code: match fips_code {
                Some(fips_code) => Some(fips_code),
                None => Some("0".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_parameter() {
        let raw_json = r#"
          {
            "Parameter": {
              "ARN": "arn:aws:ssm:us-west-2::parameter/PrivateSubnets",
              "DataType": "text",
              "LastModifiedDate": "2023-10-12T02:40:53.516Z",
              "Name": "PrivateSubnets",
              "Selector": null,
              "SourceResult": null,
              "Type": "String",
              "Value": "subnet-08d74ff09cdf9624b",
              "Version": 1
            },
            "ResultMetadata": {}
          }
        "#;
        let _deserialized = serde_json::from_str::<SSMParameter>(&raw_json).unwrap();
    }
}
