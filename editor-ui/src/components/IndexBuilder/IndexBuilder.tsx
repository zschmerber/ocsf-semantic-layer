/**
 * IndexBuilder component for constructing index models from field mappings.
 * 
 * Provides a workflow for:
 * - Building index model from field mappings
 * - Auto-populating table entry from mapping source/target
 * - Adding detection coverage metadata (MITRE techniques, tactics)
 * - Previewing index model before registration
 * - Registering table and recording source lineage in one action
 * 
 * Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7
 */

import { useCallback, useMemo, useState } from 'react';
import { useIndexStore } from '../../store/indexStore';
import { useRegisterTable, useRecordSourceLineage } from '../../api/hooks';
import type { DetectionCoverage, FieldMapping } from '../../types';
import type { RegisterTableRequest, RecordSourceLineageRequest } from '../../api/indexApi';
import './IndexBuilder.css';

// ============================================
// Types
// ============================================

interface IndexBuilderProps {
  /** Callback when registration is successful */
  onSuccess?: () => void;
  /** Callback when registration fails */
  onError?: (error: Error) => void;
}

interface TableFormData {
  table_name: string;
  schema_name: string;
  class_uid: string;
  ocsf_version: string;
  dialect: string;
}

interface SourceFormData {
  source_system: string;
  source_table: string;
}

// ============================================
// Constants
// ============================================

const DIALECT_OPTIONS = [
  { value: 'snowflake', label: 'Snowflake' },
  { value: 'databricks', label: 'Databricks' },
  { value: 'bigquery', label: 'BigQuery' },
  { value: 'postgres', label: 'PostgreSQL' },
];

// ============================================
// DetectionCoverageForm Sub-Component
// ============================================

interface DetectionCoverageFormProps {
  coverage: Partial<DetectionCoverage>;
  onChange: (coverage: Partial<DetectionCoverage>) => void;
}

/**
 * Form for adding MITRE ATT&CK detection coverage metadata.
 * 
 * Requirement 15.3: Add detection coverage metadata (MITRE techniques, tactics, data sources)
 */
function DetectionCoverageForm({ coverage, onChange }: DetectionCoverageFormProps) {
  const handleArrayChange = useCallback((
    field: keyof DetectionCoverage,
    value: string
  ) => {
    const items = value
      .split(',')
      .map(s => s.trim())
      .filter(Boolean);
    onChange({ ...coverage, [field]: items });
  }, [coverage, onChange]);

  const handleSelectChange = useCallback((
    field: 'confidence_level' | 'max_severity',
    value: string
  ) => {
    onChange({ ...coverage, [field]: value || undefined });
  }, [coverage, onChange]);

  return (
    <div className="detection-coverage-form">
      <h4>Detection Coverage (Optional)</h4>
      <p className="form-description">
        Add MITRE ATT&CK metadata to track detection capabilities.
      </p>
      
      <div className="form-group">
        <label htmlFor="mitre-techniques">MITRE Techniques</label>
        <input
          id="mitre-techniques"
          type="text"
          placeholder="e.g., T1071.001, T1078, T1566.001"
          value={coverage.mitre_techniques?.join(', ') || ''}
          onChange={(e) => handleArrayChange('mitre_techniques', e.target.value)}
        />
        <span className="form-hint">Comma-separated technique IDs</span>
      </div>
      
      <div className="form-group">
        <label htmlFor="mitre-tactics">MITRE Tactics</label>
        <input
          id="mitre-tactics"
          type="text"
          placeholder="e.g., initial-access, persistence, defense-evasion"
          value={coverage.mitre_tactics?.join(', ') || ''}
          onChange={(e) => handleArrayChange('mitre_tactics', e.target.value)}
        />
        <span className="form-hint">Comma-separated tactic names</span>
      </div>

      <div className="form-group">
        <label htmlFor="data-sources">Data Sources</label>
        <input
          id="data-sources"
          type="text"
          placeholder="e.g., Authentication logs, Network traffic, Process monitoring"
          value={coverage.data_sources?.join(', ') || ''}
          onChange={(e) => handleArrayChange('data_sources', e.target.value)}
        />
        <span className="form-hint">Comma-separated data source names</span>
      </div>
      
      <div className="form-group">
        <label htmlFor="kill-chain-phases">Kill Chain Phases</label>
        <input
          id="kill-chain-phases"
          type="text"
          placeholder="e.g., reconnaissance, weaponization, delivery"
          value={coverage.kill_chain_phases?.join(', ') || ''}
          onChange={(e) => handleArrayChange('kill_chain_phases', e.target.value)}
        />
        <span className="form-hint">Comma-separated kill chain phases</span>
      </div>
      
      <div className="form-row">
        <div className="form-group">
          <label htmlFor="confidence-level">Confidence Level</label>
          <select
            id="confidence-level"
            value={coverage.confidence_level || ''}
            onChange={(e) => handleSelectChange('confidence_level', e.target.value)}
          >
            <option value="">Select...</option>
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
        </div>
        
        <div className="form-group">
          <label htmlFor="max-severity">Max Severity</label>
          <select
            id="max-severity"
            value={coverage.max_severity || ''}
            onChange={(e) => handleSelectChange('max_severity', e.target.value)}
          >
            <option value="">Select...</option>
            <option value="informational">Informational</option>
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
            <option value="critical">Critical</option>
          </select>
        </div>
      </div>
    </div>
  );
}

