use reqwest::{
    self,
    header::{self, HeaderValue},
};

use self::model::{
    Branch, CreateBranchRequest, CreateBranchResponse, DeleteBranchResponse, Endpoint,
    EndpointType, ListBranchResponses,
};
use thiserror::Error;

pub mod model;

pub const NEON_PROJECTS_URL: &str = "https://console.neon.tech/api/v2/projects";

// Neon.tech module errors
#[derive(Error, Debug)]
pub enum NeonError {
    // Error From the Reqwest crate.
    #[error("Reqwest error")]
    Reqwest(#[from] reqwest::Error),

    /// The API Key contains invalid characters.
    #[error("invalid API Key")]
    InvalidAPIKey,
}

pub struct Client {
    client: reqwest::Client,
    project_id: String,
}

impl Client {
    /// Create a new neon.tech REAT API client for a specific project.
    pub fn new(api_key: &str, project_id: &str) -> Result<Client, NeonError> {
        let mut headers = header::HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(format!("Bearer {api_key}").as_ref())
            .map_err(|_| NeonError::InvalidAPIKey)?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Client {
            client,
            project_id: project_id.into(),
        })
    }

    /// Retrieves a list of branches for the specified project.
    ///
    /// Ref: https://api-docs.neon.tech/reference/listprojectbranches
    pub async fn get_branches(&self) -> Result<ListBranchResponses, reqwest::Error> {
        let neon_branches_url = format!("{}/{}/branches", NEON_PROJECTS_URL, self.project_id);
        self.client
            .get(&neon_branches_url)
            .send()
            .await?
            .error_for_status()?
            .json::<ListBranchResponses>()
            .await
    }

    /// Creates a branch in the specified project.
    ///
    /// Ref: https://api-docs.neon.tech/reference/createprojectbranch
    pub async fn create_branch(
        &self,
        branch_name: &str,
    ) -> Result<CreateBranchResponse, reqwest::Error> {
        let create_branch_request = CreateBranchRequest {
            endpoints: vec![Endpoint {
                r#type: EndpointType::ReadWrite,
                ..Default::default()
            }],
            branch: Branch {
                name: Some(branch_name.into()),
                ..Default::default()
            },
        };
        let neon_branches_url = format!("{}/{}/branches", NEON_PROJECTS_URL, self.project_id);
        self.client
            .post(&neon_branches_url)
            .json(&create_branch_request)
            .send()
            .await?
            .error_for_status()?
            .json::<CreateBranchResponse>()
            .await
    }

    /// Deletes the specified branch from a project, and places
    /// all endpoints into an idle state, breaking existing client connections.
    ///
    /// Ref: https://api-docs.neon.tech/reference/deleteprojectbranch
    pub async fn delete_branch(
        &self,
        branch_id: &str,
    ) -> Result<DeleteBranchResponse, reqwest::Error> {
        let neon_branches_url = format!(
            "{}/{}/branches/{}",
            NEON_PROJECTS_URL, self.project_id, branch_id
        );
        self.client
            .delete(&neon_branches_url)
            .send()
            .await?
            .error_for_status()?
            .json::<DeleteBranchResponse>()
            .await
    }
}
