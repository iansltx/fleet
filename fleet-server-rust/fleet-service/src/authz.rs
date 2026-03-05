//! Authorization module.
//!
//! Implements role-based access control (RBAC) matching the Go authorization
//! system in `server/authz/`. Determines whether a viewer (user + session)
//! is allowed to perform a specific action on a specific subject.
//!
//! The Go implementation uses OPA/Rego policies; this Rust implementation
//! uses a direct Rust translation of the same rules for simplicity and
//! performance.

use crate::{ServiceError, ServiceResult, Viewer};

/// Action represents operations that can be performed on resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Read access (list, get).
    Read,
    /// Write access (create, update, delete).
    Write,
    /// Create access (subset of write for some resources).
    Create,
    /// List access (subset of read).
    List,
    /// Run access (for queries).
    Run,
}

/// Subject represents the type of resource being accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    User,
    Session,
    Host,
    Label,
    Query,
    Pack,
    Policy,
    Team,
    AppConfig,
    Invite,
    EnrollSecret,
    Target,
    Activity,
    Carve,
    Software,
    Script,
}

/// The known Fleet roles, matching Go constants.
pub mod roles {
    pub const ADMIN: &str = "admin";
    pub const MAINTAINER: &str = "maintainer";
    pub const OBSERVER: &str = "observer";
    pub const OBSERVER_PLUS: &str = "observer_plus";
    pub const GITOPS: &str = "gitops";
    pub const TECHNICIAN: &str = "technician";
}

/// Checks if the viewer is authorized to perform the given action on the given subject.
///
/// Returns `Ok(())` if authorized, or `Err(ServiceError::Forbidden)` if not.
///
/// This matches the Go `authz.Authorize(ctx, subject, action)` pattern.
pub fn authorize(viewer: &Viewer, subject: Subject, action: Action) -> ServiceResult<()> {
    if is_authorized(viewer, subject, action) {
        Ok(())
    } else {
        Err(ServiceError::Forbidden(format!(
            "user {} is not authorized to {:?} {:?}",
            viewer.user.email, action, subject
        )))
    }
}

/// Determines whether the viewer is authorized based on role-based access control.
///
/// Rules are modeled after Go's OPA/Rego policies in `server/authz/policy.rego`.
fn is_authorized(viewer: &Viewer, subject: Subject, action: Action) -> bool {
    let user = &viewer.user;

    // Check global role first.
    if let Some(ref global_role) = user.global_role {
        return is_global_role_authorized(global_role, subject, action);
    }

    // No global role -- check team roles.
    // For team-agnostic resources, we check if any team role grants access.
    for team in &user.teams {
        if is_team_role_authorized(&team.role, subject, action) {
            return true;
        }
    }

    false
}

/// Authorization rules for global roles.
///
/// Global admin: full access to everything.
/// Global maintainer: read/write for most resources, no app config write.
/// Global observer: read-only for most resources.
/// Global observer_plus: observer + can run queries.
/// Global gitops: write access for configuration resources, no read for some.
fn is_global_role_authorized(role: &str, subject: Subject, action: Action) -> bool {
    match role {
        roles::ADMIN => {
            // Admin can do everything.
            true
        }

        roles::MAINTAINER => match subject {
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::User => matches!(action, Action::Read | Action::List),
            Subject::Invite => false,
            Subject::EnrollSecret => matches!(action, Action::Read),
            _ => {
                // Maintainer can read and write most resources.
                matches!(action, Action::Read | Action::Write | Action::Create | Action::List | Action::Run)
            }
        },

        roles::OBSERVER => match subject {
            Subject::Host | Subject::Label | Subject::Query | Subject::Pack
            | Subject::Policy | Subject::Team | Subject::Target
            | Subject::Software | Subject::Activity => {
                matches!(action, Action::Read | Action::List)
            }
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::Session => matches!(action, Action::Read | Action::Write),
            _ => false,
        },

        roles::OBSERVER_PLUS => match subject {
            Subject::Host | Subject::Label | Subject::Query | Subject::Pack
            | Subject::Policy | Subject::Team | Subject::Target
            | Subject::Software | Subject::Activity => {
                matches!(action, Action::Read | Action::List | Action::Run)
            }
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::Session => matches!(action, Action::Read | Action::Write),
            _ => false,
        },

        roles::GITOPS => match subject {
            Subject::AppConfig | Subject::Query | Subject::Pack | Subject::Policy
            | Subject::Team | Subject::Label | Subject::EnrollSecret => {
                matches!(action, Action::Write | Action::Create | Action::Read)
            }
            _ => false,
        },

        _ => false,
    }
}

