# Design Document: Editor Index Integration

## Overview

This design document describes the integration of the `ocsf-index` crate into the OCSF Semantic Model Editor application. The integration provides a complete workflow for security data engineers to:

1. Import raw log samples and transform them to OCSF format
2. Build field mappings with lineage tracking
3. Register OCSF tables with detection coverage metadata
4. Visualize data lineage and MITRE ATT&CK coverage
5. Validate and export both semantic and index models

The design follows existing patterns in the codebase:
- Backend: Axum handlers with State extraction, JSON responses
- Frontend: React functional components, Zustand stores, TanStack Query
- API: RESTful endpoints under /api prefix

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Editor UI (React)                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │  Log Import │ │  Mapping    │ │  Lineage    │ │  Detection Coverage     ││
│  │  Component  │ │  Builder    │ │  Graph      │ │  Dashboard              ││
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └───────────┬─────────────┘│
│         │               │               │                     │              │
│  ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐ ┌───────────┴─────────────┐│
│  │  Table      │ │  Statistics │ │  Index      │ │  Export/Import          ││
│  │  Registry   │ │  Viewer     │ │  Store      │ │  Manager                ││
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └───────────┬─────────────┘│
│         └───────────────┴───────────────┴───────────────────┬┘              │
│                                                             │               │
│                              API Client Layer               │               │
└─────────────────────────────────────────────────────────────┼───────────────┘
                                                              │
                                                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ocsf-editor (Axum)                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         /api/index/* endpoints                          ││
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           ││
│  │  │ tables  │ │ lineage │ │coverage │ │  stats  │ │partitions│           ││
│  └──┴────┬────┴─┴────┬────┴─┴────┬────┴─┴────┬────┴─┴────┬────┴───────────┘│
│          │           │           │           │           │                  │
│  ┌───────┴───────────┴───────────┴───────────┴───────────┴─────────────────┐│
│  │                           AppState                                       ││
│  │  ┌─────────────────────────────────────────────────────────────────────┐││
│  │  │                    SemanticIndex<B: IndexBackend>                   │││
│  │  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │││
│  │  │  │  Table    │ │  Source   │ │  Field    │ │ Detection │           │││
│  │  │  │ Registry  │ │  Lineage  │ │  Lineage  │ │ Coverage  │           │││
│  │  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘           │││
│  │  └─────────────────────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
             ┌───────────┐   ┌───────────┐   ┌───────────┐
             │  SQLite   │   │ In-Memory │   │PostgreSQL │
             │  Backend  │   │  Backend  │   │ (future)  │
             └───────────┘   └───────────┘   └───────────┘
```


### Data Flow

1. **Log Import Flow**: Raw logs → Log Parser → Parsed Fields → Mapping Builder
2. **Mapping Flow**: Field Mappings → FieldLineageRecord → SemanticIndex → API
3. **Table Registration Flow**: TableEntry → SemanticIndex → Backend Storage
4. **Visualization Flow**: API Request → SemanticIndex Query → JSON Response → React Component
5. **Export Flow**: SemanticIndex → JSON Serialization → Download/Clipboard

## Components and Interfaces

### Backend Components

#### Extended AppState

The existing `AppState` struct will be extended to include the `SemanticIndex`:

```rust
// ocsf-editor/src/lib.rs
use ocsf_index::{SemanticIndex, IndexConfig};
use ocsf_index::backend::{InMemoryBackend, SqliteBackend};

pub struct AppState {
    pub schema_service: Arc<SchemaService>,
    pub validation_service: Arc<ValidationService>,
    pub llm_service: Arc<RwLock<Option<LLMService>>>,
    // New: Semantic Index for lineage and coverage tracking
    pub semantic_index: Arc<SemanticIndex<Box<dyn IndexBackend>>>,
}

impl AppState {
    pub async fn new() -> Self {
        let schema_service = Arc::new(SchemaService::new());
        let validation_service = Arc::new(ValidationService::new(schema_service.clone()));
        let llm_service = LLMService::from_env().ok();
        
        // Initialize index backend based on environment
        let index_backend = Self::create_index_backend().await;
        let index_config = IndexConfig::default();
        let semantic_index = SemanticIndex::new(index_backend, index_config)
            .await
            .expect("Failed to initialize semantic index");
        
        Self {
            schema_service,
            validation_service,
            llm_service: Arc::new(RwLock::new(llm_service)),
            semantic_index: Arc::new(semantic_index),
        }
    }
    
    async fn create_index_backend() -> Box<dyn IndexBackend> {
        match std::env::var("OCSF_INDEX_PATH") {
            Ok(path) => Box::new(SqliteBackend::open(&path).await.unwrap()),
            Err(_) => Box::new(InMemoryBackend::new()),
        }
    }
}
```

#### Index API Router

New API routes for index operations:

```rust
// ocsf-editor/src/api/index.rs

/// Create the index API router with all endpoints.
pub fn create_index_router() -> Router<AppState> {
    Router::new()
        // Table Registry endpoints (Requirement 2)
        .route("/tables", get(list_tables).post(register_table))
        .route("/tables/:id", get(get_table).put(update_table).delete(deactivate_table))
        
        // Source Lineage endpoints (Requirement 3)
        .route("/lineage/source", get(list_source_lineage).post(record_source_lineage))
        .route("/lineage/graph", get(get_lineage_graph))
        
        // Field Lineage endpoints (Requirement 4)
        .route("/lineage/field", get(list_field_lineage).post(record_field_lineage))
        
        // Detection Coverage endpoints (Requirement 5)
        .route("/coverage/summary", get(get_coverage_summary))
        .route("/coverage/techniques/:technique_id", get(get_tables_by_technique))
        .route("/coverage/tactics/:tactic", get(get_tables_by_tactic))
        .route("/coverage/matrix", get(get_coverage_matrix))
        
        // Statistics endpoints (Requirement 6)
        .route("/statistics/:table_name", get(get_statistics))
        .route("/statistics/:table_name/collect", post(collect_statistics))
        
        // Partition endpoints (Requirement 7)
        .route("/partitions/:table_name", get(list_partitions).post(update_partition))
        
        // Export/Import endpoints (Requirements 18, 19)
        .route("/export", get(export_index_model))
        .route("/import", post(import_index_model))
}
```


#### API Request/Response Types

```rust
// ocsf-editor/src/api/index.rs

// Table Registry Types
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterTableRequest {
    pub table_name: String,
    pub schema_name: Option<String>,
    pub class_uid: u32,
    pub ocsf_version: String,
    pub dialect: String,
    pub metadata: Option<HashMap<String, String>>,
    pub detection_coverage: Option<DetectionCoverageRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionCoverageRequest {
    pub mitre_techniques: Vec<String>,
    pub mitre_tactics: Vec<String>,
    pub data_sources: Vec<String>,
    pub detection_rules: Vec<String>,
    pub kill_chain_phases: Vec<String>,
    pub confidence_level: Option<String>,
    pub max_severity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TableResponse {
    pub id: u64,
    pub table_name: String,
    pub schema_name: Option<String>,
    pub class_uid: u32,
    pub ocsf_version: String,
    pub dialect: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
    pub detection_coverage: Option<DetectionCoverageResponse>,
}

// Lineage Types
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordSourceLineageRequest {
    pub source_system: String,
    pub source_table: String,
    pub target_table: String,
    pub record_count: Option<u64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct LineageGraphResponse {
    pub nodes: Vec<LineageNode>,
    pub edges: Vec<LineageEdgeResponse>,
}

#[derive(Debug, Serialize)]
pub struct LineageNode {
    pub id: String,
    pub node_type: String, // "source" or "target"
    pub label: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct LineageEdgeResponse {
    pub source: String,
    pub target: String,
    pub timestamp: String,
    pub record_count: Option<u64>,
}

// Coverage Types
#[derive(Debug, Serialize)]
pub struct CoverageMatrixResponse {
    pub tactics: Vec<TacticColumn>,
    pub techniques_by_tactic: HashMap<String, Vec<TechniqueCell>>,
}

#[derive(Debug, Serialize)]
pub struct TacticColumn {
    pub id: String,
    pub name: String,
    pub table_count: u64,
}

#[derive(Debug, Serialize)]
pub struct TechniqueCell {
    pub id: String,
    pub name: String,
    pub table_count: u64,
    pub tables: Vec<String>,
    pub is_covered: bool,
}

// Export/Import Types
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexModelExport {
    pub version: String,
    pub exported_at: String,
    pub tables: Vec<TableResponse>,
    pub source_lineage: Vec<SourceLineageResponse>,
    pub field_lineage: Vec<FieldLineageResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CombinedModelExport {
    pub semantic_model: SemanticModel,
    pub index_model: IndexModelExport,
}
```


#### API Handler Implementations

```rust
// ocsf-editor/src/api/index.rs

/// POST /api/index/tables - Register a new table
pub async fn register_table(
    State(state): State<AppState>,
    Json(request): Json<RegisterTableRequest>,
) -> Result<Json<TableResponse>, EditorApiError> {
    // Validate class_uid > 0
    if request.class_uid == 0 {
        return Err(EditorApiError::InvalidRequest(
            "class_uid must be greater than 0".to_string()
        ));
    }
    
    let dialect = parse_dialect(&request.dialect)?;
    
    let mut table = TableEntry::new(&request.table_name, request.class_uid)
        .with_ocsf_version(&request.ocsf_version)
        .with_dialect(dialect);
    
    if let Some(schema) = &request.schema_name {
        table = table.with_schema(schema);
    }
    
    if let Some(coverage) = &request.detection_coverage {
        table = table.with_detection_coverage(coverage.into());
    }
    
    let table_id = state.semantic_index.register_table(table).await?;
    let registered = state.semantic_index.get_table(table_id).await?
        .ok_or(EditorApiError::NotFound("Table not found after registration".into()))?;
    
    Ok(Json(registered.into()))
}

/// GET /api/index/lineage/graph - Get lineage as graph data
pub async fn get_lineage_graph(
    State(state): State<AppState>,
    Query(params): Query<LineageGraphParams>,
) -> Result<Json<LineageGraphResponse>, EditorApiError> {
    let lineage_records = if let Some(target) = &params.target_table {
        state.semantic_index.get_source_lineage(target).await?
    } else {
        state.semantic_index.get_all_source_lineage().await?
    };
    
    // Build nodes and edges for graph visualization
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_sources = HashSet::new();
    let mut seen_targets = HashSet::new();
    
    for record in lineage_records {
        let source_id = format!("{}:{}", record.source_system, record.source_table);
        
        if !seen_sources.contains(&source_id) {
            nodes.push(LineageNode {
                id: source_id.clone(),
                node_type: "source".to_string(),
                label: record.source_table.clone(),
                metadata: [("system".to_string(), record.source_system.clone())].into(),
            });
            seen_sources.insert(source_id.clone());
        }
        
        if !seen_targets.contains(&record.target_table) {
            nodes.push(LineageNode {
                id: record.target_table.clone(),
                node_type: "target".to_string(),
                label: record.target_table.clone(),
                metadata: HashMap::new(),
            });
            seen_targets.insert(record.target_table.clone());
        }
        
        edges.push(LineageEdgeResponse {
            source: source_id,
            target: record.target_table,
            timestamp: record.ingestion_timestamp.to_rfc3339(),
            record_count: record.record_count,
        });
    }
    
    Ok(Json(LineageGraphResponse { nodes, edges }))
}

/// GET /api/index/coverage/matrix - Get MITRE ATT&CK matrix data
pub async fn get_coverage_matrix(
    State(state): State<AppState>,
) -> Result<Json<CoverageMatrixResponse>, EditorApiError> {
    let summary = state.semantic_index.get_detection_coverage_summary().await?;
    
    // Build matrix structure from summary
    let tactics: Vec<TacticColumn> = summary.mitre_tactics.iter()
        .map(|t| TacticColumn {
            id: t.tactic.clone(),
            name: format_tactic_name(&t.tactic),
            table_count: t.table_count,
        })
        .collect();
    
    let mut techniques_by_tactic: HashMap<String, Vec<TechniqueCell>> = HashMap::new();
    
    for technique in &summary.mitre_techniques {
        let tactic = get_tactic_for_technique(&technique.technique_id);
        let cell = TechniqueCell {
            id: technique.technique_id.clone(),
            name: format_technique_name(&technique.technique_id),
            table_count: technique.table_count,
            tables: technique.tables.clone(),
            is_covered: technique.table_count > 0,
        };
        techniques_by_tactic.entry(tactic).or_default().push(cell);
    }
    
    Ok(Json(CoverageMatrixResponse { tactics, techniques_by_tactic }))
}
```


### Frontend Components

#### TypeScript Types

```typescript
// editor-ui/src/types/index.ts (additions)

// Table Registry Types
export interface TableEntry {
  id: number;
  table_name: string;
  schema_name?: string;
  class_uid: number;
  ocsf_version: string;
  dialect: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
  metadata: Record<string, string>;
  detection_coverage?: DetectionCoverage;
}

export interface DetectionCoverage {
  mitre_techniques: string[];
  mitre_tactics: string[];
  data_sources: string[];
  detection_rules: string[];
  kill_chain_phases: string[];
  confidence_level?: 'low' | 'medium' | 'high';
  max_severity?: 'informational' | 'low' | 'medium' | 'high' | 'critical';
  tlp?: 'clear' | 'green' | 'amber' | 'amber_strict' | 'red';
}

// Lineage Types
export interface SourceLineageRecord {
  id?: number;
  source_system: string;
  source_table: string;
  target_table: string;
  ingestion_timestamp: string;
  record_count?: number;
  metadata: Record<string, string>;
}

export interface FieldLineageRecord {
  id?: number;
  source_lineage_id: number;
  source_field: string;
  target_field: string;
  transformation?: string;
  ocsf_version?: string;
}

export interface LineageGraph {
  nodes: LineageNode[];
  edges: LineageEdge[];
}

export interface LineageNode {
  id: string;
  node_type: 'source' | 'target';
  label: string;
  metadata: Record<string, string>;
}

export interface LineageEdge {
  source: string;
  target: string;
  timestamp: string;
  record_count?: number;
}

// Coverage Types
export interface DetectionCoverageSummary {
  tables_with_coverage: number;
  mitre_techniques: MitreTechniqueCoverage[];
  mitre_tactics: MitreTacticCoverage[];
  data_sources: DataSourceCoverage[];
  kill_chain_coverage: KillChainCoverage[];
}

export interface MitreTechniqueCoverage {
  technique_id: string;
  table_count: number;
  tables: string[];
}

export interface MitreTacticCoverage {
  tactic: string;
  technique_count: number;
  table_count: number;
}

export interface CoverageMatrix {
  tactics: TacticColumn[];
  techniques_by_tactic: Record<string, TechniqueCell[]>;
}

export interface TechniqueCell {
  id: string;
  name: string;
  table_count: number;
  tables: string[];
  is_covered: boolean;
}

// Statistics Types
export interface TableStatistics {
  table_name: string;
  columns: ColumnStatistics[];
  total_rows: number;
  sample_rate: number;
  collected_at: string;
}

export interface ColumnStatistics {
  column_name: string;
  distinct_count: number;
  null_count: number;
  total_count: number;
  min_value?: string;
  max_value?: string;
}

// Export/Import Types
export interface IndexModelExport {
  version: string;
  exported_at: string;
  tables: TableEntry[];
  source_lineage: SourceLineageRecord[];
  field_lineage: FieldLineageRecord[];
}

export interface CombinedModelExport {
  semantic_model: SemanticModel;
  index_model: IndexModelExport;
}

// Log Import Types
export interface ParsedLogField {
  name: string;
  value: string;
  type: 'string' | 'number' | 'boolean' | 'object' | 'array' | 'null';
  path: string;
}

export interface FieldMapping {
  source_field: string;
  target_field: string;
  transformation?: string;
}
```


#### API Client Functions

```typescript
// editor-ui/src/api/index.ts

import { get, post, put, del } from './client';
import type {
  TableEntry, SourceLineageRecord, FieldLineageRecord,
  LineageGraph, DetectionCoverageSummary, CoverageMatrix,
  TableStatistics, IndexModelExport, CombinedModelExport
} from '../types';

// Table Registry API
export function getTables(params?: { class_uid?: number; is_active?: boolean }) {
  const query = new URLSearchParams();
  if (params?.class_uid) query.set('class_uid', params.class_uid.toString());
  if (params?.is_active !== undefined) query.set('is_active', params.is_active.toString());
  return get<TableEntry[]>(`/index/tables?${query}`);
}

export function getTable(id: number) {
  return get<TableEntry>(`/index/tables/${id}`);
}

export function registerTable(table: Omit<TableEntry, 'id' | 'created_at' | 'updated_at'>) {
  return post<TableEntry>('/index/tables', table);
}

export function updateTable(id: number, table: Partial<TableEntry>) {
  return put<TableEntry>(`/index/tables/${id}`, table);
}

export function deactivateTable(id: number) {
  return del<void>(`/index/tables/${id}`);
}

// Lineage API
export function getSourceLineage(params?: { target_table?: string; source_system?: string }) {
  const query = new URLSearchParams();
  if (params?.target_table) query.set('target_table', params.target_table);
  if (params?.source_system) query.set('source_system', params.source_system);
  return get<SourceLineageRecord[]>(`/index/lineage/source?${query}`);
}

export function recordSourceLineage(lineage: Omit<SourceLineageRecord, 'id' | 'ingestion_timestamp'>) {
  return post<SourceLineageRecord>('/index/lineage/source', lineage);
}

export function getLineageGraph(params?: { target_table?: string }) {
  const query = new URLSearchParams();
  if (params?.target_table) query.set('target_table', params.target_table);
  return get<LineageGraph>(`/index/lineage/graph?${query}`);
}

export function getFieldLineage(params?: { target_field?: string; source_lineage_id?: number }) {
  const query = new URLSearchParams();
  if (params?.target_field) query.set('target_field', params.target_field);
  if (params?.source_lineage_id) query.set('source_lineage_id', params.source_lineage_id.toString());
  return get<FieldLineageRecord[]>(`/index/lineage/field?${query}`);
}

export function recordFieldLineage(lineage: Omit<FieldLineageRecord, 'id'>) {
  return post<FieldLineageRecord>('/index/lineage/field', lineage);
}

// Coverage API
export function getCoverageSummary() {
  return get<DetectionCoverageSummary>('/index/coverage/summary');
}

export function getTablesByTechnique(techniqueId: string) {
  return get<TableEntry[]>(`/index/coverage/techniques/${encodeURIComponent(techniqueId)}`);
}

export function getTablesByTactic(tactic: string) {
  return get<TableEntry[]>(`/index/coverage/tactics/${encodeURIComponent(tactic)}`);
}

export function getCoverageMatrix() {
  return get<CoverageMatrix>('/index/coverage/matrix');
}

// Statistics API
export function getStatistics(tableName: string) {
  return get<TableStatistics>(`/index/statistics/${encodeURIComponent(tableName)}`);
}

export function collectStatistics(tableName: string, sampleRate?: number) {
  return post<TableStatistics>(`/index/statistics/${encodeURIComponent(tableName)}/collect`, { sample_rate: sampleRate });
}

// Export/Import API
export function exportIndexModel() {
  return get<IndexModelExport>('/index/export');
}

export function importIndexModel(model: IndexModelExport) {
  return post<void>('/index/import', model);
}

export function exportCombinedModel() {
  return get<CombinedModelExport>('/index/export?combined=true');
}

export function importCombinedModel(model: CombinedModelExport) {
  return post<void>('/index/import?combined=true', model);
}
```


#### TanStack Query Hooks

```typescript
// editor-ui/src/api/hooks.ts (additions)

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as indexApi from './index';

// Table Registry Hooks
export function useTables(params?: { class_uid?: number; is_active?: boolean }) {
  return useQuery({
    queryKey: ['tables', params],
    queryFn: () => indexApi.getTables(params),
  });
}

export function useTable(id: number) {
  return useQuery({
    queryKey: ['table', id],
    queryFn: () => indexApi.getTable(id),
    enabled: id > 0,
  });
}

export function useRegisterTable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: indexApi.registerTable,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tables'] });
    },
  });
}

// Lineage Hooks
export function useLineageGraph(params?: { target_table?: string }) {
  return useQuery({
    queryKey: ['lineage-graph', params],
    queryFn: () => indexApi.getLineageGraph(params),
  });
}

export function useSourceLineage(params?: { target_table?: string; source_system?: string }) {
  return useQuery({
    queryKey: ['source-lineage', params],
    queryFn: () => indexApi.getSourceLineage(params),
  });
}

export function useRecordSourceLineage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: indexApi.recordSourceLineage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['source-lineage'] });
      queryClient.invalidateQueries({ queryKey: ['lineage-graph'] });
    },
  });
}

// Coverage Hooks
export function useCoverageSummary() {
  return useQuery({
    queryKey: ['coverage-summary'],
    queryFn: indexApi.getCoverageSummary,
  });
}

export function useCoverageMatrix() {
  return useQuery({
    queryKey: ['coverage-matrix'],
    queryFn: indexApi.getCoverageMatrix,
  });
}

export function useTablesByTechnique(techniqueId: string) {
  return useQuery({
    queryKey: ['tables-by-technique', techniqueId],
    queryFn: () => indexApi.getTablesByTechnique(techniqueId),
    enabled: !!techniqueId,
  });
}

// Statistics Hooks
export function useTableStatistics(tableName: string) {
  return useQuery({
    queryKey: ['statistics', tableName],
    queryFn: () => indexApi.getStatistics(tableName),
    enabled: !!tableName,
  });
}

export function useCollectStatistics() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ tableName, sampleRate }: { tableName: string; sampleRate?: number }) =>
      indexApi.collectStatistics(tableName, sampleRate),
    onSuccess: (_, { tableName }) => {
      queryClient.invalidateQueries({ queryKey: ['statistics', tableName] });
    },
  });
}
```


#### Zustand Store for Index State

```typescript
// editor-ui/src/store/indexStore.ts

import { create } from 'zustand';
import type { 
  TableEntry, SourceLineageRecord, FieldLineageRecord, 
  FieldMapping, ParsedLogField 
} from '../types';

interface IndexState {
  // Log Import State
  rawLogInput: string;
  parsedFields: ParsedLogField[];
  detectedFormat: 'json' | 'csv' | 'syslog' | 'kv' | null;
  
  // Mapping State
  fieldMappings: FieldMapping[];
  selectedSourceField: string | null;
  selectedTargetField: string | null;
  
  // Table Registration State
  pendingTable: Partial<TableEntry> | null;
  
  // UI State
  selectedTableId: number | null;
  selectedTechnique: string | null;
  lineageFilter: { source_system?: string; target_table?: string };
  
  // Actions
  setRawLogInput: (input: string) => void;
  setParsedFields: (fields: ParsedLogField[]) => void;
  setDetectedFormat: (format: 'json' | 'csv' | 'syslog' | 'kv' | null) => void;
  
  addFieldMapping: (mapping: FieldMapping) => void;
  removeFieldMapping: (sourceField: string) => void;
  updateFieldMapping: (sourceField: string, updates: Partial<FieldMapping>) => void;
  clearFieldMappings: () => void;
  
  setSelectedSourceField: (field: string | null) => void;
  setSelectedTargetField: (field: string | null) => void;
  
  setPendingTable: (table: Partial<TableEntry> | null) => void;
  setSelectedTableId: (id: number | null) => void;
  setSelectedTechnique: (technique: string | null) => void;
  setLineageFilter: (filter: { source_system?: string; target_table?: string }) => void;
  
  reset: () => void;
}

const initialState = {
  rawLogInput: '',
  parsedFields: [],
  detectedFormat: null,
  fieldMappings: [],
  selectedSourceField: null,
  selectedTargetField: null,
  pendingTable: null,
  selectedTableId: null,
  selectedTechnique: null,
  lineageFilter: {},
};

export const useIndexStore = create<IndexState>((set) => ({
  ...initialState,
  
  setRawLogInput: (input) => set({ rawLogInput: input }),
  setParsedFields: (fields) => set({ parsedFields: fields }),
  setDetectedFormat: (format) => set({ detectedFormat: format }),
  
  addFieldMapping: (mapping) => set((state) => ({
    fieldMappings: [...state.fieldMappings.filter(m => m.source_field !== mapping.source_field), mapping],
  })),
  removeFieldMapping: (sourceField) => set((state) => ({
    fieldMappings: state.fieldMappings.filter(m => m.source_field !== sourceField),
  })),
  updateFieldMapping: (sourceField, updates) => set((state) => ({
    fieldMappings: state.fieldMappings.map(m => 
      m.source_field === sourceField ? { ...m, ...updates } : m
    ),
  })),
  clearFieldMappings: () => set({ fieldMappings: [] }),
  
  setSelectedSourceField: (field) => set({ selectedSourceField: field }),
  setSelectedTargetField: (field) => set({ selectedTargetField: field }),
  
  setPendingTable: (table) => set({ pendingTable: table }),
  setSelectedTableId: (id) => set({ selectedTableId: id }),
  setSelectedTechnique: (technique) => set({ selectedTechnique: technique }),
  setLineageFilter: (filter) => set({ lineageFilter: filter }),
  
  reset: () => set(initialState),
}));
```


#### React Components

##### LineageVisualization Component

```typescript
// editor-ui/src/components/LineageVisualization/LineageVisualization.tsx

import { useCallback, useMemo } from 'react';
import { useLineageGraph, useFieldLineage } from '../../api/hooks';
import { useIndexStore } from '../../store/indexStore';
import './LineageVisualization.css';

interface LineageVisualizationProps {
  targetTable?: string;
}

export function LineageVisualization({ targetTable }: LineageVisualizationProps) {
  const { lineageFilter, setLineageFilter } = useIndexStore();
  const { data: graph, isLoading, error } = useLineageGraph({ 
    target_table: targetTable || lineageFilter.target_table 
  });
  
  const sourceNodes = useMemo(() => 
    graph?.nodes.filter(n => n.node_type === 'source') ?? [], 
    [graph]
  );
  
  const targetNodes = useMemo(() => 
    graph?.nodes.filter(n => n.node_type === 'target') ?? [], 
    [graph]
  );
  
  const handleNodeClick = useCallback((nodeId: string, nodeType: string) => {
    if (nodeType === 'target') {
      setLineageFilter({ target_table: nodeId });
    }
  }, [setLineageFilter]);
  
  if (isLoading) return <div className="lineage-loading">Loading lineage...</div>;
  if (error) return <div className="lineage-error">Error loading lineage</div>;
  if (!graph) return <div className="lineage-empty">No lineage data</div>;
  
  return (
    <div className="lineage-visualization">
      <div className="lineage-header">
        <h3>Data Lineage</h3>
        <select 
          value={lineageFilter.target_table || ''} 
          onChange={(e) => setLineageFilter({ target_table: e.target.value || undefined })}
        >
          <option value="">All Tables</option>
          {targetNodes.map(n => (
            <option key={n.id} value={n.id}>{n.label}</option>
          ))}
        </select>
      </div>
      
      <div className="lineage-graph">
        <div className="lineage-column sources">
          <h4>Source Systems</h4>
          {sourceNodes.map(node => (
            <div 
              key={node.id} 
              className="lineage-node source"
              onClick={() => handleNodeClick(node.id, 'source')}
            >
              <span className="node-label">{node.label}</span>
              <span className="node-system">{node.metadata.system}</span>
            </div>
          ))}
        </div>
        
        <div className="lineage-edges">
          <svg className="edge-canvas">
            {graph.edges.map((edge, i) => (
              <LineageEdgePath key={i} edge={edge} />
            ))}
          </svg>
        </div>
        
        <div className="lineage-column targets">
          <h4>OCSF Tables</h4>
          {targetNodes.map(node => (
            <div 
              key={node.id} 
              className="lineage-node target"
              onClick={() => handleNodeClick(node.id, 'target')}
            >
              <span className="node-label">{node.label}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
```

##### DetectionCoverageDashboard Component

```typescript
// editor-ui/src/components/DetectionCoverage/DetectionCoverageDashboard.tsx

import { useMemo, useState } from 'react';
import { useCoverageMatrix, useTablesByTechnique } from '../../api/hooks';
import { useIndexStore } from '../../store/indexStore';
import './DetectionCoverageDashboard.css';

// MITRE ATT&CK tactics in kill chain order
const TACTIC_ORDER = [
  'reconnaissance', 'resource-development', 'initial-access', 'execution',
  'persistence', 'privilege-escalation', 'defense-evasion', 'credential-access',
  'discovery', 'lateral-movement', 'collection', 'command-and-control',
  'exfiltration', 'impact'
];

export function DetectionCoverageDashboard() {
  const { selectedTechnique, setSelectedTechnique } = useIndexStore();
  const { data: matrix, isLoading, error } = useCoverageMatrix();
  const { data: techniqueDetails } = useTablesByTechnique(selectedTechnique || '');
  const [filterTactic, setFilterTactic] = useState<string | null>(null);
  
  const sortedTactics = useMemo(() => {
    if (!matrix) return [];
    return matrix.tactics.sort((a, b) => 
      TACTIC_ORDER.indexOf(a.id) - TACTIC_ORDER.indexOf(b.id)
    );
  }, [matrix]);
  
  const coverageStats = useMemo(() => {
    if (!matrix) return { covered: 0, total: 0, percentage: 0 };
    const allTechniques = Object.values(matrix.techniques_by_tactic).flat();
    const covered = allTechniques.filter(t => t.is_covered).length;
    const total = allTechniques.length;
    return { covered, total, percentage: total > 0 ? (covered / total) * 100 : 0 };
  }, [matrix]);
  
  if (isLoading) return <div className="coverage-loading">Loading coverage...</div>;
  if (error) return <div className="coverage-error">Error loading coverage</div>;
  if (!matrix) return <div className="coverage-empty">No coverage data</div>;
  
  return (
    <div className="detection-coverage-dashboard">
      <div className="coverage-header">
        <h2>MITRE ATT&CK Coverage</h2>
        <div className="coverage-stats">
          <span className="stat-covered">{coverageStats.covered} covered</span>
          <span className="stat-total">/ {coverageStats.total} techniques</span>
          <span className="stat-percentage">({coverageStats.percentage.toFixed(1)}%)</span>
        </div>
      </div>
      
      <div className="tactic-filter">
        <button 
          className={filterTactic === null ? 'active' : ''} 
          onClick={() => setFilterTactic(null)}
        >
          All Tactics
        </button>
        {sortedTactics.map(tactic => (
          <button
            key={tactic.id}
            className={filterTactic === tactic.id ? 'active' : ''}
            onClick={() => setFilterTactic(tactic.id)}
          >
            {tactic.name}
          </button>
        ))}
      </div>
      
      <div className="mitre-matrix">
        {sortedTactics
          .filter(t => !filterTactic || t.id === filterTactic)
          .map(tactic => (
            <div key={tactic.id} className="tactic-column">
              <div className="tactic-header">
                <span className="tactic-name">{tactic.name}</span>
                <span className="tactic-count">{tactic.table_count} tables</span>
              </div>
              <div className="technique-list">
                {(matrix.techniques_by_tactic[tactic.id] || []).map(technique => (
                  <div
                    key={technique.id}
                    className={`technique-cell ${technique.is_covered ? 'covered' : 'gap'}`}
                    onClick={() => setSelectedTechnique(technique.id)}
                  >
                    <span className="technique-id">{technique.id}</span>
                    <span className="technique-name">{technique.name}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
      </div>
      
      {selectedTechnique && techniqueDetails && (
        <TechniqueDetailPanel 
          techniqueId={selectedTechnique}
          tables={techniqueDetails}
          onClose={() => setSelectedTechnique(null)}
        />
      )}
    </div>
  );
}
```


##### LogImport Component

```typescript
// editor-ui/src/components/LogImport/LogImport.tsx

import { useCallback, useState } from 'react';
import { useIndexStore } from '../../store/indexStore';
import { parseLogInput, detectLogFormat } from '../../utils/logParser';
import './LogImport.css';

export function LogImport() {
  const { 
    rawLogInput, setRawLogInput, 
    setParsedFields, setDetectedFormat 
  } = useIndexStore();
  const [parseError, setParseError] = useState<string | null>(null);
  
  const handleInputChange = useCallback((value: string) => {
    setRawLogInput(value);
    setParseError(null);
    
    if (!value.trim()) {
      setParsedFields([]);
      setDetectedFormat(null);
      return;
    }
    
    try {
      const format = detectLogFormat(value);
      setDetectedFormat(format);
      
      const fields = parseLogInput(value, format);
      setParsedFields(fields);
    } catch (err) {
      setParseError(err instanceof Error ? err.message : 'Failed to parse log');
      setParsedFields([]);
    }
  }, [setRawLogInput, setParsedFields, setDetectedFormat]);
  
  const handleFileUpload = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      handleInputChange(content);
    };
    reader.readAsText(file);
  }, [handleInputChange]);
  
  return (
    <div className="log-import">
      <div className="log-import-header">
        <h3>Import Raw Log</h3>
        <input 
          type="file" 
          accept=".json,.log,.txt,.csv"
          onChange={handleFileUpload}
        />
      </div>
      
      <textarea
        className="log-input"
        placeholder="Paste raw log data here (JSON, CSV, syslog, or key-value format)"
        value={rawLogInput}
        onChange={(e) => handleInputChange(e.target.value)}
        rows={10}
      />
      
      {parseError && (
        <div className="parse-error">{parseError}</div>
      )}
      
      <ParsedFieldsPreview />
    </div>
  );
}

function ParsedFieldsPreview() {
  const { parsedFields, detectedFormat } = useIndexStore();
  
  if (parsedFields.length === 0) return null;
  
  return (
    <div className="parsed-fields-preview">
      <div className="format-indicator">
        Detected format: <strong>{detectedFormat}</strong>
      </div>
      <table className="fields-table">
        <thead>
          <tr>
            <th>Field Path</th>
            <th>Type</th>
            <th>Sample Value</th>
          </tr>
        </thead>
        <tbody>
          {parsedFields.map((field, i) => (
            <tr key={i}>
              <td className="field-path">{field.path}</td>
              <td className="field-type">{field.type}</td>
              <td className="field-value">{truncateValue(field.value)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function truncateValue(value: string, maxLen = 50): string {
  return value.length > maxLen ? value.slice(0, maxLen) + '...' : value;
}
```

##### MappingBuilder Component

```typescript
// editor-ui/src/components/MappingBuilder/MappingBuilder.tsx

import { useCallback, useMemo } from 'react';
import { useIndexStore } from '../../store/indexStore';
import { useSchema } from '../../api';
import './MappingBuilder.css';

export function MappingBuilder() {
  const { 
    parsedFields, fieldMappings, 
    selectedSourceField, selectedTargetField,
    setSelectedSourceField, setSelectedTargetField,
    addFieldMapping, removeFieldMapping, updateFieldMapping
  } = useIndexStore();
  
  const { data: schema } = useSchema();
  
  // Flatten OCSF schema attributes for target field selection
  const ocsfFields = useMemo(() => {
    if (!schema) return [];
    return flattenSchemaAttributes(schema);
  }, [schema]);
  
  const handleCreateMapping = useCallback(() => {
    if (!selectedSourceField || !selectedTargetField) return;
    
    addFieldMapping({
      source_field: selectedSourceField,
      target_field: selectedTargetField,
    });
    
    setSelectedSourceField(null);
    setSelectedTargetField(null);
  }, [selectedSourceField, selectedTargetField, addFieldMapping, setSelectedSourceField, setSelectedTargetField]);
  
  const unmappedSourceFields = useMemo(() => 
    parsedFields.filter(f => !fieldMappings.some(m => m.source_field === f.path)),
    [parsedFields, fieldMappings]
  );
  
  return (
    <div className="mapping-builder">
      <div className="mapping-header">
        <h3>Field Mappings</h3>
        <span className="mapping-count">
          {fieldMappings.length} / {parsedFields.length} fields mapped
        </span>
      </div>
      
      <div className="mapping-workspace">
        <div className="source-fields">
          <h4>Source Fields</h4>
          {unmappedSourceFields.map(field => (
            <div
              key={field.path}
              className={`source-field ${selectedSourceField === field.path ? 'selected' : ''}`}
              onClick={() => setSelectedSourceField(field.path)}
            >
              <span className="field-name">{field.path}</span>
              <span className="field-type">{field.type}</span>
            </div>
          ))}
        </div>
        
        <div className="mapping-actions">
          <button 
            className="btn primary"
            disabled={!selectedSourceField || !selectedTargetField}
            onClick={handleCreateMapping}
          >
            Create Mapping →
          </button>
        </div>
        
        <div className="target-fields">
          <h4>OCSF Fields</h4>
          <OCSFFieldSelector 
            fields={ocsfFields}
            selectedField={selectedTargetField}
            onSelect={setSelectedTargetField}
          />
        </div>
      </div>
      
      <div className="current-mappings">
        <h4>Current Mappings</h4>
        {fieldMappings.map(mapping => (
          <MappingRow 
            key={mapping.source_field}
            mapping={mapping}
            onUpdate={(updates) => updateFieldMapping(mapping.source_field, updates)}
            onRemove={() => removeFieldMapping(mapping.source_field)}
          />
        ))}
      </div>
    </div>
  );
}

interface MappingRowProps {
  mapping: FieldMapping;
  onUpdate: (updates: Partial<FieldMapping>) => void;
  onRemove: () => void;
}

function MappingRow({ mapping, onUpdate, onRemove }: MappingRowProps) {
  return (
    <div className="mapping-row">
      <span className="source">{mapping.source_field}</span>
      <span className="arrow">→</span>
      <span className="target">{mapping.target_field}</span>
      <input
        type="text"
        className="transformation"
        placeholder="Transformation (optional)"
        value={mapping.transformation || ''}
        onChange={(e) => onUpdate({ transformation: e.target.value || undefined })}
      />
      <button className="btn-remove" onClick={onRemove}>×</button>
    </div>
  );
}
```


##### TableRegistryBrowser Component

```typescript
// editor-ui/src/components/TableRegistry/TableRegistryBrowser.tsx

import { useState, useCallback } from 'react';
import { useTables, useRegisterTable, useDeactivateTable } from '../../api/hooks';
import { useIndexStore } from '../../store/indexStore';
import type { TableEntry, DetectionCoverage } from '../../types';
import './TableRegistryBrowser.css';

export function TableRegistryBrowser() {
  const [filterClassUid, setFilterClassUid] = useState<number | undefined>();
  const [showInactive, setShowInactive] = useState(false);
  const [showRegisterForm, setShowRegisterForm] = useState(false);
  
  const { selectedTableId, setSelectedTableId } = useIndexStore();
  const { data: tables, isLoading, error } = useTables({ 
    class_uid: filterClassUid, 
    is_active: showInactive ? undefined : true 
  });
  
  const selectedTable = tables?.find(t => t.id === selectedTableId);
  
  if (isLoading) return <div className="registry-loading">Loading tables...</div>;
  if (error) return <div className="registry-error">Error loading tables</div>;
  
  return (
    <div className="table-registry-browser">
      <div className="registry-header">
        <h2>Table Registry</h2>
        <div className="registry-actions">
          <button 
            className="btn primary"
            onClick={() => setShowRegisterForm(true)}
          >
            Register Table
          </button>
        </div>
      </div>
      
      <div className="registry-filters">
        <input
          type="number"
          placeholder="Filter by Class UID"
          value={filterClassUid || ''}
          onChange={(e) => setFilterClassUid(e.target.value ? parseInt(e.target.value) : undefined)}
        />
        <label>
          <input
            type="checkbox"
            checked={showInactive}
            onChange={(e) => setShowInactive(e.target.checked)}
          />
          Show inactive
        </label>
      </div>
      
      <div className="registry-content">
        <div className="table-list">
          {tables?.map(table => (
            <TableRow 
              key={table.id}
              table={table}
              isSelected={table.id === selectedTableId}
              onClick={() => setSelectedTableId(table.id)}
            />
          ))}
          {tables?.length === 0 && (
            <div className="no-tables">No tables registered</div>
          )}
        </div>
        
        {selectedTable && (
          <TableDetailPanel table={selectedTable} />
        )}
      </div>
      
      {showRegisterForm && (
        <RegisterTableModal onClose={() => setShowRegisterForm(false)} />
      )}
    </div>
  );
}

function TableRow({ table, isSelected, onClick }: { 
  table: TableEntry; 
  isSelected: boolean; 
  onClick: () => void;
}) {
  return (
    <div 
      className={`table-row ${isSelected ? 'selected' : ''} ${!table.is_active ? 'inactive' : ''}`}
      onClick={onClick}
    >
      <div className="table-name">{table.table_name}</div>
      <div className="table-meta">
        <span className="class-uid">Class: {table.class_uid}</span>
        <span className="ocsf-version">v{table.ocsf_version}</span>
        <span className="dialect">{table.dialect}</span>
      </div>
      {table.detection_coverage && (
        <div className="coverage-tags">
          {table.detection_coverage.mitre_techniques.slice(0, 3).map(t => (
            <span key={t} className="technique-tag">{t}</span>
          ))}
          {table.detection_coverage.mitre_techniques.length > 3 && (
            <span className="more-tag">+{table.detection_coverage.mitre_techniques.length - 3}</span>
          )}
        </div>
      )}
    </div>
  );
}

function TableDetailPanel({ table }: { table: TableEntry }) {
  const deactivateMutation = useDeactivateTable();
  
  return (
    <div className="table-detail-panel">
      <h3>{table.table_name}</h3>
      
      <div className="detail-section">
        <h4>Table Information</h4>
        <dl>
          <dt>Schema</dt><dd>{table.schema_name || '(none)'}</dd>
          <dt>Class UID</dt><dd>{table.class_uid}</dd>
          <dt>OCSF Version</dt><dd>{table.ocsf_version}</dd>
          <dt>Dialect</dt><dd>{table.dialect}</dd>
          <dt>Status</dt><dd>{table.is_active ? 'Active' : 'Inactive'}</dd>
          <dt>Created</dt><dd>{new Date(table.created_at).toLocaleString()}</dd>
          <dt>Updated</dt><dd>{new Date(table.updated_at).toLocaleString()}</dd>
        </dl>
      </div>
      
      {table.detection_coverage && (
        <div className="detail-section">
          <h4>Detection Coverage</h4>
          <DetectionCoverageDisplay coverage={table.detection_coverage} />
        </div>
      )}
      
      {Object.keys(table.metadata).length > 0 && (
        <div className="detail-section">
          <h4>Metadata</h4>
          <dl>
            {Object.entries(table.metadata).map(([k, v]) => (
              <><dt>{k}</dt><dd>{v}</dd></>
            ))}
          </dl>
        </div>
      )}
      
      <div className="detail-actions">
        <button className="btn">Edit</button>
        {table.is_active && (
          <button 
            className="btn danger"
            onClick={() => deactivateMutation.mutate(table.id)}
          >
            Deactivate
          </button>
        )}
      </div>
    </div>
  );
}
```

##### StatisticsViewer Component

```typescript
// editor-ui/src/components/StatisticsViewer/StatisticsViewer.tsx

import { useTableStatistics, useCollectStatistics } from '../../api/hooks';
import type { TableStatistics, ColumnStatistics } from '../../types';
import './StatisticsViewer.css';

interface StatisticsViewerProps {
  tableName: string;
}

export function StatisticsViewer({ tableName }: StatisticsViewerProps) {
  const { data: stats, isLoading, error } = useTableStatistics(tableName);
  const collectMutation = useCollectStatistics();
  
  const isStale = stats ? isStatisticsStale(stats.collected_at) : false;
  
  if (isLoading) return <div className="stats-loading">Loading statistics...</div>;
  if (error) return <div className="stats-error">Statistics not available</div>;
  if (!stats) return <div className="stats-empty">No statistics collected</div>;
  
  return (
    <div className="statistics-viewer">
      <div className="stats-header">
        <h3>Table Statistics: {tableName}</h3>
        <div className="stats-meta">
          <span className="total-rows">{stats.total_rows.toLocaleString()} rows</span>
          <span className="sample-rate">{(stats.sample_rate * 100).toFixed(0)}% sampled</span>
          <span className={`collected-at ${isStale ? 'stale' : ''}`}>
            Collected: {new Date(stats.collected_at).toLocaleString()}
            {isStale && <span className="stale-warning"> (stale)</span>}
          </span>
        </div>
        <button 
          className="btn refresh"
          onClick={() => collectMutation.mutate({ tableName })}
          disabled={collectMutation.isPending}
        >
          {collectMutation.isPending ? 'Collecting...' : 'Refresh Statistics'}
        </button>
      </div>
      
      <table className="column-stats-table">
        <thead>
          <tr>
            <th>Column</th>
            <th>Distinct</th>
            <th>Null Rate</th>
            <th>Cardinality</th>
            <th>Min</th>
            <th>Max</th>
          </tr>
        </thead>
        <tbody>
          {stats.columns.map(col => (
            <ColumnStatsRow key={col.column_name} column={col} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ColumnStatsRow({ column }: { column: ColumnStatistics }) {
  const nullRate = column.total_count > 0 
    ? (column.null_count / column.total_count) * 100 
    : 0;
  const cardinality = column.total_count > 0 
    ? (column.distinct_count / column.total_count) * 100 
    : 0;
  
  return (
    <tr>
      <td className="col-name">{column.column_name}</td>
      <td className="col-distinct">{column.distinct_count.toLocaleString()}</td>
      <td className="col-null-rate">
        <div className="rate-bar">
          <div className="rate-fill null" style={{ width: `${nullRate}%` }} />
        </div>
        <span>{nullRate.toFixed(1)}%</span>
      </td>
      <td className="col-cardinality">
        <div className="rate-bar">
          <div className="rate-fill cardinality" style={{ width: `${cardinality}%` }} />
        </div>
        <span>{cardinality.toFixed(1)}%</span>
      </td>
      <td className="col-min">{column.min_value || '-'}</td>
      <td className="col-max">{column.max_value || '-'}</td>
    </tr>
  );
}

function isStatisticsStale(collectedAt: string, maxAgeHours = 24): boolean {
  const collected = new Date(collectedAt);
  const now = new Date();
  const ageMs = now.getTime() - collected.getTime();
  return ageMs > maxAgeHours * 60 * 60 * 1000;
}
```


## Data Models

### Backend Data Models

The backend uses existing types from `ocsf-index` crate:

| Type | Description | Key Fields |
|------|-------------|------------|
| `TableEntry` | Physical table metadata | table_name, class_uid, ocsf_version, dialect, detection_coverage |
| `DetectionCoverage` | MITRE ATT&CK coverage | mitre_techniques, mitre_tactics, data_sources, kill_chain_phases |
| `SourceLineageRecord` | Source-to-target lineage | source_system, source_table, target_table, record_count |
| `FieldLineageRecord` | Field-level mapping | source_field, target_field, transformation |
| `TableStatistics` | Column statistics | columns, total_rows, sample_rate, collected_at |
| `PartitionEntry` | Partition metadata | partition_key, start_time, end_time, row_count |
| `DetectionCoverageSummary` | Aggregated coverage | mitre_techniques, mitre_tactics, data_sources |

### API Response Models

```rust
// Conversion traits for API responses
impl From<TableEntry> for TableResponse {
    fn from(entry: TableEntry) -> Self {
        Self {
            id: entry.id.map(|id| id.0).unwrap_or(0),
            table_name: entry.table_name,
            schema_name: entry.schema_name,
            class_uid: entry.class_uid,
            ocsf_version: entry.ocsf_version,
            dialect: format!("{:?}", entry.dialect).to_lowercase(),
            created_at: entry.created_at.to_rfc3339(),
            updated_at: entry.updated_at.to_rfc3339(),
            is_active: entry.is_active,
            metadata: entry.metadata,
            detection_coverage: entry.detection_coverage.map(Into::into),
        }
    }
}

impl From<DetectionCoverage> for DetectionCoverageResponse {
    fn from(coverage: DetectionCoverage) -> Self {
        Self {
            mitre_techniques: coverage.mitre_techniques,
            mitre_tactics: coverage.mitre_tactics,
            data_sources: coverage.data_sources,
            detection_rules: coverage.detection_rules,
            kill_chain_phases: coverage.kill_chain_phases,
            confidence_level: coverage.confidence_level.map(|l| format!("{:?}", l).to_lowercase()),
            max_severity: coverage.max_severity.map(|s| format!("{:?}", s).to_lowercase()),
            tlp: coverage.tlp.map(|t| format!("{:?}", t).to_lowercase()),
        }
    }
}
```

### Log Parser Utilities

```typescript
// editor-ui/src/utils/logParser.ts

export type LogFormat = 'json' | 'csv' | 'syslog' | 'kv';

export function detectLogFormat(input: string): LogFormat {
  const trimmed = input.trim();
  
  // JSON detection
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      JSON.parse(trimmed);
      return 'json';
    } catch {}
  }
  
  // CSV detection (has commas and consistent column count)
  const lines = trimmed.split('\n');
  if (lines.length > 1) {
    const firstLineCommas = (lines[0].match(/,/g) || []).length;
    const secondLineCommas = (lines[1].match(/,/g) || []).length;
    if (firstLineCommas > 0 && firstLineCommas === secondLineCommas) {
      return 'csv';
    }
  }
  
  // Syslog detection (starts with timestamp pattern)
  if (/^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}/.test(trimmed) ||
      /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/.test(trimmed)) {
    return 'syslog';
  }
  
  // Key-value detection (has key=value patterns)
  if (/\w+=\S+/.test(trimmed)) {
    return 'kv';
  }
  
  return 'json'; // Default fallback
}

export function parseLogInput(input: string, format: LogFormat): ParsedLogField[] {
  switch (format) {
    case 'json':
      return parseJsonLog(input);
    case 'csv':
      return parseCsvLog(input);
    case 'syslog':
      return parseSyslogLog(input);
    case 'kv':
      return parseKvLog(input);
  }
}

function parseJsonLog(input: string): ParsedLogField[] {
  const obj = JSON.parse(input);
  return flattenObject(obj);
}

function flattenObject(obj: unknown, prefix = ''): ParsedLogField[] {
  const fields: ParsedLogField[] = [];
  
  if (obj === null) {
    fields.push({ name: prefix || 'root', value: 'null', type: 'null', path: prefix || 'root' });
  } else if (Array.isArray(obj)) {
    fields.push({ name: prefix || 'root', value: JSON.stringify(obj), type: 'array', path: prefix || 'root' });
    obj.forEach((item, i) => {
      fields.push(...flattenObject(item, `${prefix}[${i}]`));
    });
  } else if (typeof obj === 'object') {
    for (const [key, value] of Object.entries(obj)) {
      const path = prefix ? `${prefix}.${key}` : key;
      fields.push(...flattenObject(value, path));
    }
  } else {
    const type = typeof obj as 'string' | 'number' | 'boolean';
    fields.push({ name: prefix, value: String(obj), type, path: prefix });
  }
  
  return fields;
}
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following properties have been identified for property-based testing:

### Property 1: Table Registration Round-Trip

*For any* valid TableEntry with table_name, class_uid > 0, ocsf_version, and dialect, registering the table via POST /api/index/tables and then retrieving it via GET /api/index/tables/{id} SHALL return a TableEntry with matching field values.

**Validates: Requirements 2.1, 2.3**

### Property 2: Table List Contains All Registered Tables

*For any* set of N tables registered with is_active=true, GET /api/index/tables SHALL return exactly N tables, and the returned set SHALL contain all originally registered table names.

**Validates: Requirements 2.2**

### Property 3: Table Filtering By Class UID

*For any* set of tables with various class_uid values, GET /api/index/tables?class_uid=X SHALL return only tables where class_uid equals X, and the count SHALL match the number of tables registered with that class_uid.

**Validates: Requirements 2.4**

### Property 4: Table Soft-Delete Preserves Metadata

*For any* registered table, after DELETE /api/index/tables/{id}, the table SHALL still be retrievable with is_active=false, and all original field values (table_name, class_uid, detection_coverage) SHALL be preserved.

**Validates: Requirements 2.5**

### Property 5: Table Update Reflects Changes

*For any* registered table and any valid update (changing ocsf_version, metadata, or detection_coverage), PUT /api/index/tables/{id} SHALL update only the specified fields, and subsequent GET SHALL return the updated values while preserving unchanged fields.

**Validates: Requirements 2.6, 2.8**

### Property 6: Source Lineage Round-Trip

*For any* valid SourceLineageRecord with source_system, source_table, and target_table, recording via POST /api/index/lineage/source and querying via GET /api/index/lineage/source?target_table=X SHALL return a record with matching field values.

**Validates: Requirements 3.1, 3.2, 3.3**

### Property 7: Lineage Ordering By Timestamp

*For any* set of N source lineage records for the same target_table with distinct ingestion_timestamps, GET /api/index/lineage/source?target_table=X SHALL return records ordered by ingestion_timestamp ascending.

**Validates: Requirements 3.4**

### Property 8: Lineage Graph Contains All Edges

*For any* set of source lineage records, GET /api/index/lineage/graph SHALL return a graph where the number of edges equals the number of lineage records, and each edge's source and target match the corresponding record's source_system:source_table and target_table.

**Validates: Requirements 3.5**

### Property 9: Lineage Pagination Correctness

*For any* lineage query with limit L and offset O where total records = N, the response SHALL contain min(L, N-O) records starting from the O-th record, and has_more SHALL be true if and only if O + L < N.

**Validates: Requirements 3.6**

### Property 10: Field Lineage Round-Trip

*For any* valid FieldLineageRecord with source_lineage_id, source_field, target_field, and optional transformation, recording via POST and querying via GET SHALL return a record with matching field values including the transformation expression.

**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 11: Coverage Summary Aggregation

*For any* set of tables with detection_coverage metadata, GET /api/index/coverage/summary SHALL return a summary where:
- tables_with_coverage equals the count of tables with non-empty detection_coverage
- mitre_techniques contains all unique technique IDs across all tables
- Each technique's table_count equals the number of tables containing that technique

**Validates: Requirements 5.1**

### Property 12: Coverage Technique Filtering

*For any* MITRE technique ID covered by N tables, GET /api/index/coverage/techniques/{technique_id} SHALL return exactly N tables, and each returned table's detection_coverage.mitre_techniques SHALL contain the queried technique_id.

**Validates: Requirements 5.2, 5.3**

### Property 13: Statistics Staleness Calculation

*For any* TableStatistics with collected_at timestamp T and configured max_age_secs M, is_stale SHALL be true if and only if (current_time - T) > M seconds.

**Validates: Requirements 6.4**

### Property 14: Partition Time Range Filtering

*For any* set of partitions and any time range [start, end], GET /api/index/partitions/{table}?start_time=S&end_time=E SHALL return exactly those partitions where partition.start_time < E AND partition.end_time > S (overlap condition).

**Validates: Requirements 7.2**

### Property 15: Log Format Detection

*For any* valid JSON string starting with '{' or '[', detectLogFormat SHALL return 'json'. *For any* string with consistent comma-separated values across lines, detectLogFormat SHALL return 'csv'. *For any* string matching syslog timestamp patterns, detectLogFormat SHALL return 'syslog'.

**Validates: Requirements 14.2**

### Property 16: Log Field Extraction Completeness

*For any* valid JSON log object with N leaf fields, parseLogInput SHALL return exactly N ParsedLogField entries, and each entry's path SHALL uniquely identify a field in the original object.

**Validates: Requirements 14.3**

### Property 17: Index Model Validation Correctness

*For any* index model:
- If any table has class_uid = 0, validation SHALL report an error for that table
- If any lineage record references a non-existent target_table, validation SHALL report an error
- If any detection_coverage contains an invalid MITRE technique ID format, validation SHALL report a warning

**Validates: Requirements 17.2, 17.3, 17.4**

### Property 18: Index Model Export/Import Round-Trip

*For any* valid index model containing tables, source_lineage, and field_lineage records, exporting to JSON and importing back SHALL produce an equivalent model where all table names, lineage relationships, and detection_coverage metadata match the original.

**Validates: Requirements 18.1, 18.3, 19.1, 19.2**

### Property 19: Combined Model Export/Import Round-Trip

*For any* valid combined model containing both semantic_model and index_model, exporting and importing SHALL preserve both models with all entities, attributes, metrics, tables, and lineage records matching the original.

**Validates: Requirements 18.4**
