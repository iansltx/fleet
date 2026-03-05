//! Service trait matching Go's `server/fleet/service.go`.
//!
//! This defines the core business logic interface for the Fleet server.
//! The Go interface has many methods; here we include the most important ones.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::campaign::DistributedQueryCampaign;
use crate::config::AppConfig;
use crate::enroll::EnrollSecret;
use crate::error::FleetError;
use crate::host::{Host, HostDetail, HostListOptions, HostSummary};
use crate::invite::Invite;
use crate::label::Label;
use crate::osquery::{OsqueryDistributedQueryResults, OsqueryStatus};
use crate::pack::Pack;
use crate::policy::Policy;
use crate::query::Query;
use crate::script::Script;
use crate::session::Session;
use crate::software::Software;
use crate::target::HostTargets;
use crate::team::Team;
use crate::user::User;
use crate::ListOptions;

pub type SvcResult<T> = Result<T, FleetError>;

/// OsqueryService handles communication with the osquery agent.
#[async_trait]
pub trait OsqueryService: Send + Sync {
    /// Enroll a new osquery host.
    async fn enroll_osquery(
        &self,
        enroll_secret: &str,
        host_identifier: &str,
        host_details: HashMap<String, HashMap<String, String>>,
    ) -> SvcResult<String>;

    /// Authenticate a host by its node key.
    async fn authenticate_host(&self, node_key: &str) -> SvcResult<(Host, bool)>;

    /// Get the client configuration for a host.
    async fn get_client_config(&self) -> SvcResult<serde_json::Value>;

    /// Get distributed queries for a host.
    async fn get_distributed_queries(
        &self,
    ) -> SvcResult<(
        HashMap<String, String>,
        HashMap<String, String>,
        u32,
    )>;

    /// Submit distributed query results.
    async fn submit_distributed_query_results(
        &self,
        results: OsqueryDistributedQueryResults,
        statuses: HashMap<String, OsqueryStatus>,
        messages: HashMap<String, String>,
        stats: HashMap<String, crate::campaign::Stats>,
    ) -> SvcResult<()>;

    /// Submit status logs.
    async fn submit_status_logs(&self, logs: Vec<serde_json::Value>) -> SvcResult<()>;

    /// Submit result logs.
    async fn submit_result_logs(&self, logs: Vec<serde_json::Value>) -> SvcResult<()>;
}

/// FleetService defines the core business logic methods for the Fleet server.
#[async_trait]
pub trait FleetService: Send + Sync {
    // =========================================================================
    // Users
    // =========================================================================

    async fn create_user(&self, user: &User) -> SvcResult<User>;
    async fn list_users(&self, opt: crate::user::UserListOptions) -> SvcResult<Vec<User>>;
    async fn user(&self, id: u32) -> SvcResult<User>;
    async fn modify_user(&self, id: u32, user: &User) -> SvcResult<User>;
    async fn delete_user(&self, id: u32) -> SvcResult<()>;
    async fn change_password(&self, old_password: &str, new_password: &str) -> SvcResult<()>;
    async fn require_password_reset(&self, id: u32, require: bool) -> SvcResult<User>;

    // =========================================================================
    // Sessions / Auth
    // =========================================================================

    async fn login(&self, email: &str, password: &str) -> SvcResult<(User, Session)>;
    async fn logout(&self) -> SvcResult<()>;
    async fn get_session_by_key(&self, key: &str) -> SvcResult<Session>;
    async fn delete_session_by_id(&self, id: u32) -> SvcResult<()>;

    // =========================================================================
    // AppConfig
    // =========================================================================

    async fn app_config(&self) -> SvcResult<AppConfig>;
    async fn modify_app_config(&self, config: serde_json::Value) -> SvcResult<AppConfig>;

    // =========================================================================
    // Invites
    // =========================================================================

    async fn invite_new_user(&self, invite: &Invite) -> SvcResult<Invite>;
    async fn list_invites(&self, opt: ListOptions) -> SvcResult<Vec<Invite>>;
    async fn delete_invite(&self, id: u32) -> SvcResult<()>;
    async fn verify_invite_token(&self, token: &str) -> SvcResult<Invite>;

    // =========================================================================
    // Teams
    // =========================================================================

    async fn new_team(&self, team: &Team) -> SvcResult<Team>;
    async fn modify_team(&self, id: u32, payload: serde_json::Value) -> SvcResult<Team>;
    async fn delete_team(&self, id: u32) -> SvcResult<()>;
    async fn list_teams(&self, opt: ListOptions) -> SvcResult<Vec<Team>>;
    async fn team(&self, id: u32) -> SvcResult<Team>;

