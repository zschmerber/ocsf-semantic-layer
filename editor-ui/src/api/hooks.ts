/**
 * TanStack Query hooks for the OCSF Semantic Model Editor API.
 * 
 * Provides custom hooks for each API endpoint with proper caching,
 * error handling, and retry logic.
 * 
 * Requirements: 7.1, 7.2, 7.3, 7.4
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { get, post, APIError } from './client';
import type {
  SchemaTree,
  SemanticModel,
  ValidateResponse,
  GenerateResponse,
  ResearchRequest,
  ResearchResponse,
  BatchResearchRequest,
} from '../types';

// ============================================
// Query Keys
// ============================================

import {
  getTables,
  getTable,
  registerTable,
  updateTable,
  deactivateTable,
  getSourceLineage,
  recordSourceLineage,
  getLineageGraph,
  getCoverageSummary,
  getCoverageMatrix,
  getTablesByTechnique,
  getStatistics,
  collectStatistics,
  exportIndexModel,
  importIndexModel,
} from './indexApi';
import type { ListTablesParams, ListSourceLineageParams, RegisterTableRequest } from './indexApi';
import type { IndexModelExport } from '../types';

/**
 * Query key factory for consistent cache key management.
 */
export const queryKeys = {
  all: ['editor'] as const,
  schema: () => [...queryKeys.all, 'schema'] as const,
  validation: (modelHash: string) => [...queryKeys.all, 'validation', modelHash] as const,
  generation: () => [...queryKeys.all, 'generation'] as const,
  llm: () => [...queryKeys.all, 'llm'] as const,
  llmResearch: () => [...queryKeys.llm(), 'research'] as const,
  // Index API query keys (Requirement 12.3)
  index: () => [...queryKeys.all, 'index'] as const,
  tables: (params?: ListTablesParams) => [...queryKeys.index(), 'tables', params] as const,
  table: (id: number) => [...queryKeys.index(), 'table', id] as const,
  lineageGraph: (targetTable?: string) => [...queryKeys.index(), 'lineage-graph', targetTable] as const,
  sourceLineage: (params?: ListSourceLineageParams) => [...queryKeys.index(), 'source-lineage', params] as const,
  coverageSummary: () => [...queryKeys.index(), 'coverage-summary'] as const,
  coverageMatrix: () => [...queryKeys.index(), 'coverage-matrix'] as const,
  statistics: (tableName: string) => [...queryKeys.index(), 'statistics', tableName] as const,
};

// ============================================
// Schema API (Requirement 7.1)
// ============================================

/**
 * Response type for GET /api/schema endpoint.
 */
export interface SchemaResponse {
  version: string;
  categories: SchemaTree['categories'];
  objects: SchemaTree['objects'];
}

/**
 * Fetch the OCSF schema from the backend.
 * 
 * Requirement 7.1: GET /api/schema endpoint returns loaded OCSF schema as JSON
 * 
 * @returns Promise resolving to SchemaTree
 */
async function fetchSchema(): Promise<SchemaTree> {
  const response = await get<SchemaResponse>('/schema');
  return {
    version: response.version,
    categories: response.categories,
    objects: response.objects,
  };
}

/**
 * Hook to fetch the OCSF schema.
 * 
 * Features:
 * - Caches schema data for 5 minutes (staleTime)
 * - Retries failed requests up to 3 times
 * - Returns loading and error states
 * 
 * @example
 * ```tsx
 * const { data: schema, isLoading, error } = useSchema();
 * ```
 */
