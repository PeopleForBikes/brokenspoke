use bnacore::neon::{
    model::{
        NeonBranch, NeonCreateBranchRequest, NeonCreateBranchResponse, NeonEndpoint,
        NeonEndpointType, NeonListBranchResponses,
    },
    NEON_PROJECTS_URL,
};
use color_eyre::Result;
use reqwest::header::{self, HeaderValue};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        .init();

    // Create the Neon client.
    let neon_api_key = env::var("NEON_API_KEY")?;
    let mut headers = header::HeaderMap::new();
    let mut auth_value = HeaderValue::from_str(format!("Bearer {}", neon_api_key).as_ref())?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);
    let neon = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let neon_project_id = env::var("NEON_PROJECT_ID")?;
    let neon_branches_url = format!("{}/{}/branches", NEON_PROJECTS_URL, neon_project_id);

    // Query neon API and check whether we can create a branch or not.
    let branches = neon
        .get(&neon_branches_url)
        .send()
        .await?
        .error_for_status()?
        .json::<NeonListBranchResponses>()
        .await?;
    dbg!(&branches);

    let branch_name = "usa-santa-rosa-new-mexico".to_string();
    let create_branch_request = NeonCreateBranchRequest {
        endpoints: vec![NeonEndpoint {
            r#type: NeonEndpointType::ReadWrite,
            ..Default::default()
        }],
        branch: NeonBranch {
            name: Some(branch_name),
            ..Default::default()
        },
    };
    let create_branch_response = neon
        .post(&neon_branches_url)
        .json(&create_branch_request)
        .send()
        .await?
        .error_for_status()?
        .json::<NeonCreateBranchResponse>()
        .await?;
    dbg!(&create_branch_response);

    Ok(())
}
