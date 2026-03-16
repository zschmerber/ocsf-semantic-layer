/**
 * StatisticsViewer component for displaying table column statistics.
 * 
 * Displays column-level statistics including distinct count, null count,
 * min/max values, null rate, and cardinality with visual indicators.
 * 
 * Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8
 */

import { useCallback, useMemo } from 'react';
import { useTableStatistics, useCollectStatistics } from '../../api/hooks';
import type { TableStatistics, ColumnStatistics } from '../../types';
import './StatisticsViewer.css';

// ============================================
// Types
// ============================================

interface StatisticsViewerProps {
  /** Name of the table to display statistics for */
  tableName: string;
}

interface ColumnStatsRowProps {
  /** Column statistics data */
  column: ColumnStatistics;
}

interface RateBarProps {
  /** Rate value as a percentage (0-100) */
  rate: number;
  /** Type of rate for styling */
  type: 'null' | 'cardinality';
  /** Label for accessibility */
  label: string;
}

interface StalenessIndicatorProps {
  /** Whether the statistics are stale */
  isStale: boolean;
  /** Collection timestamp */
  collectedAt: string;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Format a number with locale-specific separators.
 */
function formatNumber(value: number): string {
  return value.toLocaleString();
}

/**
 * Format a percentage value.
 */
function formatPercentage(value: number): string {
  return `${value.toFixed(1)}%`;
}

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
  
  if (diffDays > 0) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
  if (diffHours > 0) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
  if (diffMins > 0) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
  return 'just now';
}

/**
 * Truncate a string value for display.
 */
function truncateValue(value: string | undefined, maxLen: number = 20): string {
  if (!value) return '-';
  return value.length > maxLen ? `${value.slice(0, maxLen)}...` : value;
}

/**
 * Get the rate level for styling (low, medium, high).
 */
function getRateLevel(rate: number): 'low' | 'medium' | 'high' {
  if (rate < 25) return 'low';
  if (rate < 75) return 'medium';
  return 'high';
}

// ============================================
// RateBar Component
// ============================================

/**
 * Visual bar indicator for rate values.
 * 
 * Requirements: 11.3, 11.4
 */
function RateBar({ rate, type, label }: RateBarProps) {
  const level = getRateLevel(rate);
  
  return (
    <div className="rate-bar-container">
      <div 
        className="rate-bar"
        role="progressbar"
        aria-valuenow={rate}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={label}
      >
        <div 
          className={`rate-fill ${type} ${level}`}
          style={{ width: `${Math.min(rate, 100)}%` }}
        />
      </div>
      <span className={`rate-value ${level}`}>
        {formatPercentage(rate)}
      </span>
    </div>
  );
}

// ============================================
// StalenessIndicator Component
// ============================================

/**
 * Indicator showing when statistics were collected and if they're stale.
 * 
 * Requirements: 11.5, 11.8
 */
function StalenessIndicator({ isStale, collectedAt }: StalenessIndicatorProps) {
  return (
    <div className={`staleness-indicator ${isStale ? 'stale' : 'fresh'}`}>
      <span className="staleness-icon">
        {isStale ? '⚠️' : '✓'}
      </span>
      <div className="staleness-info">
        <span className="staleness-label">
          {isStale ? 'Statistics are stale' : 'Statistics are current'}
        </span>
        <span 
          className="staleness-time"
          title={`Collected: ${formatDate(collectedAt)}`}
        >
          Collected {formatRelativeTime(collectedAt)}
        </span>
      </div>
    </div>
  );
}

// ============================================
// ColumnStatsRow Component
// ============================================

/**
 * A single row in the column statistics table.
 * 
 * Displays column name, distinct count, null count, null rate,
 * cardinality, and min/max values.
 * 
 * Requirements: 11.2, 11.3, 11.4, 11.5
 */
