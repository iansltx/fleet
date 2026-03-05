//! Datastore trait matching Go's `server/fleet/datastore.go`.
//!
//! This defines the core CRUD operations for each entity type.
//! The Go interface has 200+ methods; here we include the essential ones.

use async_trait::async_trait;

use crate::campaign::DistributedQueryCampaign;
use crate::carve::{CarveListOptions, CarveMetadata};
use crate::config::AppConfig;
use crate::enroll::EnrollSecret;
use crate::error::FleetError;
use crate::host::{Host, HostListOptions};
use crate::invite::Invite;
use crate::label::{Label, LabelSpec};
use crate::pack::Pack;
use crate::policy::Policy;
use crate::query::Query;
use crate::script::Script;
use crate::session::Session;
use crate::software::Software;
use crate::team::Team;
use crate::user::{User, UserListOptions, UserSummary};
use crate::ListOptions;

/// Result type alias for datastore operations.
pub type DsResult<T> = Result<T, FleetError>;

/// Datastore defines the core data access interface for the Fleet server.
#[async_trait]
pub trait Datastore: Send + Sync {
    // =========================================================================
    // Users
    // =========================================================================

    async fn new_user(&self, user: &User) -> DsResult<User>;
    async fn list_users(&self, opt: UserListOptions) -> DsResult<Vec<User>>;
    async fn users_by_ids(&self, ids: &[u32]) -> DsResult<Vec<UserSummary>>;
    async fn user_by_email(&self, email: &str) -> DsResult<User>;
    async fn user_by_id(&self, id: u32) -> DsResult<User>;
    async fn save_user(&self, user: &User) -> DsResult<()>;
    async fn delete_user(&self, id: u32) -> DsResult<()>;

    // =========================================================================
    // Queries
    // =========================================================================

    async fn new_query(&self, query: &Query) -> DsResult<Query>;
    async fn query_by_name(&self, team_id: Option<u32>, name: &str) -> DsResult<Query>;
    async fn save_query(&self, query: &Query) -> DsResult<()>;
    async fn delete_query(&self, team_id: Option<u32>, name: &str) -> DsResult<()>;
    async fn delete_queries(&self, ids: &[u32]) -> DsResult<u32>;
    async fn list_queries(&self, opt: ListOptions, team_id: Option<u32>) -> DsResult<Vec<Query>>;

    // =========================================================================
    // Packs
    // =========================================================================

    async fn new_pack(&self, pack: &Pack) -> DsResult<Pack>;
    async fn save_pack(&self, pack: &Pack) -> DsResult<()>;
    async fn delete_pack(&self, name: &str) -> DsResult<()>;
    async fn pack(&self, id: u32) -> DsResult<Pack>;
    async fn list_packs(&self, opt: ListOptions) -> DsResult<Vec<Pack>>;

    // =========================================================================
    // Labels
    // =========================================================================

    async fn new_label(&self, label: &Label) -> DsResult<Label>;
    async fn save_label(&self, label: &Label) -> DsResult<Label>;
    async fn delete_label(&self, name: &str) -> DsResult<()>;
    async fn label(&self, id: u32) -> DsResult<Label>;
    async fn list_labels(&self, opt: ListOptions) -> DsResult<Vec<Label>>;
    async fn apply_label_specs(&self, specs: &[LabelSpec]) -> DsResult<()>;

    // =========================================================================
    // Hosts
    // =========================================================================

    async fn new_host(&self, host: &Host) -> DsResult<Host>;
    async fn save_host(&self, host: &Host) -> DsResult<()>;
    async fn delete_host(&self, id: u32) -> DsResult<()>;
    async fn host(&self, id: u32) -> DsResult<Host>;
    async fn list_hosts(&self, opt: HostListOptions) -> DsResult<Vec<Host>>;
    async fn host_by_identifier(&self, identifier: &str) -> DsResult<Host>;

    // =========================================================================
    // Teams
    // =========================================================================

    async fn new_team(&self, team: &Team) -> DsResult<Team>;
    async fn save_team(&self, team: &Team) -> DsResult<Team>;
    async fn team(&self, id: u32) -> DsResult<Team>;
    async fn delete_team(&self, id: u32) -> DsResult<()>;
    async fn team_by_name(&self, name: &str) -> DsResult<Team>;
    async fn list_teams(&self, opt: ListOptions) -> DsResult<Vec<Team>>;