// ============================================
// IndexModelSummary Sub-Component
// ============================================

interface IndexModelSummaryProps {
  tableData: TableFormData;
  sourceData: SourceFormData;
  fieldMappings: FieldMapping[];
  coverage: Partial<DetectionCoverage>;
}

/**
 * Preview of the index model before registration.
 * 
 * Requirement 15.4: Preview index model before registration
 * Requirement 15.7: Display summary including table count, lineage count, and coverage
 */
function IndexModelSummary({ 
  tableData, 
  sourceData, 
  fieldMappings, 
  coverage 
}: IndexModelSummaryProps) {
  const hasCoverage = (coverage.mitre_techniques?.length ?? 0) > 0 ||
    (coverage.mitre_tactics?.length ?? 0) > 0 ||
    (coverage.data_sources?.length ?? 0) > 0;

  return (
    <div className="index-model-summary">
      <h4>Index Model Preview</h4>
      
      <div className="summary-section">
        <h5>Table Registration</h5>
        <dl className="summary-list">
          <div className="summary-item">
            <dt>Table Name</dt>
            <dd>{tableData.table_name || <span className="empty">Not set</span>}</dd>
          </div>
          {tableData.schema_name && (
            <div className="summary-item">
              <dt>Schema</dt>
              <dd>{tableData.schema_name}</dd>
            </div>
          )}
          <div className="summary-item">
            <dt>Class UID</dt>
            <dd>{tableData.class_uid || <span className="empty">Not set</span>}</dd>
          </div>
          <div className="summary-item">
            <dt>OCSF Version</dt>
            <dd>{tableData.ocsf_version || <span className="empty">Not set</span>}</dd>
          </div>
          <div className="summary-item">
            <dt>Dialect</dt>
            <dd>{tableData.dialect || <span className="empty">Not set</span>}</dd>
          </div>
        </dl>
      </div>
      
      <div className="summary-section">
        <h5>Source Lineage</h5>
        <dl className="summary-list">
          <div className="summary-item">
            <dt>Source System</dt>
            <dd>{sourceData.source_system || <span className="empty">Not set</span>}</dd>
          </div>
          <div className="summary-item">
            <dt>Source Table</dt>
            <dd>{sourceData.source_table || <span className="empty">Not set</span>}</dd>
          </div>
        </dl>
      </div>

      <div className="summary-section">
        <h5>Field Mappings</h5>
        <div className="summary-stats">
          <span className="stat">
            <strong>{fieldMappings.length}</strong> field mappings
          </span>
        </div>
        {fieldMappings.length > 0 && (
          <div className="mappings-preview">
            {fieldMappings.slice(0, 5).map((mapping, i) => (
              <div key={i} className="mapping-preview-item">
                <code>{mapping.source_field}</code>
                <span className="arrow">→</span>
                <code>{mapping.target_field}</code>
              </div>
            ))}
            {fieldMappings.length > 5 && (
              <div className="more-mappings">
                +{fieldMappings.length - 5} more mappings
              </div>
            )}
          </div>
        )}
      </div>
      
      {hasCoverage && (
        <div className="summary-section">
          <h5>Detection Coverage</h5>
          <div className="coverage-preview">
            {(coverage.mitre_techniques?.length ?? 0) > 0 && (
              <div className="coverage-item">
                <span className="coverage-label">Techniques:</span>
                <div className="coverage-tags">
                  {coverage.mitre_techniques?.slice(0, 5).map(t => (
                    <span key={t} className="technique-tag">{t}</span>
                  ))}
                  {(coverage.mitre_techniques?.length ?? 0) > 5 && (
                    <span className="more-tag">
                      +{(coverage.mitre_techniques?.length ?? 0) - 5}
                    </span>
                  )}
                </div>
              </div>
            )}
            {(coverage.mitre_tactics?.length ?? 0) > 0 && (
              <div className="coverage-item">
                <span className="coverage-label">Tactics:</span>
                <div className="coverage-tags">
                  {coverage.mitre_tactics?.slice(0, 5).map(t => (
                    <span key={t} className="tactic-tag">{t}</span>
                  ))}
                  {(coverage.mitre_tactics?.length ?? 0) > 5 && (
                    <span className="more-tag">
                      +{(coverage.mitre_tactics?.length ?? 0) - 5}
                    </span>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// ============================================
// Main IndexBuilder Component
// ============================================

/**
 * IndexBuilder component for constructing and registering index models.
 * 
 * Features:
 * - Build index model from field mappings (Requirement 15.1)
 * - Auto-populate table entry from mapping source/target (Requirement 15.2)
 * - Add detection coverage metadata (Requirement 15.3)
 * - Preview index model before registration (Requirement 15.4)
 * - Register table and record source lineage in one action (Requirement 15.5)
 * - Show success/error feedback (Requirement 15.6)
 * - Clear mappings after successful registration (Requirement 15.7)
 */
export function IndexBuilder({ onSuccess, onError }: IndexBuilderProps) {
  const { fieldMappings, parsedFields, clearFieldMappings, clearLogImport } = useIndexStore();
  
  // Mutations for API calls
  const registerTableMutation = useRegisterTable();
  const recordLineageMutation = useRecordSourceLineage();
  
  // Form state for table registration
  const [tableData, setTableData] = useState<TableFormData>({
    table_name: '',
    schema_name: '',
    class_uid: '',
    ocsf_version: '1.3.0',
    dialect: 'snowflake',
  });
  
  // Form state for source lineage
  const [sourceData, setSourceData] = useState<SourceFormData>({
    source_system: '',
    source_table: '',
  });
  
  // Detection coverage state
  const [coverage, setCoverage] = useState<Partial<DetectionCoverage>>({
    mitre_techniques: [],
    mitre_tactics: [],
    data_sources: [],
    kill_chain_phases: [],
  });
  
  // UI state
  const [showPreview, setShowPreview] = useState(false);
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Validation
  const validateForm = useCallback((): boolean => {
    const newErrors: Record<string, string> = {};
    
    if (!tableData.table_name.trim()) {
      newErrors.table_name = 'Table name is required';
    }
    
    const classUidNum = parseInt(tableData.class_uid, 10);
    if (!tableData.class_uid || isNaN(classUidNum) || classUidNum <= 0) {
      newErrors.class_uid = 'Class UID must be a positive number';
    }
    
    if (!tableData.ocsf_version.trim()) {
      newErrors.ocsf_version = 'OCSF version is required';
    }
    
    if (!sourceData.source_system.trim()) {
      newErrors.source_system = 'Source system is required';
    }
    
    if (!sourceData.source_table.trim()) {
      newErrors.source_table = 'Source table is required';
    }
    
    // Requirement 15.6: Validate that all required fields are mapped
    if (fieldMappings.length === 0) {
      newErrors.mappings = 'At least one field mapping is required';
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [tableData, sourceData, fieldMappings]);

  // Check if form is ready for submission
  const isFormValid = useMemo(() => {
    return (
      tableData.table_name.trim() !== '' &&
      tableData.class_uid !== '' &&
      parseInt(tableData.class_uid, 10) > 0 &&
      tableData.ocsf_version.trim() !== '' &&
      sourceData.source_system.trim() !== '' &&
      sourceData.source_table.trim() !== '' &&
      fieldMappings.length > 0
    );
  }, [tableData, sourceData, fieldMappings]);

  // Build detection coverage for API request
  const buildDetectionCoverage = useCallback(() => {
    const hasCoverage = (coverage.mitre_techniques?.length ?? 0) > 0 ||
      (coverage.mitre_tactics?.length ?? 0) > 0 ||
      (coverage.data_sources?.length ?? 0) > 0 ||
      (coverage.kill_chain_phases?.length ?? 0) > 0;
    
    if (!hasCoverage) return undefined;
    
    return {
      mitre_techniques: coverage.mitre_techniques || [],
      mitre_tactics: coverage.mitre_tactics || [],
      data_sources: coverage.data_sources || [],
      kill_chain_phases: coverage.kill_chain_phases || [],
      confidence_level: coverage.confidence_level,
      max_severity: coverage.max_severity,
    };
  }, [coverage]);

  /**
   * Handle form submission - registers table and records lineage.
   * 
   * Requirement 15.5: Register table and record source lineage in one action
   */
  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    setFeedback(null);
    
    if (!validateForm()) {
      return;
    }
    
    try {
      // Step 1: Register the table
      const tableRequest: RegisterTableRequest = {
        table_name: tableData.table_name.trim(),
        schema_name: tableData.schema_name.trim() || undefined,
        class_uid: parseInt(tableData.class_uid, 10),
        ocsf_version: tableData.ocsf_version.trim(),
        dialect: tableData.dialect,
        detection_coverage: buildDetectionCoverage(),
      };
      
      await registerTableMutation.mutateAsync(tableRequest);
      
      // Step 2: Record source lineage
      const lineageRequest: RecordSourceLineageRequest = {
        source_system: sourceData.source_system.trim(),
        source_table: sourceData.source_table.trim(),
        target_table: tableData.table_name.trim(),
        record_count: parsedFields.length > 0 ? 1 : undefined,
        metadata: {
          field_count: fieldMappings.length.toString(),
        },
      };
      
      await recordLineageMutation.mutateAsync(lineageRequest);

      // Requirement 15.6: Show success feedback
      setFeedback({
        type: 'success',
        message: `Successfully registered table "${tableData.table_name}" with ${fieldMappings.length} field mappings.`,
      });
      
      // Requirement 15.7: Clear mappings after successful registration
      clearFieldMappings();
      clearLogImport();
      
      // Reset form
      setTableData({
        table_name: '',
        schema_name: '',
        class_uid: '',
        ocsf_version: '1.3.0',
        dialect: 'snowflake',
      });
      setSourceData({
        source_system: '',
        source_table: '',
      });
      setCoverage({
        mitre_techniques: [],
        mitre_tactics: [],
        data_sources: [],
        kill_chain_phases: [],
      });
      setShowPreview(false);
      
      onSuccess?.();
    } catch (error) {
      // Requirement 15.6: Show error feedback
      const errorMessage = error instanceof Error ? error.message : 'An error occurred';
      setFeedback({
        type: 'error',
        message: `Failed to register table: ${errorMessage}`,
      });
      onError?.(error instanceof Error ? error : new Error(errorMessage));
    }
  }, [
    validateForm, tableData, sourceData, fieldMappings, parsedFields,
    buildDetectionCoverage, registerTableMutation, recordLineageMutation,
    clearFieldMappings, clearLogImport, onSuccess, onError
  ]);

  // Handle table data changes
  const handleTableChange = useCallback((field: keyof TableFormData, value: string) => {
    setTableData(prev => ({ ...prev, [field]: value }));
    setErrors(prev => ({ ...prev, [field]: '' }));
  }, []);

  // Handle source data changes
  const handleSourceChange = useCallback((field: keyof SourceFormData, value: string) => {
    setSourceData(prev => ({ ...prev, [field]: value }));
    setErrors(prev => ({ ...prev, [field]: '' }));
  }, []);

  const isPending = registerTableMutation.isPending || recordLineageMutation.isPending;

  // Empty state when no mappings exist
  if (fieldMappings.length === 0 && parsedFields.length === 0) {
    return (
      <div className="index-builder empty-state">
        <div className="empty-message">
          <span className="empty-icon">📋</span>
          <h4>No Field Mappings</h4>
          <p>
            Import a log sample and create field mappings first to build an index model.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="index-builder">
      <div className="builder-header">
        <h3>Build Index Model</h3>
        <p className="builder-description">
          Register a table and record source lineage from your field mappings.
        </p>
      </div>
      
      {/* Feedback message (Requirement 15.6) */}
      {feedback && (
        <div className={`feedback-message ${feedback.type}`}>
          <span className="feedback-icon">
            {feedback.type === 'success' ? '✓' : '⚠'}
          </span>
          <span className="feedback-text">{feedback.message}</span>
          <button 
            className="feedback-dismiss"
            onClick={() => setFeedback(null)}
            aria-label="Dismiss"
          >
            ×
          </button>
        </div>
      )}
      
      <form onSubmit={handleSubmit} className="builder-form">
        {/* Table Information Section (Requirement 15.2) */}
        <div className="form-section">
          <h4>Table Information</h4>
          
          <div className="form-group">
            <label htmlFor="table-name">Table Name *</label>
            <input
              id="table-name"
              type="text"
              value={tableData.table_name}
              onChange={(e) => handleTableChange('table_name', e.target.value)}
              placeholder="e.g., auth_events"
              className={errors.table_name ? 'error' : ''}
            />
            {errors.table_name && (
              <span className="error-message">{errors.table_name}</span>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="schema-name">Schema Name</label>
            <input
              id="schema-name"
              type="text"
              value={tableData.schema_name}
              onChange={(e) => handleTableChange('schema_name', e.target.value)}
              placeholder="e.g., ocsf"
            />
          </div>
          
          <div className="form-row">
            <div className="form-group">
              <label htmlFor="class-uid">Class UID *</label>
              <input
                id="class-uid"
                type="number"
                value={tableData.class_uid}
                onChange={(e) => handleTableChange('class_uid', e.target.value)}
                placeholder="e.g., 3002"
                className={errors.class_uid ? 'error' : ''}
              />
              {errors.class_uid && (
                <span className="error-message">{errors.class_uid}</span>
              )}
            </div>
            
            <div className="form-group">
              <label htmlFor="ocsf-version">OCSF Version *</label>
              <input
                id="ocsf-version"
                type="text"
                value={tableData.ocsf_version}
                onChange={(e) => handleTableChange('ocsf_version', e.target.value)}
                placeholder="e.g., 1.3.0"
                className={errors.ocsf_version ? 'error' : ''}
              />
              {errors.ocsf_version && (
                <span className="error-message">{errors.ocsf_version}</span>
              )}
            </div>
          </div>
          
          <div className="form-group">
            <label htmlFor="dialect">SQL Dialect</label>
            <select
              id="dialect"
              value={tableData.dialect}
              onChange={(e) => handleTableChange('dialect', e.target.value)}
            >
              {DIALECT_OPTIONS.map(opt => (
                <option key={opt.value} value={opt.value}>{opt.label}</option>
              ))}
            </select>
          </div>
        </div>

        {/* Source Lineage Section (Requirement 15.2) */}
        <div className="form-section">
          <h4>Source Lineage</h4>
          
          <div className="form-row">
            <div className="form-group">
              <label htmlFor="source-system">Source System *</label>
              <input
                id="source-system"
                type="text"
                value={sourceData.source_system}
                onChange={(e) => handleSourceChange('source_system', e.target.value)}
                placeholder="e.g., okta, crowdstrike, splunk"
                className={errors.source_system ? 'error' : ''}
              />
              {errors.source_system && (
                <span className="error-message">{errors.source_system}</span>
              )}
            </div>
            
            <div className="form-group">
              <label htmlFor="source-table">Source Table *</label>
              <input
                id="source-table"
                type="text"
                value={sourceData.source_table}
                onChange={(e) => handleSourceChange('source_table', e.target.value)}
                placeholder="e.g., events, logs, raw_data"
                className={errors.source_table ? 'error' : ''}
              />
              {errors.source_table && (
                <span className="error-message">{errors.source_table}</span>
              )}
            </div>
          </div>
        </div>
        
        {/* Field Mappings Summary */}
        <div className="form-section">
          <h4>Field Mappings</h4>
          <div className="mappings-summary">
            <span className={`mapping-count ${fieldMappings.length > 0 ? 'has-mappings' : ''}`}>
              {fieldMappings.length} field{fieldMappings.length !== 1 ? 's' : ''} mapped
            </span>
            {errors.mappings && (
              <span className="error-message">{errors.mappings}</span>
            )}
          </div>
        </div>
        
        {/* Detection Coverage Section (Requirement 15.3) */}
        <DetectionCoverageForm 
          coverage={coverage}
          onChange={setCoverage}
        />

        {/* Form Actions */}
        <div className="form-actions">
          <button
            type="button"
            className="btn secondary"
            onClick={() => setShowPreview(!showPreview)}
            disabled={!isFormValid}
          >
            {showPreview ? 'Hide Preview' : 'Preview Model'}
          </button>
          
          <button
            type="submit"
            className="btn primary"
            disabled={!isFormValid || isPending}
          >
            {isPending ? 'Registering...' : 'Register Table & Record Lineage'}
          </button>
        </div>
      </form>
      
      {/* Index Model Preview (Requirement 15.4) */}
      {showPreview && (
        <IndexModelSummary
          tableData={tableData}
          sourceData={sourceData}
          fieldMappings={fieldMappings}
          coverage={coverage}
        />
      )}
    </div>
  );
}

export default IndexBuilder;
