use thiserror::Error;

/// DatastoreError covers all errors from the MySQL datastore layer.
#[derive(Debug, Error)]
pub enum DatastoreError {
    #[error("not found: {entity}")]
    NotFound {
        entity: String,
        id: Option<u64>,
        name: Option<String>,
    },

    #[error("already exists: {entity} ({name})")]
    AlreadyExists { entity: String, name: String },

    #[error("foreign key constraint: {entity} ({name})")]
    ForeignKey { entity: String, name: String },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("database error: {source}")]
    Database {
        #[from]
        source: sqlx::Error,
    },

    #[error("serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    #[error("internal error: {0}")]
    Internal(String),
}

impl DatastoreError {
    pub fn not_found(entity: &str) -> Self {
        DatastoreError::NotFound {
            entity: entity.to_string(),
            id: None,
            name: None,
        }
    }

    pub fn not_found_with_id(entity: &str, id: u64) -> Self {
        DatastoreError::NotFound {
            entity: entity.to_string(),
            id: Some(id),
            name: None,
        }
    }

    pub fn not_found_with_name(entity: &str, name: &str) -> Self {
        DatastoreError::NotFound {
            entity: entity.to_string(),
            id: None,
            name: Some(name.to_string()),
        }
    }

    pub fn already_exists(entity: &str, name: &str) -> Self {
        DatastoreError::AlreadyExists {
            entity: entity.to_string(),
            name: name.to_string(),
        }
    }

    pub fn foreign_key(entity: &str, name: &str) -> Self {
        DatastoreError::ForeignKey {
            entity: entity.to_string(),
            name: name.to_string(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, DatastoreError::NotFound { .. })
    }
}

/// Helper to check if a sqlx error is a MySQL duplicate entry error.
pub fn is_duplicate(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        // MySQL error code 1062 = Duplicate entry
        if let Some(code) = db_err.code() {
            return code == "1062";
        }
    }
    false
}

/// Helper to check if a sqlx error is a MySQL foreign key constraint error.
pub fn is_foreign_key(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        // MySQL error code 1452 = Cannot add or update a child row: a foreign key constraint fails
        if let Some(code) = db_err.code() {
            return code == "1452";
        }
    }
    false
}

/// Helper to check if a sqlx error is a child foreign key error.
pub fn is_child_foreign_key(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        // MySQL error code 1451 = Cannot delete or update a parent row: a foreign key constraint fails
        if let Some(code) = db_err.code() {
            return code == "1451";
        }
    }
    false
}

pub type Result<T> = std::result::Result<T, DatastoreError>;
