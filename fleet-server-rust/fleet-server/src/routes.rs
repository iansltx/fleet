//! Route definitions for the Fleet API.
//!
//! Maps every route from the Go handler.go to axum routes, grouped by
//! authentication type: user-authenticated, device-authenticated,
//! host-authenticated, orbit-authenticated, and no-auth.

use axum::{
    routing::{delete, get, head, patch, post, put},
    Router,
};

use crate::frontend;
use crate::handlers;


/// Build the complete axum Router with all Fleet API routes.
pub fn build_router(state: crate::AppState) -> Router {
    let api_v1 = v1_routes();
    let api_v2 = v2_routes();
    let osquery = osquery_routes();
    let orbit = orbit_routes();
    let device = device_routes();
    let no_auth = no_auth_routes();
    let frontend = frontend::frontend_routes();

    Router::new()
        .merge(api_v1)
        .merge(api_v2)
        .merge(osquery)
        .merge(orbit)
        .merge(device)
        .merge(no_auth)
        .merge(frontend)
        .with_state(state)
        // Global middleware applied to all routes
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

// ---------------------------------------------------------------------------
// API v1 user-authenticated routes
// ---------------------------------------------------------------------------

/// User-authenticated endpoints under /api/v1/fleet/...
/// These correspond to the `ue` (user-authenticated endpointer) routes in handler.go.
fn v1_routes() -> Router<crate::AppState> {
    Router::new()
        // Trigger
        .route("/api/v1/fleet/trigger", post(handlers::app_config::trigger))
        // Session / Me
        .route("/api/v1/fleet/me", get(handlers::sessions::me))
        .route("/api/v1/fleet/sessions/{id}", get(handlers::sessions::get_session_info))
        .route("/api/v1/fleet/sessions/{id}", delete(handlers::sessions::delete_session))
        // Config / Certificate
        .route("/api/v1/fleet/config/certificate", get(handlers::app_config::get_certificate))
        .route("/api/v1/fleet/config", get(handlers::app_config::get_app_config))
        .route("/api/v1/fleet/config", patch(handlers::app_config::modify_app_config))
        .route("/api/v1/fleet/spec/enroll_secret", post(handlers::app_config::apply_enroll_secret_spec))
        .route("/api/v1/fleet/spec/enroll_secret", get(handlers::app_config::get_enroll_secret_spec))
        .route("/api/v1/fleet/version", get(handlers::app_config::version))
        // Users
        .route("/api/v1/fleet/users/roles/spec", post(handlers::users::apply_user_role_specs))
        .route("/api/v1/fleet/translate", post(handlers::app_config::translate))
        .route("/api/v1/fleet/users", get(handlers::users::list_users))
        .route("/api/v1/fleet/users/admin", post(handlers::users::create_user))
        .route("/api/v1/fleet/users/{id}", get(handlers::users::get_user))
        .route("/api/v1/fleet/users/{id}", patch(handlers::users::modify_user))
        .route("/api/v1/fleet/users/{id}", delete(handlers::users::delete_user))
        .route("/api/v1/fleet/users/{id}/require_password_reset", post(handlers::users::require_password_reset))
        .route("/api/v1/fleet/users/{id}/sessions", get(handlers::users::get_user_sessions))
        .route("/api/v1/fleet/users/{id}/sessions", delete(handlers::users::delete_user_sessions))
        .route("/api/v1/fleet/change_password", post(handlers::users::change_password))
        .route("/api/v1/fleet/email/change/{token}", get(handlers::users::change_email))
        // Targets
        .route("/api/v1/fleet/targets", post(handlers::hosts::search_targets))
        .route("/api/v1/fleet/targets/count", post(handlers::hosts::count_targets))
        // Invites
        .route("/api/v1/fleet/invites", post(handlers::invites::create_invite))
        .route("/api/v1/fleet/invites", get(handlers::invites::list_invites))
        .route("/api/v1/fleet/invites/{id}", delete(handlers::invites::delete_invite))
        .route("/api/v1/fleet/invites/{id}", patch(handlers::invites::update_invite))
        // Activities
        .route("/api/v1/fleet/activities", get(handlers::activities::list_activities))
        // Policies (v1 paths with /global/)
        .route("/api/v1/fleet/global/policies", post(handlers::policies::create_global_policy))
        .route("/api/v1/fleet/global/policies", get(handlers::policies::list_global_policies))
        .route("/api/v1/fleet/policies/count", get(handlers::policies::count_global_policies))
        .route("/api/v1/fleet/global/policies/{policy_id}", get(handlers::policies::get_policy))
        .route("/api/v1/fleet/global/policies/delete", post(handlers::policies::delete_global_policies))
        .route("/api/v1/fleet/global/policies/{policy_id}", patch(handlers::policies::modify_global_policy))
        .route("/api/v1/fleet/automations/reset", post(handlers::policies::reset_automation))
        // Team Policies
        .route("/api/v1/fleet/fleets/{fleet_id}/policies", post(handlers::policies::create_team_policy))
        .route("/api/v1/fleet/fleets/{fleet_id}/policies", get(handlers::policies::list_team_policies))
        .route("/api/v1/fleet/fleets/{fleet_id}/policies/count", get(handlers::policies::count_team_policies))
        .route("/api/v1/fleet/fleets/{fleet_id}/policies/{policy_id}", get(handlers::policies::get_team_policy))
        .route("/api/v1/fleet/fleets/{fleet_id}/policies/delete", post(handlers::policies::delete_team_policies))
        .route("/api/v1/fleet/fleets/{fleet_id}/policies/{policy_id}", patch(handlers::policies::modify_team_policy))
        .route("/api/v1/fleet/spec/policies", post(handlers::policies::apply_policy_specs))
        // Certificates
        .route("/api/v1/fleet/certificates", post(handlers::app_config::create_certificate_template))
        .route("/api/v1/fleet/certificates", get(handlers::app_config::list_certificate_templates))
        .route("/api/v1/fleet/certificates/{id}", get(handlers::app_config::get_certificate_template))
        .route("/api/v1/fleet/certificates/{id}", delete(handlers::app_config::delete_certificate_template))
        .route("/api/v1/fleet/spec/certificates", post(handlers::app_config::apply_certificate_template_specs))
        .route("/api/v1/fleet/spec/certificates", delete(handlers::app_config::delete_certificate_template_specs))
        // Queries / Reports
        .route("/api/v1/fleet/reports/{id}", get(handlers::queries::get_query))
        .route("/api/v1/fleet/reports", get(handlers::queries::list_queries))
        .route("/api/v1/fleet/reports/{id}/report", get(handlers::queries::get_query_report))
        .route("/api/v1/fleet/reports", post(handlers::queries::create_query))
        .route("/api/v1/fleet/reports/{id}", patch(handlers::queries::modify_query))
        .route("/api/v1/fleet/reports/{name}", delete(handlers::queries::delete_query))
        .route("/api/v1/fleet/reports/id/{id}", delete(handlers::queries::delete_query_by_id))
        .route("/api/v1/fleet/reports/delete", post(handlers::queries::delete_queries))
        .route("/api/v1/fleet/spec/reports", post(handlers::queries::apply_query_specs))
        .route("/api/v1/fleet/spec/reports", get(handlers::queries::get_query_specs))
        .route("/api/v1/fleet/spec/reports/{name}", get(handlers::queries::get_query_spec))
        // Live queries
        .route("/api/v1/fleet/reports/{id}/run", post(handlers::queries::run_one_live_query))
        .route("/api/v1/fleet/reports/run", get(handlers::queries::run_live_query))
        .route("/api/v1/fleet/reports/run_by_identifiers", post(handlers::queries::create_distributed_query_campaign_by_identifier))
        .route("/api/v1/fleet/reports/run_by_names", post(handlers::queries::create_distributed_query_campaign_by_identifier))
        // Packs
        .route("/api/v1/fleet/packs/{id}", get(handlers::packs::get_pack))
        .route("/api/v1/fleet/packs", post(handlers::packs::create_pack))
        .route("/api/v1/fleet/packs/{id}", patch(handlers::packs::modify_pack))
        .route("/api/v1/fleet/packs", get(handlers::packs::list_packs))
        .route("/api/v1/fleet/packs/{name}", delete(handlers::packs::delete_pack))
        .route("/api/v1/fleet/packs/id/{id}", delete(handlers::packs::delete_pack_by_id))
        .route("/api/v1/fleet/spec/packs", post(handlers::packs::apply_pack_specs))
        .route("/api/v1/fleet/spec/packs", get(handlers::packs::get_pack_specs))
        .route("/api/v1/fleet/spec/packs/{name}", get(handlers::packs::get_pack_spec))
        // Scheduled queries in packs
        .route("/api/v1/fleet/packs/{id}/scheduled", get(handlers::packs::get_scheduled_queries_in_pack))
        .route("/api/v1/fleet/schedule", post(handlers::packs::schedule_query))
        .route("/api/v1/fleet/schedule/{id}", get(handlers::packs::get_scheduled_query))
        .route("/api/v1/fleet/schedule/{id}", patch(handlers::packs::modify_scheduled_query))
        .route("/api/v1/fleet/schedule/{id}", delete(handlers::packs::delete_scheduled_query))
        // Global schedule (v1 paths)
        .route("/api/v1/fleet/global/schedule", get(handlers::packs::get_global_schedule))
        .route("/api/v1/fleet/global/schedule", post(handlers::packs::global_schedule_query))
        .route("/api/v1/fleet/global/schedule/{id}", patch(handlers::packs::modify_global_schedule))
        .route("/api/v1/fleet/global/schedule/{id}", delete(handlers::packs::delete_global_schedule))
        // Team schedule
        .route("/api/v1/fleet/fleets/{fleet_id}/schedule", get(handlers::packs::get_team_schedule))
        .route("/api/v1/fleet/fleets/{fleet_id}/schedule", post(handlers::packs::team_schedule_query))
        .route("/api/v1/fleet/fleets/{fleet_id}/schedule/{report_id}", patch(handlers::packs::modify_team_schedule))
        .route("/api/v1/fleet/fleets/{fleet_id}/schedule/{report_id}", delete(handlers::packs::delete_team_schedule))
        // Software
        .route("/api/v1/fleet/software/versions", get(handlers::software::list_software_versions))
        .route("/api/v1/fleet/software/versions/{id}", get(handlers::software::get_software))
        .route("/api/v1/fleet/software", get(handlers::software::list_software))
        .route("/api/v1/fleet/software/{id}", get(handlers::software::get_software))
        .route("/api/v1/fleet/software/count", get(handlers::software::count_software))
        .route("/api/v1/fleet/software/titles", get(handlers::software::list_software_titles))
        .route("/api/v1/fleet/software/titles/{id}", get(handlers::software::get_software_title))
        .route("/api/v1/fleet/hosts/{host_id}/software/{software_title_id}/install", post(handlers::software::install_software_title))
        .route("/api/v1/fleet/hosts/{host_id}/software/{software_title_id}/uninstall", post(handlers::software::uninstall_software_title))
        // Software installers
        .route("/api/v1/fleet/software/titles/{title_id}/package", get(handlers::software::get_software_installer))
        .route("/api/v1/fleet/software/titles/{title_id}/package/token", post(handlers::software::get_software_installer_token))
        .route("/api/v1/fleet/software/package", post(handlers::software::upload_software_installer))
        .route("/api/v1/fleet/software/titles/{id}/name", patch(handlers::software::update_software_name))
        .route("/api/v1/fleet/software/titles/{id}/package", patch(handlers::software::update_software_installer))
        .route("/api/v1/fleet/software/titles/{title_id}/available_for_install", delete(handlers::software::delete_software_installer))
        .route("/api/v1/fleet/software/install/{install_uuid}/results", get(handlers::software::get_software_install_results))
        .route("/api/v1/fleet/software/batch", post(handlers::software::batch_set_software_installers))
        .route("/api/v1/fleet/software/batch/{request_uuid}", get(handlers::software::batch_set_software_installers_result))
        // Software icons
        .route("/api/v1/fleet/software/titles/{title_id}/icon", get(handlers::software::get_software_title_icon))
        .route("/api/v1/fleet/software/titles/{title_id}/icon", put(handlers::software::put_software_title_icon))
        .route("/api/v1/fleet/software/titles/{title_id}/icon", delete(handlers::software::delete_software_title_icon))
        // App store software
        .route("/api/v1/fleet/software/app_store_apps", get(handlers::software::get_app_store_apps))
        .route("/api/v1/fleet/software/app_store_apps", post(handlers::software::add_app_store_app))
        .route("/api/v1/fleet/software/titles/{title_id}/app_store_app", patch(handlers::software::update_app_store_app))
        // Fleet-maintained apps
        .route("/api/v1/fleet/software/fleet_maintained_apps", post(handlers::software::add_fleet_maintained_app))
        .route("/api/v1/fleet/software/fleet_maintained_apps", get(handlers::software::list_fleet_maintained_apps))
        .route("/api/v1/fleet/software/fleet_maintained_apps/{app_id}", get(handlers::software::get_fleet_maintained_app))
        // Software batch VPP
        .route("/api/v1/fleet/software/app_store_apps/batch", post(handlers::software::batch_associate_app_store_apps))
        // Android web apps
        .route("/api/v1/fleet/software/web_apps", post(handlers::software::create_android_web_app))
        // Setup Experience
        .route("/api/v1/fleet/setup_experience/software", put(handlers::setup::put_setup_experience_software))
        .route("/api/v1/fleet/setup_experience/software", get(handlers::setup::get_setup_experience_software))
        .route("/api/v1/fleet/setup_experience/script", get(handlers::setup::get_setup_experience_script))
        .route("/api/v1/fleet/setup_experience/script", post(handlers::setup::set_setup_experience_script))
        .route("/api/v1/fleet/setup_experience/script", delete(handlers::setup::delete_setup_experience_script))
        .route("/api/v1/fleet/setup_experience", patch(handlers::mdm::update_mdm_apple_setup))
        // Vulnerabilities
        .route("/api/v1/fleet/vulnerabilities", get(handlers::software::list_vulnerabilities))
        .route("/api/v1/fleet/vulnerabilities/{cve}", get(handlers::software::get_vulnerability))
        // Hosts
        .route("/api/v1/fleet/host_summary", get(handlers::hosts::get_host_summary))
        .route("/api/v1/fleet/hosts", get(handlers::hosts::list_hosts))
        .route("/api/v1/fleet/hosts/delete", post(handlers::hosts::delete_hosts))
        .route("/api/v1/fleet/hosts/{id}", get(handlers::hosts::get_host))
        .route("/api/v1/fleet/hosts/count", get(handlers::hosts::count_hosts))
        .route("/api/v1/fleet/hosts/search", post(handlers::hosts::search_hosts))
        .route("/api/v1/fleet/hosts/identifier/{identifier}", get(handlers::hosts::host_by_identifier))
        .route("/api/v1/fleet/hosts/identifier/{identifier}/query", post(handlers::hosts::run_live_query_on_host))
        .route("/api/v1/fleet/hosts/{id}/query", post(handlers::hosts::run_live_query_on_host_by_id))
        .route("/api/v1/fleet/hosts/{id}", delete(handlers::hosts::delete_host))
        .route("/api/v1/fleet/hosts/transfer", post(handlers::hosts::add_hosts_to_team))
        .route("/api/v1/fleet/hosts/transfer/filter", post(handlers::hosts::add_hosts_to_team_by_filter))
        .route("/api/v1/fleet/hosts/{id}/refetch", post(handlers::hosts::refetch_host))
        .route("/api/v1/fleet/hosts/{id}/device_mapping", get(handlers::hosts::list_host_device_mapping))
        .route("/api/v1/fleet/hosts/{id}/device_mapping", put(handlers::hosts::put_host_device_mapping))
        .route("/api/v1/fleet/hosts/{id}/device_mapping/idp", delete(handlers::hosts::delete_host_idp))
        .route("/api/v1/fleet/hosts/report", get(handlers::hosts::hosts_report))
        .route("/api/v1/fleet/os_versions", get(handlers::hosts::os_versions))
        .route("/api/v1/fleet/os_versions/{id}", get(handlers::hosts::get_os_version))
        .route("/api/v1/fleet/hosts/{id}/reports/{report_id}", get(handlers::hosts::get_host_query_report))
        .route("/api/v1/fleet/hosts/{id}/health", get(handlers::hosts::get_host_health))
        .route("/api/v1/fleet/hosts/{id}/labels", post(handlers::hosts::add_labels_to_host))
        .route("/api/v1/fleet/hosts/{id}/labels", delete(handlers::hosts::remove_labels_from_host))
        .route("/api/v1/fleet/hosts/{id}/software", get(handlers::hosts::get_host_software))
        .route("/api/v1/fleet/hosts/{id}/certificates", get(handlers::hosts::list_host_certificates))
        .route("/api/v1/fleet/hosts/summary/mdm", get(handlers::hosts::get_host_mdm_summary))
        .route("/api/v1/fleet/hosts/{id}/mdm", get(handlers::hosts::get_host_mdm))
        .route("/api/v1/fleet/hosts/{id}/macadmins", get(handlers::hosts::get_macadmins_data))
        .route("/api/v1/fleet/macadmins", get(handlers::hosts::get_aggregated_macadmins_data))
        .route("/api/v1/fleet/hosts/{id}/scripts", get(handlers::hosts::get_host_script_details))
        .route("/api/v1/fleet/hosts/{id}/activities/upcoming", get(handlers::hosts::list_host_upcoming_activities))
        .route("/api/v1/fleet/hosts/{id}/activities/upcoming/{activity_id}", delete(handlers::hosts::cancel_host_upcoming_activity))
        .route("/api/v1/fleet/hosts/{id}/lock", post(handlers::hosts::lock_host))
        .route("/api/v1/fleet/hosts/{id}/unlock", post(handlers::hosts::unlock_host))
        .route("/api/v1/fleet/hosts/{id}/wipe", post(handlers::hosts::wipe_host))
        // Labels
        .route("/api/v1/fleet/labels", post(handlers::labels::create_label))
        .route("/api/v1/fleet/labels/{id}", patch(handlers::labels::modify_label))
        .route("/api/v1/fleet/labels/{id}", get(handlers::labels::get_label))
        .route("/api/v1/fleet/labels", get(handlers::labels::list_labels))
        .route("/api/v1/fleet/labels/summary", get(handlers::labels::get_labels_summary))
        .route("/api/v1/fleet/labels/{id}/hosts", get(handlers::labels::list_hosts_in_label))
        .route("/api/v1/fleet/labels/{name}", delete(handlers::labels::delete_label))
        .route("/api/v1/fleet/labels/id/{id}", delete(handlers::labels::delete_label_by_id))
        .route("/api/v1/fleet/spec/labels", post(handlers::labels::apply_label_specs))
        .route("/api/v1/fleet/spec/labels", get(handlers::labels::get_label_specs))
        .route("/api/v1/fleet/spec/labels/{name}", get(handlers::labels::get_label_spec))
        // Teams
        .route("/api/v1/fleet/spec/fleets", post(handlers::teams::apply_team_specs))
        .route("/api/v1/fleet/fleets/{fleet_id}/secrets", patch(handlers::teams::modify_team_enroll_secrets))
        .route("/api/v1/fleet/fleets", post(handlers::teams::create_team))
        .route("/api/v1/fleet/fleets", get(handlers::teams::list_teams))
        .route("/api/v1/fleet/fleets/{id}", get(handlers::teams::get_team))
        .route("/api/v1/fleet/fleets/{id}", patch(handlers::teams::modify_team))
        .route("/api/v1/fleet/fleets/{id}", delete(handlers::teams::delete_team))
        .route("/api/v1/fleet/fleets/{id}/agent_options", post(handlers::teams::modify_team_agent_options))
        .route("/api/v1/fleet/fleets/{id}/users", get(handlers::teams::list_team_users))
        .route("/api/v1/fleet/fleets/{id}/users", patch(handlers::teams::add_team_users))
        .route("/api/v1/fleet/fleets/{id}/users", delete(handlers::teams::delete_team_users))
        .route("/api/v1/fleet/fleets/{id}/secrets", get(handlers::teams::team_enroll_secrets))
        // Carves
        .route("/api/v1/fleet/carves", get(handlers::carves::list_carves))
        .route("/api/v1/fleet/carves/{id}", get(handlers::carves::get_carve))
        .route("/api/v1/fleet/carves/{id}/block/{block_id}", get(handlers::carves::get_carve_block))
        // Status
        .route("/api/v1/fleet/status/result_store", get(handlers::app_config::status_result_store))
        .route("/api/v1/fleet/status/live_query", get(handlers::app_config::status_live_query))
        // Scripts
        .route("/api/v1/fleet/scripts/run", post(handlers::scripts::run_script))
        .route("/api/v1/fleet/scripts/run/sync", post(handlers::scripts::run_script_sync))
        .route("/api/v1/fleet/scripts/run/batch", post(handlers::scripts::batch_script_run))
        .route("/api/v1/fleet/scripts/results/{execution_id}", get(handlers::scripts::get_script_result))
        .route("/api/v1/fleet/scripts", post(handlers::scripts::create_script))
        .route("/api/v1/fleet/scripts", get(handlers::scripts::list_scripts))
        .route("/api/v1/fleet/scripts/{script_id}", get(handlers::scripts::get_script))
        .route("/api/v1/fleet/scripts/{script_id}", patch(handlers::scripts::update_script))
        .route("/api/v1/fleet/scripts/{script_id}", delete(handlers::scripts::delete_script))
        .route("/api/v1/fleet/scripts/batch", post(handlers::scripts::batch_set_scripts))
        .route("/api/v1/fleet/scripts/batch/{batch_execution_id}/cancel", post(handlers::scripts::batch_script_cancel))
        .route("/api/v1/fleet/scripts/batch/summary/{batch_execution_id}", get(handlers::scripts::batch_script_execution_summary))
        .route("/api/v1/fleet/scripts/batch/{batch_execution_id}/host-results", get(handlers::scripts::batch_script_execution_host_results))
        .route("/api/v1/fleet/scripts/batch/{batch_execution_id}", get(handlers::scripts::batch_script_execution_status))
        // Generative AI
        .route("/api/v1/fleet/autofill/policy", post(handlers::policies::autofill_policies))
        // Secret variables
        .route("/api/v1/fleet/spec/secret_variables", put(handlers::app_config::create_secret_variables))
        .route("/api/v1/fleet/custom_variables", post(handlers::app_config::create_secret_variable))
        .route("/api/v1/fleet/custom_variables", get(handlers::app_config::list_secret_variables))
        .route("/api/v1/fleet/custom_variables/{id}", delete(handlers::app_config::delete_secret_variable))
        // SCIM
        .route("/api/v1/fleet/scim/details", get(handlers::app_config::get_scim_details))
        // Microsoft Conditional Access
        .route("/api/v1/fleet/conditional-access/microsoft", post(handlers::app_config::conditional_access_microsoft_create))
        .route("/api/v1/fleet/conditional-access/microsoft/confirm", post(handlers::app_config::conditional_access_microsoft_confirm))
        .route("/api/v1/fleet/conditional-access/microsoft", delete(handlers::app_config::conditional_access_microsoft_delete))
        // Okta Conditional Access
        .route("/api/v1/fleet/conditional_access/idp/signing_cert", get(handlers::app_config::conditional_access_get_idp_signing_cert))
        .route("/api/v1/fleet/conditional_access/idp/apple/profile", get(handlers::app_config::conditional_access_get_idp_apple_profile))
        // MDM (user-authenticated)
        .route("/api/v1/fleet/mdm/apple/setup", patch(handlers::mdm::update_mdm_apple_setup))
        .route("/api/v1/fleet/mdm/apple/enqueue", post(handlers::mdm::enqueue_mdm_apple_command))
        .route("/api/v1/fleet/mdm/apple/commandresults", get(handlers::mdm::get_mdm_apple_command_results))
        .route("/api/v1/fleet/mdm/apple/commands", get(handlers::mdm::list_mdm_apple_commands))
        .route("/api/v1/fleet/mdm/apple/profiles/{profile_id}", get(handlers::mdm::get_mdm_apple_config_profile))
        .route("/api/v1/fleet/mdm/apple/profiles/{profile_id}", delete(handlers::mdm::delete_mdm_apple_config_profile))
        .route("/api/v1/fleet/mdm/apple/profiles", post(handlers::mdm::new_mdm_apple_config_profile))
        .route("/api/v1/fleet/mdm/apple/profiles", get(handlers::mdm::list_mdm_apple_config_profiles))
        .route("/api/v1/fleet/mdm/apple/filevault/summary", get(handlers::mdm::get_mdm_apple_filevault_summary))
        .route("/api/v1/fleet/mdm/apple/profiles/summary", get(handlers::mdm::get_mdm_apple_profiles_summary))
        .route("/api/v1/fleet/mdm/apple/enrollment_profile", post(handlers::mdm::create_mdm_apple_setup_assistant))
        .route("/api/v1/fleet/enrollment_profiles/automatic", post(handlers::mdm::create_mdm_apple_setup_assistant))
        .route("/api/v1/fleet/mdm/apple/enrollment_profile", get(handlers::mdm::get_mdm_apple_setup_assistant))
        .route("/api/v1/fleet/enrollment_profiles/automatic", get(handlers::mdm::get_mdm_apple_setup_assistant))
        .route("/api/v1/fleet/mdm/apple/enrollment_profile", delete(handlers::mdm::delete_mdm_apple_setup_assistant))
        .route("/api/v1/fleet/enrollment_profiles/automatic", delete(handlers::mdm::delete_mdm_apple_setup_assistant))
        .route("/api/v1/fleet/mdm/apple/installers", post(handlers::mdm::upload_apple_installer))
        .route("/api/v1/fleet/mdm/apple/installers/{installer_id}", get(handlers::mdm::get_apple_installer))
        .route("/api/v1/fleet/mdm/apple/installers/{installer_id}", delete(handlers::mdm::delete_apple_installer))
        .route("/api/v1/fleet/mdm/apple/installers", get(handlers::mdm::list_mdm_apple_installers))
        .route("/api/v1/fleet/mdm/apple/devices", get(handlers::mdm::list_mdm_apple_devices))
        .route("/api/v1/fleet/mdm/manual_enrollment_profile", get(handlers::mdm::get_manual_enrollment_profile))
        .route("/api/v1/fleet/enrollment_profiles/manual", get(handlers::mdm::get_manual_enrollment_profile))
        // Bootstrap package
        .route("/api/v1/fleet/mdm/bootstrap", post(handlers::mdm::upload_bootstrap_package))
        .route("/api/v1/fleet/bootstrap", post(handlers::mdm::upload_bootstrap_package))
        .route("/api/v1/fleet/mdm/bootstrap/{fleet_id}/metadata", get(handlers::mdm::bootstrap_package_metadata))
        .route("/api/v1/fleet/bootstrap/{fleet_id}/metadata", get(handlers::mdm::bootstrap_package_metadata))
        .route("/api/v1/fleet/mdm/bootstrap/{fleet_id}", delete(handlers::mdm::delete_bootstrap_package))
        .route("/api/v1/fleet/bootstrap/{fleet_id}", delete(handlers::mdm::delete_bootstrap_package))
        .route("/api/v1/fleet/mdm/bootstrap/summary", get(handlers::mdm::get_bootstrap_package_summary))
        .route("/api/v1/fleet/bootstrap/summary", get(handlers::mdm::get_bootstrap_package_summary))
        .route("/api/v1/fleet/mdm/apple/bootstrap", post(handlers::mdm::upload_bootstrap_package))
        .route("/api/v1/fleet/mdm/apple/bootstrap/{fleet_id}/metadata", get(handlers::mdm::bootstrap_package_metadata))
        .route("/api/v1/fleet/mdm/apple/bootstrap/{fleet_id}", delete(handlers::mdm::delete_bootstrap_package))
        .route("/api/v1/fleet/mdm/apple/bootstrap/summary", get(handlers::mdm::get_bootstrap_package_summary))
        // Host-specific MDM
        .route("/api/v1/fleet/mdm/hosts/{id}/lock", post(handlers::mdm::device_lock))
        .route("/api/v1/fleet/mdm/hosts/{id}/wipe", post(handlers::mdm::device_wipe))
        .route("/api/v1/fleet/mdm/hosts/{id}/profiles", get(handlers::mdm::get_host_profiles))
        .route("/api/v1/fleet/hosts/{id}/configuration_profiles", get(handlers::mdm::get_host_profiles))
        .route("/api/v1/fleet/mdm/apple", get(handlers::mdm::get_apple_mdm))
        .route("/api/v1/fleet/apns", get(handlers::mdm::get_apple_mdm))
        // EULA
        .route("/api/v1/fleet/mdm/setup/eula", post(handlers::mdm::create_mdm_eula))
        .route("/api/v1/fleet/setup_experience/eula", post(handlers::mdm::create_mdm_eula))
        .route("/api/v1/fleet/mdm/setup/eula/metadata", get(handlers::mdm::get_mdm_eula_metadata))
        .route("/api/v1/fleet/setup_experience/eula/metadata", get(handlers::mdm::get_mdm_eula_metadata))
        .route("/api/v1/fleet/mdm/setup/eula/{token}", delete(handlers::mdm::delete_mdm_eula))
        .route("/api/v1/fleet/setup_experience/eula/{token}", delete(handlers::mdm::delete_mdm_eula))
        .route("/api/v1/fleet/mdm/apple/setup/eula", post(handlers::mdm::create_mdm_eula))
        .route("/api/v1/fleet/mdm/apple/setup/eula/metadata", get(handlers::mdm::get_mdm_eula_metadata))
        .route("/api/v1/fleet/mdm/apple/setup/eula/{token}", delete(handlers::mdm::delete_mdm_eula))
        // MDM preassign
        .route("/api/v1/fleet/mdm/apple/profiles/preassign", post(handlers::mdm::preassign_mdm_apple_profile))
        .route("/api/v1/fleet/mdm/apple/profiles/match", post(handlers::mdm::match_mdm_apple_preassignment))
        // Platform-agnostic MDM
        .route("/api/v1/fleet/mdm/commands/run", post(handlers::mdm::run_mdm_command))
        .route("/api/v1/fleet/commands/run", post(handlers::mdm::run_mdm_command))
        .route("/api/v1/fleet/mdm/commandresults", get(handlers::mdm::get_mdm_command_results))
        .route("/api/v1/fleet/commands/results", get(handlers::mdm::get_mdm_command_results))
        .route("/api/v1/fleet/mdm/commands", get(handlers::mdm::list_mdm_commands))
        .route("/api/v1/fleet/commands", get(handlers::mdm::list_mdm_commands))
        .route("/api/v1/fleet/mdm/hosts/{id}/unenroll", patch(handlers::mdm::mdm_unenroll))
        .route("/api/v1/fleet/hosts/{id}/mdm", delete(handlers::mdm::mdm_unenroll))
        // Disk encryption
        .route("/api/v1/fleet/mdm/disk_encryption/summary", get(handlers::mdm::get_mdm_disk_encryption_summary))
        .route("/api/v1/fleet/disk_encryption", get(handlers::mdm::get_mdm_disk_encryption_summary))
        .route("/api/v1/fleet/mdm/hosts/{id}/encryption_key", get(handlers::mdm::get_host_encryption_key))
        .route("/api/v1/fleet/hosts/{id}/encryption_key", get(handlers::mdm::get_host_encryption_key))
        // Profile summary
        .route("/api/v1/fleet/mdm/profiles/summary", get(handlers::mdm::get_mdm_profiles_summary))
        .route("/api/v1/fleet/configuration_profiles/summary", get(handlers::mdm::get_mdm_profiles_summary))
        // Configuration profiles (platform-agnostic)
        .route("/api/v1/fleet/mdm/profiles/{profile_uuid}", get(handlers::mdm::get_mdm_config_profile))
        .route("/api/v1/fleet/configuration_profiles/{profile_uuid}", get(handlers::mdm::get_mdm_config_profile))
        .route("/api/v1/fleet/mdm/profiles/{profile_uuid}", delete(handlers::mdm::delete_mdm_config_profile))
        .route("/api/v1/fleet/configuration_profiles/{profile_uuid}", delete(handlers::mdm::delete_mdm_config_profile))
        .route("/api/v1/fleet/mdm/profiles", get(handlers::mdm::list_mdm_config_profiles))
        .route("/api/v1/fleet/configuration_profiles", get(handlers::mdm::list_mdm_config_profiles))
        .route("/api/v1/fleet/mdm/profiles", post(handlers::mdm::new_mdm_config_profile))
        .route("/api/v1/fleet/configuration_profiles", post(handlers::mdm::new_mdm_config_profile))
        .route("/api/v1/fleet/configuration_profiles/batch", post(handlers::mdm::batch_modify_mdm_config_profiles))
        .route("/api/v1/fleet/hosts/{host_id}/configuration_profiles/resend/{profile_uuid}", post(handlers::mdm::resend_host_mdm_profile))
        .route("/api/v1/fleet/hosts/{host_id}/configuration_profiles/{profile_uuid}/resend", post(handlers::mdm::resend_host_mdm_profile))
        .route("/api/v1/fleet/configuration_profiles/resend/batch", post(handlers::mdm::batch_resend_mdm_profile_to_hosts))
        .route("/api/v1/fleet/configuration_profiles/{profile_uuid}/status", get(handlers::mdm::get_mdm_config_profile_status))
        // MDM settings
        .route("/api/v1/fleet/mdm/apple/settings", patch(handlers::mdm::update_mdm_apple_settings))
        .route("/api/v1/fleet/disk_encryption", post(handlers::mdm::update_disk_encryption))
        // MDM CSR/APNs
        .route("/api/v1/fleet/mdm/apple/request_csr", post(handlers::mdm::request_mdm_apple_csr))
        .route("/api/v1/fleet/mdm/apple/request_csr", get(handlers::mdm::get_mdm_apple_csr))
        .route("/api/v1/fleet/mdm/apple/dep/key_pair", post(handlers::mdm::new_mdm_apple_dep_key_pair))
        .route("/api/v1/fleet/mdm/apple/abm_public_key", get(handlers::mdm::generate_abm_key_pair))
        .route("/api/v1/fleet/mdm/apple/apns_certificate", post(handlers::mdm::upload_mdm_apple_apns_cert))
        .route("/api/v1/fleet/mdm/apple/apns_certificate", delete(handlers::mdm::delete_mdm_apple_apns_cert))
        // ABM tokens
        .route("/api/v1/fleet/abm_tokens", post(handlers::mdm::upload_abm_token))
        .route("/api/v1/fleet/abm_tokens/{id}", delete(handlers::mdm::delete_abm_token))
        .route("/api/v1/fleet/abm_tokens", get(handlers::mdm::list_abm_tokens))
        .route("/api/v1/fleet/abm_tokens/count", get(handlers::mdm::count_abm_tokens))
        .route("/api/v1/fleet/abm_tokens/{id}/fleets", patch(handlers::mdm::update_abm_token_teams))
        .route("/api/v1/fleet/abm_tokens/{id}/renew", patch(handlers::mdm::renew_abm_token))
        // VPP tokens
        .route("/api/v1/fleet/vpp_tokens", get(handlers::mdm::get_vpp_tokens))
        .route("/api/v1/fleet/vpp_tokens", post(handlers::mdm::upload_vpp_token))
        .route("/api/v1/fleet/vpp_tokens/{id}/fleets", patch(handlers::mdm::patch_vpp_tokens_teams))
        .route("/api/v1/fleet/vpp_tokens/{id}/renew", patch(handlers::mdm::patch_vpp_token_renew))
        .route("/api/v1/fleet/vpp_tokens/{id}", delete(handlers::mdm::delete_vpp_token))
        // ABM (deprecated)
        .route("/api/v1/fleet/mdm/apple_bm", get(handlers::mdm::get_apple_bm))
        .route("/api/v1/fleet/abm", get(handlers::mdm::get_apple_bm))
        // Batch profiles (deprecated Apple-specific)
        .route("/api/v1/fleet/mdm/apple/profiles/batch", post(handlers::mdm::batch_set_mdm_apple_profiles))
        .route("/api/v1/fleet/mdm/profiles/batch", post(handlers::mdm::batch_set_mdm_profiles))
        // Certificate Authorities
        .route("/api/v1/fleet/certificate_authorities", post(handlers::app_config::create_certificate_authority))
        .route("/api/v1/fleet/certificate_authorities", get(handlers::app_config::list_certificate_authorities))
        .route("/api/v1/fleet/certificate_authorities/{id}", get(handlers::app_config::get_certificate_authority))
        .route("/api/v1/fleet/certificate_authorities/{id}", delete(handlers::app_config::delete_certificate_authority))
        .route("/api/v1/fleet/certificate_authorities/{id}", patch(handlers::app_config::update_certificate_authority))
        .route("/api/v1/fleet/certificate_authorities/{id}/request_certificate", post(handlers::app_config::request_certificate))
        .route("/api/v1/fleet/spec/certificate_authorities", post(handlers::app_config::batch_apply_certificate_authorities))
        .route("/api/v1/fleet/spec/certificate_authorities", get(handlers::app_config::get_certificate_authorities_spec))
}

// ---------------------------------------------------------------------------
// API 2022-04 user-authenticated routes (version 2 paths)
// ---------------------------------------------------------------------------

/// Routes that were introduced or moved in the 2022-04 API version.
fn v2_routes() -> Router<crate::AppState> {
    Router::new()
        // Policies (2022-04 paths without /global/)
        .route("/api/2022-04/fleet/policies", post(handlers::policies::create_global_policy))
        .route("/api/2022-04/fleet/policies", get(handlers::policies::list_global_policies))
        .route("/api/2022-04/fleet/policies/count", get(handlers::policies::count_global_policies))
        .route("/api/2022-04/fleet/policies/{policy_id}", get(handlers::policies::get_policy))
        .route("/api/2022-04/fleet/policies/delete", post(handlers::policies::delete_global_policies))
        .route("/api/2022-04/fleet/policies/{policy_id}", patch(handlers::policies::modify_global_policy))
        // Schedule (2022-04 paths)
        .route("/api/2022-04/fleet/packs/schedule", post(handlers::packs::schedule_query))
        .route("/api/2022-04/fleet/packs/schedule/{id}", patch(handlers::packs::modify_scheduled_query))
        .route("/api/2022-04/fleet/packs/schedule/{id}", delete(handlers::packs::delete_scheduled_query))
        // Global schedule (2022-04 paths)
        .route("/api/2022-04/fleet/schedule", get(handlers::packs::get_global_schedule))
        .route("/api/2022-04/fleet/schedule", post(handlers::packs::global_schedule_query))
        .route("/api/2022-04/fleet/schedule/{id}", patch(handlers::packs::modify_global_schedule))
        .route("/api/2022-04/fleet/schedule/{id}", delete(handlers::packs::delete_global_schedule))
}

// ---------------------------------------------------------------------------
// Osquery / Host-authenticated routes
// ---------------------------------------------------------------------------

/// Host-authenticated osquery endpoints.
/// These use the osquery node_key for authentication.
fn osquery_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/api/osquery/config", post(handlers::osquery::get_client_config))
        .route("/api/v1/osquery/config", post(handlers::osquery::get_client_config))
        .route("/api/osquery/distributed/read", post(handlers::osquery::get_distributed_queries))
        .route("/api/v1/osquery/distributed/read", post(handlers::osquery::get_distributed_queries))
        .route("/api/osquery/distributed/write", post(handlers::osquery::submit_distributed_query_results))
        .route("/api/v1/osquery/distributed/write", post(handlers::osquery::submit_distributed_query_results))
        .route("/api/osquery/carve/begin", post(handlers::carves::carve_begin))
        .route("/api/v1/osquery/carve/begin", post(handlers::carves::carve_begin))
        .route("/api/osquery/log", post(handlers::osquery::submit_logs))
        .route("/api/v1/osquery/log", post(handlers::osquery::submit_logs))
        .route("/api/osquery/yara/{name}", post(handlers::osquery::get_yara))
        .route("/api/v1/osquery/yara/{name}", post(handlers::osquery::get_yara))
        // Enroll (unauthenticated)
        .route("/api/osquery/enroll", post(handlers::osquery::enroll_agent))
        .route("/api/v1/osquery/enroll", post(handlers::osquery::enroll_agent))
        // Carve block (unauthenticated - uses carve session ID for auth)
        .route("/api/osquery/carve/block", post(handlers::carves::carve_block))
        .route("/api/v1/osquery/carve/block", post(handlers::carves::carve_block))
}

