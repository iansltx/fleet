//! Handler modules for Fleet API endpoints.
//!
//! Each module corresponds to a domain area of the Fleet API.
//! Handlers extract request data, call the service layer, and return
//! JSON responses matching the Go server's format.
//!
//! Note: Request body struct fields are read by serde deserialization
//! but not directly referenced in Rust code, producing dead_code warnings.

#[allow(dead_code)]
pub mod app_config;
#[allow(dead_code)]
pub mod carves;
#[allow(dead_code)]
pub mod device;
#[allow(dead_code)]
pub mod hosts;
#[allow(dead_code)]
pub mod invites;
#[allow(dead_code)]
pub mod labels;
#[allow(dead_code)]
pub mod mdm;
#[allow(dead_code)]
pub mod orbit;
#[allow(dead_code)]
pub mod osquery;
#[allow(dead_code)]
pub mod packs;
#[allow(dead_code)]
pub mod policies;
#[allow(dead_code)]
pub mod queries;
#[allow(dead_code)]
pub mod scripts;
#[allow(dead_code)]
pub mod sessions;
#[allow(dead_code)]
pub mod setup;
#[allow(dead_code)]
pub mod software;
#[allow(dead_code)]
pub mod teams;
#[allow(dead_code)]
pub mod users;
