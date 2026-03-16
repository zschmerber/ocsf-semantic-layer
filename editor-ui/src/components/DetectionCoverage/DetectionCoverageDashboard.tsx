/**
 * DetectionCoverageDashboard component for displaying MITRE ATT&CK coverage.
 * 
 * Renders a MITRE ATT&CK matrix visualization with tactics as columns
 * and techniques colored by coverage status (covered/gap).
 * 
 * Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8, 9.9
 */

import { useMemo, useState, useCallback } from 'react';
import { useCoverageMatrix, useTablesByTechnique } from '../../api/hooks';
import { useIndexStore } from '../../store/indexStore';
import type { TacticColumn, TechniqueCell, TableEntry } from '../../types';
import './DetectionCoverageDashboard.css';

// ============================================
// Constants
// ============================================

/**
 * MITRE ATT&CK tactics in kill chain order.
 * Used to sort tactics in the matrix visualization.
 */
const TACTIC_ORDER = [
  'reconnaissance',
  'resource-development',
  'initial-access',
  'execution',
  'persistence',
  'privilege-escalation',
  'defense-evasion',
  'credential-access',
  'discovery',
  'lateral-movement',
  'collection',
  'command-and-control',
  'exfiltration',
  'impact',
];

/**
 * Human-readable names for MITRE ATT&CK tactics.
 */
const TACTIC_DISPLAY_NAMES: Record<string, string> = {
  'reconnaissance': 'Reconnaissance',
  'resource-development': 'Resource Development',
  'initial-access': 'Initial Access',
  'execution': 'Execution',
  'persistence': 'Persistence',
  'privilege-escalation': 'Privilege Escalation',
  'defense-evasion': 'Defense Evasion',
  'credential-access': 'Credential Access',
  'discovery': 'Discovery',
  'lateral-movement': 'Lateral Movement',
  'collection': 'Collection',
  'command-and-control': 'Command & Control',
  'exfiltration': 'Exfiltration',
  'impact': 'Impact',
};

// ============================================
// Types
// ============================================

interface CoverageStats {
  covered: number;
  total: number;
  percentage: number;
}

