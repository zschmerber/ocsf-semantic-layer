/**
 * Index model validation utilities for the OCSF Semantic Model Editor.
 *
 * Provides validation functions for:
 * - Table entries (class_uid validity)
 * - Source lineage records (orphan detection)
 * - Detection coverage (MITRE technique/tactic format)
 *
 * Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6, 17.7
 */

import type {
  TableEntry,
  SourceLineageRecord,
  DetectionCoverage,
  ValidationError,
  ValidationWarning,
} from '../types';

// ============================================
// Validation Result Types
// ============================================

/**
 * Result of index model validation.
 */
export interface IndexValidationResult {
  /** Whether the index model is valid (no errors) */
  valid: boolean;
  /** Validation errors that must be fixed */
  errors: ValidationError[];
  /** Validation warnings for best practices */
  warnings: ValidationWarning[];
}

// ============================================
// MITRE ATT&CK Validation Constants
// ============================================

/**
 * Valid MITRE ATT&CK tactic names.
 * Based on the Enterprise ATT&CK matrix.
 */
export const VALID_MITRE_TACTICS = [
  'reconnaissance',
  'resource-development',
  'initial-access',
  'execution',
  'persistence',
  'privilege-escalation',
  'defense-evasion',
  'credential-access',
  'discovery',
  'lateral-movement',
  'collection',
  'command-and-control',
  'exfiltration',
  'impact',
] as const;

/**
 * Regex pattern for MITRE technique IDs.
 * Matches formats like: T1234, T1234.001, T1234.012
 */
export const MITRE_TECHNIQUE_PATTERN = /^T\d{4}(\.\d{3})?$/;

/**
 * Known valid OCSF class UIDs.
 * This is a subset of common class UIDs - in production, this would be
 * fetched from the schema service.
 */
export const KNOWN_OCSF_CLASS_UIDS = new Set([
  // System Activity
  1001, // File System Activity
  1002, // Kernel Extension Activity
  1003, // Kernel Activity
  1004, // Memory Activity
  1005, // Module Activity
  1006, // Scheduled Job Activity
  1007, // Process Activity
  // Network Activity
  4001, // Network Activity
  4002, // HTTP Activity
  4003, // DNS Activity
  4004, // DHCP Activity
  4005, // RDP Activity
  4006, // SMB Activity
  4007, // SSH Activity
  4008, // FTP Activity
  4009, // Email Activity
  4010, // Email File Activity
  4011, // Email URL Activity
  // Identity & Access Management
  3001, // Account Change
  3002, // Authentication
  3003, // Authorize Session
  3004, // Entity Management
  3005, // User Access Management
  3006, // Group Management
  // Findings
  2001, // Security Finding
  2002, // Vulnerability Finding
  2003, // Compliance Finding
  2004, // Detection Finding
  2005, // Incident Finding
  // Application Activity
  6001, // Web Resources Activity
  6002, // Application Lifecycle
  6003, // API Activity
  6004, // Web Resource Access Activity
  // Discovery
  5001, // Device Inventory Info
  5002, // Device Config State
  5003, // User Inventory Info
  5004, // Operating System Patch State
  5019, // Device Config State Change
]);

// ============================================
// Validation Functions
// ============================================

/**
 * Validate a MITRE technique ID format.
 *
 * Valid formats:
 * - T1234 (base technique)
 * - T1234.001 (sub-technique)
 *
 * Requirement 17.4: Validate MITRE technique ID format (T####.###)
 *
 * @param techniqueId - The technique ID to validate
 * @returns True if the format is valid
 */
export function isValidMitreTechniqueId(techniqueId: string): boolean {
  return MITRE_TECHNIQUE_PATTERN.test(techniqueId);
}

/**
 * Validate a MITRE tactic name.
 *
 * Requirement 17.5: Validate MITRE tactic names
 *
 * @param tactic - The tactic name to validate
 * @returns True if the tactic is valid
 */
export function isValidMitreTactic(tactic: string): boolean {
  return VALID_MITRE_TACTICS.includes(tactic.toLowerCase() as typeof VALID_MITRE_TACTICS[number]);
}

/**
 * Validate a class_uid value.
 *
 * Requirement 17.2: Check class_uid references valid OCSF classes
 *
 * @param classUid - The class UID to validate
 * @param knownClassUids - Optional set of known valid class UIDs
 * @returns True if the class UID is valid
 */
export function isValidClassUid(classUid: number, knownClassUids?: Set<number>): boolean {
  // Class UID must be positive
  if (classUid <= 0) {
    return false;
  }

  // If we have a set of known class UIDs, check against it
  if (knownClassUids) {
    return knownClassUids.has(classUid);
  }

  // Use the default known class UIDs
  return KNOWN_OCSF_CLASS_UIDS.has(classUid);
}