    // =========================================================================
    // Policies
    // =========================================================================

    async fn new_global_policy(&self, policy: &Policy) -> DsResult<Policy>;
    async fn policy(&self, id: u32) -> DsResult<Policy>;
    async fn save_policy(&self, policy: &Policy) -> DsResult<()>;
    async fn list_global_policies(&self, opt: ListOptions) -> DsResult<Vec<Policy>>;
    async fn delete_global_policies(&self, ids: &[u32]) -> DsResult<Vec<u32>>;
    async fn new_team_policy(&self, team_id: u32, policy: &Policy) -> DsResult<Policy>;
    async fn list_team_policies(&self, team_id: u32, opt: ListOptions) -> DsResult<Vec<Policy>>;
    async fn delete_team_policies(&self, team_id: u32, ids: &[u32]) -> DsResult<Vec<u32>>;

    // =========================================================================
    // Sessions
    // =========================================================================

    async fn new_session(&self, user_id: u32, session_key: &str) -> DsResult<Session>;
    async fn session_by_key(&self, key: &str) -> DsResult<Session>;
    async fn session_by_id(&self, id: u32) -> DsResult<Session>;
    async fn mark_session_accessed(&self, session: &Session) -> DsResult<()>;
    async fn destroy_session(&self, session: &Session) -> DsResult<()>;
    async fn destroy_all_sessions_for_user(&self, user_id: u32) -> DsResult<()>;

    // =========================================================================
    // Invites
    // =========================================================================

    async fn new_invite(&self, invite: &Invite) -> DsResult<Invite>;
    async fn list_invites(&self, opt: ListOptions) -> DsResult<Vec<Invite>>;
    async fn invite(&self, id: u32) -> DsResult<Invite>;
    async fn invite_by_email(&self, email: &str) -> DsResult<Invite>;
    async fn invite_by_token(&self, token: &str) -> DsResult<Invite>;
    async fn delete_invite(&self, id: u32) -> DsResult<()>;

    // =========================================================================
    // AppConfig
    // =========================================================================

    async fn app_config(&self) -> DsResult<AppConfig>;
    async fn save_app_config(&self, config: &AppConfig) -> DsResult<()>;

    // =========================================================================
    // Enroll Secrets
    // =========================================================================

    async fn verify_enroll_secret(&self, secret: &str) -> DsResult<EnrollSecret>;
    async fn get_enroll_secrets(&self, team_id: Option<u32>) -> DsResult<Vec<EnrollSecret>>;
    async fn apply_enroll_secrets(
        &self,
        team_id: Option<u32>,
        secrets: &[EnrollSecret],
    ) -> DsResult<()>;

    // =========================================================================
    // Campaigns
    // =========================================================================

    async fn new_distributed_query_campaign(
        &self,
        campaign: &DistributedQueryCampaign,
    ) -> DsResult<DistributedQueryCampaign>;
    async fn distributed_query_campaign(&self, id: u32) -> DsResult<DistributedQueryCampaign>;
    async fn save_distributed_query_campaign(
        &self,
        campaign: &DistributedQueryCampaign,
    ) -> DsResult<()>;

    // =========================================================================
    // Carves
    // =========================================================================

    async fn new_carve(&self, metadata: &CarveMetadata) -> DsResult<CarveMetadata>;
    async fn carve(&self, id: i64) -> DsResult<CarveMetadata>;
    async fn carve_by_session_id(&self, session_id: &str) -> DsResult<CarveMetadata>;
    async fn list_carves(&self, opt: CarveListOptions) -> DsResult<Vec<CarveMetadata>>;

    // =========================================================================
    // Scripts
    // =========================================================================

    async fn new_script(&self, script: &Script) -> DsResult<Script>;
    async fn script(&self, id: u32) -> DsResult<Script>;
    async fn delete_script(&self, id: u32) -> DsResult<()>;
    async fn list_scripts(&self, team_id: Option<u32>, opt: ListOptions) -> DsResult<Vec<Script>>;

    // =========================================================================
    // Software
    // =========================================================================

    async fn list_software(
        &self,
        opt: ListOptions,
        team_id: Option<u32>,
    ) -> DsResult<Vec<Software>>;
    async fn software_by_id(&self, id: u32) -> DsResult<Software>;
}
