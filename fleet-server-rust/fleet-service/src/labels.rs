//! Label service operations.
//!
//! Implements label CRUD, spec management, and label summary.
//! Corresponds to Go's `server/service/labels.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

/// Well-known reserved label names that cannot be created or deleted by users.
pub const RESERVED_LABEL_NAMES: &[&str] = &[
    fleet_types::label::BUILTIN_LABEL_ALL_HOSTS,
    fleet_types::label::BUILTIN_LABEL_MACOS,
    fleet_types::label::BUILTIN_LABEL_UBUNTU_LINUX,
    fleet_types::label::BUILTIN_LABEL_CENTOS_LINUX,
    fleet_types::label::BUILTIN_LABEL_WINDOWS,
];

impl FleetService {
    /// Creates a new label.
    ///
    /// Corresponds to Go's `(svc *Service) NewLabel`.
    pub async fn new_label(
        &self,
        viewer: &Viewer,
        payload: fleet_types::label::LabelPayload,
    ) -> ServiceResult<fleet_types::Label> {
        authz::authorize(viewer, Subject::Label, Action::Write)?;

        if payload.name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }

        // Cannot create a label with a reserved (built-in) name.
        if RESERVED_LABEL_NAMES.contains(&payload.name.as_str()) {
            return Err(ServiceError::invalid_argument(
                "name",
                format!(
                    "cannot add label '{}' because it conflicts with the name of a built-in label",
                    payload.name
                ),
            ));
        }

        let membership_type = if payload.query.is_empty()
            && payload.hosts.is_empty()
            && payload.host_ids.is_empty()
            && payload.criteria.is_none()
        {
            fleet_types::LabelMembershipType::Manual
        } else if payload.criteria.is_some() {
            fleet_types::LabelMembershipType::HostVitals
        } else {
            fleet_types::LabelMembershipType::Dynamic
        };

        let label = fleet_types::Label {
            id: 0,
            author_id: Some(viewer.user_id()),
            name: payload.name,
            description: payload.description,
            query: payload.query,
            platform: payload.platform,
            label_type: fleet_types::LabelType::Regular,
            label_membership_type: membership_type,
            criteria: payload.criteria,
            host_count: 0,
            team_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created = self.ds.new_label(&label).await?;

        info!(label_id = created.id, name = %created.name, "label created");
        Ok(created)
    }

    /// Gets a label by ID.
    ///
    /// Corresponds to Go's `(svc *Service) GetLabel`.
    pub async fn get_label(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Label> {
        authz::authorize(viewer, Subject::Label, Action::Read)?;
        self.ds.label(id).await
    }

    /// Modifies an existing label.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyLabel`.
    pub async fn modify_label(
        &self,
        viewer: &Viewer,
        id: u32,
        payload: fleet_types::label::ModifyLabelPayload,
    ) -> ServiceResult<fleet_types::Label> {
        let label = self.ds.label(id).await?;
        authz::authorize(viewer, Subject::Label, Action::Write)?;

        if label.label_type == fleet_types::LabelType::BuiltIn {
            return Err(ServiceError::invalid_argument(
                "label_type",
                format!("cannot modify built-in label '{}'", label.name),
            ));
        }

        let mut label = label;
        if let Some(name) = payload.name {
            // Check if the new name is a reserved label name.
            if RESERVED_LABEL_NAMES.contains(&name.as_str()) {
                return Err(ServiceError::invalid_argument(
                    "name",
                    format!(
                        "cannot rename label to '{}' because it conflicts with the name of a built-in label",
                        name
                    ),
                ));
            }
            label.name = name;
        }
        if let Some(description) = payload.description {
            label.description = description;
        }

        self.ds.save_label(&label).await
    }

    /// Lists all labels.
    ///
    /// Corresponds to Go's `(svc *Service) ListLabels`.
    pub async fn list_labels(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Label>> {
        authz::authorize(viewer, Subject::Label, Action::Read)?;
        self.ds.list_labels(opts).await
    }

    /// Returns a summary of all labels.
    ///
    /// Corresponds to Go's `(svc *Service) LabelsSummary`.
    pub async fn labels_summary(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<Vec<fleet_types::LabelSummary>> {
        authz::authorize(viewer, Subject::Label, Action::Read)?;
        self.ds.labels_summary().await
    }

    /// Deletes a label by name.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteLabel`.
    pub async fn delete_label(
        &self,
        viewer: &Viewer,
        name: &str,
    ) -> ServiceResult<()> {
        // Cannot delete built-in labels.
        if RESERVED_LABEL_NAMES.contains(&name) {
            return Err(ServiceError::invalid_argument(
                "name",
                format!("cannot delete built-in label '{}'", name),
            ));
        }

        authz::authorize(viewer, Subject::Label, Action::Write)?;

        // Verify the label exists.
        let label = self.ds.label_by_name(name).await?;
        if label.label_type == fleet_types::LabelType::BuiltIn {
            return Err(ServiceError::invalid_argument(
                "label_type",
                format!("cannot delete built-in label '{}'", name),
            ));
        }

        self.ds.delete_label(name).await?;

        info!(name = %name, "label deleted");
        Ok(())
    }

    /// Deletes a label by ID.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteLabelByID`.
    pub async fn delete_label_by_id(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<()> {
        let label = self.ds.label(id).await?;
        authz::authorize(viewer, Subject::Label, Action::Write)?;

        if label.label_type == fleet_types::LabelType::BuiltIn {
            return Err(ServiceError::invalid_argument(
                "label_type",
                format!("cannot delete built-in label '{}'", label.name),
            ));
        }
        if RESERVED_LABEL_NAMES.contains(&label.name.as_str()) {
            return Err(ServiceError::invalid_argument(
                "name",
                format!("cannot delete built-in label '{}'", label.name),
            ));
        }

        self.ds.delete_label(&label.name).await?;

        info!(label_id = id, name = %label.name, "label deleted by id");
        Ok(())
    }
}
