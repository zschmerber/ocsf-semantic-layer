/**
 * TableRegistryBrowser component for browsing and managing registered OCSF tables.
 * 
 * Displays a searchable, sortable list of tables with filtering by class_uid
 * and active status. Supports table registration, editing, and deactivation.
 * 
 * Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8
 */

import { useState, useCallback, useMemo } from 'react';
import { 
  useTables, 
  useRegisterTable, 
  useUpdateTable, 
  useDeactivateTable 
} from '../../api/hooks';
import { useIndexStore } from '../../store/indexStore';
import type { TableEntry, DetectionCoverage } from '../../types';
import type { RegisterTableRequest } from '../../api/indexApi';
import { ContextualTooltip } from '../ContextualTooltip';
import './TableRegistryBrowser.css';

// ============================================
// Types
// ============================================

interface TableRowProps {
  table: TableEntry;
  isSelected: boolean;
  onClick: () => void;
}

interface TableDetailPanelProps {
  table: TableEntry;
  onEdit: () => void;
  onClose: () => void;
}

interface RegisterTableModalProps {
  onClose: () => void;
  editTable?: TableEntry;
}

interface DetectionCoverageDisplayProps {
  coverage: DetectionCoverage;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Format a date string for display.
 */
function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleString();
}

/**
 * Format a relative time string.
 */
function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();

  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);
  
  if (diffDays > 0) return `${diffDays}d ago`;
  if (diffHours > 0) return `${diffHours}h ago`;
  if (diffMins > 0) return `${diffMins}m ago`;
  return 'just now';
}

// ============================================
// DetectionCoverageDisplay Component
// ============================================

/**
 * Displays detection coverage metadata with MITRE techniques and tactics.
 * 
 * Requirement 10.2: Show detection coverage tags
 */
function DetectionCoverageDisplay({ coverage }: DetectionCoverageDisplayProps) {
  return (
    <div className="detection-coverage-display">
      {coverage.mitre_techniques.length > 0 && (
        <div className="coverage-section">
          <span className="coverage-label">Techniques:</span>
          <div className="coverage-tags">
            {coverage.mitre_techniques.map(technique => (
              <span key={technique} className="technique-tag">{technique}</span>
            ))}
          </div>
        </div>
      )}
      
      {coverage.mitre_tactics.length > 0 && (
        <div className="coverage-section">
          <span className="coverage-label">Tactics:</span>
          <div className="coverage-tags">
            {coverage.mitre_tactics.map(tactic => (
              <span key={tactic} className="tactic-tag">{tactic}</span>
            ))}
          </div>
        </div>
      )}
      
      {coverage.data_sources.length > 0 && (
        <div className="coverage-section">
          <span className="coverage-label">Data Sources:</span>
          <div className="coverage-tags">
            {coverage.data_sources.map(source => (
              <span key={source} className="data-source-tag">{source}</span>
            ))}
          </div>
        </div>
      )}
      
      {coverage.confidence_level && (
        <div className="coverage-meta">
          <span className={`confidence-badge ${coverage.confidence_level}`}>
            {coverage.confidence_level} confidence
          </span>
        </div>
      )}
      
      {coverage.max_severity && (
        <div className="coverage-meta">
          <span className={`severity-badge ${coverage.max_severity}`}>
            {coverage.max_severity} severity
          </span>
        </div>
      )}
    </div>
  );
}

// ============================================
// TableRow Component
// ============================================

/**
 * A single row in the table list.
 * 
 * Displays table name, class_uid, ocsf_version, dialect, status,
 * and detection coverage tags.
 * 
 * Requirements: 10.1, 10.2, 10.8
 */
