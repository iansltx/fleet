//! S3-backed blob storage.
//!
//! Supports AWS S3, S3-compatible endpoints, and Google Cloud Storage.
//! Mirrors the Go `server/datastore/s3/` implementation.

use std::collections::HashSet;

use aws_sdk_s3::Client;
use chrono::{DateTime, Utc};
use fleet_types::blobstore::{BlobContent, BlobStore, BlobStoreError};

/// Configuration for S3 blob storage.
#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket: String,
    pub prefix: String,
    pub region: String,
    pub endpoint_url: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub force_path_style: bool,
    pub disable_ssl: bool,
}

/// S3-backed blob store.
///
/// Different content types (installers, icons, bootstrap packages) each get
/// their own `S3BlobStore` instance with a unique `path_prefix`.
pub struct S3BlobStore {
    client: Client,
    bucket: String,
    /// Global prefix (e.g. "fleet/")
    prefix: String,
    /// Content-type prefix (e.g. "software-installers")
    path_prefix: String,
}

impl S3BlobStore {
    /// Create a new S3 blob store from configuration.
    pub async fn new(cfg: &S3Config, path_prefix: impl Into<String>) -> Result<Self, BlobStoreError> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(cfg.region.clone()));

        if let Some(endpoint) = &cfg.endpoint_url {
            config_loader = config_loader.endpoint_url(endpoint);
        }

        if let (Some(key), Some(secret)) = (&cfg.access_key_id, &cfg.secret_access_key) {
            config_loader = config_loader.credentials_provider(
                aws_credential_types::Credentials::new(
                    key.clone(),
                    secret.clone(),
                    None,
                    None,
                    "fleet-config",
                ),
            );
        }

        let aws_config = config_loader.load().await;

        let mut s3_config = aws_sdk_s3::config::Builder::from(&aws_config);
        if cfg.force_path_style {
            s3_config = s3_config.force_path_style(true);
        }

        let client = Client::from_conf(s3_config.build());

        Ok(Self {
            client,
            bucket: cfg.bucket.clone(),
            prefix: cfg.prefix.clone(),
            path_prefix: path_prefix.into(),
        })
    }

    fn object_key(&self, file_id: &str) -> String {
        if self.prefix.is_empty() {
            format!("{}/{}", self.path_prefix, file_id)
        } else {
            format!("{}/{}/{}", self.prefix.trim_end_matches('/'), self.path_prefix, file_id)
        }
    }
}

#[async_trait::async_trait]
impl BlobStore for S3BlobStore {
    async fn get(&self, file_id: &str) -> Result<BlobContent, BlobStoreError> {
        let key = self.object_key(file_id);

        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("NoSuchKey") || msg.contains("not found") || msg.contains("404") {
                    BlobStoreError::NotFound(file_id.to_string())
                } else {
                    BlobStoreError::Storage(msg)
                }
            })?;

        let size = resp.content_length.unwrap_or(0);
        let data = resp
            .body
            .collect()
            .await
            .map_err(|e| BlobStoreError::Storage(e.to_string()))?
            .into_bytes()
            .to_vec();

        Ok(BlobContent { data, size })
    }

    async fn put(&self, file_id: &str, content: &[u8]) -> Result<(), BlobStoreError> {
        if file_id.is_empty() {
            return Err(BlobStoreError::InvalidFileId);
        }

        let key = self.object_key(file_id);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(content.to_vec().into())
            .send()
            .await
            .map_err(|e| BlobStoreError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn exists(&self, file_id: &str) -> Result<bool, BlobStoreError> {
        let key = self.object_key(file_id);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("NotFound") || msg.contains("not found") || msg.contains("404") {
                    Ok(false)
                } else {
                    Err(BlobStoreError::Storage(msg))
                }
            }
        }
    }

    async fn cleanup(
        &self,
        used_ids: &[String],
        remove_before: DateTime<Utc>,
    ) -> Result<usize, BlobStoreError> {
        let used: HashSet<&str> = used_ids.iter().map(|s| s.as_str()).collect();
        let prefix = if self.prefix.is_empty() {
            format!("{}/", self.path_prefix)
        } else {
            format!("{}/{}/", self.prefix.trim_end_matches('/'), self.path_prefix)
        };

        let mut deleted = 0;
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(token) = &continuation_token {
                req = req.continuation_token(token);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| BlobStoreError::Storage(e.to_string()))?;

            for obj in resp.contents() {
                    let key: &str = match obj.key() {
                        Some(k) => k,
                        None => continue,
                    };

                    // Extract file_id from the key
                    let file_id = key.rsplit('/').next().unwrap_or(key);
                    if used.contains(file_id) {
                        continue;
                    }

                    // Check modification time
                    if let Some(last_modified) = obj.last_modified() {
                        let secs = last_modified.secs();
                        let nanos = last_modified.subsec_nanos();
                        let modified_dt = DateTime::<Utc>::from_timestamp(secs, nanos as u32)
                            .unwrap_or_default();

                        if modified_dt >= remove_before {
                            continue;
                        }
                    }

                    // Delete the object
                    if let Err(e) = self
                        .client
                        .delete_object()
                        .bucket(&self.bucket)
                        .key(key)
                        .send()
                        .await
                    {
                        tracing::warn!(key = %key, error = %e, "failed to delete orphaned S3 blob");
                    } else {
                        deleted += 1;
                    }
            }

            if resp.is_truncated() == Some(true) {
                continuation_token = resp.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(deleted)
    }

    async fn sign(
        &self,
        file_id: &str,
        expires_in: std::time::Duration,
    ) -> Result<String, BlobStoreError> {
        let key = self.object_key(file_id);

        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
            .map_err(|e| BlobStoreError::Storage(e.to_string()))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(presigning_config)
            .await
            .map_err(|e| BlobStoreError::Storage(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    async fn delete(&self, file_id: &str) -> Result<(), BlobStoreError> {
        let key = self.object_key(file_id);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| BlobStoreError::Storage(e.to_string()))?;

        Ok(())
    }
}