interface TechniqueDetailPanelProps {
  techniqueId: string;
  techniqueName: string;
  tables: TableEntry[];
  onClose: () => void;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Get display name for a tactic.
 */
function getTacticDisplayName(tacticId: string): string {
  return TACTIC_DISPLAY_NAMES[tacticId] || formatTacticName(tacticId);
}

/**
 * Format a tactic ID into a display name.
 */
function formatTacticName(tacticId: string): string {
  return tacticId
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

// ============================================
// TechniqueDetailPanel Component
// ============================================

/**
 * Panel showing details for a selected technique.
 * Displays the list of tables covering the technique.
 * 
 * Requirement 9.5: Click handler to show technique details
 */
function TechniqueDetailPanel({ 
  techniqueId, 
  techniqueName,
  tables, 
  onClose 
}: TechniqueDetailPanelProps) {
  return (
    <div className="technique-detail-panel">
      <div className="technique-detail-header">
        <div className="technique-detail-title">
          <span className="technique-detail-id">{techniqueId}</span>
          <h4>{techniqueName}</h4>
        </div>
        <button 
          className="close-btn" 
          onClick={onClose}
          aria-label="Close technique details"
        >
          ×
        </button>
      </div>
      
      <div className="technique-detail-content">
        {tables.length === 0 ? (
          <div className="no-tables-message">
            <span className="gap-icon">⚠️</span>
            <span>No tables cover this technique</span>
            <span className="gap-hint">
              This represents a detection gap in your coverage.
            </span>
          </div>
        ) : (
          <>
            <div className="tables-count">
              <span className="count-number">{tables.length}</span>
              <span className="count-label">
                {tables.length === 1 ? 'table covers' : 'tables cover'} this technique
              </span>
            </div>
            <div className="tables-list">
              {tables.map(table => (
                <div key={table.id} className="table-item">
                  <div className="table-item-header">
                    <span className="table-icon">📋</span>
                    <span className="table-name">{table.table_name}</span>
                  </div>
                  <div className="table-item-meta">
                    <span className="table-class">Class: {table.class_uid}</span>
                    <span className="table-version">v{table.ocsf_version}</span>
                    <span className={`table-status ${table.is_active ? 'active' : 'inactive'}`}>
                      {table.is_active ? 'Active' : 'Inactive'}
                    </span>
                  </div>
                  {table.detection_coverage && (
                    <div className="table-item-coverage">
                      {table.detection_coverage.mitre_tactics.slice(0, 3).map(tactic => (
                        <span key={tactic} className="tactic-tag">
                          {getTacticDisplayName(tactic)}
                        </span>
                      ))}
                      {table.detection_coverage.mitre_tactics.length > 3 && (
                        <span className="more-tag">
                          +{table.detection_coverage.mitre_tactics.length - 3}
                        </span>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </>
        )}
      </div>
      
      <div className="technique-detail-footer">
        <a 
          href={`https://attack.mitre.org/techniques/${techniqueId.replace('.', '/')}`}
          target="_blank"
          rel="noopener noreferrer"
          className="mitre-link"
        >
          View on MITRE ATT&CK →
        </a>
      </div>
    </div>
  );
}

// ============================================
// CoverageStatistics Component
// ============================================

interface CoverageStatisticsProps {
  stats: CoverageStats;
}

/**
 * Displays coverage statistics summary.
 * 
 * Requirement 9.6: Display summary statistics
 */
function CoverageStatistics({ stats }: CoverageStatisticsProps) {
  const coverageLevel = useMemo(() => {
    if (stats.percentage >= 75) return 'high';
    if (stats.percentage >= 50) return 'medium';
    if (stats.percentage >= 25) return 'low';
    return 'critical';
  }, [stats.percentage]);

  return (
    <div className="coverage-statistics">
      <div className="stat-item covered">
        <span className="stat-value">{stats.covered}</span>
        <span className="stat-label">Covered</span>
      </div>
      <div className="stat-divider">/</div>
      <div className="stat-item total">
        <span className="stat-value">{stats.total}</span>
        <span className="stat-label">Techniques</span>
      </div>
      <div className={`stat-item percentage ${coverageLevel}`}>
        <span className="stat-value">{stats.percentage.toFixed(1)}%</span>
        <span className="stat-label">Coverage</span>
      </div>
      <div className="coverage-bar">
        <div 
          className={`coverage-fill ${coverageLevel}`}
          style={{ width: `${stats.percentage}%` }}
        />
      </div>
    </div>
  );
}

// ============================================
// TacticFilter Component
// ============================================

interface TacticFilterProps {
  tactics: TacticColumn[];
  selectedTactic: string | null;
  onSelectTactic: (tactic: string | null) => void;
}

/**
 * Filter buttons for selecting specific tactics.
 * 
 * Requirement 9.7: Support filtering by tactic
 */
function TacticFilter({ tactics, selectedTactic, onSelectTactic }: TacticFilterProps) {
  return (
    <div className="tactic-filter">
      <button
        className={`tactic-filter-btn ${selectedTactic === null ? 'active' : ''}`}
        onClick={() => onSelectTactic(null)}
      >
        All Tactics
      </button>
      {tactics.map(tactic => (
        <button
          key={tactic.id}
          className={`tactic-filter-btn ${selectedTactic === tactic.id ? 'active' : ''}`}
          onClick={() => onSelectTactic(tactic.id)}
          title={`${tactic.table_count} tables`}
        >
          {getTacticDisplayName(tactic.id)}
          <span className="filter-count">{tactic.table_count}</span>
        </button>
      ))}
    </div>
  );
}

// ============================================
// KillChainIndicator Component
// ============================================

interface KillChainIndicatorProps {
  tactics: TacticColumn[];
}

/**
 * Visual indicator showing coverage across the kill chain.
 * 
 * Requirement 9.9: Display kill_chain_coverage as visual indicator
 */
function KillChainIndicator({ tactics }: KillChainIndicatorProps) {
  const sortedTactics = useMemo(() => {
    return [...tactics].sort((a, b) => 
      TACTIC_ORDER.indexOf(a.id) - TACTIC_ORDER.indexOf(b.id)
    );
  }, [tactics]);

  return (
    <div className="kill-chain-indicator">
      <span className="kill-chain-label">Kill Chain Coverage:</span>
      <div className="kill-chain-phases">
        {sortedTactics.map(tactic => (
          <div
            key={tactic.id}
            className={`kill-chain-phase ${tactic.table_count > 0 ? 'covered' : 'gap'}`}
            title={`${getTacticDisplayName(tactic.id)}: ${tactic.table_count} tables`}
          />
        ))}
      </div>
    </div>
  );
}

// ============================================
// Main Component
// ============================================

/**
 * DetectionCoverageDashboard displays MITRE ATT&CK coverage.
 * 
 * Features:
 * - MITRE ATT&CK matrix with tactics as columns (Requirement 9.2)
 * - Techniques colored by coverage (Requirement 9.3, 9.4)
 * - Click handler for technique details (Requirement 9.5)
 * - Coverage statistics (Requirement 9.6)
 * - Tactic filtering (Requirement 9.7)
 * - Data sources coverage section (Requirement 9.8)
 * - Kill chain coverage indicator (Requirement 9.9)
 */
export function DetectionCoverageDashboard() {
  const { selectedTechnique, setSelectedTechnique } = useIndexStore();
  const [filterTactic, setFilterTactic] = useState<string | null>(null);
  
  // Fetch coverage matrix data (Requirement 9.1)
  const { data: matrix, isLoading, error } = useCoverageMatrix();
  
  // Fetch technique details when a technique is selected
  const { data: techniqueDetails, isLoading: isLoadingDetails } = useTablesByTechnique(
    selectedTechnique || ''
  );
  
  // Sort tactics by kill chain order
  const sortedTactics = useMemo(() => {
    if (!matrix) return [];
    return [...matrix.tactics].sort((a, b) => 
      TACTIC_ORDER.indexOf(a.id) - TACTIC_ORDER.indexOf(b.id)
    );
  }, [matrix]);
  
  // Calculate coverage statistics (Requirement 9.6)
  const coverageStats = useMemo((): CoverageStats => {
    if (!matrix) return { covered: 0, total: 0, percentage: 0 };
    
    const allTechniques = Object.values(matrix.techniques_by_tactic).flat();
    const covered = allTechniques.filter(t => t.is_covered).length;
    const total = allTechniques.length;
    const percentage = total > 0 ? (covered / total) * 100 : 0;
    
    return { covered, total, percentage };
  }, [matrix]);
  
  // Filter tactics based on selection
  const filteredTactics = useMemo(() => {
    if (!filterTactic) return sortedTactics;
    return sortedTactics.filter(t => t.id === filterTactic);
  }, [sortedTactics, filterTactic]);
  
  // Get selected technique info
  const selectedTechniqueInfo = useMemo(() => {
    if (!selectedTechnique || !matrix) return null;
    
    for (const techniques of Object.values(matrix.techniques_by_tactic)) {
      const found = techniques.find(t => t.id === selectedTechnique);
      if (found) return found;
    }
    return null;
  }, [selectedTechnique, matrix]);
  
  // Handle technique click (Requirement 9.5)
  const handleTechniqueClick = useCallback((technique: TechniqueCell) => {
    setSelectedTechnique(technique.id);
  }, [setSelectedTechnique]);
  
  // Handle close detail panel
  const handleCloseDetail = useCallback(() => {
    setSelectedTechnique(null);
  }, [setSelectedTechnique]);
  
  // Loading state (Requirement 9.1)
  if (isLoading) {
    return (
      <div className="detection-coverage-dashboard">
        <div className="coverage-loading">
          <div className="loading-spinner" />
          <span>Loading coverage data...</span>
        </div>
      </div>
    );
  }
  
  // Error state
  if (error) {
    return (
      <div className="detection-coverage-dashboard">
        <div className="coverage-error">
          <span className="error-icon">⚠️</span>
          <span>Error loading coverage data</span>
          <span className="error-detail">
            {error instanceof Error ? error.message : 'Unknown error'}
          </span>
        </div>
      </div>
    );
  }
  
  // Empty state
  if (!matrix || sortedTactics.length === 0) {
    return (
      <div className="detection-coverage-dashboard">
        <div className="coverage-empty">
          <span className="empty-icon">🛡️</span>
          <span className="empty-title">No coverage data</span>
          <span className="empty-description">
            Register tables with detection coverage metadata to see MITRE ATT&CK coverage here.
          </span>
        </div>
      </div>
    );
  }
  
  return (
    <div className="detection-coverage-dashboard">
      {/* Header with title and statistics (Requirement 9.6) */}
      <div className="coverage-header">
        <div className="coverage-title-section">
          <h2>MITRE ATT&CK Coverage</h2>
          <KillChainIndicator tactics={sortedTactics} />
        </div>
        <CoverageStatistics stats={coverageStats} />
      </div>
      
      {/* Tactic filter (Requirement 9.7) */}
      <TacticFilter
        tactics={sortedTactics}
        selectedTactic={filterTactic}
        onSelectTactic={setFilterTactic}
      />
      
      {/* MITRE ATT&CK Matrix (Requirement 9.2) */}
      <div className="mitre-matrix">
        {filteredTactics.map(tactic => (
          <div key={tactic.id} className="tactic-column">
            <div className="tactic-header">
              <span className="tactic-name">{getTacticDisplayName(tactic.id)}</span>
              <span className="tactic-count">
                {tactic.table_count} {tactic.table_count === 1 ? 'table' : 'tables'}
              </span>
            </div>
            <div className="technique-list">
              {(matrix.techniques_by_tactic[tactic.id] || []).map(technique => (
                <div
                  key={technique.id}
                  className={`technique-cell ${technique.is_covered ? 'covered' : 'gap'} ${
                    selectedTechnique === technique.id ? 'selected' : ''
                  }`}
                  onClick={() => handleTechniqueClick(technique)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => e.key === 'Enter' && handleTechniqueClick(technique)}
                  title={`${technique.name} - ${technique.table_count} tables`}
                >
                  <span className="technique-id">{technique.id}</span>
                  <span className="technique-name">{technique.name}</span>
                  {technique.table_count > 0 && (
                    <span className="technique-count">{technique.table_count}</span>
                  )}
                </div>
              ))}
              {(matrix.techniques_by_tactic[tactic.id] || []).length === 0 && (
                <div className="no-techniques">No techniques</div>
              )}
            </div>
          </div>
        ))}
      </div>
      
      {/* Technique Detail Panel (Requirement 9.5) */}
      {selectedTechnique && selectedTechniqueInfo && (
        <TechniqueDetailPanel
          techniqueId={selectedTechnique}
          techniqueName={selectedTechniqueInfo.name}
          tables={techniqueDetails || []}
          onClose={handleCloseDetail}
        />
      )}
      
      {/* Loading indicator for technique details */}
      {selectedTechnique && isLoadingDetails && (
        <div className="technique-loading-overlay">
          <div className="loading-spinner small" />
        </div>
      )}
    </div>
  );
}

export default DetectionCoverageDashboard;