function TableRow({ table, isSelected, onClick }: TableRowProps) {
  return (
    <div 
      className={`table-row ${isSelected ? 'selected' : ''} ${!table.is_active ? 'inactive' : ''}`}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === 'Enter' && onClick()}
    >
      <div className="table-row-main">
        <div className="table-name-section">
          <span className="table-icon">📋</span>
          <div className="table-name-info">
            <span className="table-name">{table.table_name}</span>
            {table.schema_name && (
              <span className="schema-name">{table.schema_name}</span>
            )}
          </div>
        </div>
        
        <div className="table-meta">
          <span className="meta-item class-uid" title="OCSF Class UID">
            Class: {table.class_uid}
          </span>
          <span className="meta-item ocsf-version" title="OCSF Version">
            v{table.ocsf_version}
          </span>
          <span className="meta-item dialect" title="SQL Dialect">
            {table.dialect}
          </span>
          <span className={`meta-item status ${table.is_active ? 'active' : 'inactive'}`}>
            {table.is_active ? 'Active' : 'Inactive'}
          </span>
        </div>
      </div>
      
      {/* Detection coverage tags (Requirement 10.2) */}
      {table.detection_coverage && table.detection_coverage.mitre_techniques.length > 0 && (
        <div className="table-row-coverage">
          {table.detection_coverage.mitre_techniques.slice(0, 3).map(technique => (
            <span key={technique} className="technique-tag small">{technique}</span>
          ))}
          {table.detection_coverage.mitre_techniques.length > 3 && (
            <span className="more-tag">
              +{table.detection_coverage.mitre_techniques.length - 3}
            </span>
          )}
        </div>
      )}
      
      {/* Timestamps (Requirement 10.8) */}
      <div className="table-row-timestamps">
        <span className="timestamp" title={`Created: ${formatDate(table.created_at)}`}>
          Created {formatRelativeTime(table.created_at)}
        </span>
        <span className="timestamp" title={`Updated: ${formatDate(table.updated_at)}`}>
          Updated {formatRelativeTime(table.updated_at)}
        </span>
      </div>
    </div>
  );
}

// ============================================
// TableDetailPanel Component
// ============================================

/**
 * Side panel showing full table details.
 * 
 * Displays all table metadata, detection coverage, and actions.
 * 
 * Requirements: 10.4, 10.6, 10.7, 10.8
 */
function TableDetailPanel({ table, onEdit, onClose }: TableDetailPanelProps) {
  const deactivateMutation = useDeactivateTable();
  
  const handleDeactivate = useCallback(() => {
    if (window.confirm(`Are you sure you want to deactivate "${table.table_name}"?`)) {
      deactivateMutation.mutate(table.id, {
        onSuccess: () => {
          onClose();
        },
      });
    }
  }, [table.id, table.table_name, deactivateMutation, onClose]);
  
  return (
    <div className="table-detail-panel">
      <div className="detail-panel-header">
        <div className="detail-title">
          <span className="detail-icon">📋</span>
          <h3>{table.table_name}</h3>
        </div>
        <button 
          className="close-btn" 
          onClick={onClose}
          aria-label="Close details"
        >
          ×
        </button>
      </div>
      
      <div className="detail-panel-content">
        {/* Table Information (Requirement 10.1) */}
        <div className="detail-section">
          <h4>Table Information</h4>
          <dl className="detail-list">
            <div className="detail-item">
              <dt>Table Name</dt>
              <dd>{table.table_name}</dd>
            </div>
            <div className="detail-item">
              <dt>Schema</dt>
              <dd>{table.schema_name || '(none)'}</dd>
            </div>
            <div className="detail-item">
              <dt>Class UID</dt>
              <dd>{table.class_uid}</dd>
            </div>
            <div className="detail-item">
              <dt>OCSF Version</dt>
              <dd>{table.ocsf_version}</dd>
            </div>
            <div className="detail-item">
              <dt>Dialect</dt>
              <dd>{table.dialect}</dd>
            </div>
            <div className="detail-item">
              <dt>Status</dt>
              <dd>
                <span className={`status-badge ${table.is_active ? 'active' : 'inactive'}`}>
                  {table.is_active ? 'Active' : 'Inactive'}
                </span>
              </dd>
            </div>
          </dl>
        </div>
        
        {/* Timestamps (Requirement 10.8) */}
        <div className="detail-section">
          <h4>Timestamps</h4>
          <dl className="detail-list">
            <div className="detail-item">
              <dt>Created</dt>
              <dd>{formatDate(table.created_at)}</dd>
            </div>
            <div className="detail-item">
              <dt>Updated</dt>
              <dd>{formatDate(table.updated_at)}</dd>
            </div>
          </dl>
        </div>
        
        {/* Detection Coverage (Requirement 10.2) */}
        {table.detection_coverage && (
          <div className="detail-section">
            <h4>
              <ContextualTooltip term="detection_coverage">
                <span>Detection Coverage</span>
              </ContextualTooltip>
            </h4>
            <DetectionCoverageDisplay coverage={table.detection_coverage} />
          </div>
        )}
        
        {/* Metadata */}
        {Object.keys(table.metadata).length > 0 && (
          <div className="detail-section">
            <h4>Metadata</h4>
            <dl className="detail-list">
              {Object.entries(table.metadata).map(([key, value]) => (
                <div key={key} className="detail-item">
                  <dt>{key}</dt>
                  <dd>{value}</dd>
                </div>
              ))}
            </dl>
          </div>
        )}
      </div>
      
      {/* Actions (Requirements 10.6, 10.7) */}
      <div className="detail-panel-actions">
        <button 
          className="btn secondary"
          onClick={onEdit}
        >
          Edit Table
        </button>
        {table.is_active && (
          <button 
            className="btn danger"
            onClick={handleDeactivate}
            disabled={deactivateMutation.isPending}
          >
            {deactivateMutation.isPending ? 'Deactivating...' : 'Deactivate'}
          </button>
        )}
      </div>
    </div>
  );
}

