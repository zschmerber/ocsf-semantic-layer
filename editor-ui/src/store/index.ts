/**
 * Store exports for the OCSF Semantic Model Editor.
 */

export {
  useEditorStore,
  selectEntity,
  selectMetric,
  selectCurrentEntity,
  selectCurrentMetric,
  selectAllDimensions,
  selectHasErrors,
  selectHasWarnings,
  type EditorState,
  type LoadingOperation,
} from './editorStore';

export {
  useIndexStore,
  selectFieldMapping,
  selectIsFieldMapped,
  selectMappedTargetFields,
  selectMappingCount,
  type IndexState,
  type LogFormat,
} from './indexStore';

export { useGuideStore, type GuideState } from './guideStore';

export {
  initializePersistence,
  cleanupPersistence,
  saveNow,
  saveModelToStorage,
  loadModelFromStorage,
  clearStoredModel,
  startAutoSave,
  stopAutoSave,
  restoreFromStorage,
} from './persistence';
