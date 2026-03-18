import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import './App.css';
import { usePersistence, useKeyboardShortcuts } from './hooks';
import { useEditorStore } from './store';
import { 
  SchemaBrowser, 
  EntityEditor, 
  MetricBuilder, 
  ValidationPanel, 
  HeaderLoadingIndicator, 
  LoadingOverlay,
  LineageVisualization,
  DetectionCoverageDashboard,
  TableRegistryBrowser,
  StatisticsViewer,
  ArchitectureDiagram,
  LogImport,
  MappingBuilder,
  IndexBuilder,
  CatalogBrowser,
  CatalogEntryForm,
  CatalogEntryDetail,
  PluginDashboard,
  EtlDashboard,
  GuidedProgressBar,
  StepHint,
  EmptyStateGuide,
  NextStepPrompt,
  GuideErrorBoundary,
  EventPastePanel,
} from './components';
import { useTables } from './api/hooks';
import { useGuideStore } from './store/guideStore';
import { checkPrerequisites, shouldShowNextPrompt, getNextStep } from './store/guideLogic';
import { useReferenceEventStore } from './store/referenceEventStore';
import type { GuideStepId, TabId as GuideTabId } from './types/guide';
import { GUIDE_STEPS } from './types/guide';
import { exportModelAsYaml, modelToYaml, openFilePicker, importModelFromYaml, type ImportError } from './utils';
import { getLLMConfig, setLLMConfig } from './api/client';
import { getTables, importIndexModel, exportIndexModel } from './api/indexApi';
import type { LLMProvider, IndexModelExport } from './types';
import type { CatalogEntry } from './types';
import { createExampleModel } from './types';

type TabId = 'schema' | 'entities' | 'metrics' | 'validation' | 'index' | 'architecture' | 'catalog' | 'etl';
type IndexViewId = 'lineage' | 'coverage' | 'tables' | 'statistics' | 'import' | 'mapping' | 'builder';
type CatalogViewId = 'browse' | 'plugins';

interface Tab {
  id: TabId;
  label: string;
  hasSubNav?: boolean;
}

interface IndexSubNav {
  id: IndexViewId;
  label: string;
  icon: string;
}

const tabs: Tab[] = [
  { id: 'schema', label: 'Schema' },
  { id: 'entities', label: 'Entities' },
  { id: 'metrics', label: 'Metrics' },
  { id: 'validation', label: 'Validation' },
  { id: 'index', label: 'Index', hasSubNav: true },
  { id: 'architecture', label: 'Architecture' },
  { id: 'catalog', label: 'Catalog' },
  { id: 'etl', label: 'ETL' },
];

const indexSubNavItems: IndexSubNav[] = [
  { id: 'import', label: 'Import', icon: '📥' },
  { id: 'mapping', label: 'Mapping', icon: '🔀' },
  { id: 'builder', label: 'Builder', icon: '🏗️' },
  { id: 'tables', label: 'Tables', icon: '📋' },
  { id: 'lineage', label: 'Lineage', icon: '🔗' },
  { id: 'coverage', label: 'Coverage', icon: '🛡️' },
  { id: 'statistics', label: 'Statistics', icon: '📊' },
];

// ============================================
// Statistics Viewer Wrapper Component
// ============================================

interface StatisticsViewerWrapperProps {
  selectedTable: string | null;
  onSelectTable: (tableName: string | null) => void;
}

/**
 * Wrapper component for StatisticsViewer that adds table selection.
 */
function StatisticsViewerWrapper({ selectedTable, onSelectTable }: StatisticsViewerWrapperProps) {
  const { data: tables, isLoading: tablesLoading } = useTables({ is_active: true });
  
  if (!selectedTable) {
    return (
      <div className="statistics-table-selector">
        <div className="selector-header">
          <h3>📊 Table Statistics</h3>
          <p>Select a table to view its column statistics.</p>
        </div>
        
        {tablesLoading ? (
          <div className="selector-loading">
            <div className="spinner"></div>
            <span>Loading tables...</span>
          </div>
        ) : tables && tables.length > 0 ? (
          <div className="selector-list">
            {tables.map(table => (
              <button
                key={table.id}
                className="selector-item"
                onClick={() => onSelectTable(table.table_name)}
              >
                <span className="selector-item-icon">📋</span>
                <div className="selector-item-info">
                  <span className="selector-item-name">{table.table_name}</span>
                  {table.schema_name && (
                    <span className="selector-item-schema">{table.schema_name}</span>
                  )}
                </div>
                <span className="selector-item-meta">Class {table.class_uid}</span>
              </button>
            ))}
          </div>
        ) : (
          <div className="selector-empty">
            <span>No tables registered. Register tables in the Tables view first.</span>
          </div>
        )}
      </div>
    );
  }
  
  return (
    <div className="statistics-viewer-container">
      <div className="statistics-back-nav">
        <button 
          className="btn secondary back-btn"
          onClick={() => onSelectTable(null)}
        >
          ← Back to table list
        </button>
      </div>
      <StatisticsViewer tableName={selectedTable} />
    </div>
  );
}

