/**
 * LineageVisualization component for displaying data lineage graphs.
 * 
 * Renders source systems on the left, OCSF tables on the right,
 * with SVG edges connecting them to visualize data flow.
 * 
 * Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8
 */

import { useCallback, useMemo, useState, useRef, useEffect } from 'react';
import { useLineageGraph } from '../../api/hooks';
import { useIndexStore } from '../../store/indexStore';
import { useReferenceEventStore } from '../../store/referenceEventStore';
import type { LineageEdge, LineageNode } from '../../types';
import type { MappingEntry, MappingCoverage } from '../../types/referenceEvent';
import './LineageVisualization.css';

// ============================================
// Types
// ============================================

interface LineageVisualizationProps {
  /** Optional target table to filter lineage by */
  targetTable?: string;
}

interface NodePosition {
  id: string;
  x: number;
  y: number;
  height: number;
}

interface EdgePathProps {
  edge: LineageEdge;
  sourcePositions: Map<string, NodePosition>;
  targetPositions: Map<string, NodePosition>;
  isSelected: boolean;
  onClick: () => void;
}

// ============================================
// LineageEdgePath Component
// ============================================

/**
 * SVG path component for rendering lineage edges.
 * Uses bezier curves for smooth connections between nodes.
 */
function LineageEdgePath({ 
  edge, 
  sourcePositions, 
  targetPositions, 
  isSelected,
  onClick 
}: EdgePathProps) {
  const sourcePos = sourcePositions.get(edge.source);
  const targetPos = targetPositions.get(edge.target);
  
  if (!sourcePos || !targetPos) {
    return null;
  }
  
  // Calculate bezier curve control points
  const startX = sourcePos.x;
  const startY = sourcePos.y + sourcePos.height / 2;
  const endX = targetPos.x;
  const endY = targetPos.y + targetPos.height / 2;
  
  // Control points for smooth curve
  const midX = (startX + endX) / 2;
  const controlX1 = startX + (midX - startX) * 0.8;
  const controlX2 = endX - (endX - midX) * 0.8;
  
  const pathD = `M ${startX} ${startY} C ${controlX1} ${startY}, ${controlX2} ${endY}, ${endX} ${endY}`;
  
  return (
    <g className={`lineage-edge-group ${isSelected ? 'selected' : ''}`} onClick={onClick}>
      {/* Invisible wider path for easier clicking */}
      <path
        d={pathD}
        className="edge-hitbox"
        fill="none"
        stroke="transparent"
        strokeWidth={12}
      />
      {/* Visible edge path */}
      <path
        d={pathD}
        className="edge-path"
        fill="none"
      />
      {/* Record count label */}
      {edge.record_count !== undefined && (
        <text
          x={midX}
          y={(startY + endY) / 2 - 8}
          className="edge-label"
          textAnchor="middle"
        >
          {formatRecordCount(edge.record_count)}
        </text>
      )}
    </g>
  );
}

// ============================================
// Helper Functions
// ============================================

/**
 * Format record count for display (e.g., 1.2M, 500K)
 */
function formatRecordCount(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  }
  return count.toString();
}

// ============================================
// Mapping Lineage Types
// ============================================

type ConfidenceFilter = 'All' | 'High' | 'Medium' | 'Low';
type VerificationFilter = 'All' | 'Verified' | 'Unverified' | 'Conflict';
type SortField = 'confidence' | 'verification';

const CONFIDENCE_ORDER: Record<string, number> = { High: 0, Medium: 1, Low: 2 };
const VERIFICATION_ORDER: Record<string, number> = { Verified: 0, Unverified: 1, Conflict: 2 };

// ============================================
// Confidence Badge Component
// ============================================

function ConfidenceBadge({ confidence }: { confidence: 'High' | 'Medium' | 'Low' }) {
  const className = `mapping-badge confidence-${confidence.toLowerCase()}`;
  return <span className={className}>{confidence}</span>;
}

// ============================================
// Verification Badge Component
// ============================================

function VerificationBadge({ status, conflictDetail }: { status: string; conflictDetail?: string | null }) {
  const className = `mapping-badge verification-${status.toLowerCase()}`;
  return (
    <span className={className} title={status === 'Conflict' && conflictDetail ? conflictDetail : undefined}>
      {status}
    </span>
  );
}

// ============================================
// Coverage Summary Panel Component
// ============================================