/**
 * Validate detection coverage metadata.
 *
 * Requirements 17.4, 17.5: Validate MITRE technique IDs and tactic names
 *
 * @param coverage - The detection coverage to validate
 * @param path - The path prefix for error messages
 * @returns Validation errors and warnings
 */
export function validateDetectionCoverage(
  coverage: DetectionCoverage,
  path: string
): { errors: ValidationError[]; warnings: ValidationWarning[] } {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];

  // Validate MITRE technique IDs (Requirement 17.4)
  coverage.mitre_techniques.forEach((techniqueId, index) => {
    if (!isValidMitreTechniqueId(techniqueId)) {
      errors.push({
        path: `${path}.mitre_techniques[${index}]`,
        message: `Invalid MITRE technique ID format: "${techniqueId}". Expected format: T####.### (e.g., T1071.001)`,
        code: 'INVALID_MITRE_TECHNIQUE_ID',
      });
    }
  });

  // Validate MITRE tactic names (Requirement 17.5)
  coverage.mitre_tactics.forEach((tactic, index) => {
    if (!isValidMitreTactic(tactic)) {
      errors.push({
        path: `${path}.mitre_tactics[${index}]`,
        message: `Invalid MITRE tactic name: "${tactic}". Valid tactics: ${VALID_MITRE_TACTICS.join(', ')}`,
        code: 'INVALID_MITRE_TACTIC',
      });
    }
  });

  // Best practice warnings (Requirement 17.7)
  if (coverage.mitre_techniques.length === 0 && coverage.mitre_tactics.length > 0) {
    warnings.push({
      path: `${path}.mitre_techniques`,
      message: 'Detection coverage has tactics but no techniques. Consider adding specific technique IDs.',
      code: 'MISSING_TECHNIQUES',
    });
  }

  if (coverage.data_sources.length === 0) {
    warnings.push({
      path: `${path}.data_sources`,
      message: 'No data sources specified. Consider adding data source information for better coverage tracking.',
      code: 'MISSING_DATA_SOURCES',
    });
  }

  if (!coverage.confidence_level) {
    warnings.push({
      path: `${path}.confidence_level`,
      message: 'No confidence level specified. Consider setting a confidence level (low, medium, high).',
      code: 'MISSING_CONFIDENCE_LEVEL',
    });
  }

  return { errors, warnings };
}

/**
 * Validate a single table entry.
 *
 * Requirements 17.1, 17.2: Validate index model structure and class_uid
 *
 * @param table - The table entry to validate
 * @param index - The index of the table in the list
 * @param knownClassUids - Optional set of known valid class UIDs
 * @returns Validation errors and warnings
 */
export function validateTableEntry(
  table: TableEntry,
  index: number,
  knownClassUids?: Set<number>
): { errors: ValidationError[]; warnings: ValidationWarning[] } {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];
  const path = `tables[${index}]`;

  // Validate table_name (Requirement 17.1)
  if (!table.table_name || table.table_name.trim() === '') {
    errors.push({
      path: `${path}.table_name`,
      message: 'Table name is required',
      code: 'MISSING_TABLE_NAME',
    });
  }

  // Validate class_uid (Requirement 17.2)
  if (table.class_uid <= 0) {
    errors.push({
      path: `${path}.class_uid`,
      message: `Invalid class_uid: ${table.class_uid}. Class UID must be a positive integer.`,
      code: 'INVALID_CLASS_UID',
    });
  } else if (!isValidClassUid(table.class_uid, knownClassUids)) {
    warnings.push({
      path: `${path}.class_uid`,
      message: `Unknown class_uid: ${table.class_uid}. This may not be a valid OCSF event class.`,
      code: 'UNKNOWN_CLASS_UID',
    });
  }

  // Validate ocsf_version (Requirement 17.1)
  if (!table.ocsf_version || table.ocsf_version.trim() === '') {
    errors.push({
      path: `${path}.ocsf_version`,
      message: 'OCSF version is required',
      code: 'MISSING_OCSF_VERSION',
    });
  }

  // Validate dialect (Requirement 17.1)
  if (!table.dialect || table.dialect.trim() === '') {
    errors.push({
      path: `${path}.dialect`,
      message: 'SQL dialect is required',
      code: 'MISSING_DIALECT',
    });
  }

  // Validate detection coverage if present (Requirements 17.4, 17.5)
  if (table.detection_coverage) {
    const coverageResult = validateDetectionCoverage(
      table.detection_coverage,
      `${path}.detection_coverage`
    );
    errors.push(...coverageResult.errors);
    warnings.push(...coverageResult.warnings);
  }

  // Best practice warnings (Requirement 17.7)
  if (!table.schema_name) {
    warnings.push({
      path: `${path}.schema_name`,
      message: 'No schema name specified. Consider adding a schema name for better organization.',
      code: 'MISSING_SCHEMA_NAME',
    });
  }

  return { errors, warnings };
}