// ---------------------------------------------------------------------------
// Orbit-authenticated routes
// ---------------------------------------------------------------------------

/// Orbit-authenticated endpoints.
fn orbit_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/api/fleet/orbit/device_token", post(handlers::orbit::set_or_update_device_token))
        .route("/api/fleet/orbit/config", post(handlers::orbit::get_orbit_config))
        .route("/api/fleet/orbit/scripts/request", post(handlers::orbit::get_orbit_script))
        .route("/api/fleet/orbit/scripts/result", post(handlers::orbit::post_orbit_script_result))
        .route("/api/fleet/orbit/device_mapping", put(handlers::orbit::put_orbit_device_mapping))
        .route("/api/fleet/orbit/software_install/result", post(handlers::orbit::post_orbit_software_install_result))
        .route("/api/fleet/orbit/software_install/package", post(handlers::orbit::orbit_download_software_installer))
        .route("/api/fleet/orbit/software_install/details", post(handlers::orbit::get_orbit_software_install_details))
        .route("/api/fleet/orbit/setup_experience/init", post(handlers::orbit::orbit_setup_experience_init))
        .route("/api/fleet/orbit/setup_experience/status", post(handlers::orbit::get_orbit_setup_experience_status))
        .route("/api/fleet/orbit/disk_encryption_key", post(handlers::orbit::post_orbit_disk_encryption_key))
        .route("/api/fleet/orbit/luks_data", post(handlers::orbit::post_orbit_luks))
        // Orbit enroll (unauthenticated)
        .route("/api/fleet/orbit/enroll", post(handlers::orbit::enroll_orbit))
        // Orbit ping (unauthenticated)
        .route("/api/fleet/orbit/ping", head(handlers::orbit::orbit_ping))
        // Android authenticated endpoints
        .route("/api/fleetd/certificates/{id}", get(handlers::orbit::get_device_certificate_template))
        .route("/api/fleetd/certificates/{id}/status", put(handlers::orbit::update_certificate_status))
}

