/**
 * TypeScript types for the ETL Engine GUI.
 * These types mirror the Rust ETL Engine backend types for API compatibility.
 */

// ============================================
// API Response Types
// ============================================

/**
 * Health check response from the ETL Engine backend.
 */
export interface HealthResponse {
  status: string;
  tangent_available: boolean;
}

/**
 * Summary of an ETL job returned by the list endpoint.
 */
export interface EtlJobSummary {
  job_id: string;
  plugin_name: string;
  status: string;
}

/**
 * Detailed ETL job information returned by the detail endpoint.
 */
export interface EtlJobDetail {
  job_id: string;
  plugin_name: string;
  ocsf_class_uid: number;
  status: string;
  artifact_files: string[];
  log_lines: number;
}

/**
 * Response from a job lifecycle action (generate, compile, test, run, stop).
 */
export interface JobActionResponse {
  job_id: string;
  status: string;
}

/**
 * A catalog entry available for creating ETL jobs.
 */
export interface EtlCatalogEntry {
  id: number;
  name: string;
  ocsf_version: string;
}

// ============================================
// Job Creation Types
// ============================================

/**
 * Discriminant for source configuration types.
 */
export type SourceConfigType = 'File' | 'Tcp' | 'Sqs' | 'Kafka';

/**
 * File-based source configuration.
 */
export interface FileSourceConfig {
  type: 'File';
  path: string;
}

/**
 * TCP listener source configuration.
 */
export interface TcpSourceConfig {
  type: 'Tcp';
  bind_address: string;
}

/**
 * AWS SQS source configuration.
 */
export interface SqsSourceConfig {
  type: 'Sqs';
  queue_url: string;
  region: string;
}

/**
 * Apache Kafka source configuration.
 */
export interface KafkaSourceConfig {
  type: 'Kafka';
  brokers: string[];
  topic: string;
}

/**
 * Discriminated union of all source configuration types.
 */
export type SourceConfig =
  | FileSourceConfig
  | TcpSourceConfig
  | SqsSourceConfig
  | KafkaSourceConfig;

/**
 * SQL cast target types for field transformations.
 */
export type CastType = 'Integer' | 'BigInt' | 'Float' | 'Double' | 'Boolean' | 'String' | 'Timestamp' | 'Date';

/**
 * Optional transformation applied to a field mapping.
 * `null` means no transformation; `{ Cast: CastType }` applies a SQL cast.
 */
export type TransformType = null | 'Lower' | 'Upper' | 'Trim' | { Cast: CastType };

/**
 * A source-to-OCSF field mapping with confidence score and optional transform.
 */
export interface FieldMapping {
  source_field: string;
  target_ocsf_path: string;
  transformation: TransformType;
  confidence: number;
}

/**
 * Reference to a semantic catalog entry used to seed an ETL job.
 */
export interface CatalogRef {
  entry_id: number;
  catalog_version: number;
  catalog_url: string | null;
}

/**
 * S3 storage configuration for Delta Lake output.
 */
export interface S3Config {
  bucket: string;
  region: string;
  endpoint: string | null;
  access_key_id: string | null;
  secret_access_key: string | null;
}

/**
 * Full ETL job definition submitted to the create endpoint.
 */
export interface EtlJob {
  job_id: string;
  plugin_name: string;
  ocsf_class_uid: number;
  ocsf_version: string;
  mappings: FieldMapping[];
  source_config: SourceConfig;
  delta_table_uri: string;
  s3_config: S3Config | null;
  catalog_ref: CatalogRef | null;
}

// ============================================
// UI State Types
// ============================================

/**
 * View identifiers for ETL dashboard sub-navigation.
 */
export type EtlViewId = 'list' | 'form' | 'detail';

/**
 * Job status values used for badge rendering and lifecycle control.
 * Known statuses are enumerated; failed jobs use the pattern "Failed: <message>".
 */
export type JobStatus =
  | 'Pending'
  | 'Generating'
  | 'Compiling'
  | 'Testing'
  | 'Running'
  | 'Completed'
  | string; // "Failed: <message>"
