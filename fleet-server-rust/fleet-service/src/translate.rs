//! Translate service operations.
//!
//! Translates identifiers (email, name, hostname, etc.) to database IDs.
//! Corresponds to Go's `server/service/translator.go`.

use serde::{Deserialize, Serialize};

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

/// The type of entity to translate.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TranslateType {
    User,
    Label,
    Team,
    Host,
}

/// A single translate payload: type + identifier in, type + identifier + id out.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslatePayload {
    #[serde(rename = "type")]
    pub payload_type: TranslateType,
    pub payload: TranslateIdentifier,
}

/// The identifier/id pair within a translate payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslateIdentifier {
    pub identifier: String,
    #[serde(default)]
    pub id: u32,
}

impl FleetService {
    /// Translates a list of identifiers to their corresponding database IDs.
    ///
    /// Each payload specifies a type (user, label, team, host) and an identifier
    /// (email for users, name for labels/teams, identifier for hosts). The
    /// returned payloads have the `id` field populated.
    ///
    /// Corresponds to Go's `Service.Translate`.
    pub async fn translate_identifiers(
        &self,
        viewer: &Viewer,
        payloads: Vec<TranslatePayload>,
    ) -> ServiceResult<Vec<TranslatePayload>> {
        if payloads.is_empty() {
            return Err(ServiceError::bad_request(
                "payloads must not be empty",
            ));
        }

        let mut results = Vec::with_capacity(payloads.len());

        for payload in payloads {
            let id = match payload.payload_type {
                TranslateType::User => {
                    authz::authorize(viewer, Subject::User, Action::Read)?;
                    let user = self.ds.user_by_email(&payload.payload.identifier).await?;
                    user.id
                }
                TranslateType::Label => {
                    authz::authorize(viewer, Subject::Label, Action::Read)?;
                    let label = self.ds.label_by_name(&payload.payload.identifier).await?;
                    label.id
                }
                TranslateType::Team => {
                    authz::authorize(viewer, Subject::Team, Action::Read)?;
                    let team = self.ds.team_by_name(&payload.payload.identifier).await?;
                    team.id
                }
                TranslateType::Host => {
                    authz::authorize(viewer, Subject::Host, Action::Read)?;
                    let host = self.ds.host_by_identifier(&payload.payload.identifier).await?;
                    host.id
                }
            };

            results.push(TranslatePayload {
                payload_type: payload.payload_type,
                payload: TranslateIdentifier {
                    identifier: payload.payload.identifier,
                    id,
                },
            });
        }

        Ok(results)
    }
}
