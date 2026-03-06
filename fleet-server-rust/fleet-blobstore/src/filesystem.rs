//! Local filesystem blob storage backend.
//!
//! Stores blobs as files under `<root_dir>/<prefix>/<file_id>`.
//! Mirrors the Go `server/datastore/filesystem/` implementation.

use std::collections::HashSet;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use fleet_types::blobstore::{BlobContent, BlobStore, BlobStoreError};

/// Filesystem-backed blob store.
///
/// Different content types (installers, icons, bootstrap packages) each get
/// their own `FilesystemBlobStore` instance with a unique `prefix`.
pub struct FilesystemBlobStore {
    root_dir: PathBuf,
    prefix: String,
}

impl FilesystemBlobStore {
    /// Create a new filesystem blob store.
    ///
    /// `root_dir` is the base directory; `prefix` is the subdirectory for this
    /// content type (e.g. "software-installers", "software-title-icons").
    pub fn new(root_dir: impl Into<PathBuf>, prefix: impl Into<String>) -> Result<Self, BlobStoreError> {
        let store = Self {
            root_dir: root_dir.into(),
            prefix: prefix.into(),
        };
        // Ensure the directory exists.
        std::fs::create_dir_all(store.dir())?;
        Ok(store)
    }

    fn dir(&self) -> PathBuf {
        self.root_dir.join(&self.prefix)
    }

    fn file_path(&self, file_id: &str) -> PathBuf {
        self.dir().join(file_id)
    }
}

#[async_trait::async_trait]
impl BlobStore for FilesystemBlobStore {
    async fn get(&self, file_id: &str) -> Result<BlobContent, BlobStoreError> {
        let path = self.file_path(file_id);
        match tokio::fs::read(&path).await {
            Ok(data) => {
                let size = data.len() as i64;
                Ok(BlobContent { data, size })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(BlobStoreError::NotFound(file_id.to_string()))
            }
            Err(e) => Err(BlobStoreError::Io(e)),
        }
    }

    async fn put(&self, file_id: &str, content: &[u8]) -> Result<(), BlobStoreError> {
        if file_id.is_empty() {
            return Err(BlobStoreError::InvalidFileId);
        }
        let path = self.file_path(file_id);
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    async fn exists(&self, file_id: &str) -> Result<bool, BlobStoreError> {
        let path = self.file_path(file_id);
        match tokio::fs::metadata(&path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(BlobStoreError::Io(e)),
        }
    }

    async fn cleanup(
        &self,
        used_ids: &[String],
        remove_before: DateTime<Utc>,
    ) -> Result<usize, BlobStoreError> {
        let used: HashSet<&str> = used_ids.iter().map(|s| s.as_str()).collect();
        let mut deleted = 0;

        let mut entries = tokio::fs::read_dir(self.dir()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if used.contains(name.as_ref()) {
                continue;
            }

            let meta = entry.metadata().await?;
            if let Ok(modified) = meta.modified() {
                let modified_dt: DateTime<Utc> = modified.into();
                if modified_dt < remove_before {
                    if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                        tracing::warn!(file = %name, error = %e, "failed to delete orphaned blob");
                    } else {
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }

    async fn sign(
        &self,
        _file_id: &str,
        _expires_in: std::time::Duration,
    ) -> Result<String, BlobStoreError> {
        Err(BlobStoreError::SigningNotSupported)
    }

    async fn delete(&self, file_id: &str) -> Result<(), BlobStoreError> {
        let path = self.file_path(file_id);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(BlobStoreError::NotFound(file_id.to_string()))
            }
            Err(e) => Err(BlobStoreError::Io(e)),
        }
    }
}
