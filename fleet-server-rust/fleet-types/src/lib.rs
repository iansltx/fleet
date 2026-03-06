//! Fleet Types - Core data types for the Fleet server.
//!
//! This crate contains all the core data models used throughout the Fleet server,
//! ported from the Go implementation. All types are serialization-compatible with
//! the Go types for JSON API responses.

pub mod activity;
pub mod blobstore;
pub mod campaign;
pub mod carve;
pub mod certificate;
pub mod config;
pub mod datastore;
pub mod enroll;
pub mod error;
pub mod host;
pub mod invite;
pub mod label;
pub mod mdm;
pub mod osquery;
pub mod pack;
pub mod policy;
pub mod query;
pub mod script;
pub mod service;
pub mod session;
pub mod software;
pub mod target;
pub mod team;
pub mod user;
pub mod vulnerability;

// Re-export commonly used types at the crate root.
pub use activity::{Activity, UpcomingActivity};
pub use campaign::*;
pub use carve::*;
pub use config::*;
pub use enroll::*;
pub use error::*;
pub use host::{
    AggregatedMDMData, AggregatedMDMSolutions, AggregatedMDMStatus, AggregatedMacadminsData,
    AggregatedMunkiIssue, AggregatedMunkiVersion, Host, HostDetail, HostListOptions, HostMDM,
    HostMunkiInfo, HostMunkiIssue, HostStatus, HostSummary, HostUser, MacadminsData, MDMSolution,
    MunkiIssue, OSVersionStats,
};
pub use invite::*;
pub use label::{Label, LabelMembershipType, LabelSpec, LabelSummary, LabelType};
pub use osquery::*;
pub use pack::{Pack, PackStats, ScheduledQuery};
pub use policy::{HostPolicy, Policy, PolicyData};
pub use query::{Query, QueryResultRow};
pub use script::Script;
pub use session::Session;
pub use software::Software;
pub use target::*;
pub use team::{Team, TeamConfig, TeamUser};
pub use user::User;
pub use vulnerability::{CVE, CVEMeta, SoftwareVulnerability};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Timestamps for entity creation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CreateTimestamp {
    pub created_at: DateTime<Utc>,
}

/// Timestamps for entity updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateTimestamp {
    pub updated_at: DateTime<Utc>,
}

/// Combined create and update timestamps, matching Go's UpdateCreateTimestamps.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateCreateTimestamps {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// OrderDirection specifies the direction of ordering (ascending or descending).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum OrderDirection {
    #[default]
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// ListOptions defines options related to paging and ordering.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ListOptions {
    /// Which page to return (must be positive integer).
    pub page: u32,
    /// How many results per page (0 indicates unlimited).
    pub per_page: u32,
    /// Key to use for ordering.
    pub order_key: String,
    /// Direction of ordering.
    pub order_direction: OrderDirection,
    /// Query string to match against columns of the entity.
    pub match_query: String,
    /// Row to start from (used with cursor pagination).
    pub after: String,
    /// Whether to include pagination metadata in the response.
    pub include_metadata: bool,
}
