//! Certificate management service operations.
//!
//! Implements certificate template and certificate authority CRUD.

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceResult, Viewer};

impl FleetService {
    /// Creates a certificate template.
    pub async fn create_certificate_template(
        &self,
        viewer: &Viewer,
        team_id: u32,
        ca_id: i32,
        name: &str,
        subject_name: &str,
    ) -> ServiceResult<fleet_types::certificate::CertificateTemplate> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.create_certificate_template(team_id, ca_id, name, subject_name).await
    }

    /// Gets a certificate template by ID.
    pub async fn get_certificate_template(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::certificate::CertificateTemplate> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.get_certificate_template(id).await
    }

    /// Lists certificate templates.
    pub async fn list_certificate_templates(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<Vec<fleet_types::certificate::CertificateTemplate>> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.list_certificate_templates().await
    }

    /// Deletes a certificate template.
    pub async fn delete_certificate_template(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.delete_certificate_template(id).await
    }

    /// Creates a certificate authority.
    pub async fn create_certificate_authority(
        &self,
        viewer: &Viewer,
        ca_type: &str,
        name: &str,
        url: &str,
    ) -> ServiceResult<fleet_types::certificate::CertificateAuthority> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.create_certificate_authority(ca_type, name, url).await
    }

    /// Gets a certificate authority by ID.
    pub async fn get_certificate_authority(
        &self,
        viewer: &Viewer,
        id: i32,
    ) -> ServiceResult<fleet_types::certificate::CertificateAuthority> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.get_certificate_authority(id).await
    }

    /// Lists certificate authorities.
    pub async fn list_certificate_authorities(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<Vec<fleet_types::certificate::CertificateAuthority>> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.list_certificate_authorities().await
    }

    /// Deletes a certificate authority.
    pub async fn delete_certificate_authority(
        &self,
        viewer: &Viewer,
        id: i32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.delete_certificate_authority(id).await
    }

    /// Updates a certificate authority.
    pub async fn update_certificate_authority(
        &self,
        viewer: &Viewer,
        id: i32,
        name: &str,
        url: &str,
    ) -> ServiceResult<fleet_types::certificate::CertificateAuthority> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.update_certificate_authority(id, name, url).await
    }
}
