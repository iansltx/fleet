package service

// PublicRoutePatterns is a static list of route patterns that should be
// accessible from the public internet, derived from the documentation in
// articles/what-api-endpoints-to-expose-to-the-public-internet.md.
//
// Each pattern uses a simple wildcard syntax:
//   - "*" matches a single path segment (e.g. a version string or ID)
//   - "**" matches one or more remaining path segments (suffix wildcard)
//
// These patterns are used in tests to verify that all routes tagged as public
// are covered by the documented set of public-facing endpoints. This ensures
// that WAF/load balancer rules stay in sync with the codebase as new endpoints
// are added.
var PublicRoutePatterns = []string{
	// Osquery endpoints (managing hosts outside VPN/intranet)
	"/api/osquery/**",
	"/api/v1/osquery/**",

	// Fleet Desktop and device-authenticated endpoints
	"/api/*/fleet/device/**",
	"/api/fleet/device/ping",

	// Orbit endpoints
	"/api/fleet/orbit/**",

	// fleetctl CLI endpoints (using Fleet from outside the network)
	"/api/setup",
	"/api/*/setup",
	"/api/*/fleet/**",

	// Apple MDM endpoints
	"/mdm/apple/scep",
	"/mdm/apple/mdm",
	"/api/mdm/apple/enroll",
	"/api/mdm/apple/installer",
	"/api/mdm/apple/account_driven_enroll",

	// MDM SSO endpoints
	"/api/v1/fleet/sso",
	"/api/v1/fleet/sso/callback",

	// Microsoft MDM endpoints
	"/api/mdm/microsoft/management",
	"/api/mdm/microsoft/discovery",
	"/api/mdm/microsoft/policy",
	"/api/mdm/microsoft/enroll",
	"/api/mdm/microsoft/tos",
	"/api/mdm/microsoft/auth",

	// iOS/iPadOS/Android enrollment
	"/enroll",
	"/api/*/fleet/enrollment_profiles/ota",
	"/api/*/fleet/software/titles/*/in_house_app",
	"/api/*/fleet/software/titles/*/in_house_app/manifest",

	// Android endpoints
	"/api/*/fleet/android_enterprise/**",
	"/api/fleetd/**",

	// SCEP proxy
	"/mdm/scep/proxy/**",

	// Static assets and frontend device pages
	"/assets/**",
	"/device/**",
}