/// Authorization rules for team roles.
///
/// Team roles apply to team-scoped resources. For team-agnostic checks
/// (like labels or app config), we allow if any team role grants it.
fn is_team_role_authorized(role: &str, subject: Subject, action: Action) -> bool {
    match role {
        roles::ADMIN => match subject {
            // Team admins can manage team-scoped resources.
            Subject::Host | Subject::Label | Subject::Query | Subject::Pack
            | Subject::Policy | Subject::Team | Subject::Target
            | Subject::Software | Subject::EnrollSecret => {
                matches!(action, Action::Read | Action::Write | Action::Create | Action::List | Action::Run)
            }
            Subject::User | Subject::Invite => {
                matches!(action, Action::Read | Action::Write | Action::List)
            }
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::Session => matches!(action, Action::Read | Action::Write),
            Subject::Activity => matches!(action, Action::Read | Action::List),
            _ => false,
        },

        roles::MAINTAINER => match subject {
            Subject::Host | Subject::Label | Subject::Query | Subject::Pack
            | Subject::Policy | Subject::Target | Subject::Software => {
                matches!(action, Action::Read | Action::Write | Action::Create | Action::List | Action::Run)
            }
            Subject::Team => matches!(action, Action::Read | Action::List),
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::Session => matches!(action, Action::Read | Action::Write),
            Subject::Activity => matches!(action, Action::Read | Action::List),
            _ => false,
        },

        roles::OBSERVER | roles::OBSERVER_PLUS => match subject {
            Subject::Host | Subject::Label | Subject::Query | Subject::Pack
            | Subject::Policy | Subject::Team | Subject::Target
            | Subject::Software | Subject::Activity => {
                let base = matches!(action, Action::Read | Action::List);
                if role == roles::OBSERVER_PLUS {
                    base || matches!(action, Action::Run)
                } else {
                    base
                }
            }
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::Session => matches!(action, Action::Read | Action::Write),
            _ => false,
        },

        roles::GITOPS => match subject {
            Subject::Query | Subject::Pack | Subject::Policy | Subject::Team
            | Subject::Label | Subject::EnrollSecret => {
                matches!(action, Action::Write | Action::Create | Action::Read)
            }
            _ => false,
        },

        roles::TECHNICIAN => match subject {
            Subject::Host => matches!(action, Action::Read | Action::List | Action::Write),
            Subject::Label | Subject::Query | Subject::Pack
            | Subject::Policy | Subject::Team | Subject::Software
            | Subject::Activity => {
                matches!(action, Action::Read | Action::List)
            }
            Subject::AppConfig => matches!(action, Action::Read),
            Subject::Session => matches!(action, Action::Read | Action::Write),
            _ => false,
        },

        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a minimal Team for use in UserTeam.
    fn make_test_team(id: u32, name: &str) -> fleet_types::Team {
        fleet_types::Team {
            id,
            gitops_filename: None,
            created_at: chrono::Utc::now(),
            name: name.to_string(),
            description: String::new(),
            config: Default::default(),
            user_count: 0,
            users: Vec::new(),
            host_count: 0,
            hosts: Vec::new(),
            secrets: None,
        }
    }

    fn make_viewer(global_role: Option<&str>, teams: Vec<(&str, &str)>) -> Viewer {
        let user = fleet_types::User {
            id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            password: Vec::new(),
            salt: String::new(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            admin_forced_password_reset: false,
            gravatar_url: String::new(),
            position: String::new(),
            sso_enabled: false,
            mfa_enabled: false,
            global_role: global_role.map(String::from),
            api_only: false,
            teams: teams
                .into_iter()
                .map(|(role, name)| fleet_types::team::UserTeam {
                    team: make_test_team(1, name),
                    role: role.to_string(),
                })
                .collect(),
            settings: None,
        };
        let session = fleet_types::Session {
            id: 1,
            created_at: chrono::Utc::now(),
            accessed_at: chrono::Utc::now(),
            user_id: 1,
            key: "test-key".to_string(),
            api_only: Some(false),
        };
        Viewer { user, session }
    }

    #[test]
    fn test_admin_can_do_everything() {
        let viewer = make_viewer(Some("admin"), vec![]);
        assert!(authorize(&viewer, Subject::User, Action::Write).is_ok());
        assert!(authorize(&viewer, Subject::Host, Action::Read).is_ok());
        assert!(authorize(&viewer, Subject::AppConfig, Action::Write).is_ok());
        assert!(authorize(&viewer, Subject::Policy, Action::Write).is_ok());
    }

    #[test]
    fn test_observer_read_only() {
        let viewer = make_viewer(Some("observer"), vec![]);
        assert!(authorize(&viewer, Subject::Host, Action::Read).is_ok());
        assert!(authorize(&viewer, Subject::Host, Action::Write).is_err());
        assert!(authorize(&viewer, Subject::User, Action::Write).is_err());
    }

    #[test]
    fn test_maintainer_no_app_config_write() {
        let viewer = make_viewer(Some("maintainer"), vec![]);
        assert!(authorize(&viewer, Subject::AppConfig, Action::Read).is_ok());
        assert!(authorize(&viewer, Subject::AppConfig, Action::Write).is_err());
        assert!(authorize(&viewer, Subject::Host, Action::Write).is_ok());
    }

    #[test]
    fn test_observer_plus_can_run_queries() {
        let viewer = make_viewer(Some("observer_plus"), vec![]);
        assert!(authorize(&viewer, Subject::Query, Action::Run).is_ok());
        assert!(authorize(&viewer, Subject::Query, Action::Write).is_err());
    }

    #[test]
    fn test_no_role_denied() {
        let viewer = make_viewer(None, vec![]);
        assert!(authorize(&viewer, Subject::Host, Action::Read).is_err());
    }

    #[test]
    fn test_team_admin() {
        let viewer = make_viewer(None, vec![("admin", "Team1")]);
        assert!(authorize(&viewer, Subject::Host, Action::Write).is_ok());
        assert!(authorize(&viewer, Subject::AppConfig, Action::Read).is_ok());
        assert!(authorize(&viewer, Subject::AppConfig, Action::Write).is_err());
    }

    #[test]
    fn test_team_observer() {
        let viewer = make_viewer(None, vec![("observer", "Team1")]);
        assert!(authorize(&viewer, Subject::Host, Action::Read).is_ok());
        assert!(authorize(&viewer, Subject::Host, Action::Write).is_err());
    }

    #[test]
    fn test_gitops_write_access() {
        let viewer = make_viewer(Some("gitops"), vec![]);
        assert!(authorize(&viewer, Subject::Query, Action::Write).is_ok());
        assert!(authorize(&viewer, Subject::Pack, Action::Write).is_ok());
        assert!(authorize(&viewer, Subject::Host, Action::Read).is_err());
    }
}
