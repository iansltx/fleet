package tables

import (
	"database/sql"
	"fmt"
)

func init() {
	MigrationClient.AddMigration(Up_20260303000000, Down_20260303000000)
}

func Up_20260303000000(tx *sql.Tx) error {
	// This migration replaces all uses of MD5() with SHA2(, 256) in generated
	// columns and re-hashes stored checksums. This is required because MySQL 9.6
	// removed MD5() and SHA1() by default. This migration must be run BEFORE
	// upgrading MySQL from 9.5 to 9.6.

	// 1. mdm_windows_configuration_profiles: change generated column from
	//    BINARY(16) AS (UNHEX(MD5(syncml))) to BINARY(32) AS (UNHEX(SHA2(syncml, 256)))
	//    MySQL does not support ALTER COLUMN for generated columns, so we must
	//    drop and re-add.
	if columnExists(tx, "mdm_windows_configuration_profiles", "checksum") {
		_, err := tx.Exec(`ALTER TABLE mdm_windows_configuration_profiles
			DROP COLUMN checksum`)
		if err != nil {
			return fmt.Errorf("dropping checksum from mdm_windows_configuration_profiles: %w", err)
		}
		_, err = tx.Exec(`ALTER TABLE mdm_windows_configuration_profiles
			ADD COLUMN checksum BINARY(32) AS (UNHEX(SHA2(syncml, 256))) STORED`)
		if err != nil {
			return fmt.Errorf("re-adding checksum to mdm_windows_configuration_profiles: %w", err)
		}
	}

	// 2. mdm_apple_declarations: change generated token column from
	//    BINARY(16) GENERATED ALWAYS AS (UNHEX(MD5(...))) to BINARY(32) with SHA2
	if columnExists(tx, "mdm_apple_declarations", "token") {
		_, err := tx.Exec(`ALTER TABLE mdm_apple_declarations
			DROP COLUMN token`)
		if err != nil {
			return fmt.Errorf("dropping token from mdm_apple_declarations: %w", err)
		}
		_, err = tx.Exec(`ALTER TABLE mdm_apple_declarations
			ADD COLUMN token BINARY(32) GENERATED ALWAYS AS
				(UNHEX(SHA2(CONCAT(raw_json, IFNULL(secrets_updated_at, '')), 256))) STORED NULL`)
		if err != nil {
			return fmt.Errorf("re-adding token to mdm_apple_declarations: %w", err)
		}
	}

	// 3. host_mdm_apple_declarations: widen token from BINARY(16) to BINARY(32)
	//    and re-sync from the generated column.
	//    The idx_token index must be dropped and re-created after column resize.
	if columnExists(tx, "host_mdm_apple_declarations", "token") {
		_, err := tx.Exec(`ALTER TABLE host_mdm_apple_declarations
			DROP INDEX idx_token,
			MODIFY COLUMN token BINARY(32) NOT NULL`)
		if err != nil {
			return fmt.Errorf("widening token in host_mdm_apple_declarations: %w", err)
		}
		_, err = tx.Exec(`UPDATE host_mdm_apple_declarations hmad
			JOIN mdm_apple_declarations mad ON hmad.declaration_uuid = mad.declaration_uuid
			SET hmad.token = mad.token`)
		if err != nil {
			return fmt.Errorf("updating token values in host_mdm_apple_declarations: %w", err)
		}
		_, err = tx.Exec(`ALTER TABLE host_mdm_apple_declarations
			ADD INDEX idx_token (token)`)
		if err != nil {
			return fmt.Errorf("re-adding idx_token to host_mdm_apple_declarations: %w", err)
		}
	}

	// 4. host_mdm_windows_profiles: widen checksum from BINARY(16) to BINARY(32)
	//    and re-sync from the generated column.
	if columnExists(tx, "host_mdm_windows_profiles", "checksum") {
		_, err := tx.Exec(`ALTER TABLE host_mdm_windows_profiles
			MODIFY COLUMN checksum BINARY(32) NOT NULL DEFAULT 0`)
		if err != nil {
			return fmt.Errorf("widening checksum in host_mdm_windows_profiles: %w", err)
		}
		_, err = tx.Exec(`UPDATE host_mdm_windows_profiles hmwp
			JOIN mdm_windows_configuration_profiles mwcp ON mwcp.profile_uuid = hmwp.profile_uuid
			SET hmwp.checksum = mwcp.checksum`)
		if err != nil {
			return fmt.Errorf("updating checksum values in host_mdm_windows_profiles: %w", err)
		}
	}

	// 5. mdm_apple_configuration_profiles: widen checksum from BINARY(16) to
	//    BINARY(32) and re-hash with SHA2.
	if columnExists(tx, "mdm_apple_configuration_profiles", "checksum") {
		_, err := tx.Exec(`ALTER TABLE mdm_apple_configuration_profiles
			MODIFY COLUMN checksum BINARY(32) NOT NULL`)
		if err != nil {
			return fmt.Errorf("widening checksum in mdm_apple_configuration_profiles: %w", err)
		}
		_, err = tx.Exec(`UPDATE mdm_apple_configuration_profiles
			SET checksum = UNHEX(SHA2(mobileconfig, 256))`)
		if err != nil {
			return fmt.Errorf("re-hashing checksums in mdm_apple_configuration_profiles: %w", err)
		}
	}

	// 6. host_mdm_apple_profiles: widen checksum from BINARY(16) to BINARY(32)
	//    and re-sync from source table.
	if columnExists(tx, "host_mdm_apple_profiles", "checksum") {
		_, err := tx.Exec(`ALTER TABLE host_mdm_apple_profiles
			MODIFY COLUMN checksum BINARY(32) NOT NULL`)
		if err != nil {
			return fmt.Errorf("widening checksum in host_mdm_apple_profiles: %w", err)
		}
		_, err = tx.Exec(`UPDATE host_mdm_apple_profiles hmap
			JOIN mdm_apple_configuration_profiles macp ON macp.profile_uuid = hmap.profile_uuid
			SET hmap.checksum = macp.checksum`)
		if err != nil {
			return fmt.Errorf("updating checksum values in host_mdm_apple_profiles: %w", err)
		}
	}

	// 7. policies: widen checksum from BINARY(16) to BINARY(32)
	//    and re-hash with SHA2.
	if columnExists(tx, "policies", "checksum") {
		_, err := tx.Exec(`ALTER TABLE policies
			MODIFY COLUMN checksum BINARY(32) NOT NULL`)
		if err != nil {
			return fmt.Errorf("widening checksum in policies: %w", err)
		}
		_, err = tx.Exec(`UPDATE policies SET checksum = UNHEX(
			SHA2(
				CONCAT_WS(CHAR(0),
					COALESCE(team_id, ''),
					name
				),
			256)
		)`)
		if err != nil {
			return fmt.Errorf("re-hashing checksums in policies: %w", err)
		}
	}

	// 8. software: widen checksum from BINARY(16) to BINARY(32).
	//    Re-hashing is done in multiple passes to handle the conditional
	//    inclusion of application_id and upgrade_code in the hash, matching
	//    the Go ComputeRawChecksum logic.
	if columnExists(tx, "software", "checksum") {
		// First, drop the unique index so we can resize without constraint issues
		_, err := tx.Exec(`ALTER TABLE software
			DROP INDEX idx_software_checksum,
			MODIFY COLUMN checksum BINARY(32) NOT NULL`)
		if err != nil {
			return fmt.Errorf("widening checksum in software: %w", err)
		}

		// Case 1: no application_id, no upgrade_code
		_, err = tx.Exec(`UPDATE software SET checksum = UNHEX(SHA2(
			CONCAT_WS(CHAR(0),
				version, source, COALESCE(bundle_identifier, ''),
				` + "`release`" + `, arch, vendor,
				COALESCE(extension_for, ''), extension_id, name
			), 256))
			WHERE (application_id IS NULL OR application_id = '')
			  AND (upgrade_code IS NULL OR upgrade_code = '')`)
		if err != nil {
			return fmt.Errorf("re-hashing software checksums (case 1): %w", err)
		}

		// Case 2: has application_id, no upgrade_code
		_, err = tx.Exec(`UPDATE software SET checksum = UNHEX(SHA2(
			CONCAT_WS(CHAR(0),
				version, source, COALESCE(bundle_identifier, ''),
				` + "`release`" + `, arch, vendor,
				COALESCE(extension_for, ''), extension_id, name,
				application_id
			), 256))
			WHERE application_id IS NOT NULL AND application_id != ''
			  AND (upgrade_code IS NULL OR upgrade_code = '')`)
		if err != nil {
			return fmt.Errorf("re-hashing software checksums (case 2): %w", err)
		}

		// Case 3: no application_id, has upgrade_code
		_, err = tx.Exec(`UPDATE software SET checksum = UNHEX(SHA2(
			CONCAT_WS(CHAR(0),
				version, source, COALESCE(bundle_identifier, ''),
				` + "`release`" + `, arch, vendor,
				COALESCE(extension_for, ''), extension_id, name,
				upgrade_code
			), 256))
			WHERE (application_id IS NULL OR application_id = '')
			  AND upgrade_code IS NOT NULL AND upgrade_code != ''`)
		if err != nil {
			return fmt.Errorf("re-hashing software checksums (case 3): %w", err)
		}

		// Case 4: has both application_id and upgrade_code
		_, err = tx.Exec(`UPDATE software SET checksum = UNHEX(SHA2(
			CONCAT_WS(CHAR(0),
				version, source, COALESCE(bundle_identifier, ''),
				` + "`release`" + `, arch, vendor,
				COALESCE(extension_for, ''), extension_id, name,
				application_id, upgrade_code
			), 256))
			WHERE application_id IS NOT NULL AND application_id != ''
			  AND upgrade_code IS NOT NULL AND upgrade_code != ''`)
		if err != nil {
			return fmt.Errorf("re-hashing software checksums (case 4): %w", err)
		}

		// Re-add the unique index
		_, err = tx.Exec(`ALTER TABLE software
			ADD UNIQUE INDEX idx_software_checksum (checksum)`)
		if err != nil {
			return fmt.Errorf("re-adding unique index to software: %w", err)
		}
	}

	return nil
}

func Down_20260303000000(_ *sql.Tx) error {
	return nil
}