/**
 * Check for orphan lineage records (target_table not in registry).
 *
 * Requirement 17.3: Check for orphan lineage records
 *
 * @param lineageRecords - Source lineage records to check
 * @param tables - Registered tables
 * @returns Validation errors for orphan records
 */
export function findOrphanLineageRecords(
  lineageRecords: SourceLineageRecord[],
  tables: TableEntry[]
): ValidationError[] {
  const errors: ValidationError[] = [];
  const tableNames = new Set(tables.map((t) => t.table_name));

  lineageRecords.forEach((record, index) => {
    if (!tableNames.has(record.target_table)) {
      errors.push({
        path: `source_lineage[${index}]`,
        message: `Orphan lineage record: target_table "${record.target_table}" is not registered in the table registry`,
        code: 'ORPHAN_LINEAGE_RECORD',
      });
    }
  });

  return errors;
}

/**
 * Validate the complete index model.
 *
 * Requirements 17.1-17.7: Complete index model validation
 *
 * @param tables - Registered tables
 * @param sourceLineage - Source lineage records
 * @param knownClassUids - Optional set of known valid class UIDs
 * @returns Complete validation result
 */
export function validateIndexModel(
  tables: TableEntry[],
  sourceLineage: SourceLineageRecord[],
  knownClassUids?: Set<number>
): IndexValidationResult {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];

  // Validate each table entry (Requirements 17.1, 17.2, 17.4, 17.5)
  tables.forEach((table, index) => {
    const tableResult = validateTableEntry(table, index, knownClassUids);
    errors.push(...tableResult.errors);
    warnings.push(...tableResult.warnings);
  });

  // Check for orphan lineage records (Requirement 17.3)
  const orphanErrors = findOrphanLineageRecords(sourceLineage, tables);
  errors.push(...orphanErrors);

  // Check for duplicate table names (Requirement 17.1)
  const tableNameCounts = new Map<string, number>();
  tables.forEach((table) => {
    const count = tableNameCounts.get(table.table_name) || 0;
    tableNameCounts.set(table.table_name, count + 1);
  });
  tableNameCounts.forEach((count, name) => {
    if (count > 1) {
      errors.push({
        path: 'tables',
        message: `Duplicate table name: "${name}" appears ${count} times`,
        code: 'DUPLICATE_TABLE_NAME',
      });
    }
  });

  // Best practice warnings (Requirement 17.7)
  if (tables.length === 0) {
    warnings.push({
      path: 'tables',
      message: 'No tables registered. Consider registering at least one table.',
      code: 'NO_TABLES',
    });
  }

  const tablesWithCoverage = tables.filter((t) => t.detection_coverage);
  if (tables.length > 0 && tablesWithCoverage.length === 0) {
    warnings.push({
      path: 'tables',
      message: 'No tables have detection coverage metadata. Consider adding MITRE ATT&CK coverage information.',
      code: 'NO_DETECTION_COVERAGE',
    });
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

// ============================================
// Utility Functions
// ============================================

/**
 * Format a validation error for display.
 *
 * @param error - The validation error
 * @returns Formatted error string
 */
export function formatValidationError(error: ValidationError): string {
  return `[${error.code}] ${error.path}: ${error.message}`;
}

/**
 * Format a validation warning for display.
 *
 * @param warning - The validation warning
 * @returns Formatted warning string
 */
export function formatValidationWarning(warning: ValidationWarning): string {
  return `[${warning.code}] ${warning.path}: ${warning.message}`;
}

/**
 * Get a summary of validation results.
 *
 * @param result - The validation result
 * @returns Summary string
 */
export function getValidationSummary(result: IndexValidationResult): string {
  if (result.valid && result.warnings.length === 0) {
    return 'Index model is valid with no warnings.';
  }
  if (result.valid) {
    return `Index model is valid with ${result.warnings.length} warning(s).`;
  }
  return `Index model has ${result.errors.length} error(s) and ${result.warnings.length} warning(s).`;
}

// ============================================
// Export Model Validation
// ============================================

/**
 * Validate an IndexModelExport object.
 * 
 * This is a convenience wrapper for validateIndexModel that accepts
 * the export format directly.
 * 
 * @param exportModel - The exported index model to validate
 * @param knownClassUids - Optional set of known valid class UIDs
 * @returns Complete validation result
 */
export function validateIndexModelExport(
  exportModel: { tables: TableEntry[]; source_lineage: SourceLineageRecord[] },
  knownClassUids?: Set<number>
): IndexValidationResult {
  return validateIndexModel(
    exportModel.tables || [],
    exportModel.source_lineage || [],
    knownClassUids
  );
}