// ============================================
// RegisterTableModal Component
// ============================================

/**
 * Modal form for registering new tables or editing existing ones.
 * 
 * Requirements: 10.5, 10.6
 */
function RegisterTableModal({ onClose, editTable }: RegisterTableModalProps) {
  const registerMutation = useRegisterTable();
  const updateMutation = useUpdateTable();
  
  const isEditing = !!editTable;
  
  // Form state
  const [tableName, setTableName] = useState(editTable?.table_name || '');
  const [schemaName, setSchemaName] = useState(editTable?.schema_name || '');
  const [classUid, setClassUid] = useState(editTable?.class_uid?.toString() || '');
  const [ocsfVersion, setOcsfVersion] = useState(editTable?.ocsf_version || '1.3.0');
  const [dialect, setDialect] = useState(editTable?.dialect || 'snowflake');
  
  // Detection coverage state
  const [mitreTechniques, setMitreTechniques] = useState(
    editTable?.detection_coverage?.mitre_techniques.join(', ') || ''
  );
  const [mitreTactics, setMitreTactics] = useState(
    editTable?.detection_coverage?.mitre_tactics.join(', ') || ''
  );
  const [dataSources, setDataSources] = useState(
    editTable?.detection_coverage?.data_sources.join(', ') || ''
  );
  
  const [errors, setErrors] = useState<Record<string, string>>({});
  
  const validateForm = useCallback((): boolean => {
    const newErrors: Record<string, string> = {};
    
    if (!tableName.trim()) {
      newErrors.tableName = 'Table name is required';
    }
    
    const classUidNum = parseInt(classUid, 10);
    if (!classUid || isNaN(classUidNum) || classUidNum <= 0) {
      newErrors.classUid = 'Class UID must be a positive number';
    }
    
    if (!ocsfVersion.trim()) {
      newErrors.ocsfVersion = 'OCSF version is required';
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [tableName, classUid, ocsfVersion]);
  
  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }
    
    const tableData: RegisterTableRequest = {
      table_name: tableName.trim(),
      schema_name: schemaName.trim() || undefined,
      class_uid: parseInt(classUid, 10),
      ocsf_version: ocsfVersion.trim(),
      dialect,
      detection_coverage: {
        mitre_techniques: mitreTechniques.split(',').map(s => s.trim()).filter(Boolean),
        mitre_tactics: mitreTactics.split(',').map(s => s.trim()).filter(Boolean),
        data_sources: dataSources.split(',').map(s => s.trim()).filter(Boolean),
      },
    };
    
    if (isEditing && editTable) {
      updateMutation.mutate(
        { id: editTable.id, table: tableData },
        {
          onSuccess: () => {
            onClose();
          },
        }
      );
    } else {
      registerMutation.mutate(tableData, {
        onSuccess: () => {
          onClose();
        },
      });
    }
  }, [
    validateForm, tableName, schemaName, classUid, ocsfVersion, dialect,
    mitreTechniques, mitreTactics, dataSources, isEditing, editTable,
    registerMutation, updateMutation, onClose
  ]);
  
  const isPending = registerMutation.isPending || updateMutation.isPending;
  const error = registerMutation.error || updateMutation.error;
  
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="register-table-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{isEditing ? 'Edit Table' : 'Register New Table'}</h3>
          <button 
            className="close-btn" 
            onClick={onClose}
            aria-label="Close modal"
          >
            ×
          </button>
        </div>
        
        <form onSubmit={handleSubmit} className="modal-form">
          <div className="form-section">
            <h4>Table Information</h4>
            
            <div className="form-group">
              <label htmlFor="tableName">Table Name *</label>
              <input
                id="tableName"
                type="text"
                value={tableName}
                onChange={e => setTableName(e.target.value)}
                placeholder="e.g., auth_events"
                className={errors.tableName ? 'error' : ''}
              />
              {errors.tableName && (
                <span className="error-message">{errors.tableName}</span>
              )}
            </div>
            
            <div className="form-group">
              <label htmlFor="schemaName">Schema Name</label>
              <input
                id="schemaName"
                type="text"
                value={schemaName}
                onChange={e => setSchemaName(e.target.value)}
                placeholder="e.g., ocsf"
              />
            </div>
            
            <div className="form-row">
              <div className="form-group">
                <label htmlFor="classUid">Class UID *</label>
                <input
                  id="classUid"
                  type="number"
                  value={classUid}
                  onChange={e => setClassUid(e.target.value)}
                  placeholder="e.g., 3002"
                  className={errors.classUid ? 'error' : ''}
                />
                {errors.classUid && (
                  <span className="error-message">{errors.classUid}</span>
                )}
              </div>
              
              <div className="form-group">
                <label htmlFor="ocsfVersion">OCSF Version *</label>
                <input
                  id="ocsfVersion"
                  type="text"
                  value={ocsfVersion}
                  onChange={e => setOcsfVersion(e.target.value)}
                  placeholder="e.g., 1.3.0"
                  className={errors.ocsfVersion ? 'error' : ''}
                />
                {errors.ocsfVersion && (
                  <span className="error-message">{errors.ocsfVersion}</span>
                )}
              </div>
            </div>
            
            <div className="form-group">
              <label htmlFor="dialect">SQL Dialect</label>
              <select
                id="dialect"
                value={dialect}
                onChange={e => setDialect(e.target.value)}
              >
                <option value="snowflake">Snowflake</option>
                <option value="databricks">Databricks</option>
                <option value="bigquery">BigQuery</option>
                <option value="postgres">PostgreSQL</option>
              </select>
            </div>
          </div>
          
          <div className="form-section">
            <h4>
              <ContextualTooltip term="detection_coverage">
                <span>Detection Coverage</span>
              </ContextualTooltip>
              {' '}(Optional)
            </h4>
            
            <div className="form-group">
              <label htmlFor="mitreTechniques">MITRE Techniques</label>
              <input
                id="mitreTechniques"
                type="text"
                value={mitreTechniques}
                onChange={e => setMitreTechniques(e.target.value)}
                placeholder="e.g., T1071.001, T1078"
              />
              <span className="form-hint">Comma-separated technique IDs</span>
            </div>
            
            <div className="form-group">
              <label htmlFor="mitreTactics">MITRE Tactics</label>
              <input
                id="mitreTactics"
                type="text"
                value={mitreTactics}
                onChange={e => setMitreTactics(e.target.value)}
                placeholder="e.g., initial-access, persistence"
              />
              <span className="form-hint">Comma-separated tactic names</span>
            </div>
            
            <div className="form-group">
              <label htmlFor="dataSources">Data Sources</label>
              <input
                id="dataSources"
                type="text"
                value={dataSources}
                onChange={e => setDataSources(e.target.value)}
                placeholder="e.g., Authentication logs, Network traffic"
              />
              <span className="form-hint">Comma-separated data source names</span>
            </div>
          </div>
          
          {error && (
            <div className="form-error">
              {error instanceof Error ? error.message : 'An error occurred'}
            </div>
          )}
          
          <div className="modal-actions">
            <button 
              type="button" 
              className="btn secondary"
              onClick={onClose}
              disabled={isPending}
            >
              Cancel
            </button>
            <button 
              type="submit" 
              className="btn primary"
              disabled={isPending}
            >
              {isPending 
                ? (isEditing ? 'Updating...' : 'Registering...') 
                : (isEditing ? 'Update Table' : 'Register Table')
              }
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// ============================================
// Main Component
// ============================================

