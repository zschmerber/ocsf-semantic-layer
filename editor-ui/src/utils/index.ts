/**
 * Utility functions for the OCSF Semantic Model Editor.
 */

export { modelToYaml, downloadFile, exportModelAsYaml } from './exportYaml';
export { 
  importModelFromYaml, 
  openFilePicker, 
  formatImportErrors,
  type ImportResult,
  type ImportError,
} from './importYaml';

// Log parser utilities (Requirements 14.2, 14.3)
export {
  detectLogFormat,
  parseLogInput,
  flattenObject,
  type LogFormat,
} from './logParser';

// ETL helper utilities (Requirements 4.3, 5.8, 5.11, 7.2, 8.2–8.6, 9.3)
export {
  getStatusBadgeClass,
  getMonacoLanguage,
  getEnabledActions,
  generateJobId,
  validateJobForm,
  type JobFormData,
} from './etlHelpers';

// Index validation utilities (Requirements 17.1-17.7)
export {
  validateIndexModel,
  validateTableEntry,
  validateDetectionCoverage,
  findOrphanLineageRecords,
  isValidMitreTechniqueId,
  isValidMitreTactic,
  isValidClassUid,
  formatValidationError,
  formatValidationWarning,
  getValidationSummary,
  VALID_MITRE_TACTICS,
  MITRE_TECHNIQUE_PATTERN,
  KNOWN_OCSF_CLASS_UIDS,
  type IndexValidationResult,
} from './indexValidation';