export function useSchema() {
  return useQuery({
    queryKey: queryKeys.schema(),
    queryFn: fetchSchema,
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 30 * 60 * 1000, // 30 minutes (formerly cacheTime)
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

// ============================================
// Validation API (Requirement 7.2)
// ============================================

/**
 * Request body for POST /api/validate endpoint.
 */
export interface ValidateRequest {
  model: SemanticModel;
}

/**
 * Validate a semantic model against the OCSF schema.
 * 
 * Requirement 7.2: POST /api/validate endpoint validates semantic model against schema
 * 
 * @param model - The semantic model to validate
 * @returns Promise resolving to validation results
 */
async function validateModel(model: SemanticModel): Promise<ValidateResponse> {
  return post<ValidateResponse, ValidateRequest>('/validate', { model });
}

/**
 * Hook to validate a semantic model.
 * 
 * Uses mutation pattern since validation is triggered on-demand
 * and we don't want to cache validation results.
 * 
 * @example
 * ```tsx
 * const { mutate: validate, isPending, error } = useValidation();
 * validate(model, {
 *   onSuccess: (result) => console.log('Valid:', result.valid),
 * });
 * ```
 */
export function useValidation() {
  return useMutation({
    mutationFn: validateModel,
    retry: 2,
    retryDelay: 1000,
  });
}

/**
 * Hook to validate a model with automatic re-validation on model changes.
 * 
 * This is useful for real-time validation as the user edits the model.
 * Uses query pattern with the model as part of the key.
 * 
 * @param model - The semantic model to validate
 * @param enabled - Whether to enable automatic validation
 */
export function useAutoValidation(model: SemanticModel, enabled: boolean = true) {
  // Create a simple hash of the model for cache key
  const modelHash = JSON.stringify(model).length.toString();
  
  return useQuery({
    queryKey: queryKeys.validation(modelHash),
    queryFn: () => validateModel(model),
    enabled,
    staleTime: 0, // Always re-validate when model changes
    gcTime: 0, // Don't cache validation results
    retry: 1,
  });
}

// ============================================
// Generation API (Requirement 7.3)
// ============================================

/**
 * Request body for POST /api/generate endpoint.
 */
export interface GenerateRequest {
  model: SemanticModel;
  dialect: 'snowflake' | 'databricks' | 'bigquery' | 'postgres';
  artifacts: ('dbt' | 'cubejs' | 'views' | 'etl')[];
}

/**
 * Generate warehouse artifacts from a semantic model.
 * 
 * Requirement 7.3: POST /api/generate endpoint generates warehouse artifacts
 * 
 * @param request - Generation request with model, dialect, and artifact types
 * @returns Promise resolving to generated files
 */
async function generateArtifacts(request: GenerateRequest): Promise<GenerateResponse> {
  return post<GenerateResponse, GenerateRequest>('/generate', request);
}

/**
 * Hook to generate warehouse artifacts.
 * 
 * @example
 * ```tsx
 * const { mutate: generate, isPending, data } = useGenerate();
 * generate({
 *   model,
 *   dialect: 'snowflake',
 *   artifacts: ['dbt', 'views'],
 * });
 * ```
 */
export function useGenerate() {
  return useMutation({
    mutationFn: generateArtifacts,
    retry: 1,
  });
}

// ============================================
// LLM Research API (Requirement 7.4)
// ============================================

/**
 * Query the LLM for semantic enrichments.
 * 
 * Requirement 7.4: POST /api/llm/research endpoint queries LLM for semantic enrichments
 * 
 * @param request - Research request with target and context
 * @returns Promise resolving to LLM suggestions
 */
async function llmResearch(request: ResearchRequest): Promise<ResearchResponse> {
  return post<ResearchResponse, ResearchRequest>('/llm/research', request, {
    timeout: 60000, // LLM requests may take longer
    retries: 2,
  });
}

/**
 * Hook for single-target LLM research.
 * 
 * @example
 * ```tsx
 * const { mutate: research, isPending, data } = useLLMResearch();
 * research({
 *   target_type: 'entity',
 *   entity_name: 'Authentication',
 *   ocsf_context: { ... },
 *   requested_fields: ['description', 'synonyms'],
 * });
 * ```
 */
export function useLLMResearch() {
  return useMutation({
    mutationFn: llmResearch,
    retry: 1,
    retryDelay: 2000,
  });
}

/**
 * Batch research for multiple targets.
 * 
 * Requirement 4.7: Support batch research for multiple targets
 * 
 * @param request - Batch research request with multiple targets
 * @returns Promise resolving to array of research responses
 */
async function llmBatchResearch(request: BatchResearchRequest): Promise<ResearchResponse[]> {
  return post<ResearchResponse[], BatchResearchRequest>('/llm/research/batch', request, {
    timeout: 120000, // Batch requests may take even longer
    retries: 1,
  });
}

/**
 * Hook for batch LLM research.
 * 
 * @example
 * ```tsx
 * const { mutate: batchResearch, isPending } = useLLMBatchResearch();
 * batchResearch({
 *   targets: [
 *     { target_type: 'attribute', entity_name: 'Auth', attribute_name: 'user', ... },
 *     { target_type: 'attribute', entity_name: 'Auth', attribute_name: 'ip', ... },
 *   ],
 * });
 * ```
 */
export function useLLMBatchResearch() {
  return useMutation({
    mutationFn: llmBatchResearch,
    retry: 0, // Don't retry batch requests automatically
  });
}

// ============================================
// Utility Hooks
// ============================================

/**
 * Hook to invalidate and refetch the schema.
 * 
 * Useful when the backend schema is updated.
 */
export function useRefreshSchema() {
  const queryClient = useQueryClient();
  
  return () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.schema() });
  };
}

