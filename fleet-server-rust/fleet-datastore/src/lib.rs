//! Fleet Datastore - MySQL datastore layer for the Fleet server.
//!
//! This crate provides the `MysqlDatastore` struct that implements
//! all database operations using the same table/column names and query
//! patterns as the Go implementation.

pub mod datastore_impl;
pub mod activities;
pub mod campaigns;
pub mod app_config;
pub mod carves;
pub mod certificates;
pub mod enroll;
pub mod error;
pub mod hosts;
pub mod invites;
pub mod labels;
pub mod mysql;
pub mod packs;
pub mod policies;
pub mod queries;
pub mod scripts;
pub mod sessions;
pub mod setup_experience;
pub mod software;
pub mod teams;
pub mod users;

pub use error::DatastoreError;
pub use mysql::{MysqlDatastore, MysqlDatastoreConfig};