function ColumnStatsRow({ column }: ColumnStatsRowProps) {
  // Calculate null rate and cardinality
  const nullRate = column.null_rate ?? (
    column.total_count > 0 
      ? (column.null_count / column.total_count) * 100 
      : 0
  );
  
  const cardinality = column.cardinality ?? (
    column.total_count > 0 
      ? (column.distinct_count / column.total_count) * 100 
      : 0
  );
  
  return (
    <tr className="column-stats-row">
      {/* Column Name */}
      <td className="col-name">
        <span className="column-name-text" title={column.column_name}>
          {column.column_name}
        </span>
      </td>
      
      {/* Distinct Count (Requirement 11.2) */}
      <td className="col-distinct">
        <span className="stat-value">{formatNumber(column.distinct_count)}</span>
      </td>
      
      {/* Null Count (Requirement 11.2) */}
      <td className="col-null-count">
        <span className="stat-value">{formatNumber(column.null_count)}</span>
      </td>
      
      {/* Null Rate with visual bar (Requirement 11.3) */}
      <td className="col-null-rate">
        <RateBar 
          rate={nullRate} 
          type="null" 
          label={`Null rate for ${column.column_name}`}
        />
      </td>
      
      {/* Cardinality with visual bar (Requirement 11.4) */}
      <td className="col-cardinality">
        <RateBar 
          rate={cardinality} 
          type="cardinality" 
          label={`Cardinality for ${column.column_name}`}
        />
      </td>
      
      {/* Min Value (Requirement 11.2) */}
      <td className="col-min">
        <span 
          className="stat-value min-max"
          title={column.min_value || 'No minimum value'}
        >
          {truncateValue(column.min_value)}
        </span>
      </td>
      
      {/* Max Value (Requirement 11.2) */}
      <td className="col-max">
        <span 
          className="stat-value min-max"
          title={column.max_value || 'No maximum value'}
        >
          {truncateValue(column.max_value)}
        </span>
      </td>
    </tr>
  );
}

// ============================================
// StatisticsSummary Component
// ============================================

interface StatisticsSummaryProps {
  stats: TableStatistics;
}

/**
 * Summary section showing total rows and sample rate.
 * 
 * Requirements: 11.7, 11.8
 */
function StatisticsSummary({ stats }: StatisticsSummaryProps) {
  const samplePercentage = (stats.sample_rate * 100).toFixed(0);
  
  return (
    <div className="statistics-summary">
      <div className="summary-item">
        <span className="summary-label">Total Rows</span>
        <span className="summary-value">{formatNumber(stats.total_rows)}</span>
      </div>
      <div className="summary-item">
        <span className="summary-label">Sample Rate</span>
        <span className="summary-value">{samplePercentage}%</span>
      </div>
      <div className="summary-item">
        <span className="summary-label">Columns</span>
        <span className="summary-value">{stats.columns.length}</span>
      </div>
    </div>
  );
}

// ============================================
// Main Component
// ============================================

/**
 * StatisticsViewer displays column-level statistics for a table.
 * 
 * Features:
 * - Column statistics table (Requirement 11.1, 11.2)
 * - Null rate with visual bar (Requirement 11.3)
 * - Cardinality with visual bar (Requirement 11.4)
 * - Staleness indicator (Requirement 11.5)
 * - Refresh button (Requirement 11.6)
 * - Sample rate display (Requirement 11.7)
 * - Collection timestamp (Requirement 11.8)
 */