/**
 * TableRegistryBrowser displays and manages registered OCSF tables.
 * 
 * Features:
 * - Searchable, sortable table list (Requirement 10.1)
 * - Detection coverage tags (Requirement 10.2)
 * - Filtering by class_uid and active status (Requirement 10.3)
 * - Click to view details in side panel (Requirement 10.4)
 * - Register new tables (Requirement 10.5)
 * - Edit existing tables (Requirement 10.6)
 * - Deactivate tables (Requirement 10.7)
 * - Show timestamps (Requirement 10.8)
 */
export function TableRegistryBrowser() {
  // Filter state (Requirement 10.3)
  const [filterClassUid, setFilterClassUid] = useState<string>('');
  const [showInactive, setShowInactive] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  
  // Modal state
  const [showRegisterModal, setShowRegisterModal] = useState(false);
  const [editingTable, setEditingTable] = useState<TableEntry | undefined>();
  
  // Selection state from store
  const { selectedTableId, setSelectedTableId } = useIndexStore();
  
  // Build filter params
  const filterParams = useMemo(() => {
    const params: { class_uid?: number; is_active?: boolean } = {};
    
    const classUidNum = parseInt(filterClassUid, 10);
    if (!isNaN(classUidNum) && classUidNum > 0) {
      params.class_uid = classUidNum;
    }
    
    if (!showInactive) {
      params.is_active = true;
    }
    
    return params;
  }, [filterClassUid, showInactive]);
  
  // Fetch tables with filters
  const { data: tables, isLoading, error } = useTables(filterParams);
  
  // Filter tables by search query
  const filteredTables = useMemo(() => {
    if (!tables) return [];
    if (!searchQuery.trim()) return tables;
    
    const query = searchQuery.toLowerCase();
    return tables.filter(table => 
      table.table_name.toLowerCase().includes(query) ||
      table.schema_name?.toLowerCase().includes(query) ||
      table.dialect.toLowerCase().includes(query) ||
      table.class_uid.toString().includes(query)
    );
  }, [tables, searchQuery]);
  
  // Get selected table
  const selectedTable = useMemo(() => 
    tables?.find(t => t.id === selectedTableId),
    [tables, selectedTableId]
  );
  
  // Handlers
  const handleTableClick = useCallback((tableId: number) => {
    setSelectedTableId(tableId === selectedTableId ? null : tableId);
  }, [selectedTableId, setSelectedTableId]);
  
  const handleEdit = useCallback(() => {
    if (selectedTable) {
      setEditingTable(selectedTable);
      setShowRegisterModal(true);
    }
  }, [selectedTable]);
  
  const handleCloseModal = useCallback(() => {
    setShowRegisterModal(false);
    setEditingTable(undefined);
  }, []);
  
  const handleCloseDetail = useCallback(() => {
    setSelectedTableId(null);
  }, [setSelectedTableId]);
  
  // Loading state
  if (isLoading) {
    return (
      <div className="table-registry-browser">
        <div className="registry-loading">
          <div className="loading-spinner" />
          <span>Loading tables...</span>
        </div>
      </div>
    );
  }
  
  // Error state
  if (error) {
    return (
      <div className="table-registry-browser">
        <div className="registry-error">
          <span className="error-icon">⚠️</span>
          <span>Error loading tables</span>
          <span className="error-detail">
            {error instanceof Error ? error.message : 'Unknown error'}
          </span>
        </div>
      </div>
    );
  }
  
  return (
    <div className="table-registry-browser">
      {/* Header */}
      <div className="registry-header">
        <div className="registry-title">
          <h2>Table Registry</h2>
          <span className="table-count">
            {filteredTables.length} {filteredTables.length === 1 ? 'table' : 'tables'}
          </span>
        </div>
        <button 
          className="btn primary"
          onClick={() => setShowRegisterModal(true)}
        >
          Register Table
        </button>
      </div>
      
      {/* Filters (Requirement 10.3) */}
      <div className="registry-filters">
        <div className="filter-group search">
          <input
            type="text"
            placeholder="Search tables..."
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>
        
        <div className="filter-group">
          <label htmlFor="filterClassUid">Class UID:</label>
          <input
            id="filterClassUid"
            type="number"
            placeholder="Filter by class"
            value={filterClassUid}
            onChange={e => setFilterClassUid(e.target.value)}
            className="filter-input"
          />
        </div>
        
        <div className="filter-group checkbox">
          <label>
            <input
              type="checkbox"
              checked={showInactive}
              onChange={e => setShowInactive(e.target.checked)}
            />
            Show inactive
          </label>
        </div>
      </div>
      
      {/* Content */}
      <div className="registry-content">
        {/* Table List */}
        <div className={`table-list ${selectedTable ? 'with-panel' : ''}`}>
          {filteredTables.length === 0 ? (
            <div className="no-tables">
              <span className="empty-icon">📋</span>
              <span className="empty-title">No tables found</span>
              <span className="empty-description">
                {tables?.length === 0 
                  ? 'Register your first table to get started.'
                  : 'Try adjusting your filters or search query.'}
              </span>
            </div>
          ) : (
            filteredTables.map(table => (
              <TableRow
                key={table.id}
                table={table}
                isSelected={table.id === selectedTableId}
                onClick={() => handleTableClick(table.id)}
              />
            ))
          )}
        </div>
        
        {/* Detail Panel (Requirement 10.4) */}
        {selectedTable && (
          <TableDetailPanel
            table={selectedTable}
            onEdit={handleEdit}
            onClose={handleCloseDetail}
          />
        )}
      </div>
      
      {/* Register/Edit Modal (Requirements 10.5, 10.6) */}
      {showRegisterModal && (
        <RegisterTableModal
          onClose={handleCloseModal}
          editTable={editingTable}
        />
      )}
    </div>
  );
}

export default TableRegistryBrowser;