// ---------------------------------------------------------------------------
// Device-authenticated routes
// ---------------------------------------------------------------------------

/// Device-authenticated endpoints (Fleet Desktop / device API).
fn device_routes() -> Router<crate::AppState> {
    Router::new()
        .route("/api/v1/fleet/device/{token}", get(handlers::device::get_device_host))
        .route("/api/v1/fleet/device/{token}/desktop", get(handlers::device::get_fleet_desktop))
        .route("/api/v1/fleet/device/{token}/ping", head(handlers::device::device_ping))
        .route("/api/v1/fleet/device/{token}/refetch", post(handlers::device::refetch_device_host))
        .route("/api/v1/fleet/device/{token}/device_mapping", get(handlers::device::list_device_host_device_mapping))
        .route("/api/v1/fleet/device/{token}/macadmins", get(handlers::device::get_device_macadmins_data))
        .route("/api/v1/fleet/device/{token}/policies", get(handlers::device::list_device_policies))
        .route("/api/v1/fleet/device/{token}/transparency", get(handlers::device::transparency_url))
        .route("/api/v1/fleet/device/{token}/debug/errors", post(handlers::device::fleetd_error))
        .route("/api/v1/fleet/device/{token}/software", get(handlers::device::get_device_software))
        .route("/api/v1/fleet/device/{token}/software/install/{software_title_id}", post(handlers::device::submit_self_service_software_install))
        .route("/api/v1/fleet/device/{token}/software/uninstall/{software_title_id}", post(handlers::device::submit_device_software_uninstall))
        .route("/api/v1/fleet/device/{token}/software/install/{install_uuid}/results", get(handlers::device::get_device_software_install_results))
        .route("/api/v1/fleet/device/{token}/software/uninstall/{execution_id}/results", get(handlers::device::get_device_software_uninstall_results))
        .route("/api/v1/fleet/device/{token}/certificates", get(handlers::device::list_device_certificates))
        .route("/api/v1/fleet/device/{token}/setup_experience/status", post(handlers::device::get_device_setup_experience_status))
        .route("/api/v1/fleet/device/{token}/software/titles/{software_title_id}/icon", get(handlers::device::get_device_software_icon))
        .route("/api/v1/fleet/device/{token}/mdm/linux/trigger_escrow", post(handlers::device::trigger_linux_disk_encryption_escrow))
        .route("/api/v1/fleet/device/{token}/bypass_conditional_access", post(handlers::device::bypass_conditional_access))
        // Device + Apple MDM
        .route("/api/v1/fleet/device/{token}/mdm/apple/manual_enrollment_profile", get(handlers::device::get_device_mdm_manual_enroll_profile))
        .route("/api/v1/fleet/device/{token}/software/commands/{command_uuid}/results", get(handlers::device::get_device_mdm_command_results))
        .route("/api/v1/fleet/device/{token}/configuration_profiles/{profile_uuid}/resend", post(handlers::device::resend_device_configuration_profile))
        .route("/api/v1/fleet/device/{token}/migrate_mdm", post(handlers::device::migrate_mdm_device))
        // Device ping (unauthenticated)
        .route("/api/fleet/device/ping", head(handlers::device::device_ping_unauth))
}

