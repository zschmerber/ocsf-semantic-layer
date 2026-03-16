/**
 * Unit tests for index model validation utilities.
 *
 * Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6, 17.7
 */

import { describe, it, expect } from 'vitest';
import {
  validateIndexModel,
  validateTableEntry,
  validateDetectionCoverage,
  findOrphanLineageRecords,
  isValidMitreTechniqueId,
  isValidMitreTactic,
  isValidClassUid,
  VALID_MITRE_TACTICS,
} from './indexValidation';
import type { TableEntry, SourceLineageRecord, DetectionCoverage } from '../types';

// ============================================
// Test Fixtures
// ============================================

function createValidTableEntry(overrides: Partial<TableEntry> = {}): TableEntry {
  return {
    id: 1,
    table_name: 'test_table',
    class_uid: 4003, // DNS Activity
    ocsf_version: '1.6.0',
    dialect: 'snowflake',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    is_active: true,
    metadata: {},
    ...overrides,
  };
}

function createValidSourceLineage(overrides: Partial<SourceLineageRecord> = {}): SourceLineageRecord {
  return {
    id: 1,
    source_system: 'test_system',
    source_table: 'raw_events',
    target_table: 'test_table',
    ingestion_timestamp: '2024-01-01T00:00:00Z',
    metadata: {},
    ...overrides,
  };
}

function createValidDetectionCoverage(overrides: Partial<DetectionCoverage> = {}): DetectionCoverage {
  return {
    mitre_techniques: ['T1071', 'T1071.001'],
    mitre_tactics: ['command-and-control', 'exfiltration'],
    data_sources: ['Network Traffic'],
    detection_rules: ['rule_001'],
    kill_chain_phases: ['C2'],
    confidence_level: 'high',
    ...overrides,
  };
}

// ============================================
// MITRE Technique ID Validation Tests
// ============================================

describe('isValidMitreTechniqueId', () => {
  it('should accept valid base technique IDs', () => {
    expect(isValidMitreTechniqueId('T1071')).toBe(true);
    expect(isValidMitreTechniqueId('T1234')).toBe(true);
    expect(isValidMitreTechniqueId('T0001')).toBe(true);
    expect(isValidMitreTechniqueId('T9999')).toBe(true);
  });

  it('should accept valid sub-technique IDs', () => {
    expect(isValidMitreTechniqueId('T1071.001')).toBe(true);
    expect(isValidMitreTechniqueId('T1234.012')).toBe(true);
    expect(isValidMitreTechniqueId('T0001.999')).toBe(true);
  });

  it('should reject invalid technique IDs', () => {
    expect(isValidMitreTechniqueId('T123')).toBe(false); // Too short
    expect(isValidMitreTechniqueId('T12345')).toBe(false); // Too long
    expect(isValidMitreTechniqueId('T1071.01')).toBe(false); // Sub-technique too short
    expect(isValidMitreTechniqueId('T1071.0001')).toBe(false); // Sub-technique too long
    expect(isValidMitreTechniqueId('1071')).toBe(false); // Missing T prefix
    expect(isValidMitreTechniqueId('TABC')).toBe(false); // Non-numeric
    expect(isValidMitreTechniqueId('')).toBe(false); // Empty
    expect(isValidMitreTechniqueId('T1071.ABC')).toBe(false); // Non-numeric sub-technique
  });
});

// ============================================
// MITRE Tactic Validation Tests
// ============================================

describe('isValidMitreTactic', () => {
  it('should accept all valid MITRE tactics', () => {
    VALID_MITRE_TACTICS.forEach((tactic) => {
      expect(isValidMitreTactic(tactic)).toBe(true);
    });
  });

  it('should be case-insensitive', () => {
    expect(isValidMitreTactic('RECONNAISSANCE')).toBe(true);
    expect(isValidMitreTactic('Initial-Access')).toBe(true);
    expect(isValidMitreTactic('COMMAND-AND-CONTROL')).toBe(true);
  });

  it('should reject invalid tactics', () => {
    expect(isValidMitreTactic('invalid-tactic')).toBe(false);
    expect(isValidMitreTactic('')).toBe(false);
    expect(isValidMitreTactic('recon')).toBe(false);
    expect(isValidMitreTactic('c2')).toBe(false);
  });
});

// ============================================
// Class UID Validation Tests
// ============================================

