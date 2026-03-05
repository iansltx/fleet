//! Pack service operations.
//!
//! Implements pack CRUD and spec management.
//! Corresponds to Go's `server/service/packs.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Gets a pack by ID.
    ///
    /// Corresponds to Go's `(svc *Service) GetPack`.
    pub async fn get_pack(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Pack> {
        authz::authorize(viewer, Subject::Pack, Action::Read)?;
        self.ds.pack(id).await
    }

    /// Creates a new pack.
    ///
    /// Corresponds to Go's `(svc *Service) NewPack`.
    pub async fn new_pack(
        &self,
        viewer: &Viewer,
        payload: PackPayload,
    ) -> ServiceResult<fleet_types::Pack> {
        authz::authorize(viewer, Subject::Pack, Action::Write)?;

        if payload.name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }

        let pack = fleet_types::Pack {
            id: 0,
            name: payload.name,
            description: payload.description.unwrap_or_default(),
            platform: payload.platform.unwrap_or_default(),
            disabled: payload.disabled.unwrap_or(false),
            pack_type: None,
            host_ids: payload.host_ids.unwrap_or_default(),
            label_ids: payload.label_ids.unwrap_or_default(),
            team_ids: payload.team_ids.unwrap_or_default(),
            labels: Vec::new(),
            hosts: Vec::new(),
            teams: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created = self.ds.new_pack(&pack).await?;

        info!(pack_id = created.id, name = %created.name, "pack created");
        Ok(created)
    }

    /// Modifies an existing pack.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyPack`.
    pub async fn modify_pack(
        &self,
        viewer: &Viewer,
        id: u32,
        payload: ModifyPackPayload,
    ) -> ServiceResult<fleet_types::Pack> {
        authz::authorize(viewer, Subject::Pack, Action::Write)?;

        let mut pack = self.ds.pack(id).await?;

        let is_editable = pack.editable_pack_type();

        if let Some(name) = payload.name {
            if is_editable {
                pack.name = name;
            }
        }
        if let Some(description) = payload.description {
            if is_editable {
                pack.description = description;
            }
        }
        if let Some(platform) = payload.platform {
            pack.platform = platform;
        }
        if let Some(disabled) = payload.disabled {
            pack.disabled = disabled;
        }
        if let Some(host_ids) = payload.host_ids {
            if is_editable {
                pack.host_ids = host_ids;
            }
        }
        if let Some(label_ids) = payload.label_ids {
            if is_editable {
                pack.label_ids = label_ids;
            }
        }
        if let Some(team_ids) = payload.team_ids {
            if is_editable {
                pack.team_ids = team_ids;
            }
        }

        self.ds.save_pack(&pack).await?;

        info!(pack_id = pack.id, name = %pack.name, "pack modified");
        Ok(pack)
    }

    /// Lists all packs.
    ///
    /// Corresponds to Go's `(svc *Service) ListPacks`.
    pub async fn list_packs(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Pack>> {
        authz::authorize(viewer, Subject::Pack, Action::Read)?;
        self.ds.list_packs(opts).await
    }

    /// Deletes a pack by name.
    ///
    /// Corresponds to Go's `(svc *Service) DeletePack`.
    pub async fn delete_pack(
        &self,
        viewer: &Viewer,
        name: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Pack, Action::Write)?;

        // Check that the pack exists and is editable.
        if let Ok(Some(pack)) = self.ds.pack_by_name(name).await {
            if !pack.editable_pack_type() {
                return Err(ServiceError::bad_request(format!(
                    "cannot delete pack_type {}",
                    pack.pack_type.unwrap_or_default()
                )));
            }
        }

        self.ds.delete_pack(name).await?;

        info!(name = %name, "pack deleted");
        Ok(())
    }

    /// Deletes a pack by ID.
    ///
    /// Corresponds to Go's `(svc *Service) DeletePackByID`.
    pub async fn delete_pack_by_id(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Pack, Action::Write)?;

        let pack = self.ds.pack(id).await?;

        if !pack.editable_pack_type() {
            return Err(ServiceError::bad_request(format!(
                "cannot delete pack_type {}",
                pack.pack_type.unwrap_or_default()
            )));
        }

        self.ds.delete_pack(&pack.name).await?;

        info!(pack_id = id, name = %pack.name, "pack deleted by id");
        Ok(())
    }
}

/// Payload for creating a new pack.
#[derive(Debug, Clone, Default)]
pub struct PackPayload {
    pub name: String,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub disabled: Option<bool>,
    pub host_ids: Option<Vec<u32>>,
    pub label_ids: Option<Vec<u32>>,
    pub team_ids: Option<Vec<u32>>,
}

/// Payload for modifying an existing pack.
#[derive(Debug, Clone, Default)]
pub struct ModifyPackPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub disabled: Option<bool>,
    pub host_ids: Option<Vec<u32>>,
    pub label_ids: Option<Vec<u32>>,
    pub team_ids: Option<Vec<u32>>,
}
