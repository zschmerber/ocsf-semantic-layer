/**
 * Index API client functions for the OCSF Semantic Model Editor.
 *
 * Provides functions for interacting with the /api/index/* endpoints.
 *
 * Requirements: 12.1, 12.4, 12.5
 */

import { get, post, put, del } from './client';
import type {
  TableEntry,
  SourceLineageRecord,
  FieldLineageRecord,
  LineageGraph,
  DetectionCoverageSummary,
  CoverageMatrix,
  TableStatistics,
  PartitionEntry,
  IndexModelExport,
  ImportResult,
} from '../types';

// ============================================
// Table Registry API
// ============================================

/**
 * Parameters for listing tables.
 */
export interface ListTablesParams {
  /** Filter by OCSF class UID */
  class_uid?: number;
  /** Filter by active status */
  is_active?: boolean;
}

/**
 * Get all registered tables with optional filtering.
 *
 * @param params - Optional filter parameters
 * @returns List of table entries
 */
export function getTables(params?: ListTablesParams): Promise<TableEntry[]> {
  const query = new URLSearchParams();
  if (params?.class_uid !== undefined) {
    query.set('class_uid', params.class_uid.toString());
  }
  if (params?.is_active !== undefined) {
    query.set('is_active', params.is_active.toString());
  }
  const queryString = query.toString();
  return get<TableEntry[]>(`/index/tables${queryString ? `?${queryString}` : ''}`);
}

/**
 * Get a single table by ID.
 *
 * @param id - Table ID
 * @returns Table entry
 */
export function getTable(id: number): Promise<TableEntry> {
  return get<TableEntry>(`/index/tables/${id}`);
}

/**
 * Request body for registering a new table.
 */
export interface RegisterTableRequest {
  table_name: string;
  schema_name?: string;
  class_uid: number;
  ocsf_version: string;
  dialect: string;
  metadata?: Record<string, string>;
  detection_coverage?: {
    mitre_techniques?: string[];
    mitre_tactics?: string[];
    data_sources?: string[];
    detection_rules?: string[];
    kill_chain_phases?: string[];
    confidence_level?: string;
    max_severity?: string;
  };
}

/**
 * Register a new table in the index.
 *
 * @param table - Table registration request
 * @returns Created table entry with assigned ID
 */
export function registerTable(table: RegisterTableRequest): Promise<TableEntry> {
  return post<TableEntry>('/index/tables', table);
}

/**
 * Update an existing table.
 *
 * @param id - Table ID
 * @param table - Updated table data
 * @returns Updated table entry
 */
export function updateTable(id: number, table: RegisterTableRequest): Promise<TableEntry> {
  return put<TableEntry>(`/index/tables/${id}`, table);
}

/**
 * Deactivate (soft-delete) a table.
 *
 * @param id - Table ID
 * @returns Deactivated table entry
 */
export function deactivateTable(id: number): Promise<TableEntry> {
  return del<TableEntry>(`/index/tables/${id}`);
}

// ============================================
// Lineage API
// ============================================

/**
 * Parameters for listing source lineage records.
 */
export interface ListSourceLineageParams {
  /** Filter by target table name */
  target_table?: string;
  /** Filter by source system */
  source_system?: string;
  /** Maximum number of records to return */
  limit?: number;
  /** Number of records to skip */
  offset?: number;
}

/**
 * Get source lineage records with optional filtering.
 *
 * @param params - Optional filter parameters
 * @returns List of source lineage records
 */
export function getSourceLineage(params?: ListSourceLineageParams): Promise<SourceLineageRecord[]> {
  const query = new URLSearchParams();
  if (params?.target_table) {
    query.set('target_table', params.target_table);
  }
  if (params?.source_system) {
    query.set('source_system', params.source_system);
  }
  if (params?.limit !== undefined) {
    query.set('limit', params.limit.toString());
  }
  if (params?.offset !== undefined) {
    query.set('offset', params.offset.toString());
  }
  const queryString = query.toString();
  return get<SourceLineageRecord[]>(`/index/lineage/source${queryString ? `?${queryString}` : ''}`);
}

/**
 * Request body for recording source lineage.
 */
export interface RecordSourceLineageRequest {
  source_system: string;
  source_table: string;
  target_table: string;
  record_count?: number;
  metadata?: Record<string, string>;
}

/**
 * Record a new source lineage entry.
 *
 * @param lineage - Source lineage record to create
 * @returns Created source lineage record with assigned ID
 */
export function recordSourceLineage(lineage: RecordSourceLineageRequest): Promise<SourceLineageRecord> {
  return post<SourceLineageRecord>('/index/lineage/source', lineage);
}

/**
 * Get lineage data formatted as a graph for visualization.
 *
 * @param targetTable - Optional target table to filter by
 * @returns Lineage graph with nodes and edges
 */
export function getLineageGraph(targetTable?: string): Promise<LineageGraph> {
  const query = targetTable ? `?target_table=${encodeURIComponent(targetTable)}` : '';
  return get<LineageGraph>(`/index/lineage/graph${query}`);
}

/**
 * Parameters for listing field lineage records.
 */
export interface ListFieldLineageParams {
  /** Filter by target field name */
  target_field?: string;
  /** Filter by source lineage ID */
  source_lineage_id?: number;
}

/**
 * Get field lineage records with optional filtering.
 *
 * @param params - Optional filter parameters
 * @returns List of field lineage records
 */
