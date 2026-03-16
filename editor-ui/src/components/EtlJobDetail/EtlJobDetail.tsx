/**
 * EtlJobDetail — Job metadata, lifecycle buttons, artifact viewer, and log panel.
 *
 * Polls job status every 3 seconds. Provides lifecycle action buttons
 * (Generate, Compile, Test, Run, Stop) with enablement driven by
 * the current job status state machine.
 *
 * Requirements: 7.1, 7.2, 7.3, 7.4, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8
 */

import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEtlJob, etlPost, etlKeys } from '../../api/etlClient';
import { getStatusBadgeClass, getEnabledActions } from '../../utils/etlHelpers';
import { EtlArtifactViewer } from '../EtlArtifactViewer';
import { EtlLogPanel } from '../EtlLogPanel';
import type { JobActionResponse } from '../../types/etl';
import './EtlJobDetail.css';

interface EtlJobDetailProps {
  jobId: string;
  onBack: () => void;
}

const LIFECYCLE_ACTIONS = [
  { label: 'Generate', action: 'generate' },
  { label: 'Compile', action: 'compile' },
  { label: 'Test', action: 'test' },
  { label: 'Run', action: 'run' },
  { label: 'Stop', action: 'stop' },
] as const;

export function EtlJobDetail({ jobId, onBack }: EtlJobDetailProps) {
  const { data: job, isLoading, error: fetchError } = useEtlJob(jobId);
  const queryClient = useQueryClient();

  const [activeAction, setActiveAction] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const actionMutation = useMutation({
    mutationFn: (action: string) =>
      etlPost<JobActionResponse>(`/api/jobs/${jobId}/${action}`),
    onSuccess: () => {
      setActionError(null);
      queryClient.invalidateQueries({ queryKey: etlKeys.job(jobId) });
      queryClient.invalidateQueries({ queryKey: etlKeys.jobs });
    },
    onError: (err: Error) => {
      setActionError(err.message);
    },
    onSettled: () => {
      setActiveAction(null);
    },
  });

  const handleAction = (action: string) => {
    setActiveAction(action);
    setActionError(null);
    actionMutation.mutate(action);
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="etl-job-detail">
        <div className="etl-job-detail-loading">
          <span className="spinner" /> Loading job details…
        </div>
      </div>
    );
  }

  // Error state
  if (fetchError || !job) {
    return (
      <div className="etl-job-detail">
        <div className="etl-job-detail-error">
          <span>{fetchError?.message ?? 'Failed to load job details'}</span>
          <button className="btn-back" onClick={onBack} type="button">
            ← Back to Jobs
          </button>
        </div>
      </div>
    );
  }

  const enabledActions = getEnabledActions(job.status);
  const badgeClass = getStatusBadgeClass(job.status);

  return (
    <div className="etl-job-detail">
      {/* Header */}
      <div className="etl-job-detail-header">
        <button className="btn-back" onClick={onBack} type="button">
          ← Back to Jobs
        </button>
        <h3>Job Detail</h3>
      </div>

      {/* Metadata */}
      <dl className="etl-job-metadata">
        <dt>Job ID</dt>
        <dd>{job.job_id}</dd>

        <dt>Plugin Name</dt>
        <dd>{job.plugin_name}</dd>

        <dt>OCSF Class UID</dt>
        <dd>{job.ocsf_class_uid}</dd>

        <dt>Status</dt>
        <dd>
          <span className={`status-badge ${badgeClass}`}>{job.status}</span>
        </dd>
      </dl>

      {/* Lifecycle action buttons */}
      <div className="etl-actions">
        {LIFECYCLE_ACTIONS.map(({ label, action }) => {
          const isActive = activeAction === action;
          return (
            <button
              key={action}
              className="etl-action-btn"
              disabled={!enabledActions.has(label) || actionMutation.isPending}
              onClick={() => handleAction(action)}
              type="button"
            >
              {isActive && <span className="spinner" />}
              {label}
            </button>
          );
        })}
      </div>

      {actionError && (
        <div className="etl-action-error">{actionError}</div>
      )}

      {/* Artifact viewer */}
      <EtlArtifactViewer artifactFiles={job.artifact_files} jobId={jobId} />

      {/* Log panel */}
      <EtlLogPanel jobId={jobId} jobStatus={job.status} />
    </div>
  );
}

export default EtlJobDetail;