export function StatisticsViewer({ tableName }: StatisticsViewerProps) {
  // Fetch statistics (Requirement 11.1)
  const { data: stats, isLoading, error, refetch } = useTableStatistics(tableName);
  
  // Mutation for collecting statistics (Requirement 11.6)
  const collectMutation = useCollectStatistics();
  
  // Handle refresh button click (Requirement 11.6)
  const handleRefresh = useCallback(() => {
    collectMutation.mutate(
      { tableName },
      {
        onSuccess: () => {
          // Refetch statistics after collection
          refetch();
        },
      }
    );
  }, [tableName, collectMutation, refetch]);
  
  // Determine if statistics are stale (Requirement 11.5)
  const isStale = stats?.is_stale ?? false;
  
  // Sort columns alphabetically
  const sortedColumns = useMemo(() => {
    if (!stats?.columns) return [];
    return [...stats.columns].sort((a, b) => 
      a.column_name.localeCompare(b.column_name)
    );
  }, [stats?.columns]);
  
  // Loading state
  if (isLoading) {
    return (
      <div className="statistics-viewer">
        <div className="statistics-loading">
          <div className="loading-spinner" />
          <span>Loading statistics for {tableName}...</span>
        </div>
      </div>
    );
  }
  
  // Error state (Requirement 11.1 - handle 404)
  if (error) {
    const is404 = error instanceof Error && error.message.includes('404');
    
    return (
      <div className="statistics-viewer">
        <div className="statistics-error">
          <span className="error-icon">📊</span>
          <span className="error-title">
            {is404 ? 'No statistics available' : 'Error loading statistics'}
          </span>
          <span className="error-description">
            {is404 
              ? `Statistics have not been collected for "${tableName}" yet.`
              : (error instanceof Error ? error.message : 'Unknown error')
            }
          </span>
          <button 
            className="btn primary"
            onClick={handleRefresh}
            disabled={collectMutation.isPending}
          >
            {collectMutation.isPending ? 'Collecting...' : 'Collect Statistics'}
          </button>
        </div>
      </div>
    );
  }
  
  // Empty state
  if (!stats || stats.columns.length === 0) {
    return (
      <div className="statistics-viewer">
        <div className="statistics-empty">
          <span className="empty-icon">📊</span>
          <span className="empty-title">No column statistics</span>
          <span className="empty-description">
            Click the button below to collect statistics for this table.
          </span>
          <button 
            className="btn primary"
            onClick={handleRefresh}
            disabled={collectMutation.isPending}
          >
            {collectMutation.isPending ? 'Collecting...' : 'Collect Statistics'}
          </button>
        </div>
      </div>
    );
  }
  
  return (
    <div className="statistics-viewer">
      {/* Header with title and refresh button */}
      <div className="statistics-header">
        <div className="statistics-title-section">
          <h3>Table Statistics</h3>
          <span className="table-name-badge">{tableName}</span>
        </div>
        
        <div className="statistics-actions">
          {/* Staleness indicator (Requirements 11.5, 11.8) */}
          <StalenessIndicator 
            isStale={isStale} 
            collectedAt={stats.collected_at} 
          />
          
          {/* Refresh button (Requirement 11.6) */}
          <button 
            className={`btn refresh ${isStale ? 'primary' : 'secondary'}`}
            onClick={handleRefresh}
            disabled={collectMutation.isPending}
            title={isStale ? 'Statistics are stale - click to refresh' : 'Refresh statistics'}
          >
            <span className="refresh-icon">🔄</span>
            {collectMutation.isPending ? 'Collecting...' : 'Refresh'}
          </button>
        </div>
      </div>
      
      {/* Summary section (Requirements 11.7, 11.8) */}
      <StatisticsSummary stats={stats} />
      
      {/* Column statistics table (Requirements 11.1, 11.2, 11.3, 11.4) */}
      <div className="statistics-table-container">
        <table className="column-stats-table">
          <thead>
            <tr>
              <th className="col-header col-name">Column</th>
              <th className="col-header col-distinct">Distinct</th>
              <th className="col-header col-null-count">Nulls</th>
              <th className="col-header col-null-rate">Null Rate</th>
              <th className="col-header col-cardinality">Cardinality</th>
              <th className="col-header col-min">Min</th>
              <th className="col-header col-max">Max</th>
            </tr>
          </thead>
          <tbody>
            {sortedColumns.map(column => (
              <ColumnStatsRow 
                key={column.column_name} 
                column={column} 
              />
            ))}
          </tbody>
        </table>
      </div>
      
      {/* Collection error message */}
      {collectMutation.error && (
        <div className="collection-error">
          <span className="error-icon">⚠️</span>
          <span>
            Failed to collect statistics: {
              collectMutation.error instanceof Error 
                ? collectMutation.error.message 
                : 'Unknown error'
            }
          </span>
        </div>
      )}
    </div>
  );
}

export default StatisticsViewer;