function CoverageSummaryPanel({ coverage }: { coverage: MappingCoverage }) {
  const [showUnmapped, setShowUnmapped] = useState(false);
  const [showUnobserved, setShowUnobserved] = useState(false);

  return (
    <div className="mapping-coverage-panel">
      <h4 className="coverage-title">Coverage Summary</h4>
      <div className="coverage-bars">
        <div className="coverage-bar-row">
          <span className="coverage-label">Event fields mapped</span>
          <div className="coverage-bar-track">
            <div
              className="coverage-bar-fill coverage-mapped"
              style={{ width: `${coverage.percentEventFieldsMapped}%` }}
            />
          </div>
          <span className="coverage-pct">{coverage.percentEventFieldsMapped.toFixed(0)}%</span>
        </div>
        <div className="coverage-bar-row">
          <span className="coverage-label">Unmapped fields</span>
          <div className="coverage-bar-track">
            <div
              className="coverage-bar-fill coverage-unmapped"
              style={{ width: `${coverage.percentEventFieldsUnmapped}%` }}
            />
          </div>
          <span className="coverage-pct">{coverage.percentEventFieldsUnmapped.toFixed(0)}%</span>
        </div>
        <div className="coverage-bar-row">
          <span className="coverage-label">Unobserved mappings</span>
          <div className="coverage-bar-track">
            <div
              className="coverage-bar-fill coverage-unobserved"
              style={{ width: `${coverage.percentMappingFieldsUnobserved}%` }}
            />
          </div>
          <span className="coverage-pct">{coverage.percentMappingFieldsUnobserved.toFixed(0)}%</span>
        </div>
      </div>
      {coverage.unmappedFieldPaths.length > 0 && (
        <div className="coverage-detail-section">
          <button
            className="coverage-toggle-btn"
            onClick={() => setShowUnmapped(!showUnmapped)}
            aria-expanded={showUnmapped}
          >
            {showUnmapped ? '▾' : '▸'} Unmapped fields ({coverage.unmappedFieldPaths.length})
          </button>
          {showUnmapped && (
            <ul className="coverage-field-list">
              {coverage.unmappedFieldPaths.map((p) => (
                <li key={p}><code>{p}</code></li>
              ))}
            </ul>
          )}
        </div>
      )}
      {coverage.unobservedMappingFields.length > 0 && (
        <div className="coverage-detail-section">
          <button
            className="coverage-toggle-btn"
            onClick={() => setShowUnobserved(!showUnobserved)}
            aria-expanded={showUnobserved}
          >
            {showUnobserved ? '▾' : '▸'} Unobserved mapping fields ({coverage.unobservedMappingFields.length})
          </button>
          {showUnobserved && (
            <ul className="coverage-field-list">
              {coverage.unobservedMappingFields.map((p) => (
                <li key={p}><code>{p}</code></li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

// ============================================
// Mapping Lineage Overlay Component
// ============================================

/**
 * Displays LLM-interpreted mapping entries as a lineage overlay.
 * Shows raw_field → ocsf_field with transformation, confidence, and verification badges.
 * Includes filter/sort controls and a coverage summary panel.
 *
 * Requirements: 6.5, 6.6, 15.13, 15.14, 16.5, 16.6, 17.2, 17.3
 */
function MappingLineageOverlay({
  entries,
  coverage,
}: {
  entries: MappingEntry[];
  coverage: MappingCoverage | null;
}) {
  const [confidenceFilter, setConfidenceFilter] = useState<ConfidenceFilter>('All');
  const [verificationFilter, setVerificationFilter] = useState<VerificationFilter>('All');
  const [sortBy, setSortBy] = useState<SortField>('confidence');

  const filteredAndSorted = useMemo(() => {
    let result = [...entries];

    if (confidenceFilter !== 'All') {
      result = result.filter((e) => e.confidence === confidenceFilter);
    }
    if (verificationFilter !== 'All') {
      result = result.filter((e) => e.verificationStatus === verificationFilter);
    }

    result.sort((a, b) => {
      if (sortBy === 'confidence') {
        return (CONFIDENCE_ORDER[a.confidence] ?? 3) - (CONFIDENCE_ORDER[b.confidence] ?? 3);
      }
      return (VERIFICATION_ORDER[a.verificationStatus] ?? 3) - (VERIFICATION_ORDER[b.verificationStatus] ?? 3);
    });

    return result;
  }, [entries, confidenceFilter, verificationFilter, sortBy]);

  return (
    <div className="mapping-lineage-overlay">
      <div className="mapping-lineage-banner" role="status">
        <span className="banner-icon">🔗</span>
        Lineage auto-populated from LLM-interpreted mapping — review and edit
      </div>

      <div className="mapping-lineage-controls">
        <div className="mapping-filter-group">
          <label htmlFor="confidence-filter">Confidence:</label>
          <select
            id="confidence-filter"
            value={confidenceFilter}
            onChange={(e) => setConfidenceFilter(e.target.value as ConfidenceFilter)}
          >
            <option value="All">All</option>
            <option value="High">High</option>
            <option value="Medium">Medium</option>
            <option value="Low">Low</option>
          </select>
        </div>
        <div className="mapping-filter-group">
          <label htmlFor="verification-filter">Verification:</label>
          <select
            id="verification-filter"
            value={verificationFilter}
            onChange={(e) => setVerificationFilter(e.target.value as VerificationFilter)}
          >
            <option value="All">All</option>
            <option value="Verified">Verified</option>
            <option value="Unverified">Unverified</option>
            <option value="Conflict">Conflict</option>
          </select>
        </div>
        <div className="mapping-filter-group">
          <label htmlFor="sort-field">Sort by:</label>
          <select
            id="sort-field"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortField)}
          >
            <option value="confidence">Confidence</option>
            <option value="verification">Verification Status</option>
          </select>
        </div>
      </div>

      <div className="mapping-entries-table" role="table" aria-label="Mapping entries">
        <div className="mapping-table-header" role="row">
          <span className="mapping-col-field" role="columnheader">Raw Field → OCSF Field</span>
          <span className="mapping-col-transform" role="columnheader">Transformation</span>
          <span className="mapping-col-confidence" role="columnheader">Confidence</span>
          <span className="mapping-col-verification" role="columnheader">Verification</span>
        </div>
        {filteredAndSorted.length === 0 ? (
          <div className="mapping-empty-row">No entries match the current filters.</div>
        ) : (
          filteredAndSorted.map((entry, i) => (
            <div className="mapping-table-row" role="row" key={`${entry.rawField}-${entry.ocsfField}-${i}`}>
              <span className="mapping-col-field" role="cell">
                <code className="field-raw">{entry.rawField}</code>
                <span className="field-arrow">→</span>
                <code className="field-ocsf">{entry.ocsfField}</code>
              </span>
              <span className="mapping-col-transform" role="cell" title={entry.explanation ?? undefined}>
                {entry.transformation || '—'}
                {entry.explanation && <span className="transform-hint" title={entry.explanation}> ℹ️</span>}
              </span>
              <span className="mapping-col-confidence" role="cell">
                <ConfidenceBadge confidence={entry.confidence} />
              </span>
              <span className="mapping-col-verification" role="cell">
                <VerificationBadge status={entry.verificationStatus} conflictDetail={entry.conflictDetail} />
              </span>
            </div>
          ))
        )}
      </div>

      {coverage && <CoverageSummaryPanel coverage={coverage} />}
    </div>
  );
}

// ============================================
// Main Component
// ============================================

/**
 * LineageVisualization displays data lineage from source systems to OCSF tables.
 * 
 * Features:
 * - Source systems column on the left (Requirement 8.2)
 * - OCSF tables column on the right (Requirement 8.3)
 * - SVG edges with record count labels (Requirement 8.4)
 * - Click handlers for filtering (Requirement 8.5, 8.6, 8.7)
 * - Loading, error, and empty states (Requirement 8.8)
 */
export function LineageVisualization({ targetTable }: LineageVisualizationProps) {
  const { lineageFilter, setLineageFilter } = useIndexStore();
  const { interpretedMapping, mappingCoverage } = useReferenceEventStore();
  const [selectedEdge, setSelectedEdge] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  
  // Refs for calculating node positions
  const sourcesRef = useRef<HTMLDivElement>(null);
  const targetsRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  
  // Node position state for SVG edge rendering
  const [sourcePositions, setSourcePositions] = useState<Map<string, NodePosition>>(new Map());
  const [targetPositions, setTargetPositions] = useState<Map<string, NodePosition>>(new Map());
  
  // Fetch lineage data (Requirement 8.1)
  const { data: graph, isLoading, error } = useLineageGraph(
    targetTable || lineageFilter.target_table
  );
  
  // Separate nodes by type
  const sourceNodes = useMemo(() => 
    graph?.nodes.filter(n => n.node_type === 'source') ?? [], 
    [graph]
  );
  
  const targetNodes = useMemo(() => 
    graph?.nodes.filter(n => n.node_type === 'target') ?? [], 
    [graph]
  );
  
  // Calculate node positions after render
  useEffect(() => {
    if (!sourcesRef.current || !targetsRef.current || !containerRef.current) {
      return;
    }
    
    const containerRect = containerRef.current.getBoundingClientRect();
    
    // Calculate source node positions
    const newSourcePositions = new Map<string, NodePosition>();
    const sourceNodeElements = sourcesRef.current.querySelectorAll('.lineage-node');
    sourceNodeElements.forEach((el) => {
      const nodeId = el.getAttribute('data-node-id');
      if (nodeId) {
        const rect = el.getBoundingClientRect();
        newSourcePositions.set(nodeId, {
          id: nodeId,
          x: rect.right - containerRect.left,
          y: rect.top - containerRect.top,
          height: rect.height,
        });
      }
    });
    setSourcePositions(newSourcePositions);
    
    // Calculate target node positions
    const newTargetPositions = new Map<string, NodePosition>();
    const targetNodeElements = targetsRef.current.querySelectorAll('.lineage-node');
    targetNodeElements.forEach((el) => {
      const nodeId = el.getAttribute('data-node-id');
      if (nodeId) {
        const rect = el.getBoundingClientRect();
        newTargetPositions.set(nodeId, {
          id: nodeId,
          x: rect.left - containerRect.left,
          y: rect.top - containerRect.top,
          height: rect.height,
        });
      }
    });
    setTargetPositions(newTargetPositions);
  }, [graph, sourceNodes, targetNodes]);
  
  // Handle node click for filtering (Requirement 8.5, 8.6)
  const handleNodeClick = useCallback((node: LineageNode) => {
    setSelectedNode(node.id);
    setSelectedEdge(null);
    
    if (node.node_type === 'target') {
      // Filter by target table
      setLineageFilter({ target_table: node.id });
    } else if (node.node_type === 'source') {
      // Filter by source system
      const system = node.metadata.system;
      if (system) {
        setLineageFilter({ source_system: system });
      }
    }
  }, [setLineageFilter]);
  
  // Handle edge click for field-level mappings (Requirement 8.5)
  const handleEdgeClick = useCallback((edge: LineageEdge) => {
    const edgeId = `${edge.source}->${edge.target}`;
    setSelectedEdge(edgeId);
    setSelectedNode(null);
    // Edge click could trigger field lineage display in a detail panel
  }, []);
  
  // Handle filter change
  const handleFilterChange = useCallback((value: string) => {
    setLineageFilter({ target_table: value || undefined });
    setSelectedNode(null);
    setSelectedEdge(null);
  }, [setLineageFilter]);
  
  // Clear filter
  const handleClearFilter = useCallback(() => {
    setLineageFilter({});
    setSelectedNode(null);
    setSelectedEdge(null);
  }, [setLineageFilter]);
  
  // Loading state (Requirement 8.8)
  if (isLoading) {
    return (
      <div className="lineage-visualization">
        <div className="lineage-loading">
          <div className="loading-spinner" />
          <span>Loading lineage data...</span>
        </div>
      </div>
    );
  }
  
  // Error state (Requirement 8.8)
  if (error) {
    return (
      <div className="lineage-visualization">
        <div className="lineage-error">
          <span className="error-icon">⚠️</span>
          <span>Error loading lineage data</span>
          <span className="error-detail">
            {error instanceof Error ? error.message : 'Unknown error'}
          </span>
        </div>
      </div>
    );
  }
  
  // Empty state (Requirement 8.8)
  if (!graph || (graph.nodes.length === 0 && graph.edges.length === 0)) {
    return (
      <div className="lineage-visualization">
        {/* Show mapping overlay even when no lineage graph data (Requirement 6.5, 6.6) */}
        {interpretedMapping && (
          <MappingLineageOverlay
            entries={interpretedMapping.entries}
            coverage={mappingCoverage}
          />
        )}
        <div className="lineage-empty">
          <span className="empty-icon">📊</span>
          <span className="empty-title">No lineage data</span>
          <span className="empty-description">
            Import logs and create mappings to see data lineage here.
          </span>
        </div>
      </div>
    );
  }
  
  const hasActiveFilter = lineageFilter.target_table || lineageFilter.source_system;
  
  return (
    <div className="lineage-visualization">
      {/* Mapping lineage overlay when LLM-interpreted mapping available (Requirement 6.5, 6.6) */}
      {interpretedMapping && (
        <MappingLineageOverlay
          entries={interpretedMapping.entries}
          coverage={mappingCoverage}
        />
      )}

      {/* Header with title and filter (Requirement 8.7) */}
      <div className="lineage-header">
        <h3>Data Lineage</h3>
        <div className="lineage-controls">
          <select 
            className="lineage-filter-select"
            value={lineageFilter.target_table || ''} 
            onChange={(e) => handleFilterChange(e.target.value)}
            aria-label="Filter by target table"
          >
            <option value="">All Tables</option>
            {targetNodes.map(n => (
              <option key={n.id} value={n.id}>{n.label}</option>
            ))}
          </select>
          {hasActiveFilter && (
            <button 
              className="clear-filter-btn"
              onClick={handleClearFilter}
              aria-label="Clear filter"
            >
              Clear Filter
            </button>
          )}
        </div>
      </div>
      
      {/* Graph container */}
      <div className="lineage-graph" ref={containerRef}>
        {/* Source systems column (Requirement 8.2) */}
        <div className="lineage-column sources" ref={sourcesRef}>
          <h4>Source Systems</h4>
          <div className="node-list">
            {sourceNodes.length === 0 ? (
              <div className="no-nodes">No source systems</div>
            ) : (
              sourceNodes.map(node => (
                <div 
                  key={node.id}
                  data-node-id={node.id}
                  className={`lineage-node source ${selectedNode === node.id ? 'selected' : ''}`}
                  onClick={() => handleNodeClick(node)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => e.key === 'Enter' && handleNodeClick(node)}
                >
                  <span className="node-icon">🔌</span>
                  <div className="node-content">
                    <span className="node-label">{node.label}</span>
                    {node.metadata.system && (
                      <span className="node-system">{node.metadata.system}</span>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
        
        {/* SVG edges container (Requirement 8.4) */}
        <div className="lineage-edges">
          <svg className="edge-canvas" width="100%" height="100%">
            {graph.edges.map((edge, i) => {
              const edgeId = `${edge.source}->${edge.target}`;
              return (
                <LineageEdgePath 
                  key={i} 
                  edge={edge}
                  sourcePositions={sourcePositions}
                  targetPositions={targetPositions}
                  isSelected={selectedEdge === edgeId}
                  onClick={() => handleEdgeClick(edge)}
                />
              );
            })}
          </svg>
        </div>
        
        {/* Target OCSF tables column (Requirement 8.3) */}
        <div className="lineage-column targets" ref={targetsRef}>
          <h4>OCSF Tables</h4>
          <div className="node-list">
            {targetNodes.length === 0 ? (
              <div className="no-nodes">No OCSF tables</div>
            ) : (
              targetNodes.map(node => (
                <div 
                  key={node.id}
                  data-node-id={node.id}
                  className={`lineage-node target ${selectedNode === node.id ? 'selected' : ''}`}
                  onClick={() => handleNodeClick(node)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => e.key === 'Enter' && handleNodeClick(node)}
                >
                  <span className="node-icon">📋</span>
                  <div className="node-content">
                    <span className="node-label">{node.label}</span>
                    {node.metadata.class_uid && (
                      <span className="node-class">Class: {node.metadata.class_uid}</span>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
      
      {/* Selected edge detail panel */}
      {selectedEdge && (
        <SelectedEdgeDetail 
          edge={graph.edges.find(e => `${e.source}->${e.target}` === selectedEdge)}
          onClose={() => setSelectedEdge(null)}
        />
      )}
    </div>
  );
}

// ============================================
// Selected Edge Detail Component
// ============================================

interface SelectedEdgeDetailProps {
  edge?: LineageEdge;
  onClose: () => void;
}

/**
 * Detail panel shown when an edge is selected.
 * Displays field-level mappings for the source-to-target relationship.
 */
function SelectedEdgeDetail({ edge, onClose }: SelectedEdgeDetailProps) {
  if (!edge) return null;
  
  return (
    <div className="edge-detail-panel">
      <div className="edge-detail-header">
        <h4>Lineage Details</h4>
        <button 
          className="close-btn" 
          onClick={onClose}
          aria-label="Close details"
        >
          ×
        </button>
      </div>
      <div className="edge-detail-content">
        <div className="detail-row">
          <span className="detail-label">Source:</span>
          <span className="detail-value">{edge.source}</span>
        </div>
        <div className="detail-row">
          <span className="detail-label">Target:</span>
          <span className="detail-value">{edge.target}</span>
        </div>
        <div className="detail-row">
          <span className="detail-label">Timestamp:</span>
          <span className="detail-value">
            {new Date(edge.timestamp).toLocaleString()}
          </span>
        </div>
        {edge.record_count !== undefined && (
          <div className="detail-row">
            <span className="detail-label">Records:</span>
            <span className="detail-value">
              {edge.record_count.toLocaleString()}
            </span>
          </div>
        )}
      </div>
      <div className="edge-detail-footer">
        <span className="hint">Click to view field-level mappings</span>
      </div>
    </div>
  );
}

export default LineageVisualization;