/**
 * Hook to prefetch the schema.
 * 
 * Useful for preloading schema data before it's needed.
 */
export function usePrefetchSchema() {
  const queryClient = useQueryClient();
  
  return () => {
    queryClient.prefetchQuery({
      queryKey: queryKeys.schema(),
      queryFn: fetchSchema,
      staleTime: 5 * 60 * 1000,
    });
  };
}

// ============================================
// Index API Hooks (Requirement 12.3)
// ============================================

// Table Registry Hooks

/**
 * Hook to fetch all registered tables with optional filtering.
 * 
 * @param params - Optional filter parameters (class_uid, is_active)
 * @returns Query result with table entries
 * 
 * @example
 * ```tsx
 * const { data: tables, isLoading } = useTables({ is_active: true });
 * ```
 */
export function useTables(params?: ListTablesParams) {
  return useQuery({
    queryKey: queryKeys.tables(params),
    queryFn: () => getTables(params),
    staleTime: 30 * 1000, // 30 seconds
  });
}

/**
 * Hook to fetch a single table by ID.
 * 
 * @param id - Table ID
 * @returns Query result with table entry
 * 
 * @example
 * ```tsx
 * const { data: table, isLoading } = useTable(123);
 * ```
 */
export function useTable(id: number) {
  return useQuery({
    queryKey: queryKeys.table(id),
    queryFn: () => getTable(id),
    enabled: id > 0,
  });
}

/**
 * Hook to register a new table.
 * 
 * Invalidates the tables query on success.
 * 
 * @returns Mutation result for table registration
 * 
 * @example
 * ```tsx
 * const { mutate: register, isPending } = useRegisterTable();
 * register({ table_name: 'auth_events', class_uid: 3002, ... });
 * ```
 */
export function useRegisterTable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: registerTable,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tables() });
    },
  });
}

/**
 * Hook to update an existing table.
 * 
 * Invalidates both the tables list and specific table queries on success.
 * 
 * @returns Mutation result for table update
 * 
 * @example
 * ```tsx
 * const { mutate: update, isPending } = useUpdateTable();
 * update({ id: 123, table: { table_name: 'auth_events', ... } });
 * ```
 */
export function useUpdateTable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, table }: { id: number; table: RegisterTableRequest }) =>
      updateTable(id, table),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tables() });
      queryClient.invalidateQueries({ queryKey: queryKeys.table(id) });
    },
  });
}

/**
 * Hook to deactivate (soft-delete) a table.
 * 
 * Invalidates the tables query on success.
 * 
 * @returns Mutation result for table deactivation
 * 
 * @example
 * ```tsx
 * const { mutate: deactivate, isPending } = useDeactivateTable();
 * deactivate(123);
 * ```
 */
export function useDeactivateTable() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deactivateTable,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tables() });
    },
  });
}

// Lineage Hooks

/**
 * Hook to fetch lineage data as a graph for visualization.
 * 
 * @param targetTable - Optional target table to filter by
 * @returns Query result with lineage graph (nodes and edges)
 * 
 * @example
 * ```tsx
 * const { data: graph, isLoading } = useLineageGraph('auth_events');
 * ```
 */
export function useLineageGraph(targetTable?: string) {
  return useQuery({
    queryKey: queryKeys.lineageGraph(targetTable),
    queryFn: () => getLineageGraph(targetTable),
  });
}

/**
 * Hook to fetch source lineage records with optional filtering.
 * 
 * @param params - Optional filter parameters (target_table, source_system, limit, offset)
 * @returns Query result with source lineage records
 * 
 * @example
 * ```tsx
 * const { data: lineage, isLoading } = useSourceLineage({ target_table: 'auth_events' });
 * ```
 */
export function useSourceLineage(params?: ListSourceLineageParams) {
  return useQuery({
    queryKey: queryKeys.sourceLineage(params),
    queryFn: () => getSourceLineage(params),
  });
}

/**
 * Hook to record a new source lineage entry.
 * 
 * Invalidates both source lineage and lineage graph queries on success.
 * 
 * @returns Mutation result for recording source lineage
 * 
 * @example
 * ```tsx
 * const { mutate: record, isPending } = useRecordSourceLineage();
 * record({ source_system: 'okta', source_table: 'events', target_table: 'auth_events' });
 * ```
 */
export function useRecordSourceLineage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: recordSourceLineage,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sourceLineage() });
      queryClient.invalidateQueries({ queryKey: queryKeys.lineageGraph() });
    },
  });
}

