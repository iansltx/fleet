//! Handler modules for Fleet API endpoints.
//!
//! Each module corresponds to a domain area of the Fleet API.
//! Handlers extract request data, call the service layer, and return
//! JSON responses matching the Go server's format.

pub mod app_config;
pub mod carves;
pub mod device;
pub mod hosts;
pub mod invites;
pub mod labels;
pub mod mdm;
pub mod orbit;
pub mod osquery;
pub mod packs;
pub mod policies;
pub mod queries;
pub mod scripts;
pub mod sessions;
pub mod setup;
pub mod software;
pub mod teams;
pub mod users;
