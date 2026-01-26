pub mod s3;

use reqwest::StatusCode;
use serde::Deserialize;
use std::{collections::HashMap, env};
use thiserror::Error;
use time::OffsetDateTime;

/// AWS module errors
#[derive(Error, Debug)]
pub enum AWSError {
    /// Secret not found.
    #[error("secret `{0}` not found")]
    SecretNotFound(String),

    /// Secret was found but the secret key does not exist.
    #[error("no value matching the key `{secret_key}` in secret `{secret_name}`")]
    SecretKeyNotFound {
        secret_name: String,
        secret_key: String,
    },

    /// SSM parameter not found.
    #[error("parameter `{0}` not found")]
    ParameterNotFound(String),

    /// S3 Error.
    #[error("S3 operation failed: `{0}`")]
    S3Error(String),
}

/// Represent the contents of the encrypted fields SecretString or SecretBinary
/// from the specified version of a secret, whichever contains content.
/// https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetSecretValue.html
#[derive(Deserialize, Debug)]
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
}

impl SecretValue {
    /// Read the secret string as a collection of key/value pairs.
    pub fn parse_secret_string(&self) -> serde_json::Result<HashMap<String, String>> {
        serde_json::from_str::<HashMap<String, String>>(&self.secret_string)
    }

    /// Extract the value of a specific secret from the secret string.
    pub fn extract_secret_value(&self, key: &str) -> serde_json::Result<Option<String>> {
        let secrets = self.parse_secret_string()?;
        match secrets.get(key) {
            Some(s) => Ok(Some(s.clone())),
            None => Ok(None),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum ParameterType {
    SecureString,
    String,
    StringList,
}

/// Represent a single parameter from the store.
///
/// Ref: https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/ssm/client/get_parameter.html
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResultMetadata {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SSMParameter {
    pub parameter: Parameter,
    pub result_metadata: ResultMetadata,
}

/// Retrieve a secret from the AWS Secrets Manager using the Lambda caching layer.
///
/// Ref: <https://docs.aws.amazon.com/secretsmanager/latest/userguide/retrieving-secrets_lambda.html>
pub async fn get_aws_secrets(secret_id: &str) -> Result<SecretValue, crate::Error> {
    let aws_session_token = env::var("AWS_SESSION_TOKEN")?;
    let res = reqwest::Client::new()
        .get(format!(
            "http://localhost:2773/secretsmanager/get?secretId={secret_id}"
        ))
        .header("X-Aws-Parameters-Secrets-Token", aws_session_token)
        .send()
        .await?
        .error_for_status();
    match res {
        Ok(res) => Ok(res.json::<SecretValue>().await?),

        Err(err) => match err.status() {
            Some(StatusCode::NOT_FOUND) => Err(crate::Error::BNAAWS(AWSError::SecretNotFound(
                secret_id.to_string(),
            ))),
            _ => Err(crate::Error::Reqwest(err)),
        },
    }
}

/// Retrieve a specific value out off a secret from AWS Secrets Manager.
pub async fn get_aws_secrets_value(
    secret_name: &str,
    secret_key: &str,
) -> Result<String, crate::Error> {
    let secret = get_aws_secrets(secret_name).await?;
    let value = secret.extract_secret_value(secret_key)?;
    value.ok_or(crate::Error::BNAAWS(AWSError::SecretKeyNotFound {
        secret_name: secret_name.into(),
        secret_key: secret_key.into(),
    }))
}

/// Retrieve a parameter from the parameter store.
///
/// Ref: https://docs.aws.amazon.com/systems-manager/latest/userguide/ps-integration-lambda-extensions.html
pub async fn get_aws_parameter(name: &str) -> Result<SSMParameter, crate::Error> {
    let aws_session_token = env::var("AWS_SESSION_TOKEN")?;
    let res = reqwest::Client::new()
        .get(format!(
            "http://localhost:2773/systemsmanager/parameters/get/?name={name}"
        ))
        .header("X-Aws-Parameters-Secrets-Token", aws_session_token)
        .send()
        .await?
        .error_for_status();
    match res {
        Ok(res) => Ok(res.json::<SSMParameter>().await?),
        Err(err) => match err.status() {
            Some(StatusCode::NOT_FOUND) => Err(crate::Error::BNAAWS(AWSError::ParameterNotFound(
                name.to_string(),
            ))),
            _ => Err(crate::Error::Reqwest(err)),
        },
    }
}

/// Convenience function to extract a value from a parameter directly.
///
/// If the name exists as an environment variable, it will be returned, otherwise
/// AWS SSM service will be queried.
pub async fn get_aws_parameter_value(name: &str) -> Result<String, crate::Error> {
    let value = match env::var(name) {
        Ok(v) => v,
        Err(_) => {
            let parameter = get_aws_parameter(name).await?;
            parameter.parameter.value
        }
    };
    Ok(value)
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
        let _deserialized = serde_json::from_str::<SSMParameter>(raw_json).unwrap();
    }

    #[test]
    fn test_deserialize_secret() {
        let raw_json = r#"
          {
            "ARN": "arn:aws:secretsmanager:us-west-2:123456789012:secret:staging/DATABASE_URL-W9OPPc",
            "Name": "staging/DATABASE_URL",
            "VersionId": "2da56f31-38b6-4ea3-92b0-b15d1189f4d2",
            "SecretString": "{\"DATABASE_URL\":\"postgresql://user:password@host:5432/database?sslmode=require\"}",
            "VersionStages": [
                "AWSCURRENT"
            ],
            "CreatedDate": "2023-12-28T16:37:14.751000-06:00"
        }
      "#;
        let secret = serde_json::from_str::<SecretValue>(raw_json).unwrap();
        let value = secret.extract_secret_value("DATABASE_URL").unwrap();
        assert_eq!(
            value,
            Some("postgresql://user:password@host:5432/database?sslmode=require".to_string())
        )
    }
}
