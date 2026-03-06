//! Blob storage trait for software installers, icons, and bootstrap packages.
//!
//! Mirrors the Go `SoftwareInstallerStore`, `SoftwareTitleIconStore`, and
//! `MDMBootstrapPackageStore` interfaces from `server/fleet/`.

use chrono::{DateTime, Utc};

/// Errors returned by blob storage operations.
#[derive(Debug, thiserror::Error)]
pub enum BlobStoreError {
    #[error("blob not found: {0}")]
    NotFound(String),

    #[error("signing not supported by this backend")]
    SigningNotSupported,

    #[error("invalid file id")]
    InvalidFileId,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("storage error: {0}")]
    Storage(String),
}

impl BlobStoreError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, BlobStoreError::NotFound(_))
    }
}

/// A retrieved blob with its content and metadata.
pub struct BlobContent {
    /// The raw bytes of the blob.
    pub data: Vec<u8>,
    /// Total size in bytes.
    pub size: i64,
}

/// Trait for blob storage backends (filesystem, S3, GCS).
///
/// Separate instances are used for different content types:
/// - Software installers
/// - Software title icons
/// - MDM bootstrap packages
///
/// Each instance stores files under its own prefix.
#[async_trait::async_trait]
pub trait BlobStore: Send + Sync {
    /// Retrieve a blob by its file ID.
    ///
    /// Returns the content bytes and total size. Returns
    /// `BlobStoreError::NotFound` if the blob doesn't exist.
    async fn get(&self, file_id: &str) -> Result<BlobContent, BlobStoreError>;

    /// Store a blob with the given file ID.
    ///
    /// Overwrites any existing blob with the same ID.
    async fn put(&self, file_id: &str, content: &[u8]) -> Result<(), BlobStoreError>;

    /// Check whether a blob exists.
    async fn exists(&self, file_id: &str) -> Result<bool, BlobStoreError>;

    /// Delete unused blobs older than the given timestamp.
    ///
    /// `used_ids` contains the set of file IDs that are still referenced.
    /// Only blobs not in `used_ids` and created before `remove_before` are deleted.
    /// Returns the number of deleted blobs.
    async fn cleanup(
        &self,
        used_ids: &[String],
        remove_before: DateTime<Utc>,
    ) -> Result<usize, BlobStoreError>;

    /// Generate a pre-signed URL for direct download.
    ///
    /// Only supported by cloud backends (S3/CloudFront). Filesystem
    /// implementations return `BlobStoreError::SigningNotSupported`.
    async fn sign(
        &self,
        file_id: &str,
        expires_in: std::time::Duration,
    ) -> Result<String, BlobStoreError>;

    /// Delete a specific blob by file ID.
    async fn delete(&self, file_id: &str) -> Result<(), BlobStoreError>;
}
