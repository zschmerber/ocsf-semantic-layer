/**
 * EtlJobList — Job table with polling and status badges.
 *
 * Polls GET /api/jobs every 5 seconds via useEtlJobs() hook.
 * Renders a table of jobs with clickable rows, status badges,
 * and action buttons for creating new jobs or importing from catalog.
 *
 * Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6
 */

import { useEtlJobs } from '../../api/etlClient';
import { getStatusBadgeClass } from '../../utils/etlHelpers';
import './EtlJobList.css';

interface EtlJobListProps {
  onSelectJob: (jobId: string) => void;
  onNewJob: () => void;
  onFromCatalog: () => void;
}

export function EtlJobList({ onSelectJob, onNewJob, onFromCatalog }: EtlJobListProps) {
  const { data: jobs, isLoading, error } = useEtlJobs();

  return (
    <div className="etl-job-list">
      {/* Header with action buttons */}
      <div className="etl-job-list-header">
        <h3>ETL Jobs</h3>
        <div className="etl-job-list-actions">
          <button className="btn-primary" onClick={onNewJob} type="button">
            + New Job
          </button>
          <button className="btn-secondary" onClick={onFromCatalog} type="button">
            From Catalog
          </button>
        </div>
      </div>

      {/* Error state */}
      {error && (
        <div className="etl-job-list-error">
          {error instanceof Error ? error.message : 'Failed to load jobs'}
        </div>
      )}

      {/* Loading state */}
      {isLoading && (
        <div className="etl-job-list-loading">
          <span className="spinner" /> Loading jobs…
        </div>
      )}

      {/* Empty state */}
      {!isLoading && !error && jobs && jobs.length === 0 && (
        <div className="etl-job-list-empty">
          No ETL jobs yet. Create a new job to get started.
        </div>
      )}

      {/* Job table */}
      {!isLoading && jobs && jobs.length > 0 && (
        <table className="etl-job-table">
          <thead>
            <tr>
              <th>Plugin Name</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((job) => (
              <tr
                key={job.job_id}
                className="etl-job-row"
                onClick={() => onSelectJob(job.job_id)}
              >
                <td>{job.plugin_name}</td>
                <td>
                  <span className={`status-badge ${getStatusBadgeClass(job.status)}`}>
                    {job.status}
                  </span>
                </td>
                <td>
                  <button
                    className="btn-view"
                    onClick={(e) => {
                      e.stopPropagation();
                      onSelectJob(job.job_id);
                    }}
                    type="button"
                  >
                    View
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

export default EtlJobList;