function App() {
  const [activeTab, setActiveTab] = useState<TabId>('schema');
  const [activeIndexView, setActiveIndexView] = useState<IndexViewId>('import');
  const [activeCatalogView, setActiveCatalogView] = useState<CatalogViewId>('browse');
  const [selectedCatalogEntry, setSelectedCatalogEntry] = useState<CatalogEntry | null>(null);
  const [showCatalogForm, setShowCatalogForm] = useState(false);
  const [editingCatalogEntry, setEditingCatalogEntry] = useState<CatalogEntry | null>(null);
  const [indexBackendConfigured, setIndexBackendConfigured] = useState<boolean | null>(null);
  const [importErrors, setImportErrors] = useState<ImportError[] | null>(null);
  const [showNewModelConfirm, setShowNewModelConfirm] = useState(false);
  const [showLoadExampleConfirm, setShowLoadExampleConfirm] = useState(false);
  const [yamlPreview, setYamlPreview] = useState<string | null>(null);
  const [showExportOptions, setShowExportOptions] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [llmConfigured, setLlmConfigured] = useState(false);
  const [llmProvider, setLlmProvider] = useState<LLMProvider | null>(null);
  const [settingsProvider, setSettingsProvider] = useState<LLMProvider>('anthropic');
  const [settingsApiKey, setSettingsApiKey] = useState('');
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [settingsSaving, setSettingsSaving] = useState(false);
  const [selectedStatisticsTable, setSelectedStatisticsTable] = useState<string | null>(null);
  const { wasRestored, save } = usePersistence();
  const isDirty = useEditorStore((state) => state.isDirty);
  const modelName = useEditorStore((state) => state.model.name);
  const setModel = useEditorStore((state) => state.setModel);
  const resetModel = useEditorStore((state) => state.resetModel);
  const canUndo = useEditorStore((state) => state.canUndo);
  const canRedo = useEditorStore((state) => state.canRedo);
  const undo = useEditorStore((state) => state.undo);
  const redo = useEditorStore((state) => state.redo);
  
  // Loading state (Requirement: 8.5)
  const isLoading = useEditorStore((state) => state.isLoading);
  const startLoading = useEditorStore((state) => state.startLoading);
  const stopLoading = useEditorStore((state) => state.stopLoading);

  // ============================================
  // Guide Store State (Tasks 9.1–9.4)
  // ============================================
  const guideVisible = useGuideStore((s) => s.guideVisible);
  const stepStatuses = useGuideStore((s) => s.stepStatuses);
  const shownPrompts = useGuideStore((s) => s.shownPrompts);
  const model = useEditorStore((state) => state.model);
  const { data: guideTables } = useTables({ is_active: true });
  const tablesRegistered = (guideTables?.length ?? 0) > 0;
  const classDetection = useReferenceEventStore((s) => s.classDetection);
  const referenceRawJson = useReferenceEventStore((s) => s.rawJson);
  const schema = useEditorStore((state) => state.schema);

  // Track previous step statuses for NextStepPrompt detection
  const prevStepStatusesRef = useRef<Record<GuideStepId, string> | null>(null);
  const [activePromptStepId, setActivePromptStepId] = useState<GuideStepId | null>(null);

  // Refresh step statuses when model or classDetection change (Requirement 10.1, 10.2)
  useEffect(() => {
    useGuideStore.getState().refreshStepStatuses(activeTab as GuideTabId, classDetection);
  }, [model, classDetection, activeTab]);

  // Auto-set EditorStore selected class and expand schema tree on class detection (Requirements 2.7, 2.8)
  useEffect(() => {
    if (!classDetection?.classUid || !classDetection?.categoryUid) return;
    const store = useEditorStore.getState();
    // Expand the category node and class node in the schema tree
    store.expandNode(`category-${classDetection.categoryUid}`);
    store.expandNode(`class-${classDetection.classUid}`);
  }, [classDetection]);

  // Rehydrate ReferenceEventStore when schema loads with persisted rawJson (Requirement 14.3)
  // Queues detection: if rawJson was restored from localStorage but schema wasn't available yet,
  // rehydrate computes classDetection/observables/mismatches once schema is ready.
  useEffect(() => {
    if (!schema || !referenceRawJson) return;
    const { classDetection: currentDetection } = useReferenceEventStore.getState();
    if (currentDetection) return; // Already hydrated
    useReferenceEventStore.getState().rehydrate(schema);
  }, [schema, referenceRawJson]);

  // Detect step completion transitions for NextStepPrompt (Requirement 6.1)
  useEffect(() => {
    const prev = prevStepStatusesRef.current;
    if (prev && guideVisible) {
      const stepIds: GuideStepId[] = [1, 2, 3, 4];
      for (const id of stepIds) {
        if (prev[id] !== 'complete' && stepStatuses[id] === 'complete') {
          if (shouldShowNextPrompt(id, guideVisible, shownPrompts)) {
            setActivePromptStepId(id);
            break;
          }
        }
      }
    }
    prevStepStatusesRef.current = { ...stepStatuses };
  }, [stepStatuses, guideVisible, shownPrompts]);

  // Build guide steps for GuidedProgressBar
  const guideSteps = useMemo(() =>
    GUIDE_STEPS.map((s) => ({ ...s, status: stepStatuses[s.id] })),
    [stepStatuses]
  );

  // Helper: check if tab content is empty
  const isTabContentEmpty = useCallback((tab: string): boolean => {
    switch (tab) {
      case 'schema':
        return !model.entities.some(
          (e) => Array.isArray(e.source_event_classes) && e.source_event_classes.length > 0
        );
      case 'entities':
        return model.entities.length === 0;
      case 'metrics':
        return model.metrics.length === 0;
      case 'index':
        return !tablesRegistered;
      default:
        return false;
    }
  }, [model, tablesRegistered]);

  // Load LLM config on mount
  useEffect(() => {
    getLLMConfig()
      .then((config) => {
        setLlmConfigured(config.configured);
        setLlmProvider(config.provider);
      })
      .catch(() => {
        // Ignore errors - LLM config is optional
      });
  }, []);

  // Check if index backend is configured when Index tab is selected (Requirement 13.4)
  useEffect(() => {
    if (activeTab === 'index' && indexBackendConfigured === null) {
      // Try to fetch tables to check if backend is configured
      getTables({ limit: 1 } as any)
        .then(() => {
          setIndexBackendConfigured(true);
        })
        .catch(() => {
          setIndexBackendConfigured(false);
        });
    }
  }, [activeTab, indexBackendConfigured]);

  const handleExport = useCallback(() => {
    // Show export options modal
    setShowExportOptions(true);
  }, []);

  const handleExportTypeSelect = useCallback(async (type: 'semantic' | 'index' | 'combined') => {
    setShowExportOptions(false);
    
    if (type === 'semantic') {
      // Generate YAML preview for semantic model
      const model = useEditorStore.getState().model;
      const yaml = modelToYaml(model);
      setYamlPreview(yaml);
    } else {
      // Export index or combined as JSON
      startLoading('export', `Exporting ${type} model...`);
      try {
        const combined = type === 'combined';
        const data = await exportIndexModel(combined);
        
        // Generate filename
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
        const prefix = type === 'combined' ? 'ocsf-combined-model' : 'ocsf-index-model';
        const filename = `${prefix}-${timestamp}.json`;
        
        // Download as JSON
        const jsonString = JSON.stringify(data, null, 2);
        const blob = new Blob([jsonString], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
      } catch (error) {
        console.error('Export failed:', error);
      } finally {
        stopLoading();
      }
    }
  }, [startLoading, stopLoading]);

  const confirmExport = useCallback(() => {
    // Start loading indicator
    startLoading('export', 'Exporting model...');
    
    try {
      // Save to localStorage first
      save();
      // Export the current model as a YAML file download
      exportModelAsYaml(useEditorStore.getState().model);
    } finally {
      // Stop loading indicator
      stopLoading();
      setYamlPreview(null);
    }
  }, [save, startLoading, stopLoading]);

  const cancelExport = useCallback(() => {
    setYamlPreview(null);
  }, []);

  const copyYamlToClipboard = useCallback(() => {
    if (yamlPreview) {
      navigator.clipboard.writeText(yamlPreview);
    }
  }, [yamlPreview]);

  // Register keyboard shortcuts (Requirement: 8.4)
  useKeyboardShortcuts({ onSave: handleExport });

  const handleImport = async () => {
    // Open file picker and get the YAML content
    const result = await openFilePicker();
    if (!result) {
      // User cancelled the file picker
      return;
    }

    // Start loading indicator
    startLoading('import', 'Importing model...');

    try {
      // Parse and validate the YAML
      const importResult = importModelFromYaml(result.content);
      
      if (!importResult.success || !importResult.model) {
        // Show error modal with parse errors
        setImportErrors(importResult.errors);
        return;
      }

      // Successfully imported - update the store
      setModel(importResult.model);
    } finally {
      // Stop loading indicator
      stopLoading();
    }
  };

  const closeErrorModal = () => {
    setImportErrors(null);
  };

  const handleNewModel = () => {
    // If there are unsaved changes, show confirmation dialog
    if (isDirty) {
      setShowNewModelConfirm(true);
    } else {
      // No unsaved changes, reset immediately
      resetModel();
    }
  };

  const confirmNewModel = () => {
    resetModel();
    setShowNewModelConfirm(false);
  };

  const cancelNewModel = () => {
    setShowNewModelConfirm(false);
  };

  // Example index data for DNS Activity
  const loadExampleIndex = async () => {
    const exampleIndex: IndexModelExport = {
      version: "1.0.0",
      exported_at: new Date().toISOString(),
      tables: [
        {
          id: 1,
          table_name: "dns_activity",
          schema_name: "ocsf",
          class_uid: 4003,
          ocsf_version: "1.3.0",
          dialect: "snowflake",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          is_active: true,
          metadata: {
            source: "zeek",
            retention_days: "90",
            partition_key: "time"
          },
          detection_coverage: {
            mitre_techniques: ["T1071.004", "T1568.002", "T1048.003"],
            mitre_tactics: ["command-and-control", "exfiltration"],
            data_sources: ["dns_query", "dns_response", "network_traffic"],
            detection_rules: ["dns_tunneling", "dga_detection", "dns_exfil"],
            kill_chain_phases: ["command-and-control", "actions-on-objectives"],
            confidence_level: "high",
            max_severity: "critical"
          }
        },
        {
          id: 2,
          table_name: "dns_activity_raw",
          schema_name: "raw",
          class_uid: 4003,
          ocsf_version: "1.3.0",
          dialect: "snowflake",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          is_active: true,
          metadata: {
            source: "packetbeat",
            retention_days: "30"
          }
        }
      ],
      source_lineage: [
        {
          id: 1,
          source_system: "zeek",
          source_table: "raw.dns_activity_raw",
          target_table: "ocsf.dns_activity",
          ingestion_timestamp: new Date().toISOString(),
          record_count: 1000000,
          metadata: {
            pipeline: "dns_normalization",
            schedule: "*/5 * * * *"
          }
        }
      ],
      field_lineage: [
        {
          id: 1,
          source_lineage_id: 1,
          source_field: "query_name",
          target_field: "query.hostname",
          transformation: "LOWER(TRIM(query_name))",
          ocsf_version: "1.3.0"
        },
        {
          id: 2,
          source_lineage_id: 1,
          source_field: "query_type",
          target_field: "query.type",
          transformation: "CASE WHEN query_type = 'A' THEN 1 WHEN query_type = 'AAAA' THEN 28 ELSE 0 END",
          ocsf_version: "1.3.0"
        },
        {
          id: 3,
          source_lineage_id: 1,
          source_field: "src_ip",
          target_field: "src_endpoint.ip",
          transformation: "src_ip",
          ocsf_version: "1.3.0"
        },
        {
          id: 4,
          source_lineage_id: 1,
          source_field: "dst_ip",
          target_field: "dst_endpoint.ip",
          transformation: "dst_ip",
          ocsf_version: "1.3.0"
        },
        {
          id: 5,
          source_lineage_id: 1,
          source_field: "response_code",
          target_field: "rcode_id",
          transformation: "CAST(response_code AS INTEGER)",
          ocsf_version: "1.3.0"
        },
        {
          id: 6,
          source_lineage_id: 1,
          source_field: "timestamp",
          target_field: "time",
          transformation: "TO_TIMESTAMP(timestamp)",
          ocsf_version: "1.3.0"
        }
      ]
    };

    try {
      await importIndexModel(exampleIndex);
      // Refresh the index backend configured state
      setIndexBackendConfigured(true);
    } catch (err) {
      console.error('Failed to load example index:', err);
    }
  };

  const handleLoadExample = () => {
    // If there are unsaved changes, show confirmation dialog
    if (isDirty) {
      setShowLoadExampleConfirm(true);
    } else {
      // No unsaved changes, load immediately
      setModel(createExampleModel());
      loadExampleIndex();
      setActiveTab('entities');
    }
  };

  const confirmLoadExample = () => {
    setModel(createExampleModel());
    loadExampleIndex();
    setShowLoadExampleConfirm(false);
    setActiveTab('entities');
  };

  const cancelLoadExample = () => {
    setShowLoadExampleConfirm(false);
  };

  const openSettings = () => {
    setSettingsError(null);
    setSettingsApiKey('');
    setShowSettings(true);
  };

  const closeSettings = () => {
    setShowSettings(false);
    setSettingsError(null);
    setSettingsApiKey('');
  };

  const handleSaveSettings = async () => {
    if (!settingsApiKey.trim()) {
      setSettingsError('API key is required');
      return;
    }

    setSettingsSaving(true);
    setSettingsError(null);

    try {
      const result = await setLLMConfig({
        provider: settingsProvider,
        api_key: settingsApiKey.trim(),
      });
      setLlmConfigured(result.configured);
      setLlmProvider(result.provider);
      closeSettings();
    } catch (err) {
      setSettingsError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSettingsSaving(false);
    }
  };

  return (
    <div className="app">
        <header className="header">
          <h1 className="header-title">
            <span className="header-icon">🔷</span>
            OCSF Semantic Model Editor
            {modelName && <span className="model-name"> - {modelName}</span>}
            {isDirty && <span className="dirty-indicator">*</span>}
          </h1>
          <nav className="tabs">
            {tabs.map((tab) => (
              <div key={tab.id} className="tab-container">
                <button
                  className={`tab ${activeTab === tab.id ? 'active' : ''}`}
                  onClick={() => setActiveTab(tab.id)}
                >
                  {tab.label}
                </button>
                {/* Index sub-navigation (Requirements 13.1, 13.2, 13.3) */}
                {tab.id === 'index' && activeTab === 'index' && (
                  <div className="tab-subnav">
                    {indexSubNavItems.map((item) => (
                      <button
                        key={item.id}
                        className={`tab-subnav-item ${activeIndexView === item.id ? 'active' : ''}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          setActiveIndexView(item.id);
                        }}
                        title={item.label}
                      >
                        <span className="tab-subnav-icon">{item.icon}</span>
                        <span className="tab-subnav-label">{item.label}</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </nav>
          <div className="header-actions">
            {!guideVisible && (
              <button
                className="btn icon-btn"
                onClick={() => useGuideStore.getState().setGuideVisible(true)}
                title="Show Guide"
                aria-label="Show onboarding guide"
              >
                📖
              </button>
            )}
            <HeaderLoadingIndicator />
            <button 
              className="btn icon-btn" 
              onClick={undo} 
              disabled={!canUndo || isLoading}
              title="Undo (Ctrl+Z)"
            >
              ↶
            </button>
            <button 
              className="btn icon-btn" 
              onClick={redo} 
              disabled={!canRedo || isLoading}
              title="Redo (Ctrl+Y)"
            >
              ↷
            </button>
            <button className="btn" onClick={handleNewModel} disabled={isLoading}>New</button>
            <button className="btn" onClick={handleLoadExample} disabled={isLoading} title="Load DNS Analytics Example">Example</button>
            <button className="btn" onClick={handleImport} disabled={isLoading}>Import</button>
            <button className="btn primary" onClick={handleExport} disabled={isLoading} title="Export (Ctrl+S)">Export</button>
            <button 
              className={`btn icon-btn settings-btn ${llmConfigured ? 'configured' : ''}`}
              onClick={openSettings}
              title={llmConfigured ? `LLM: ${llmProvider}` : 'Configure LLM API Key'}
            >
              ⚙️
            </button>
          </div>
        </header>

        {/* Guided Progress Bar (Task 9.1) */}
        {guideVisible && (
          <GuideErrorBoundary>
            <GuidedProgressBar
              steps={guideSteps}
              currentTab={activeTab as GuideTabId}
              onStepClick={(step) => setActiveTab(step.targetTab as TabId)}
              onDismiss={() => useGuideStore.getState().setGuideVisible(false)}
            />
          </GuideErrorBoundary>
        )}

        {/* Import Error Modal */}
        {importErrors && (
          <div className="modal-overlay" onClick={closeErrorModal}>
            <div className="modal" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h3 className="modal-title">Import Error</h3>
                <button className="modal-close" onClick={closeErrorModal}>×</button>
              </div>
              <div className="modal-body">
                <p className="modal-description">
                  The YAML file could not be imported due to the following errors:
                </p>
                <div className="error-list">
                  {importErrors.map((error, index) => (
                    <div key={index} className="error-item">
                      {error.line !== undefined && (
                        <span className="error-location">
                          Line {error.line}
                          {error.column !== undefined && `, Col ${error.column}`}:
                        </span>
                      )}
                      {error.path && (
                        <span className="error-path">[{error.path}]</span>
                      )}
                      <span className="error-message">{error.message}</span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="modal-footer">
                <button className="btn primary" onClick={closeErrorModal}>Close</button>
              </div>
            </div>
          </div>
        )}

        {/* New Model Confirmation Modal */}
        {showNewModelConfirm && (
          <div className="modal-overlay" onClick={cancelNewModel}>
            <div className="modal modal-confirm" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h3 className="modal-title modal-title-warning">Discard Changes?</h3>
                <button className="modal-close" onClick={cancelNewModel}>×</button>
              </div>
              <div className="modal-body">
                <p className="modal-description">
                  You have unsaved changes. Creating a new model will discard all current changes.
                </p>
                <p className="modal-description">
                  Are you sure you want to continue?
                </p>
              </div>
              <div className="modal-footer">
                <button className="btn" onClick={cancelNewModel}>Cancel</button>
                <button className="btn danger" onClick={confirmNewModel}>Discard Changes</button>
              </div>
            </div>
          </div>
        )}

        {/* Load Example Confirmation Modal */}
        {showLoadExampleConfirm && (
          <div className="modal-overlay" onClick={cancelLoadExample}>
            <div className="modal modal-confirm" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h3 className="modal-title modal-title-warning">Load Example?</h3>
                <button className="modal-close" onClick={cancelLoadExample}>×</button>
              </div>
              <div className="modal-body">
                <p className="modal-description">
                  You have unsaved changes. Loading the example model will discard all current changes.
                </p>
                <p className="modal-description">
                  Are you sure you want to continue?
                </p>
              </div>
              <div className="modal-footer">
                <button className="btn" onClick={cancelLoadExample}>Cancel</button>
                <button className="btn danger" onClick={confirmLoadExample}>Load Example</button>
              </div>
            </div>
          </div>
        )}

        {/* YAML Preview Modal */}
        {yamlPreview && (
          <div className="modal-overlay" onClick={cancelExport}>
            <div className="modal modal-preview" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h3 className="modal-title">YAML Preview</h3>
                <button className="modal-close" onClick={cancelExport}>×</button>
              </div>
              <div className="modal-body">
                <div className="yaml-preview-container">
                  <pre className="yaml-preview">{yamlPreview}</pre>
                </div>
              </div>
              <div className="modal-footer">
                <button className="btn" onClick={copyYamlToClipboard} title="Copy to clipboard">
                  📋 Copy
                </button>
                <button className="btn" onClick={cancelExport}>Cancel</button>
                <button className="btn primary" onClick={confirmExport}>Download</button>
              </div>
            </div>
          </div>
        )}

        {/* Settings Modal */}
        {showSettings && (
          <div className="modal-overlay" onClick={closeSettings}>
            <div className="modal modal-settings" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h3 className="modal-title">⚙️ Settings</h3>
                <button className="modal-close" onClick={closeSettings}>×</button>
              </div>
              <div className="modal-body">
                <div className="settings-section">
                  <h4 className="settings-section-title">LLM Configuration</h4>
                  <p className="settings-description">
                    Configure an API key to enable AI-powered research features for generating
                    descriptions, synonyms, and security context.
                  </p>
                  
                  {llmConfigured && (
                    <div className="settings-status configured">
                      ✓ Currently configured: <strong>{llmProvider}</strong>
                    </div>
                  )}
                  
                  <div className="form-group">
                    <label className="form-label" htmlFor="settings-provider">
                      Provider
                    </label>
                    <select
                      id="settings-provider"
                      className="form-select"
                      value={settingsProvider}
                      onChange={(e) => setSettingsProvider(e.target.value as LLMProvider)}
                    >
                      <option value="anthropic">Anthropic (Claude)</option>
                      <option value="openai">OpenAI (GPT)</option>
                    </select>
                  </div>
                  
                  <div className="form-group">
                    <label className="form-label" htmlFor="settings-api-key">
                      API Key
                    </label>
                    <input
                      id="settings-api-key"
                      type="password"
                      className="form-input"
                      value={settingsApiKey}
                      onChange={(e) => setSettingsApiKey(e.target.value)}
                      placeholder={`Enter your ${settingsProvider === 'anthropic' ? 'Anthropic' : 'OpenAI'} API key`}
                    />
                    <span className="form-hint">
                      Your API key is sent to the local server only and is not stored permanently.
                    </span>
                  </div>
                  
                  {settingsError && (
                    <div className="settings-error">
                      {settingsError}
                    </div>
                  )}
                </div>
              </div>
              <div className="modal-footer">
                <button className="btn" onClick={closeSettings}>Cancel</button>
                <button 
                  className="btn primary" 
                  onClick={handleSaveSettings}
                  disabled={settingsSaving}
                >
                  {settingsSaving ? 'Saving...' : 'Save'}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Export Options Modal */}
        {showExportOptions && (
          <div className="modal-overlay" onClick={() => setShowExportOptions(false)}>
            <div className="modal modal-export-options" onClick={(e) => e.stopPropagation()}>
              <div className="modal-header">
                <h3 className="modal-title">📤 Export Options</h3>
                <button className="modal-close" onClick={() => setShowExportOptions(false)}>×</button>
              </div>
              <div className="modal-body">
                <p className="modal-description">
                  Choose what to export:
                </p>
                <div className="export-options-grid">
                  <button 
                    className="export-option-card"
                    onClick={() => handleExportTypeSelect('semantic')}
                  >
                    <span className="export-option-icon">📝</span>
                    <span className="export-option-title">Semantic Model</span>
                    <span className="export-option-desc">
                      Entities, attributes, metrics, and observable config (YAML)
                    </span>
                    <span className="export-option-format">Format: YAML</span>
                  </button>
                  
                  <button 
                    className="export-option-card"
                    onClick={() => handleExportTypeSelect('index')}
                  >
                    <span className="export-option-icon">🗂️</span>
                    <span className="export-option-title">Index Model</span>
                    <span className="export-option-desc">
                      Table registry, source lineage, and field lineage
                    </span>
                    <span className="export-option-format">Format: JSON</span>
                  </button>
                  
                  <button 
                    className="export-option-card export-option-combined"
                    onClick={() => handleExportTypeSelect('combined')}
                  >
                    <span className="export-option-icon">📦</span>
                    <span className="export-option-title">Combined Export</span>
                    <span className="export-option-desc">
                      Both semantic model and index data together
                    </span>
                    <span className="export-option-format">Format: JSON</span>
                  </button>
                </div>
              </div>
              <div className="modal-footer">
                <button className="btn" onClick={() => setShowExportOptions(false)}>Cancel</button>
              </div>
            </div>
          </div>
        )}

        {/* Full-screen Loading Overlay for blocking operations (Requirement: 8.5) */}
        <LoadingOverlay fullScreen />

        {wasRestored && (
          <div className="restore-notice">
            Restored from previous session
          </div>
        )}

        <main className="content">
          <aside className={`sidebar ${activeTab === 'schema' ? 'schema-focused' : ''}`}>
            {/* StepHint for Schema tab (Task 9.2) */}
            {activeTab === 'schema' && guideVisible && (
              <GuideErrorBoundary>
                <StepHint
                  stepId={1}
                  tabId="schema"
                  isStepComplete={stepStatuses[1] === 'complete'}
                  onDismissHint={(id) => useGuideStore.getState().dismissHint(id)}
                />
              </GuideErrorBoundary>
            )}
            {/* EmptyStateGuide for Schema tab (Task 9.2) */}
            {activeTab === 'schema' && isTabContentEmpty('schema') && (
              <GuideErrorBoundary>
                <EmptyStateGuide
                  tabId="schema"
                  stepId={1}
                  prerequisitesMet={true}
                  onNavigateToPrerequisite={(tab) => setActiveTab(tab as TabId)}
                />
              </GuideErrorBoundary>
            )}
            {/* EventPastePanel for data-first workflow (Task 7.3) — Requirements 1.1, 2.7, 2.8, 12.1 */}
            {activeTab === 'schema' && (
              <EventPastePanel onSkip={() => { /* no-op: analyst continues with manual schema browsing */ }} />
            )}
            <SchemaBrowser />
          </aside>

          <section className={`main-panel ${activeTab === 'schema' ? 'hidden' : ''}`}>
            {activeTab === 'entities' && (
              <>
                {/* StepHint for Entities tab (Task 9.2) */}
                <GuideErrorBoundary>
                  <StepHint
                    stepId={stepStatuses[2] === 'complete' ? 3 : 2}
                    tabId="entities"
                    isStepComplete={stepStatuses[2] === 'complete' && stepStatuses[3] === 'complete'}
                    onDismissHint={(id) => useGuideStore.getState().dismissHint(id)}
                  />
                </GuideErrorBoundary>
                {/* EmptyStateGuide for Entities tab (Task 9.2) */}
                {isTabContentEmpty('entities') && (
                  <GuideErrorBoundary>
                    <EmptyStateGuide
                      tabId="entities"
                      stepId={2}
                      prerequisitesMet={checkPrerequisites(2, stepStatuses).met}
                      onNavigateToPrerequisite={(tab) => setActiveTab(tab as TabId)}
                    />
                  </GuideErrorBoundary>
                )}
                <EntityEditor />
              </>
            )}

            {activeTab === 'metrics' && (
              <>
                {/* StepHint for Metrics tab (Task 9.2) */}
                <GuideErrorBoundary>
                  <StepHint
                    stepId={3}
                    tabId="metrics"
                    isStepComplete={stepStatuses[3] === 'complete'}
                    onDismissHint={(id) => useGuideStore.getState().dismissHint(id)}
                  />
                </GuideErrorBoundary>
                {/* EmptyStateGuide for Metrics tab (Task 9.2) */}
                {isTabContentEmpty('metrics') && (
                  <GuideErrorBoundary>
                    <EmptyStateGuide
                      tabId="metrics"
                      stepId={3}
                      prerequisitesMet={checkPrerequisites(3, stepStatuses).met}
                      onNavigateToPrerequisite={(tab) => setActiveTab(tab as TabId)}
                    />
                  </GuideErrorBoundary>
                )}
                <MetricBuilder />
              </>
            )}

            {activeTab === 'validation' && (
              <>
                {/* StepHint for Validation tab (Task 9.2) */}
                <GuideErrorBoundary>
                  <StepHint
                    stepId={4}
                    tabId="validation"
                    isStepComplete={stepStatuses[4] === 'complete'}
                    onDismissHint={(id) => useGuideStore.getState().dismissHint(id)}
                  />
                </GuideErrorBoundary>
                <ValidationPanel
                  onNavigateToEntity={() => {
                    setActiveTab('entities');
                  }}
                  onNavigateToMetric={() => {
                    setActiveTab('metrics');
                  }}
                />
              </>
            )}

            {/* Index Views (Requirements 13.1, 13.2, 13.3, 13.4) */}
            {activeTab === 'index' && (
              <div className="index-view-container">
                {/* Index tab is optional — not part of 4-step workflow (Requirement 6.1, 6.2) */}
                {/* EmptyStateGuide for Index tab — optional enrichment */}
                {isTabContentEmpty('index') && (
                  <GuideErrorBoundary>
                    <EmptyStateGuide
                      tabId="index"
                      stepId={null}
                      prerequisitesMet={true}
                      onNavigateToPrerequisite={(tab) => setActiveTab(tab as TabId)}
                    />
                  </GuideErrorBoundary>
                )}
                {/* Configuration prompt when backend not configured (Requirement 13.4) */}
                {indexBackendConfigured === false && (
                  <div className="index-config-prompt">
                    <div className="index-config-prompt-icon">⚠️</div>
                    <h3>Index Backend Not Configured</h3>
                    <p>
                      The index backend is not configured. To use lineage tracking, detection coverage,
                      and table registry features, please configure the backend.
                    </p>
                    <div className="index-config-prompt-details">
                      <p>Set the following environment variables on the server:</p>
                      <code>OCSF_INDEX_BACKEND=sqlite</code>
                      <code>OCSF_INDEX_PATH=/path/to/index.db</code>
                    </div>
                    <p className="index-config-prompt-note">
                      Without configuration, an in-memory backend will be used (data will not persist).
                    </p>
                  </div>
                )}

                {/* Loading state */}
                {indexBackendConfigured === null && (
                  <div className="index-loading">
                    <div className="spinner"></div>
                    <p>Checking index backend configuration...</p>
                  </div>
                )}

                {/* Index views when backend is available */}
                {indexBackendConfigured !== false && indexBackendConfigured !== null && (
                  <>
                    {activeIndexView === 'import' && (
                      <LogImport />
                    )}

                    {activeIndexView === 'mapping' && (
                      <MappingBuilder />
                    )}

                    {activeIndexView === 'builder' && (
                      <IndexBuilder 
                        onSuccess={() => setActiveIndexView('tables')} 
                      />
                    )}

                    {activeIndexView === 'tables' && (
                      <TableRegistryBrowser />
                    )}

                    {activeIndexView === 'lineage' && (
                      <LineageVisualization />
                    )}

                    {activeIndexView === 'coverage' && (
                      <DetectionCoverageDashboard />
                    )}

                    {activeIndexView === 'statistics' && (
                      <StatisticsViewerWrapper 
                        selectedTable={selectedStatisticsTable}
                        onSelectTable={setSelectedStatisticsTable}
                      />
                    )}
                  </>
                )}
              </div>
            )}

            {activeTab === 'architecture' && (
              <ArchitectureDiagram />
            )}

            {activeTab === 'catalog' && (
              <div className="catalog-view-container">
                <div className="catalog-subnav">
                  <button
                    className={`catalog-subnav-item ${activeCatalogView === 'browse' ? 'active' : ''}`}
                    onClick={() => setActiveCatalogView('browse')}
                  >
                    📚 Browse
                  </button>
                  <button
                    className={`catalog-subnav-item ${activeCatalogView === 'plugins' ? 'active' : ''}`}
                    onClick={() => setActiveCatalogView('plugins')}
                  >
                    🔌 Plugins
                  </button>
                  <button
                    className="btn primary catalog-new-btn"
                    onClick={() => { setEditingCatalogEntry(null); setShowCatalogForm(true); }}
                  >
                    + New Entry
                  </button>
                </div>

                <div className="catalog-main">
                  {activeCatalogView === 'browse' && (
                    <div className="catalog-browse-layout">
                      <div className={`catalog-list-pane ${selectedCatalogEntry ? 'with-detail' : ''}`}>
                        <CatalogBrowser
                          onSelect={(entry) => setSelectedCatalogEntry(entry)}
                          onEdit={(entry) => { setEditingCatalogEntry(entry); setShowCatalogForm(true); }}
                        />
                      </div>
                      {selectedCatalogEntry && (
                        <div className="catalog-detail-pane">
                          <CatalogEntryDetail
                            entry={selectedCatalogEntry}
                            onClose={() => setSelectedCatalogEntry(null)}
                          />
                        </div>
                      )}
                    </div>
                  )}

                  {activeCatalogView === 'plugins' && (
                    <PluginDashboard />
                  )}
                </div>

                {showCatalogForm && (
                  <div className="modal-overlay" onClick={() => setShowCatalogForm(false)}>
                    <div className="modal modal-catalog-form" onClick={(e) => e.stopPropagation()}>
                      <div className="modal-header">
                        <h3 className="modal-title">
                          {editingCatalogEntry ? 'Edit Catalog Entry' : 'New Catalog Entry'}
                        </h3>
                        <button className="modal-close" onClick={() => setShowCatalogForm(false)}>×</button>
                      </div>
                      <div className="modal-body">
                        <CatalogEntryForm
                          initial={editingCatalogEntry ?? undefined}
                          onSuccess={(entry) => {
                            setShowCatalogForm(false);
                            setSelectedCatalogEntry(entry);
                            setActiveCatalogView('browse');
                          }}
                          onCancel={() => setShowCatalogForm(false)}
                        />
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {activeTab === 'etl' && (
              <EtlDashboard />
            )}

            {/* NextStepPrompt — slide-in banner on step completion (Task 9.3) */}
            {activePromptStepId !== null && guideVisible && (() => {
              const { nextStepId, nextTab } = getNextStep(activePromptStepId);
              return (
                <GuideErrorBoundary>
                  <NextStepPrompt
                    completedStepId={activePromptStepId}
                    nextStepId={nextStepId}
                    nextTabId={nextTab}
                    onNavigate={(tab) => {
                      useGuideStore.getState().markPromptShown(activePromptStepId);
                      setActivePromptStepId(null);
                      setActiveTab(tab as TabId);
                    }}
                    onDismiss={() => {
                      useGuideStore.getState().markPromptShown(activePromptStepId);
                      setActivePromptStepId(null);
                    }}
                  />
                </GuideErrorBoundary>
              );
            })()}
          </section>
        </main>
      </div>
  );
}

export default App;
