/**
 * Custom hooks for the OCSF Semantic Model Editor.
 */

export { usePersistence, type UsePersistenceResult } from './usePersistence';
export { useKeyboardShortcuts, type UseKeyboardShortcutsOptions } from './useKeyboardShortcuts';

// Re-export API hooks for convenience
export {
  useSchema,
  useValidation,
  useAutoValidation,
  useGenerate,
  useLLMResearch,
  useLLMBatchResearch,
  useRefreshSchema,
  usePrefetchSchema,
} from '../api';
