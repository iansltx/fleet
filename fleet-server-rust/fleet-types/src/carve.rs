//! Carve types matching Go's `server/fleet/carves.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ListOptions;

/// CarveMetadata represents metadata for a file carve operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarveMetadata {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub host_id: u32,
    pub name: String,
    pub block_count: i64,
    pub block_size: i64,
    pub carve_size: i64,
    pub carve_id: String,
    pub request_id: String,
    pub session_id: String,
    pub expired: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub max_block: i64,
}

impl CarveMetadata {
    /// Returns true if all blocks have been received.
    pub fn blocks_complete(&self) -> bool {
        self.max_block == self.block_count - 1
    }
}

/// CarveListOptions configures carve listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CarveListOptions {
    #[serde(flatten)]
    pub list_options: ListOptions,
    pub expired: bool,
}

/// CarveBeginPayload is the payload to initiate a carve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarveBeginPayload {
    pub block_count: i64,
    pub block_size: i64,
    pub carve_size: i64,
    pub carve_id: String,
    pub request_id: String,
}

/// CarveBlockPayload is the payload for a single carve data block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarveBlockPayload {
    pub session_id: String,
    pub request_id: String,
    pub block_id: i64,
    #[serde(with = "serde_bytes_base64")]
    pub data: Vec<u8>,
}

/// Helper module for base64 serialization of byte data.
mod serde_bytes_base64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::Serialize;
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<u8> = Vec::deserialize(deserializer)?;
        Ok(v)
    }
}