describe('isValidClassUid', () => {
  it('should accept known OCSF class UIDs', () => {
    expect(isValidClassUid(4003)).toBe(true); // DNS Activity
    expect(isValidClassUid(3002)).toBe(true); // Authentication
    expect(isValidClassUid(2001)).toBe(true); // Security Finding
  });

  it('should reject invalid class UIDs', () => {
    expect(isValidClassUid(0)).toBe(false);
    expect(isValidClassUid(-1)).toBe(false);
  });

  it('should warn about unknown class UIDs', () => {
    // Unknown but positive class UID - returns false with default set
    expect(isValidClassUid(99999)).toBe(false);
  });

  it('should use custom class UID set when provided', () => {
    const customSet = new Set([100, 200, 300]);
    expect(isValidClassUid(100, customSet)).toBe(true);
    expect(isValidClassUid(4003, customSet)).toBe(false);
  });
});

// ============================================
// Detection Coverage Validation Tests
// ============================================

describe('validateDetectionCoverage', () => {
  it('should pass for valid detection coverage', () => {
    const coverage = createValidDetectionCoverage();
    const result = validateDetectionCoverage(coverage, 'test');
    
    expect(result.errors).toHaveLength(0);
  });

  it('should error on invalid MITRE technique IDs', () => {
    const coverage = createValidDetectionCoverage({
      mitre_techniques: ['T1071', 'INVALID', 'T123'],
    });
    const result = validateDetectionCoverage(coverage, 'test');
    
    expect(result.errors).toHaveLength(2);
    expect(result.errors[0].code).toBe('INVALID_MITRE_TECHNIQUE_ID');
    expect(result.errors[1].code).toBe('INVALID_MITRE_TECHNIQUE_ID');
  });

  it('should error on invalid MITRE tactics', () => {
    const coverage = createValidDetectionCoverage({
      mitre_tactics: ['command-and-control', 'invalid-tactic'],
    });
    const result = validateDetectionCoverage(coverage, 'test');
    
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].code).toBe('INVALID_MITRE_TACTIC');
  });

  it('should warn when tactics exist but no techniques', () => {
    const coverage = createValidDetectionCoverage({
      mitre_techniques: [],
      mitre_tactics: ['execution'],
    });
    const result = validateDetectionCoverage(coverage, 'test');
    
    expect(result.warnings.some((w) => w.code === 'MISSING_TECHNIQUES')).toBe(true);
  });

  it('should warn when no data sources specified', () => {
    const coverage = createValidDetectionCoverage({
      data_sources: [],
    });
    const result = validateDetectionCoverage(coverage, 'test');
    
    expect(result.warnings.some((w) => w.code === 'MISSING_DATA_SOURCES')).toBe(true);
  });

  it('should warn when no confidence level specified', () => {
    const coverage = createValidDetectionCoverage({
      confidence_level: undefined,
    });
    const result = validateDetectionCoverage(coverage, 'test');
    
    expect(result.warnings.some((w) => w.code === 'MISSING_CONFIDENCE_LEVEL')).toBe(true);
  });
});

// ============================================
// Table Entry Validation Tests
// ============================================

describe('validateTableEntry', () => {
  it('should pass for valid table entry', () => {
    const table = createValidTableEntry();
    const result = validateTableEntry(table, 0);
    
    expect(result.errors).toHaveLength(0);
  });

  it('should error on missing table name', () => {
    const table = createValidTableEntry({ table_name: '' });
    const result = validateTableEntry(table, 0);
    
    expect(result.errors.some((e) => e.code === 'MISSING_TABLE_NAME')).toBe(true);
  });

  it('should error on invalid class_uid', () => {
    const table = createValidTableEntry({ class_uid: 0 });
    const result = validateTableEntry(table, 0);
    
    expect(result.errors.some((e) => e.code === 'INVALID_CLASS_UID')).toBe(true);
  });

  it('should warn on unknown class_uid', () => {
    const table = createValidTableEntry({ class_uid: 99999 });
    const result = validateTableEntry(table, 0);
    
    expect(result.warnings.some((w) => w.code === 'UNKNOWN_CLASS_UID')).toBe(true);
  });

  it('should error on missing ocsf_version', () => {
    const table = createValidTableEntry({ ocsf_version: '' });
    const result = validateTableEntry(table, 0);
    
    expect(result.errors.some((e) => e.code === 'MISSING_OCSF_VERSION')).toBe(true);
  });

  it('should error on missing dialect', () => {
    const table = createValidTableEntry({ dialect: '' });
    const result = validateTableEntry(table, 0);
    
    expect(result.errors.some((e) => e.code === 'MISSING_DIALECT')).toBe(true);
  });

  it('should warn on missing schema_name', () => {
    const table = createValidTableEntry({ schema_name: undefined });
    const result = validateTableEntry(table, 0);
    
    expect(result.warnings.some((w) => w.code === 'MISSING_SCHEMA_NAME')).toBe(true);
  });

  it('should validate detection coverage when present', () => {
    const table = createValidTableEntry({
      detection_coverage: createValidDetectionCoverage({
        mitre_techniques: ['INVALID'],
      }),
    });
    const result = validateTableEntry(table, 0);
    
    expect(result.errors.some((e) => e.code === 'INVALID_MITRE_TECHNIQUE_ID')).toBe(true);
  });
});