// Coverage Hooks

/**
 * Hook to fetch aggregated detection coverage summary.
 * 
 * @returns Query result with detection coverage summary
 * 
 * @example
 * ```tsx
 * const { data: summary, isLoading } = useCoverageSummary();
 * ```
 */
export function useCoverageSummary() {
  return useQuery({
    queryKey: queryKeys.coverageSummary(),
    queryFn: getCoverageSummary,
  });
}

/**
 * Hook to fetch coverage data formatted as a MITRE ATT&CK matrix.
 * 
 * @returns Query result with coverage matrix
 * 
 * @example
 * ```tsx
 * const { data: matrix, isLoading } = useCoverageMatrix();
 * ```
 */
export function useCoverageMatrix() {
  return useQuery({
    queryKey: queryKeys.coverageMatrix(),
    queryFn: getCoverageMatrix,
  });
}

/**
 * Hook to fetch tables that cover a specific MITRE ATT&CK technique.
 * 
 * @param techniqueId - MITRE technique ID (e.g., "T1071.001")
 * @returns Query result with tables covering the technique
 * 
 * @example
 * ```tsx
 * const { data: tables, isLoading } = useTablesByTechnique('T1071.001');
 * ```
 */
export function useTablesByTechnique(techniqueId: string) {
  return useQuery({
    queryKey: [...queryKeys.index(), 'tables-by-technique', techniqueId] as const,
    queryFn: () => getTablesByTechnique(techniqueId),
    enabled: !!techniqueId,
  });
}

// Statistics Hooks

/**
 * Hook to fetch statistics for a specific table.
 * 
 * @param tableName - Name of the table
 * @returns Query result with table statistics
 * 
 * @example
 * ```tsx
 * const { data: stats, isLoading } = useTableStatistics('auth_events');
 * ```
 */
export function useTableStatistics(tableName: string) {
  return useQuery({
    queryKey: queryKeys.statistics(tableName),
    queryFn: () => getStatistics(tableName),
    enabled: !!tableName,
  });
}

/**
 * Hook to trigger statistics collection for a table.
 * 
 * Invalidates the statistics query for the table on success.
 * 
 * @returns Mutation result for statistics collection
 * 
 * @example
 * ```tsx
 * const { mutate: collect, isPending } = useCollectStatistics();
 * collect({ tableName: 'auth_events', sampleRate: 0.1 });
 * ```
 */
export function useCollectStatistics() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ tableName, sampleRate }: { tableName: string; sampleRate?: number }) =>
      collectStatistics(tableName, sampleRate),
    onSuccess: (_, { tableName }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.statistics(tableName) });
    },
  });
}

// Export/Import Hooks

/**
 * Hook to export the index model.
 * 
 * @param combined - If true, include semantic model in export
 * @returns Mutation result for export
 * 
 * @example
 * ```tsx
 * const { mutate: exportModel, isPending, data } = useExportIndexModel();
 * exportModel({ combined: false });
 * ```
 */
export function useExportIndexModel() {
  return useMutation({
    mutationFn: ({ combined }: { combined?: boolean }) => exportIndexModel(combined),
  });
}

/**
 * Hook to import an index model.
 * 
 * Invalidates all index-related queries on success.
 * 
 * @returns Mutation result for import
 * 
 * @example
 * ```tsx
 * const { mutate: importModel, isPending } = useImportIndexModel();
 * importModel(indexModelExport);
 * ```
 */
export function useImportIndexModel() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (model: IndexModelExport) => importIndexModel(model),
    onSuccess: () => {
      // Invalidate all index-related queries after import
      queryClient.invalidateQueries({ queryKey: queryKeys.index() });
    },
  });
}

// ============================================
// Error Handling Utilities
// ============================================

/**
 * Check if an error is an API error.
 */
export function isAPIError(error: unknown): error is APIError {
  return error instanceof APIError;
}

/**
 * Get a user-friendly error message from an API error.
 */
export function getErrorMessage(error: unknown): string {
  if (isAPIError(error)) {
    // Handle specific error codes
    switch (error.code) {
      case 'NETWORK_ERROR':
        return 'Unable to connect to the server. Please check your connection.';
      case 'TIMEOUT':
        return 'The request timed out. Please try again.';
      case 'SchemaNotLoaded':
        return 'No OCSF schema is loaded. Please load a schema first.';
      default:
        return error.message;
    }
  }
  
  if (error instanceof Error) {
    return error.message;
  }
  
  return 'An unexpected error occurred';
}
