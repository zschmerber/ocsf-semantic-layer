/**
 * API module for the OCSF Semantic Model Editor.
 * 
 * Exports the API client, hooks, and utilities for interacting
 * with the backend API.
 * 
 * Requirements: 7.1, 7.2, 7.3, 7.4, 12.1, 12.4, 12.5
 */

// Client exports
export {
  API_BASE_URL,
  APIError,
  apiRequest,
  get,
  post,
  put,
  del,
} from './client';

export type { APIErrorResponse, RequestOptions } from './client';

// Hook exports
export {
  // Query keys
  queryKeys,
  
  // Schema hooks (Requirement 7.1)
  useSchema,
  useRefreshSchema,
  usePrefetchSchema,
  
  // Validation hooks (Requirement 7.2)
  useValidation,
  useAutoValidation,
  
  // Generation hooks (Requirement 7.3)
  useGenerate,
  
  // LLM Research hooks (Requirement 7.4)
  useLLMResearch,
  useLLMBatchResearch,
  
  // Index API hooks (Requirement 12.3)
  // Table Registry hooks
  useTables,
  useTable,
  useRegisterTable,
  useUpdateTable,
  useDeactivateTable,
  // Lineage hooks
  useLineageGraph,
  useSourceLineage,
  useRecordSourceLineage,
  // Coverage hooks
  useCoverageSummary,
  useCoverageMatrix,
  useTablesByTechnique,
  // Statistics hooks
  useTableStatistics,
  useCollectStatistics,
  
  // Error utilities
  isAPIError,
  getErrorMessage,
} from './hooks';

export type {
  SchemaResponse,
  ValidateRequest,
  GenerateRequest,
} from './hooks';

// Index API exports (Requirements 12.1, 12.4, 12.5)
export {
  // Table Registry API
  getTables,
  getTable,
  registerTable,
  updateTable,
  deactivateTable,
  
  // Lineage API
  getSourceLineage,
  recordSourceLineage,
  getLineageGraph,
  getFieldLineage,
  recordFieldLineage,
  
  // Coverage API
  getCoverageSummary,
  getTablesByTechnique,
  getTablesByTactic,
  getCoverageMatrix,
  
  // Statistics API
  getStatistics,
  collectStatistics,
  
  // Partition API
  getPartitions,
  updatePartition,
  
  // Export/Import API
  exportIndexModel,
  importIndexModel,
} from './indexApi';

export type {
  ListTablesParams,
  RegisterTableRequest,
  ListSourceLineageParams,
  RecordSourceLineageRequest,
  ListFieldLineageParams,
  RecordFieldLineageRequest,
  ListPartitionsParams,
  UpdatePartitionRequest,
} from './indexApi';

// Catalog API
export * from './catalogApi';
export * from './catalogHooks';
