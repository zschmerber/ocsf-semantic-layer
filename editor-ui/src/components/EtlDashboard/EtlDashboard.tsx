/**
 * EtlDashboard — Top-level ETL tab container with sub-navigation,
 * health indicator, and catalog entry selector.
 *
 * Manages view state (list / form / detail) and delegates to
 * EtlJobList, EtlJobForm, or EtlJobDetail based on current view.
 *
 * Requirements: 1.1–1.4, 3.1–3.4, 11.1–11.5, 12.2–12.3
 */

import { useState } from 'react';
import {
  useEtlHealth,
  useEtlCatalogEntries,
  useCreateJobFromCatalog,
} from '../../api/etlClient';
import type { EtlViewId } from '../../types/etl';
import { EtlJobList } from '../EtlJobList';
import { EtlJobForm } from '../EtlJobForm';
import { EtlJobDetail } from '../EtlJobDetail';
import './EtlDashboard.css';

export function EtlDashboard(): JSX.Element {
  const [etlView, setEtlView] = useState<EtlViewId>('list');
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [showCatalogSelector, setShowCatalogSelector] = useState(false);
  const [catalogError, setCatalogError] = useState<string | null>(null);

  // Health polling (every 30s)
  const { data: health, error: healthError } = useEtlHealth();

  // Catalog entries (fetched when selector is open)
  const { data: catalogEntries, isLoading: catalogLoading } = useEtlCatalogEntries();

  const createFromCatalog = useCreateJobFromCatalog();

  // --- Navigation callbacks ---

  const onSelectJob = (jobId: string) => {
    setSelectedJobId(jobId);
    setEtlView('detail');
  };

  const onNewJob = () => {
    setEtlView('form');
  };

  const onBack = () => {
    setSelectedJobId(null);
    setEtlView('list');
  };

  const onFromCatalog = () => {
    setCatalogError(null);
    setShowCatalogSelector(true);
  };

  const onCancelForm = () => {
    setEtlView('list');
  };

  const onJobCreated = (jobId: string) => {
    setSelectedJobId(jobId);
    setEtlView('detail');
  };

  // --- Catalog entry selection ---

  const handleCatalogSelect = (entryId: number) => {
    setCatalogError(null);
    createFromCatalog.mutate(entryId, {
      onSuccess: (data) => {
        setShowCatalogSelector(false);
        setSelectedJobId(data.job_id);
        setEtlView('detail');
      },
      onError: (err) => {
        setCatalogError(err instanceof Error ? err.message : 'Failed to create job from catalog');
      },
    });
  };

  // --- Health indicator state ---
  const isHealthy = !!health && !healthError;

  return (
    <div className="etl-dashboard">
      {/* Header with health indicator */}
      <div className="etl-dashboard-header">
        <h2 className="etl-dashboard-title">ETL Engine</h2>
        <div className="etl-health-indicator">
          <span
            className={`health-dot ${isHealthy ? 'health-connected' : 'health-disconnected'}`}
          />
          <span className="health-label">
            {isHealthy ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Configuration hint banner when health check fails */}
      {healthError && (
        <div className="etl-config-banner">
          Cannot connect to ETL Engine. Ensure the backend is running at{' '}
          <code>http://localhost:3030</code>
        </div>
      )}

      {/* Main content area — view router */}
      <div className="etl-dashboard-content">
        {etlView === 'list' && (
          <EtlJobList
            onSelectJob={onSelectJob}
            onNewJob={onNewJob}
            onFromCatalog={onFromCatalog}
          />
        )}

        {etlView === 'form' && (
          <EtlJobForm onCreated={onJobCreated} onCancel={onCancelForm} />
        )}

        {etlView === 'detail' && selectedJobId && (
          <EtlJobDetail jobId={selectedJobId} onBack={onBack} />
        )}
      </div>

      {/* Catalog entry selector modal */}
      {showCatalogSelector && (
        <div className="catalog-selector-overlay">
          <div className="catalog-selector-modal">
            <div className="catalog-selector-header">
              <h3>Create Job from Catalog</h3>
              <button
                className="btn-close"
                onClick={() => setShowCatalogSelector(false)}
                type="button"
                aria-label="Close catalog selector"
              >
                ×
              </button>
            </div>

            {catalogError && (
              <div className="catalog-selector-error">{catalogError}</div>
            )}

            {catalogLoading && (
              <div className="catalog-selector-loading">
                <span className="spinner" /> Loading catalog entries…
              </div>
            )}

            {!catalogLoading && catalogEntries && catalogEntries.length === 0 && (
              <div className="catalog-selector-empty">
                No catalog entries available.
              </div>
            )}

            {!catalogLoading && catalogEntries && catalogEntries.length > 0 && (
              <ul className="catalog-selector-list">
                {catalogEntries.map((entry) => (
                  <li key={entry.id} className="catalog-selector-item">
                    <button
                      className="catalog-selector-btn"
                      onClick={() => handleCatalogSelect(entry.id)}
                      disabled={createFromCatalog.isPending}
                      type="button"
                    >
                      <span className="catalog-entry-name">{entry.name}</span>
                      <span className="catalog-entry-version">
                        OCSF {entry.ocsf_version}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default EtlDashboard;
