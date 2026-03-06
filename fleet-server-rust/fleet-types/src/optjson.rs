//! Tri-state JSON types for PATCH semantics.
//!
//! These types distinguish between three states in JSON payloads:
//! - **Absent**: The key was not present in the JSON → don't touch the existing value
//! - **Null**: The key was explicitly set to `null` → clear/reset the field
//! - **Value**: The key was set to a non-null value → update the field
//!
//! This matches Go's `pkg/optjson` package behavior.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A tri-state value for PATCH-style JSON semantics.
///
/// - `set = false` → key was absent from JSON (do not modify)
/// - `set = true, valid = false` → key was `null` (clear/reset)
/// - `set = true, valid = true` → key had a value (update to `value`)
#[derive(Debug, Clone, PartialEq)]
pub struct OptJson<T> {
    /// Whether the key was present in the JSON payload.
    pub set: bool,
    /// Whether the value was non-null.
    pub valid: bool,
    /// The actual value (meaningful only when `valid` is true).
    pub value: T,
}

impl<T: Default> Default for OptJson<T> {
    fn default() -> Self {
        Self {
            set: false,
            valid: false,
            value: T::default(),
        }
    }
}

impl<T> OptJson<T> {
    /// Create a set, valid value.
    pub fn some(value: T) -> Self {
        Self {
            set: true,
            valid: true,
            value,
        }
    }

    /// Create a set but null value.
    pub fn null() -> Self
    where
        T: Default,
    {
        Self {
            set: true,
            valid: false,
            value: T::default(),
        }
    }

    /// Returns `true` if the key was present in the JSON.
    pub fn is_set(&self) -> bool {
        self.set
    }

    /// Returns `true` if the value is non-null.
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Convert to `Option<T>`, returning `Some` only when set and valid.
    pub fn into_option(self) -> Option<T> {
        if self.set && self.valid {
            Some(self.value)
        } else {
            None
        }
    }

    /// Get a reference to the value if set and valid.
    pub fn as_option(&self) -> Option<&T> {
        if self.set && self.valid {
            Some(&self.value)
        } else {
            None
        }
    }
}

impl<T: Serialize> Serialize for OptJson<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.valid {
            self.value.serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de, T: DeserializeOwned + Default> Deserialize<'de> for OptJson<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // If deserialize is called, the key was present (set = true).
        let v: Option<T> = Option::deserialize(deserializer)?;
        match v {
            Some(value) => Ok(Self {
                set: true,
                valid: true,
                value,
            }),
            None => Ok(Self {
                set: true,
                valid: false,
                value: T::default(),
            }),
        }
    }
}

// Type aliases matching Go's optjson package.

/// Tri-state String (Go: `optjson.String`).
pub type OptString = OptJson<String>;

/// Tri-state bool (Go: `optjson.Bool`).
pub type OptBool = OptJson<bool>;

/// Tri-state integer (Go: `optjson.Int`).
pub type OptInt = OptJson<i64>;

/// Tri-state slice (Go: `optjson.Slice[T]`).
pub type OptSlice<T> = OptJson<Vec<T>>;