// ============================================
// Orphan Lineage Detection Tests
// ============================================

describe('findOrphanLineageRecords', () => {
  it('should return empty array when all lineage records have valid targets', () => {
    const tables = [createValidTableEntry({ table_name: 'table_a' })];
    const lineage = [createValidSourceLineage({ target_table: 'table_a' })];
    
    const errors = findOrphanLineageRecords(lineage, tables);
    
    expect(errors).toHaveLength(0);
  });

  it('should detect orphan lineage records', () => {
    const tables = [createValidTableEntry({ table_name: 'table_a' })];
    const lineage = [
      createValidSourceLineage({ target_table: 'table_a' }),
      createValidSourceLineage({ target_table: 'nonexistent_table' }),
    ];
    
    const errors = findOrphanLineageRecords(lineage, tables);
    
    expect(errors).toHaveLength(1);
    expect(errors[0].code).toBe('ORPHAN_LINEAGE_RECORD');
    expect(errors[0].message).toContain('nonexistent_table');
  });

  it('should detect multiple orphan records', () => {
    const tables: TableEntry[] = [];
    const lineage = [
      createValidSourceLineage({ target_table: 'orphan_1' }),
      createValidSourceLineage({ target_table: 'orphan_2' }),
    ];
    
    const errors = findOrphanLineageRecords(lineage, tables);
    
    expect(errors).toHaveLength(2);
  });
});

// ============================================
// Full Index Model Validation Tests
// ============================================

describe('validateIndexModel', () => {
  it('should pass for valid index model', () => {
    const tables = [createValidTableEntry()];
    const lineage = [createValidSourceLineage()];
    
    const result = validateIndexModel(tables, lineage);
    
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('should detect duplicate table names', () => {
    const tables = [
      createValidTableEntry({ id: 1, table_name: 'duplicate' }),
      createValidTableEntry({ id: 2, table_name: 'duplicate' }),
    ];
    
    const result = validateIndexModel(tables, []);
    
    expect(result.errors.some((e) => e.code === 'DUPLICATE_TABLE_NAME')).toBe(true);
  });

  it('should warn when no tables registered', () => {
    const result = validateIndexModel([], []);
    
    expect(result.warnings.some((w) => w.code === 'NO_TABLES')).toBe(true);
  });

  it('should warn when no tables have detection coverage', () => {
    const tables = [
      createValidTableEntry({ detection_coverage: undefined }),
    ];
    
    const result = validateIndexModel(tables, []);
    
    expect(result.warnings.some((w) => w.code === 'NO_DETECTION_COVERAGE')).toBe(true);
  });

  it('should aggregate errors from all validation checks', () => {
    const tables = [
      createValidTableEntry({ table_name: '', class_uid: 0 }),
    ];
    const lineage = [
      createValidSourceLineage({ target_table: 'nonexistent' }),
    ];
    
    const result = validateIndexModel(tables, lineage);
    
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBeGreaterThan(2);
  });

  it('should include path information in errors', () => {
    const tables = [
      createValidTableEntry({ table_name: '' }),
    ];
    
    const result = validateIndexModel(tables, []);
    
    const tableNameError = result.errors.find((e) => e.code === 'MISSING_TABLE_NAME');
    expect(tableNameError?.path).toBe('tables[0].table_name');
  });
});
