# Fleet Rust Rewrite — Execution Plan

## Current State Summary

| Metric | Go | Rust | Coverage |
|---|---|---|---|
| Source files (non-test) | ~1,300 | 142 | ~11% |
| Source lines (total) | ~620,000 | ~98,000 | ~16% |
| API routes | ~394 | ~419 | ~100% (routes defined) |
| Datastore methods | ~901 | ~449 | ~50% |
| Service interface methods | ~436 | ~554 fn defs | ~70% (many are premium-gated stubs) |
| Type definitions (lines) | ~27,600 | ~3,000 | ~11% |
| Test files | 675 | 0 (only inline #[cfg(test)]) | ~0% |

**Key finding:** Routes are wired up but most handlers delegate to service methods
that either return `require_premium()` errors or call datastore methods that return
placeholder values (`Ok(Vec::new())`, `Ok(0)`, `Ok(None)`). The wiring is done;
the business logic and data layer are not.

---

## Phase 1: Type System & Data Models (Foundation)

The Go `server/fleet/` package has ~27,600 lines of type definitions. The Rust
`fleet-types` crate has ~3,000. Almost every subsequent phase depends on having
correct, complete types.

### 1.1 Core entity types
- [ ] `Host` — full struct with all 80+ fields (Go: `server/fleet/hosts.go`, 1,671 lines)
- [ ] `HostDetail`, `HostListOptions`, `HostSummary`, `HostMDM`, `HostMunkiInfo`
- [ ] `User` — full struct with roles, teams, API-only fields (Go: `server/fleet/users.go`)
- [ ] `Team` — full struct with agent options, secrets, config (Go: `server/fleet/teams.go`, 742 lines)
- [ ] `TeamConfig`, `TeamSpecsDryRunResult`, `TeamIntegrations`
- [ ] `Query` — full struct with stats, observer fields (Go: `server/fleet/queries.go`)
- [ ] `Pack`, `PackSpec`, `PackTarget` (Go: `server/fleet/packs.go`)
- [ ] `Label`, `LabelSpec`, `LabelQueryExecution` (Go: `server/fleet/labels.go`)
- [ ] `Policy`, `PolicyPayload`, `PolicyData` (Go: `server/fleet/policies.go`)
- [ ] `Activity` — all 100+ activity types (Go: `server/fleet/activities.go`, 1,746 lines)

### 1.2 MDM types
- [ ] Apple MDM types (Go: `server/fleet/apple_mdm.go`, 1,133 lines)
  - `MDMAppleCommand`, `MDMAppleEnrollmentProfile`, `MDMAppleDEPDevice`
  - `MDMAppleConfigProfile`, `MDMAppleDeclaration`
- [ ] Microsoft MDM types (Go: `server/fleet/microsoft_mdm.go`, 1,706 lines)
  - `MDMWindowsCommand`, `MDMWindowsConfigProfile`, `MDMWindowsEnrolledDevice`
- [ ] Generic MDM types (Go: `server/fleet/mdm.go`, 1,229 lines)
  - `MDMPlatform`, `MDMCommandResult`, `MDMProfileStatus`, `MDMSolution`

### 1.3 Software & vulnerability types
- [ ] `Software`, `SoftwareTitle`, `SoftwareVersion` (Go: `server/fleet/software.go`, 851 lines)
- [ ] `SoftwareInstaller`, `SoftwareInstallerPayload` (Go: `server/fleet/software_installer.go`, 1,140 lines)
- [ ] `Vulnerability`, `CVE`, `CVEMeta` (Go: `server/fleet/vulnerabilities.go`)
- [ ] VPP types: `VPPApp`, `VPPToken`, `VPPAppTeam`

### 1.4 Configuration & integration types
- [ ] `AppConfig` — massive struct, 20+ nested sub-configs (Go: `server/fleet/app.go`, 1,742 lines)
  - `OrgInfo`, `ServerSettings`, `SMTPSettings`, `SSOSettings`
  - `HostExpirySettings`, `AgentOptions`, `WebhookSettings`
  - `Integrations` (Jira, Zendesk, Google Calendar)
  - `MDMConfig`, `Features`, `FleetDesktopSettings`
- [ ] `EnrollSecret`, `EnrollSecretSpec`
- [ ] `InvitePayload`, `Invite`
- [ ] `SessionInfo`, `Session`

### 1.5 Error types
- [ ] Match all Go error types (Go: `server/fleet/errors.go`, 569 lines)
  - `NotFoundError`, `AuthFailedError`, `AuthRequiredError`
  - `InvalidArgumentError`, `ConflictError`, `ValidationError`
  - `LicenseError`, `PermissionError`
- [ ] HTTP status code mapping for each error type

### 1.6 Remaining types
- [ ] Scripts types (Go: `server/fleet/scripts.go`, 706 lines)
- [ ] Certificate authorities types (Go: `server/fleet/certificate_authorities.go`, 764 lines)
- [ ] Integrations types (Go: `server/fleet/integrations.go`, 686 lines)
- [ ] Carve types (Go: `server/fleet/carves.go`)
- [ ] Calendar types, SCIM types, Android types

**Estimated scope:** ~24,000 lines of Go → ~8,000-12,000 lines of Rust

---

## Phase 2: Datastore Layer (Data Access)

Go has ~901 datastore methods across `server/datastore/mysql/`. Rust has ~449,
but many return placeholder values. The MySQL implementation is critical path.

### 2.1 Audit existing Rust datastore — fix placeholders
- [ ] `cleanup.rs` — 14 placeholder returns, 11 stub methods
- [ ] `datastore_impl.rs` — 8 placeholder returns
- [ ] `policies.rs` — 6 placeholder returns
- [ ] `users.rs` — 5 placeholder returns
- [ ] `invites.rs` — 5 placeholder returns
- [ ] `campaigns.rs` — 4 placeholder returns
- [ ] `software.rs` — 3 placeholder returns
- [ ] `queries.rs` — 3 placeholder returns
- [ ] `activities.rs` — 3 placeholder returns
- [ ] `setup_experience.rs` — 2 placeholder returns
- [ ] `jobs.rs` — 2 placeholder returns
- [ ] `enroll.rs` — 2 placeholder returns

### 2.2 Missing datastore domains (~452 methods missing)
- [ ] **Hosts** — Go has ~6,500 lines of MySQL host queries; Rust has ~608 lines
  - `ListHosts`, `CountHosts`, `HostByIdentifier`, `SearchHosts`
  - `UpdateHostSoftware`, `UpdateHostOperatingSystem`
  - Host detail loading (disk encryption, MDM, munki, macadmins)
  - Host filtering (labels, teams, status, OS, software, policy)
- [ ] **Software** — Go has ~6,484 lines; Rust has ~520 lines
  - `ListSoftware`, `ListSoftwareTitles`, `CountSoftwareTitles`
  - Software installer CRUD, VPP app management
  - Software install results tracking
- [ ] **Apple MDM** — Go has ~7,241 lines; Rust has 0 dedicated lines
  - Profile management (install/remove/status)
  - DEP enrollment, device commands
  - Bootstrap packages, setup assistants
  - Declaration management (DDM)
- [ ] **Microsoft MDM** — Go has ~2,569 lines; Rust has 0 dedicated lines
  - Windows profile management
  - Windows enrollment, device commands
- [ ] **Generic MDM** — Go has ~2,916 lines; Rust has ~652 lines
  - Command results, profile status aggregation
  - MDM solutions tracking
- [ ] **Policies** — Go has ~2,438 lines; Rust has ~290 lines
  - Policy results aggregation, team policy inheritance
  - Policy automation (webhook, installer triggers)
- [ ] **Labels** — Go has ~1,634 lines; Rust has ~333 lines
  - Label membership computation, dynamic label queries
  - Label spec application
- [ ] **Scripts** — Go has ~3,052 lines; Rust has ~254 lines
  - Script execution tracking, results storage
  - Pending script management
- [ ] **Activities** — Go has ~2,218 lines (test); Rust has ~270 lines
  - Activity logging for all 100+ activity types
  - Activity list with pagination and filtering
- [ ] **VPP** — Go has ~2,695 lines; Rust has 0
- [ ] **In-house apps** — Go has ~1,563 lines; Rust has 0
- [ ] **Android** — Go has ~1,864 lines; Rust has 0
- [ ] **SCIM** — Go has ~2,725 lines (test); Rust has 0
- [ ] **Certificate templates** — Go has dedicated files; Rust has 0

### 2.3 Database migrations
- [ ] Verify migration system handles Go's schema.sql correctly
- [ ] Test migration rollback support
- [ ] Add migration versioning/tracking

**Estimated scope:** ~450 missing methods, ~80,000 lines of Go → ~25,000-35,000 lines of Rust

---

## Phase 3: Service Layer (Business Logic)

The Go service layer (`server/service/`) has ~200K lines including tests.
The Rust service layer has ~7,843 lines. Most Rust methods do basic
CRUD delegation; the complex business logic is missing.

### 3.1 Osquery protocol — real implementation
- [ ] Enrollment with proper secret validation and host creation
- [ ] Config endpoint — return real osquery config (packs, decorators, options)
- [ ] Distributed read — query campaigns, policy queries, label queries
- [ ] Distributed write — process query results, update host details
- [ ] Log endpoint — process status/result/snapshot logs
- [ ] Carve begin/block — file carving protocol

Go reference: `server/service/osquery.go` (3,360 lines)
Rust current: `fleet-service/src/osquery.rs` (340 lines)

### 3.2 Host management — full business logic
- [ ] Host listing with complex filtering (labels, teams, software, policies, OS, etc.)
- [ ] Host detail aggregation (software, policies, MDM status, disk encryption)
- [ ] Host transfer between teams
- [ ] Host refetch, delete, bulk operations
- [ ] Host health/vitals endpoints
- [ ] Macadmins data aggregation

Go reference: `server/service/hosts.go` (3,630 lines)
Rust current: `fleet-service/src/hosts.rs` (457 lines)

### 3.3 App config — complex merge logic
- [ ] Apply app config with validation of all nested fields
- [ ] Team-level config overrides
- [ ] Agent options validation and merging
- [ ] Integration settings (Jira, Zendesk, Google Calendar) validation

Go reference: `server/service/appconfig.go` (2,196 lines)
Rust current: `fleet-service/src/app_config.rs` (258 lines)

### 3.4 Live queries — full pipeline
- [ ] Campaign creation with target resolution (hosts, labels, teams)
- [ ] Campaign status tracking via Redis pub/sub
- [ ] WebSocket connection for streaming results
- [ ] Campaign cleanup on disconnect

Go reference: `server/service/live_query.go` + `server/live_query/` (~742 lines)
Rust current: `fleet-service/src/live_query.rs` (276 lines)

### 3.5 MDM operations (Apple + Microsoft)
- [ ] Apple MDM profile management with conflict detection
- [ ] Apple DEP enrollment flow
- [ ] Apple MDM commands (lock, wipe, restart, etc.)
- [ ] Apple DDM (Declarative Device Management)
- [ ] Apple bootstrap packages
- [ ] Apple setup assistant / setup experience
- [ ] Microsoft MDM profile management
- [ ] Microsoft MDM commands
- [ ] Cross-platform MDM summary/status aggregation

Go reference: `server/service/apple_mdm.go` (7,310 lines) + `server/service/microsoft_mdm.go` (2,836 lines) + `server/service/mdm.go` (3,654 lines)
Rust current: `fleet-service/src/mdm.rs` (580 lines)

### 3.6 Software management
- [ ] Software installer upload/download/install
- [ ] VPP app management
- [ ] Software title aggregation
- [ ] Install/uninstall tracking
- [ ] Self-service software

Go reference: multiple files totaling ~5,000+ lines
Rust current: `fleet-service/src/software.rs` (374 lines)

### 3.7 Orbit agent endpoints
- [ ] Orbit enrollment and config
- [ ] Orbit device mapping
- [ ] Script execution via Orbit
- [ ] Nudge/software update notifications
- [ ] Orbit extensions/capabilities

Go reference: `server/service/orbit.go` (1,939 lines)
Rust current: `fleet-service/src/orbit.rs` (266 lines)

### 3.8 User management — complete flows
- [ ] Password reset flow (request + complete)
- [ ] Email change with token verification
- [ ] API-only user management
- [ ] User role management with team scoping

Go reference: ~1,200 lines across service files
Rust current: `fleet-service/src/users.rs` (508 lines)

### 3.9 Scripts
- [ ] Script execution flow (run, check status, get results)
- [ ] Script content management (upload, download)
- [ ] Batch script operations for teams
- [ ] Lock/unlock/wipe via scripts

Go reference: `server/service/scripts.go` (1,687 lines)
Rust current: `fleet-service/src/scripts.rs` (included in service but thin)

**Estimated scope:** ~30,000 lines of Go business logic → ~10,000-15,000 lines of Rust

---

## Phase 4: Enterprise Features (Premium/License-Gated)

The Go codebase has a separate `ee/server/` directory (21,060 lines, 49 files)
with enterprise-only service implementations. The Rust server gates these behind
`require_premium()` but has zero actual premium implementations.

### 4.1 Enterprise service layer
- [ ] `ee/server/service/` — premium service method overrides
  - Teams: full team management (create, modify, delete, specs)
  - Software: installer management, VPP, self-service
  - MDM: all Apple/Microsoft MDM operations
  - Calendar: Google Calendar integration
  - Device management: premium device endpoints
  - Policies: automation, installers, calendar integration

### 4.2 Enterprise calendar integration
- [ ] Google Calendar event creation/management
- [ ] Calendar-based policy automation

### 4.3 Enterprise MDM features
- [ ] End-user MDM migration
- [ ] MDM SSO authentication
- [ ] Advanced profile management

**Estimated scope:** ~21,000 lines of Go → ~7,000-10,000 lines of Rust

---

## Phase 5: MDM Protocol Stack

The Go `server/mdm/` directory is a major subsystem (30,060 lines, 191 files).
The Rust server has no equivalent — MDM protocol handling, SCEP, DEP, and
nano MDM integration are all missing.

### 5.1 Apple MDM protocol (NanoMDM)
- [ ] MDM command queue and response processing
- [ ] Push notification integration (APNs)
- [ ] DEP (Device Enrollment Program) sync and assignment
- [ ] SCEP certificate issuance
- [ ] MDM check-in protocol handlers
- [ ] Token update / authenticate / checkout handlers

### 5.2 Apple profile/declaration management
- [ ] Configuration profile XML generation and parsing
- [ ] DDM declaration management
- [ ] Profile verification and conflict resolution
- [ ] FileVault key escrow

### 5.3 Microsoft MDM protocol
- [ ] MS-MDE2 enrollment protocol
- [ ] SyncML command handling
- [ ] Windows CSP profile translation
- [ ] Discovery, policy, enrollment services

### 5.4 MDM lifecycle
- [ ] Host MDM enrollment/unenrollment lifecycle
- [ ] MDM migration between MDM solutions
- [ ] MDM maintenance (cleanup, reconciliation)

### 5.5 Maintained apps
- [ ] Maintained apps catalog and installation
- [ ] App version tracking

**Estimated scope:** ~30,000 lines of Go → ~10,000-15,000 lines of Rust

---

## Phase 6: Background Workers & Cron Jobs

### 6.1 Worker system
- [ ] Job queue with MySQL-backed persistence (Go: `server/worker/worker.go`)
- [ ] Apple MDM worker (profile delivery, DEP operations)
- [ ] Jira/Zendesk integration workers
- [ ] macOS setup assistant worker
- [ ] Software installer worker
- [ ] VPP verification worker
- [ ] Batch activities worker

Go reference: `server/worker/` (3,240 lines)
Rust current: `fleet-service/src/cron.rs` (324 lines) — basic cron scheduler exists

### 6.2 Cron schedules (verify real implementations)
- [ ] Host status cleanup
- [ ] Policy automation processing
- [ ] Software vulnerability scanning trigger
- [ ] MDM profile reconciliation
- [ ] Activity cleanup
- [ ] Statistics collection
- [ ] Certificate rotation

### 6.3 Vulnerability processing
- [ ] NVD data feed download and processing
- [ ] CPE matching for software
- [ ] CVE detection and host association
- [ ] OVAL/MSRC processing for OS vulnerabilities
- [ ] Vulnerability webhook/integration notifications

Go reference: `server/vulnerabilities/` (15,348 lines, 109 files)
Rust current: 0 lines

**Estimated scope:** ~18,500 lines of Go → ~6,000-9,000 lines of Rust

---

## Phase 7: Infrastructure & Integration

### 7.1 SSO/SAML — real implementation
- [ ] SAML request generation with proper XML signing
- [ ] SAML response validation and attribute extraction
- [ ] SSO initiate/callback with session creation
- [ ] JIT user provisioning from SAML attributes
- [ ] IdP metadata parsing and validation

Go reference: `server/sso/` (540 lines)
Rust current: has SSO route handlers but SAML protocol parsing is stubbed

### 7.2 Mail/SMTP
- [ ] SMTP connection with TLS/STARTTLS
- [ ] Email template rendering (invitation, password reset)
- [ ] SMTP configuration test endpoint

Go reference: `server/mail/` (5 files)
Rust current: has mail module but unclear if it sends real emails

### 7.3 Logging/result store
- [ ] Osquery result log forwarding (filesystem, Firehose, Kinesis, Lambda, Kafka, PubSub, stdout)
- [ ] Status log forwarding to configured backends
- [ ] Log rotation and cleanup

Go reference: `server/logging/` (14 files)
Rust current: basic tracing setup only

### 7.4 S3/blob storage — verify real implementation
- [ ] Installers storage (upload/download)
- [ ] Carve block storage
- [ ] Bootstrap package storage
- [ ] Script content storage

Go reference: `server/datastore/s3/` (7 files)
Rust current: has `blobstore` module — verify it works end-to-end

### 7.5 Redis — verify all operations
- [ ] Live query campaign pub/sub
- [ ] Distributed query result collection
- [ ] Rate limiting state storage
- [ ] IP banning state
- [ ] Session caching (if used)

Go reference: `server/pubsub/` (4 files, 314 lines) + `server/datastore/redis/`
Rust current: `fleet-redis/` (1,507 lines) — appears partially implemented

### 7.6 WebSocket support
- [ ] Live query streaming results over WebSocket
- [ ] Connection lifecycle management
- [ ] Authentication for WebSocket connections

Go reference: `server/websocket/` (1 file)
Rust current: 9 references to WebSocket — verify completeness

**Estimated scope:** ~5,000+ lines of Go → ~2,000-4,000 lines of Rust

---

## Phase 8: CLI & Server Bootstrap

### 8.1 Server command
- [ ] Full CLI flag parity with Go's `fleet serve` command
- [ ] Configuration file loading (YAML)
- [ ] Environment variable overrides
- [ ] TLS configuration
- [ ] Graceful shutdown handling

Go reference: `cmd/fleet/` (4,656 lines)
Rust current: `fleet-server/src/main.rs` + `config.rs` — basic startup exists

### 8.2 Fleet CLI tools
- [ ] `fleet prepare db` — database preparation
- [ ] `fleet version` — version info
- [ ] `fleet config_dump` — configuration dump
- [ ] Other CLI subcommands

**Estimated scope:** ~4,600 lines of Go → ~1,500-2,500 lines of Rust

---

## Phase 9: Authorization (RBAC)

### 9.1 Full authorization policy
- [ ] Complete RBAC matrix matching Go's authorization checks
  - Global admin, maintainer, observer, observer+, GitOps roles
  - Team admin, maintainer, observer, observer+, GitOps roles
- [ ] Per-endpoint authorization validation
- [ ] Team-scoped authorization (user can have different roles per team)
- [ ] Authorization for MDM, software, scripts operations

Go reference: `server/authz/` + authorization checks in every service method
Rust current: `fleet-service/src/authz.rs` (348 lines) — basic structure exists

**Estimated scope:** Authorization touches every service method; ~2,000-4,000 lines of Rust

---

## Phase 10: Testing

The Go codebase has 675 test files. The Rust codebase has essentially zero tests
outside of a few inline `#[cfg(test)]` blocks.

### 10.1 Unit test infrastructure
- [ ] Mock datastore trait implementation for unit tests
- [ ] Mock Redis implementation
- [ ] Test fixtures and factories for all entity types
- [ ] Test helper utilities

### 10.2 Service layer unit tests
- [ ] Auth/session tests
- [ ] User management tests
- [ ] Host management tests
- [ ] Query/pack/label tests
- [ ] Policy tests
- [ ] Team tests
- [ ] Osquery protocol tests

### 10.3 Datastore integration tests
- [ ] MySQL test harness (create/destroy test databases)
- [ ] Host CRUD tests
- [ ] User CRUD tests
- [ ] Software CRUD tests
- [ ] Label CRUD tests
- [ ] Policy CRUD tests
- [ ] MDM data tests

### 10.4 API integration tests
- [ ] HTTP handler tests (request/response validation)
- [ ] Authentication/authorization integration tests
- [ ] End-to-end enrollment flow tests
- [ ] Live query flow tests

### 10.5 Parity tests
- [ ] Response format compatibility tests (compare Go vs Rust JSON output)
- [ ] API contract tests against Go server

**Estimated scope:** Go has ~200K lines of tests; target ~20,000-40,000 lines of Rust tests

---

## Phase 11: Observability & Production Readiness

### 11.1 Metrics
- [ ] Prometheus metrics endpoint
- [ ] Request duration histograms
- [ ] Active host count gauges
- [ ] Error rate counters
- [ ] Database query metrics

### 11.2 Structured logging
- [ ] Request/response logging with correlation IDs
- [ ] Audit logging for sensitive operations
- [ ] Log level configuration

### 11.3 Health checks
- [ ] Database health check
- [ ] Redis health check
- [ ] Disk space check
- [ ] Certificate expiry check

### 11.4 Graceful operations
- [ ] Graceful shutdown with in-flight request draining
- [ ] Database connection pool management
- [ ] Redis connection pool management

**Estimated scope:** ~2,000-3,000 lines of Rust

---

## Priority Order & Dependencies

```
Phase 1 (Types)
  └─→ Phase 2 (Datastore)
       └─→ Phase 3 (Service Logic)
            ├─→ Phase 4 (Enterprise)
            ├─→ Phase 5 (MDM Protocol)
            └─→ Phase 6 (Workers)
  └─→ Phase 7 (Infrastructure) — can parallel with Phase 2-3
  └─→ Phase 8 (CLI) — can parallel with Phase 2-3
  └─→ Phase 9 (AuthZ) — should interleave with Phase 3

Phase 10 (Testing) — should run continuously alongside all phases
Phase 11 (Observability) — can parallel with later phases
```

## Total Estimated Effort

| Phase | Go Lines | Est. Rust Lines | Priority |
|---|---|---|---|
| 1. Types | ~27,600 | 8,000-12,000 | P0 — blocks everything |
| 2. Datastore | ~80,000 | 25,000-35,000 | P0 — blocks service layer |
| 3. Service Logic | ~30,000 | 10,000-15,000 | P0 — core functionality |
| 4. Enterprise | ~21,000 | 7,000-10,000 | P1 — premium features |
| 5. MDM Protocol | ~30,000 | 10,000-15,000 | P1 — major feature area |
| 6. Workers/Vuln | ~18,500 | 6,000-9,000 | P1 — background processing |
| 7. Infrastructure | ~5,000 | 2,000-4,000 | P1 — integrations |
| 8. CLI | ~4,600 | 1,500-2,500 | P2 — operational |
| 9. Authorization | spread | 2,000-4,000 | P0 — security critical |
| 10. Testing | ~200,000 | 20,000-40,000 | P0 — ongoing |
| 11. Observability | — | 2,000-3,000 | P2 — production |
| **Total** | **~420,000** | **~93,500-149,500** | |

Current Rust: ~98,000 lines (including boilerplate/stubs)
Estimated remaining: **~60,000-100,000 lines** of meaningful new Rust code
