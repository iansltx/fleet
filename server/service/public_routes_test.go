package service

import (
	"log/slog"
	"sort"
	"strings"
	"testing"

	"github.com/fleetdm/fleet/v4/server/config"
	"github.com/fleetdm/fleet/v4/server/mock"
	"github.com/fleetdm/fleet/v4/server/platform/endpointer"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/throttled/throttled/v2/store/memstore"
)

// TestPublicRoutesMatchPatterns verifies that every route tagged as "public"
// in the handler registry is matched by at least one entry in
// PublicRoutePatterns (the static list derived from the public-internet
// endpoint documentation).
//
// This test is intentionally conservative: it will fail when a new public
// route is added to the code but not yet documented in PublicRoutePatterns.
// To fix a failure, either add the missing pattern to PublicRoutePatterns
// (if the route should be public-facing) or remove the public tag from
// the route registration.
func TestPublicRoutesMatchPatterns(t *testing.T) {
	ds := new(mock.Store)
	svc, _ := newTestService(t, ds, nil, nil)
	limitStore, _ := memstore.New(0)
	cfg := config.TestConfig()

	_, registry := MakeHandlerWithRegistry(svc, cfg, slog.New(slog.DiscardHandler), limitStore, nil, nil, nil)
	require.NotNil(t, registry)

	publicRoutes := registry.RoutesByTag(endpointer.RouteTagPublic)
	require.NotEmpty(t, publicRoutes, "expected at least one public-tagged route")

	sort.Strings(publicRoutes)

	var unmatched []string
	for _, route := range publicRoutes {
		if !matchesAnyPattern(route, PublicRoutePatterns) {
			unmatched = append(unmatched, route)
		}
	}

	assert.Empty(t, unmatched,
		"the following public-tagged routes are not covered by PublicRoutePatterns; "+
			"add matching patterns to PublicRoutePatterns or remove the public tag:\n%s",
		strings.Join(unmatched, "\n"))
}

func TestMatchPattern(t *testing.T) {
	tests := []struct {
		pattern string
		path    string
		want    bool
	}{
		// Exact match
		{"/api/fleet/device/ping", "/api/fleet/device/ping", true},
		{"/api/fleet/device/ping", "/api/fleet/device/other", false},

		// Single wildcard
		{"/api/*/fleet/me", "/api/v1/fleet/me", true},
		{"/api/*/fleet/me", "/api/_version_/fleet/me", true},
		{"/api/*/fleet/me", "/api/v1/fleet/you", false},

		// Double wildcard (suffix)
		{"/api/osquery/**", "/api/osquery/config", true},
		{"/api/osquery/**", "/api/osquery/distributed/read", true},
		{"/api/osquery/**", "/api/osquery", false},     // ** requires at least 1 segment
		{"/api/*/fleet/**", "/api/v1/fleet/hosts", true},
		{"/api/*/fleet/**", "/api/v1/fleet/hosts/123", true},

		// Mixed wildcards
		{"/api/*/fleet/device/*/desktop", "/api/v1/fleet/device/abc/desktop", true},
		{"/api/*/fleet/device/*/desktop", "/api/v1/fleet/device/abc/other", false},

		// No match - different prefix
		{"/api/osquery/**", "/mdm/apple/scep", false},
	}

	for _, tt := range tests {
		t.Run(tt.pattern+"_"+tt.path, func(t *testing.T) {
			got := matchPattern(tt.pattern, tt.path)
			assert.Equal(t, tt.want, got)
		})
	}
}

// matchesAnyPattern returns true if path matches at least one pattern.
func matchesAnyPattern(path string, patterns []string) bool {
	for _, pattern := range patterns {
		if matchPattern(pattern, path) {
			return true
		}
	}
	return false
}

// matchPattern checks if path matches pattern using simple wildcard rules:
//   - "*" matches exactly one path segment
//   - "**" at the end of a pattern matches one or more remaining segments
//   - literal segments must match exactly
//
// Both pattern and path are split on "/" for segment-by-segment comparison.
// The _version_ placeholder in registered routes is treated as a single
// segment, matching "*" in patterns.
func matchPattern(pattern, path string) bool {
	patParts := strings.Split(strings.Trim(pattern, "/"), "/")
	pathParts := strings.Split(strings.Trim(path, "/"), "/")

	pi := 0
	for _, pp := range patParts {
		if pp == "**" {
			// "**" must be the last element and matches one or more remaining segments.
			return pi < len(pathParts)
		}
		if pi >= len(pathParts) {
			return false
		}
		if pp == "*" {
			// matches any single segment
			pi++
			continue
		}
		if pp != pathParts[pi] {
			return false
		}
		pi++
	}
	return pi == len(pathParts)
}
