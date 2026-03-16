/**
 * Pure helper functions for the ETL Engine GUI.
 *
 * Provides utility functions for:
 * - Status badge CSS class mapping
 * - Monaco Editor language detection from file extensions
 * - Lifecycle action button enablement based on job status
 * - Client-side UUID generation
 * - Job creation form validation
 *
 * Requirements: 4.3, 5.8, 5.11, 7.2, 8.2–8.6, 9.3
 */

import type { FieldMapping, SourceConfig } from '../types/etl';

// ============================================
// Form Data Types
// ============================================

/**
 * Shape of the job creation form data used for validation.
 */
export interface JobFormData {
  plugin_name: string;
  mappings: FieldMapping[];
  source_config: SourceConfig | null;
  delta_table_uri: string;
}

// ============================================
// Status Badge Mapping
// ============================================

/**
 * Maps a job status string to the corresponding CSS class for badge rendering.
 *
 * | Status              | CSS Class          |
 * |---------------------|--------------------|
 * | Pending             | status-pending     |
 * | Generating/Compiling/Testing | status-active |
 * | Running             | status-running     |
 * | Completed           | status-completed   |
 * | Failed:*            | status-failed      |
 * | Unknown             | status-pending     |
 */
export function getStatusBadgeClass(status: string): string {
  if (status.startsWith('Failed')) {
    return 'status-failed';
  }

  switch (status) {
    case 'Pending':
      return 'status-pending';
    case 'Generating':
    case 'Compiling':
    case 'Testing':
      return 'status-active';
    case 'Running':
      return 'status-running';
    case 'Completed':
      return 'status-completed';
    default:
      return 'status-pending';
  }
}

// ============================================
// Monaco Language Detection
// ============================================

/** Map of file extensions to Monaco Editor language IDs. */
const EXTENSION_LANGUAGE_MAP: Record<string, string> = {
  '.go': 'go',
  '.mod': 'go',
  '.sql': 'sql',
  '.yaml': 'yaml',
  '.yml': 'yaml',
  '.json': 'json',
};

/**
 * Determines the Monaco Editor language ID from a file path's extension.
 *
 * Supported extensions: `.go`, `.mod` → "go"; `.sql` → "sql";
 * `.yaml`, `.yml` → "yaml"; `.json` → "json".
 * Returns "plaintext" for unrecognized extensions.
 */
export function getMonacoLanguage(filePath: string): string {
  const lastDot = filePath.lastIndexOf('.');
  if (lastDot === -1) {
    return 'plaintext';
  }
  const ext = filePath.slice(lastDot).toLowerCase();
  return EXTENSION_LANGUAGE_MAP[ext] ?? 'plaintext';
}

// ============================================
// Lifecycle Action Enablement
// ============================================

/**
 * Returns the set of lifecycle action names enabled for a given job status.
 *
 * - "Pending"   → { "Generate" }
 * - "Compiling" → { "Compile" }
 * - "Testing"   → { "Test" }
 * - "Running"   → { "Stop" }
 * - "Completed", "Failed:*", or any other status → empty set
 */
export function getEnabledActions(status: string): Set<string> {
  if (status === 'Completed' || status.startsWith('Failed')) {
    return new Set();
  }

  switch (status) {
    case 'Pending':
      return new Set(['Generate']);
    case 'Compiling':
      return new Set(['Compile']);
    case 'Testing':
      return new Set(['Test']);
    case 'Running':
      return new Set(['Stop']);
    default:
      return new Set();
  }
}

// ============================================
// UUID Generation
// ============================================

/**
 * Generates a UUID v4 string for use as a client-side job ID.
 * Uses the Web Crypto API (`crypto.randomUUID()`).
 */
export function generateJobId(): string {
  return crypto.randomUUID();
}

// ============================================
// Form Validation
// ============================================

/**
 * Validates that all required fields in the job creation form are present.
 *
 * Returns `true` only when:
 * - `plugin_name` is non-empty (after trim)
 * - `mappings` has at least one entry
 * - `source_config` is not null/undefined
 * - `delta_table_uri` is non-empty (after trim)
 */
export function validateJobForm(form: JobFormData): boolean {
  if (!form.plugin_name.trim()) {
    return false;
  }
  if (!form.mappings || form.mappings.length === 0) {
    return false;
  }
  if (!form.source_config) {
    return false;
  }
  if (!form.delta_table_uri.trim()) {
    return false;
  }
  return true;
}