export function getFieldLineage(params?: ListFieldLineageParams): Promise<FieldLineageRecord[]> {
  const query = new URLSearchParams();
  if (params?.target_field) {
    query.set('target_field', params.target_field);
  }
  if (params?.source_lineage_id !== undefined) {
    query.set('source_lineage_id', params.source_lineage_id.toString());
  }
  const queryString = query.toString();
  return get<FieldLineageRecord[]>(`/index/lineage/field${queryString ? `?${queryString}` : ''}`);
}

/**
 * Request body for recording field lineage.
 */
export interface RecordFieldLineageRequest {
  source_lineage_id: number;
  source_field: string;
  target_field: string;
  transformation?: string;
  ocsf_version?: string;
}

/**
 * Record a new field lineage entry.
 *
 * @param lineage - Field lineage record to create
 * @returns Created field lineage record with assigned ID
 */
export function recordFieldLineage(lineage: RecordFieldLineageRequest): Promise<FieldLineageRecord> {
  return post<FieldLineageRecord>('/index/lineage/field', lineage);
}

// ============================================
// Coverage API
// ============================================

/**
 * Get aggregated detection coverage summary across all tables.
 *
 * @returns Detection coverage summary
 */
export function getCoverageSummary(): Promise<DetectionCoverageSummary> {
  return get<DetectionCoverageSummary>('/index/coverage/summary');
}

/**
 * Get tables that cover a specific MITRE ATT&CK technique.
 *
 * @param techniqueId - MITRE technique ID (e.g., "T1071.001")
 * @returns List of tables covering the technique
 */
export function getTablesByTechnique(techniqueId: string): Promise<TableEntry[]> {
  return get<TableEntry[]>(`/index/coverage/techniques/${encodeURIComponent(techniqueId)}`);
}

/**
 * Get tables that cover a specific MITRE ATT&CK tactic.
 *
 * @param tactic - MITRE tactic name (e.g., "initial-access")
 * @returns List of tables covering the tactic
 */
export function getTablesByTactic(tactic: string): Promise<TableEntry[]> {
  return get<TableEntry[]>(`/index/coverage/tactics/${encodeURIComponent(tactic)}`);
}

/**
 * Get coverage data formatted as a MITRE ATT&CK matrix.
 *
 * @returns Coverage matrix with tactics and techniques
 */
export function getCoverageMatrix(): Promise<CoverageMatrix> {
  return get<CoverageMatrix>('/index/coverage/matrix');
}

// ============================================
// Statistics API
// ============================================

/**
 * Get statistics for a specific table.
 *
 * @param tableName - Name of the table
 * @returns Table statistics including column-level stats
 */
export function getStatistics(tableName: string): Promise<TableStatistics> {
  return get<TableStatistics>(`/index/statistics/${encodeURIComponent(tableName)}`);
}

/**
 * Trigger statistics collection for a table.
 *
 * @param tableName - Name of the table
 * @param sampleRate - Optional sample rate (0.0 to 1.0)
 * @returns Collected table statistics
 */
export function collectStatistics(tableName: string, sampleRate?: number): Promise<TableStatistics> {
  return post<TableStatistics>(`/index/statistics/${encodeURIComponent(tableName)}/collect`, {
    sample_rate: sampleRate,
  });
}

// ============================================
// Partition API
// ============================================

/**
 * Parameters for listing partitions.
 */
export interface ListPartitionsParams {
  /** Filter partitions starting from this time (ISO 8601) */
  start_time?: string;
  /** Filter partitions ending before this time (ISO 8601) */
  end_time?: string;
}

/**
 * Get partition metadata for a table.
 *
 * @param tableName - Name of the table
 * @param params - Optional time range filter
 * @returns List of partition entries
 */
export function getPartitions(tableName: string, params?: ListPartitionsParams): Promise<PartitionEntry[]> {
  const query = new URLSearchParams();
  if (params?.start_time) {
    query.set('start_time', params.start_time);
  }
  if (params?.end_time) {
    query.set('end_time', params.end_time);
  }
  const queryString = query.toString();
  return get<PartitionEntry[]>(
    `/index/partitions/${encodeURIComponent(tableName)}${queryString ? `?${queryString}` : ''}`
  );
}

/**
 * Request body for updating partition metadata.
 */
export interface UpdatePartitionRequest {
  table_name: string;
  partition_key: string;
  start_time: string;
  end_time: string;
  row_count: number;
  size_bytes: number;
}

/**
 * Update or create partition metadata.
 *
 * @param partition - Partition metadata to update/create
 * @returns Updated partition entry
 */
export function updatePartition(partition: UpdatePartitionRequest): Promise<PartitionEntry> {
  return post<PartitionEntry>('/index/partitions', partition);
}

// ============================================
// Export/Import API
// ============================================

/**
 * Export the index model as JSON.
 *
 * @param combined - If true, include semantic model in export
 * @returns Exported index model
 */
export function exportIndexModel(combined?: boolean): Promise<IndexModelExport> {
  const query = combined ? '?combined=true' : '';
  return get<IndexModelExport>(`/index/export${query}`);
}

/**
 * Import an index model from JSON.
 *
 * @param model - Index model to import
 * @returns Import result with counts
 */
export function importIndexModel(model: IndexModelExport): Promise<ImportResult> {
  return post<ImportResult>('/index/import', model);
}