// ---------------------------------------------------------------------------
// No-auth routes
// ---------------------------------------------------------------------------

/// Unauthenticated endpoints (login, setup, SSO, etc.).
fn no_auth_routes() -> Router<crate::AppState> {
    Router::new()
        // Setup
        .route("/api/v1/setup", post(handlers::setup::setup))
        .route("/api/setup", post(handlers::setup::setup))
        // Login / Auth
        .route("/api/v1/fleet/login", post(handlers::sessions::login))
        .route("/api/v1/fleet/sessions", post(handlers::sessions::session_create))
        .route("/api/v1/fleet/logout", post(handlers::sessions::logout))
        .route("/api/v1/fleet/forgot_password", post(handlers::sessions::forgot_password))
        .route("/api/v1/fleet/reset_password", post(handlers::sessions::reset_password))
        .route("/api/v1/fleet/perform_required_password_reset", post(handlers::sessions::perform_required_password_reset))
        // User creation from invite
        .route("/api/v1/fleet/users", post(handlers::users::create_user_from_invite))
        .route("/api/v1/fleet/invites/{token}", get(handlers::invites::verify_invite))
        // SSO
        .route("/api/v1/fleet/sso", post(handlers::sessions::initiate_sso))
        .route("/api/v1/fleet/sso/callback", post(handlers::sessions::callback_sso))
        .route("/api/v1/fleet/sso", get(handlers::sessions::settings_sso))
        // MDM SSO
        .route("/api/v1/fleet/mdm/sso", post(handlers::mdm::initiate_mdm_sso))
        .route("/api/v1/fleet/mdm/sso/callback", post(handlers::mdm::callback_mdm_sso))
        // Software download
        .route("/api/v1/fleet/software/titles/{title_id}/package/token/{token}", get(handlers::software::download_software_installer))
        .route("/api/v1/fleet/software/titles/{title_id}/in_house_app", get(handlers::software::get_in_house_app_package))
        .route("/api/v1/fleet/software/titles/{title_id}/in_house_app/manifest", get(handlers::software::get_in_house_app_manifest))
        // Calendar webhook
        .route("/api/v1/fleet/calendar/webhook/{event_uuid}", post(handlers::app_config::calendar_webhook))
        // MDM enrollment (token-authenticated, unauthenticated for API layer)
        .route("/api/v1/fleet/mdm/bootstrap", get(handlers::mdm::download_bootstrap_package))
        .route("/api/v1/fleet/bootstrap", get(handlers::mdm::download_bootstrap_package))
        .route("/api/v1/fleet/mdm/apple/bootstrap", get(handlers::mdm::download_bootstrap_package))
        .route("/api/v1/fleet/mdm/setup/eula/{token}", get(handlers::mdm::get_mdm_eula))
        .route("/api/v1/fleet/setup_experience/eula/{token}", get(handlers::mdm::get_mdm_eula))
        .route("/api/v1/fleet/mdm/apple/setup/eula/{token}", get(handlers::mdm::get_mdm_eula))
        .route("/api/v1/fleet/ota_enrollment", post(handlers::mdm::mdm_apple_ota))
        .route("/api/v1/fleet/enrollment_profiles/ota", get(handlers::mdm::get_ota_profile))
        // Websocket for live query results (prefix-based)
        .route("/api/v1/fleet/results/{campaign_id}", get(handlers::queries::stream_campaign_results))
        // Metrics
        .route("/metrics", get(handlers::app_config::metrics))
}
