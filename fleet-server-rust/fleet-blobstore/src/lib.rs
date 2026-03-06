//! Blob storage backends for Fleet server.
//!
//! Provides filesystem and S3 implementations of the `BlobStore` trait
//! for storing software installers, icons, and MDM bootstrap packages.

pub mod filesystem;
pub mod s3;

pub use filesystem::FilesystemBlobStore;
pub use s3::S3BlobStore;