    // =========================================================================
    // Enroll Secrets
    // =========================================================================

    async fn get_enroll_secrets(&self, team_id: Option<u32>) -> SvcResult<Vec<EnrollSecret>>;
    async fn apply_enroll_secrets(
        &self,
        team_id: Option<u32>,
        secrets: &[EnrollSecret],
    ) -> SvcResult<()>;

    // =========================================================================
    // Hosts
    // =========================================================================

    async fn list_hosts(&self, opt: HostListOptions) -> SvcResult<Vec<Host>>;
    async fn get_host(&self, id: u32) -> SvcResult<HostDetail>;
    async fn get_host_lite(&self, id: u32) -> SvcResult<Host>;
    async fn host_by_identifier(&self, identifier: &str) -> SvcResult<HostDetail>;
    async fn delete_host(&self, id: u32) -> SvcResult<()>;
    async fn count_hosts(&self, opt: HostListOptions) -> SvcResult<u32>;
    async fn get_host_summary(&self, team_id: Option<u32>) -> SvcResult<HostSummary>;
    async fn refetch_host(&self, id: u32) -> SvcResult<()>;

    // =========================================================================
    // Labels
    // =========================================================================

    async fn new_label(&self, label: &Label) -> SvcResult<Label>;
    async fn modify_label(&self, id: u32, payload: serde_json::Value) -> SvcResult<Label>;
    async fn get_label(&self, id: u32) -> SvcResult<Label>;
    async fn list_labels(&self, opt: ListOptions) -> SvcResult<Vec<Label>>;
    async fn delete_label(&self, name: &str) -> SvcResult<()>;

    // =========================================================================
    // Queries
    // =========================================================================

    async fn new_query(&self, query: &Query) -> SvcResult<Query>;
    async fn modify_query(&self, id: u32, payload: serde_json::Value) -> SvcResult<Query>;
    async fn delete_query(&self, team_id: Option<u32>, name: &str) -> SvcResult<()>;
    async fn delete_queries(&self, ids: &[u32]) -> SvcResult<u32>;
    async fn list_queries(&self, opt: ListOptions, team_id: Option<u32>) -> SvcResult<Vec<Query>>;
    async fn get_query(&self, id: u32) -> SvcResult<Query>;

    // =========================================================================
    // Packs
    // =========================================================================

    async fn new_pack(&self, pack: &Pack) -> SvcResult<Pack>;
    async fn modify_pack(&self, id: u32, payload: serde_json::Value) -> SvcResult<Pack>;
    async fn list_packs(&self, opt: ListOptions) -> SvcResult<Vec<Pack>>;
    async fn get_pack(&self, id: u32) -> SvcResult<Pack>;
    async fn delete_pack(&self, name: &str) -> SvcResult<()>;

    // =========================================================================
    // Policies
    // =========================================================================

    async fn new_global_policy(&self, policy: &Policy) -> SvcResult<Policy>;
    async fn list_global_policies(&self, opt: ListOptions) -> SvcResult<Vec<Policy>>;
    async fn get_policy_by_id(&self, id: u32) -> SvcResult<Policy>;
    async fn delete_global_policies(&self, ids: &[u32]) -> SvcResult<Vec<u32>>;
    async fn new_team_policy(&self, team_id: u32, policy: &Policy) -> SvcResult<Policy>;
    async fn list_team_policies(&self, team_id: u32, opt: ListOptions) -> SvcResult<Vec<Policy>>;

    // =========================================================================
    // Live Queries / Campaigns
    // =========================================================================

    async fn new_distributed_query_campaign(
        &self,
        query_string: &str,
        targets: HostTargets,
    ) -> SvcResult<DistributedQueryCampaign>;

    // =========================================================================
    // Scripts
    // =========================================================================

    async fn list_scripts(&self, team_id: Option<u32>, opt: ListOptions) -> SvcResult<Vec<Script>>;
    async fn get_script(&self, id: u32) -> SvcResult<Script>;
    async fn delete_script(&self, id: u32) -> SvcResult<()>;

    // =========================================================================
    // Software
    // =========================================================================

    async fn list_software(
        &self,
        opt: ListOptions,
        team_id: Option<u32>,
    ) -> SvcResult<Vec<Software>>;
}
