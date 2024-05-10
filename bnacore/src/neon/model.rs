use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
pub struct NeonError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListBranchResponse {
    #[serde(flatten)]
    pub branch: Branch,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListBranchResponses {
    pub branches: Vec<ListBranchResponse>,
}

/// The compute endpoint type. Either read_write or read_only.
/// The read_only compute endpoint type is not yet supported (Oct 2023).
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    ReadOnly,
    #[default]
    ReadWrite,
}

/// The Neon compute provisioner.
/// Specify the k8s-neonvm provisioner to create a compute endpoint that supports Autoscaling.
#[derive(Debug, Deserialize, Serialize)]
pub enum ComputeProvisioner {
    #[serde(rename = "k8s-pod")]
    K8sPod,
    #[serde(rename = "k8s-neonvm")]
    K8sNeonVM,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointState {
    Active,
    Idle,
    Init,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    // pg_settings: NeonPGSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PGSettings {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolerMode {
    Transaction,
}

#[skip_serializing_none]
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Endpoint {
    /// The maximum number of Compute Units.
    pub autoscaling_limit_max_cu: Option<f32>,
    /// The minimum number of Compute Units.
    /// The minimum value is 0.25.
    pub autoscaling_limit_min_cu: Option<f32>,
    /// The ID of the branch that the compute endpoint is associated with.
    pub branch_id: Option<String>,
    /// A timestamp indicating when the compute endpoint was created.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub created_at: Option<OffsetDateTime>,
    /// The state of the compute endpoint.
    pub current_state: Option<EndpointState>,
    /// Whether to restrict connections to the compute endpoint
    pub disabled: Option<bool>,
    /// The hostname of the compute endpoint.
    /// This is the hostname specified when connecting to a Neon database.
    pub host: Option<String>,
    /// The compute endpoint ID.
    /// Compute endpoint IDs have an ep- prefix. For example: ep-little-smoke-851426
    pub id: Option<String>,
    /// A timestamp indicating when the compute endpoint was last active.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub last_active: Option<OffsetDateTime>,
    /// Whether to permit passwordless access to the compute endpoint.
    pub passwordless_access: Option<bool>,
    /// The state of the compute endpoint.
    pub pending_state: Option<EndpointState>,
    /// Whether connection pooling is enabled for the compute endpoint.
    pub pooler_enabled: Option<bool>,
    /// The connection pooler mode. Neon supports PgBouncer in transaction mode only.
    pub pooler_mode: Option<PoolerMode>,
    /// The ID of the project to which the compute endpoint belongs.
    pub project_id: Option<String>,
    /// The Neon compute provisioner.
    pub provisioner: Option<ComputeProvisioner>,
    /// The region identifier.
    pub region_id: Option<String>,
    /// A collection of settings for a compute endpoint
    pub settings: Option<Settings>,
    /// Duration of inactivity in seconds after which the compute endpoint is
    /// automatically suspended. The value 0 means use the global default.
    /// The value -1 means never suspend. The default value is 300 seconds (5 minutes).
    /// The maximum value is 604800 seconds (1 week).
    pub suspend_timeout_seconds: Option<u64>,
    /// The compute endpoint type.
    pub r#type: EndpointType,
    /// A timestamp indicating when the compute endpoint was last updated.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchState {
    Init,
    Ready,
}

#[skip_serializing_none]
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Branch {
    pub active_time_seconds: Option<u64>,
    pub compute_time_seconds: Option<u64>,
    /// CPU seconds used by all the endpoints of the branch, including deleted ones.
    /// This value is reset at the beginning of each billing period.
    pub cpu_used_sec: Option<u64>,
    /// A timestamp indicating when the branch was created.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub created_at: Option<OffsetDateTime>,
    /// The branch creation source.
    pub creation_source: Option<String>,
    /// The branch state.
    pub current_state: Option<BranchState>,
    pub data_transfer_bytes: Option<u64>,
    /// The branch ID.
    pub id: Option<String>,
    /// The logical size of the branch, in bytes
    pub logical_size: Option<u64>,
    /// The branch name.
    pub name: Option<String>,
    /// The branch_id of the parent branch.
    /// If omitted or empty, the branch will be created from the project's primary branch.
    pub parent_id: Option<String>,
    /// The ID of the project to which the branch belongs.
    pub project_id: Option<String>,
    /// A Log Sequence Number (LSN) on the parent branch.
    /// The branch will be created with data from this LSN.
    pub parent_lsn: Option<String>,
    /// The branch state.
    pub pending_state: Option<BranchState>,
    /// Whether the branch is the project's primary branch
    pub primary: Option<bool>,
    /// A timestamp identifying a point in time on the parent branch.
    /// The branch will be created with data starting from this point in time.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub timestamp: Option<OffsetDateTime>,
    /// A timestamp indicating when the branch was last updated.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub updated_at: Option<OffsetDateTime>,
    pub written_data_bytes: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    ApplyConfig,
    ApplyStorageConfig,
    CheckAvailability,
    CreateBranch,
    CreateCompute,
    CreateTimeline,
    DeleteTimeline,
    DisableMaintenance,
    ReplaceSafekeeper,
    StartCompute,
    SuspendCompute,
    TenantAttach,
    TenantDetach,
    TenantIgnore,
    TenantReattach,
}

#[skip_serializing_none]
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Operation {
    /// The action performed by the operation.
    pub action: Option<Action>,
    /// The branch ID.
    pub branch_id: Option<String>,
    /// A timestamp indicating when the operation was created.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub created_at: Option<OffsetDateTime>,
    /// The endpoint ID.
    pub endpoint_id: Option<String>,
    /// The error that occured.
    pub error: Option<String>,
    /// The operation ID.
    pub id: Option<String>,
    /// The number of times the operation failed.
    pub failures_count: Option<u32>,
    /// The Neon project ID.
    pub project_id: Option<String>,
    /// A timestamp indicating when the operation was last retried.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub retry_at: Option<OffsetDateTime>,
    /// The status of the operation.
    pub status: Option<String>,
    /// The total duration of the operation in milliseconds.
    pub total_duration_ms: Option<u32>,
    /// A timestamp indicating when the operation was last updated.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub updated_at: Option<OffsetDateTime>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Role {
    /// The ID of the branch to which the role belongs.
    pub branch_id: Option<String>,
    /// A timestamp indicating when the role was created.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub created_at: Option<OffsetDateTime>,
    /// The role name.
    pub name: Option<String>,
    /// The role password.
    pub password: Option<String>,
    /// Whether or not the role is system-protected.
    pub role: Option<bool>,
    /// A timestamp indicating when the role was last updated.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub updated_at: Option<OffsetDateTime>,
}

#[skip_serializing_none]
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Database {
    /// The ID of the branch to which the database belongs.
    pub branch_id: Option<String>,
    /// A timestamp indicating when the database was created.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub created_at: Option<OffsetDateTime>,
    /// The database ID.
    pub id: Option<u32>,
    /// The database name.
    pub name: Option<String>,
    /// The name of role that owns the database.
    pub owner_name: Option<String>,
    /// A timestamp indicating when the database was last updated.
    #[serde(with = "time::serde::iso8601::option")]
    #[serde(default)]
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateBranchRequest {
    pub endpoints: Vec<Endpoint>,
    pub branch: Branch,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateBranchResponse {
    pub branch: Branch,
    pub endpoints: Vec<Endpoint>,
    pub operations: Vec<Operation>,
    pub roles: Vec<Role>,
    pub databases: Vec<Database>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteBranchResponse {
    pub branch: Branch,
    pub operations: Vec<Operation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_list_branch() {
        let raw_json = r#"
        {
          "branches": [
            {
              "id": "br-round-pine-192368",
              "project_id": "patient-smoke-782429",
              "name": "main",
              "current_state": "ready",
              "logical_size": 39215104,
              "creation_source": "console",
              "primary": true,
              "cpu_used_sec": 594,
              "compute_time_seconds": 594,
              "active_time_seconds": 2376,
              "written_data_bytes": 11787549,
              "data_transfer_bytes": 12271,
              "created_at": "2023-04-17T13:53:48Z",
              "updated_at": "2023-10-11T15:31:10Z"
            },
            {
              "id": "br-still-breeze-64375152",
              "project_id": "patient-smoke-782429",
              "parent_id": "br-round-pine-192368",
              "parent_lsn": "0/35F2340",
              "name": "remy-is-testing",
              "current_state": "ready",
              "logical_size": 39215104,
              "creation_source": "console",
              "primary": false,
              "cpu_used_sec": 78,
              "compute_time_seconds": 78,
              "active_time_seconds": 312,
              "written_data_bytes": 0,
              "data_transfer_bytes": 0,
              "created_at": "2023-10-11T17:12:32Z",
              "updated_at": "2023-10-11T17:31:40Z"
            },
            {
              "id": "br-soft-star-53161169",
              "project_id": "patient-smoke-782429",
              "parent_id": "br-round-pine-192368",
              "parent_lsn": "0/35B7D70",
              "name": "usa-santa-rosa-new-mexico",
              "current_state": "ready",
              "logical_size": 39165952,
              "creation_source": "console",
              "primary": false,
              "cpu_used_sec": 78,
              "compute_time_seconds": 78,
              "active_time_seconds": 312,
              "written_data_bytes": 20968,
              "data_transfer_bytes": 0,
              "created_at": "2023-10-05T21:53:02Z",
              "updated_at": "2023-10-11T03:13:00Z"
            }
          ]
        }"#;
        let v = serde_json::from_str::<ListBranchResponses>(raw_json).unwrap();
        assert_eq!(v.branches.len(), 3)
    }

    #[test]
    fn test_ser_de_create_branch_request() {
        let branch_name = "usa-santa-rosa-new-mexico".to_string();
        let create_branch_request = CreateBranchRequest {
            endpoints: vec![Endpoint {
                r#type: EndpointType::ReadWrite,
                ..Default::default()
            }],
            branch: Branch {
                name: Some(branch_name),
                ..Default::default()
            },
        };
        let serialized = dbg!(serde_json::to_string(&create_branch_request)).unwrap();
        let _deserialized = serde_json::from_str::<CreateBranchRequest>(&serialized).unwrap();
    }

    #[test]
    fn test_deser_create_branch_response() {
        // https://api-docs.neon.tech/reference/createprojectbranch
        let raw_json = r#"
        {
          "branch": {
            "id": "br-odd-dream-88611736",
            "project_id": "patient-smoke-782429",
            "parent_id": "br-round-pine-192368",
            "parent_lsn": "0/3BE68658",
            "name": "help-me-debug",
            "current_state": "init",
            "pending_state": "ready",
            "creation_source": "console",
            "primary": false,
            "cpu_used_sec": 0,
            "compute_time_seconds": 0,
            "active_time_seconds": 0,
            "written_data_bytes": 0,
            "data_transfer_bytes": 0,
            "created_at": "2023-10-13T20:15:28Z",
            "updated_at": "2023-10-13T20:15:28Z"
          },
          "endpoints": [
            {
              "host": "ep-super-unit-07200292.us-west-2.aws.neon.tech",
              "id": "ep-super-unit-07200292",
              "project_id": "patient-smoke-782429",
              "branch_id": "br-odd-dream-88611736",
              "autoscaling_limit_min_cu": 0.25,
              "autoscaling_limit_max_cu": 0.25,
              "region_id": "aws-us-west-2",
              "type": "read_write",
              "current_state": "init",
              "pending_state": "active",
              "settings": {},
              "pooler_enabled": false,
              "pooler_mode": "transaction",
              "disabled": false,
              "passwordless_access": true,
              "creation_source": "console",
              "created_at": "2023-10-13T20:15:28Z",
              "updated_at": "2023-10-13T20:15:28Z",
              "proxy_host": "us-west-2.aws.neon.tech",
              "suspend_timeout_seconds": 0,
              "provisioner": "k8s-pod"
            }
          ],
          "operations": [
            {
              "id": "73fa0fb1-b96e-43ba-968f-571836a0ffbe",
              "project_id": "patient-smoke-782429",
              "branch_id": "br-odd-dream-88611736",
              "action": "create_branch",
              "status": "running",
              "failures_count": 0,
              "created_at": "2023-10-13T20:15:28Z",
              "updated_at": "2023-10-13T20:15:28Z",
              "total_duration_ms": 0
            },
            {
              "id": "ff80b203-2842-455c-8bb1-168fd1f8f364",
              "project_id": "patient-smoke-782429",
              "branch_id": "br-odd-dream-88611736",
              "endpoint_id": "ep-super-unit-07200292",
              "action": "start_compute",
              "status": "scheduling",
              "failures_count": 0,
              "created_at": "2023-10-13T20:15:28Z",
              "updated_at": "2023-10-13T20:15:28Z",
              "total_duration_ms": 2
            }
          ],
          "roles": [
            {
              "branch_id": "br-odd-dream-88611736",
              "name": "rgreinho",
              "protected": false,
              "created_at": "2023-04-17T13:53:48Z",
              "updated_at": "2023-04-17T13:53:48Z"
            }
          ],
          "databases": [
            {
              "id": 6595608,
              "branch_id": "br-odd-dream-88611736",
              "name": "bna",
              "owner_name": "rgreinho",
              "created_at": "2023-04-17T22:48:06Z",
              "updated_at": "2023-04-17T22:48:06Z"
            }
          ],
          "connection_uris": [
            {
              "connection_uri": "postgres://rgreinho:password@ep-super-unit-07200292.us-west-2.aws.neon.tech/bna",
              "connection_parameters": {
                "database": "bna",
                "password": "password",
                "role": "rgreinho",
                "host": "ep-super-unit-07200292.us-west-2.aws.neon.tech",
                "pooler_host": "ep-super-unit-07200292-pooler.us-west-2.aws.neon.tech"
              }
            }
          ]
        }
      "#;
        let deserialized = serde_json::from_str::<CreateBranchResponse>(raw_json).unwrap();
        let _serialized = serde_json::to_string(&deserialized).unwrap();
    }

    #[test]
    fn test_deserialize_delete_branch() {
        //https://api-docs.neon.tech/reference/deleteprojectbranch
        let raw_json = r#"
        {
          "branch": {
            "id": "br-aged-salad-637688",
            "project_id": "shiny-wind-028834",
            "name": "main",
            "current_state": "ready",
            "logical_size": 28,
            "created_at": "2022-11-23T17:42:25Z",
            "updated_at": "2022-11-23T17:42:26Z",
            "data_transfer_bytes": 1000000,
            "written_data_bytes": 100800,
            "compute_time_seconds": 100,
            "active_time_seconds": 100,
            "cpu_used_sec": 100,
            "primary": true,
            "creation_source": "console"
          },
          "operations": [
            {
              "id": "b6afbc21-2990-4a76-980b-b57d8c2948f2",
              "project_id": "shiny-wind-028834",
              "branch_id": "br-sweet-breeze-497520",
              "endpoint_id": "ep-soft-violet-752733",
              "action": "suspend_compute",
              "status": "running",
              "failures_count": 0,
              "created_at": "2022-12-01T19:53:05Z",
              "updated_at": "2022-12-01T19:53:05Z",
              "total_duration_ms": 100
            },
            {
              "id": "b6afbc21-2990-4a76-980b-b57d8c2948f2",
              "project_id": "shiny-wind-028834",
              "branch_id": "br-sweet-breeze-497520",
              "action": "delete_timeline",
              "status": "scheduling",
              "failures_count": 0,
              "created_at": "2022-12-01T19:53:05Z",
              "updated_at": "2022-12-01T19:53:05Z",
              "total_duration_ms": 100
            }
          ]
        }"#;
        let _deserialized = serde_json::from_str::<DeleteBranchResponse>(raw_json).unwrap();
    }
}
