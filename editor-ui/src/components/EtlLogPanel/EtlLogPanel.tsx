/**
 * EtlLogPanel — SSE-driven terminal log display for ETL jobs.
 *
 * Connects to the ETL Engine SSE endpoint to stream real-time log lines.
 * Renders in a dark, monospace, auto-scrolling container.
 *
 * Requirements: 10.1, 10.2, 10.3, 10.4, 10.5
 */

import { useEffect, useRef, useState } from 'react';
import './EtlLogPanel.css';

const ETL_BASE_URL = 'http://localhost:3030';

interface EtlLogPanelProps {
  jobId: string;
  jobStatus: string;
}

function isTerminalStatus(status: string): boolean {
  return status === 'Completed' || status.startsWith('Failed');
}

export function EtlLogPanel({ jobId, jobStatus }: EtlLogPanelProps) {
  const [logLines, setLogLines] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Manage EventSource lifecycle
  useEffect(() => {
    if (jobStatus === 'Pending' || isTerminalStatus(jobStatus)) {
      return;
    }

    const url = `${ETL_BASE_URL}/api/jobs/${jobId}/logs`;
    const eventSource = new EventSource(url);

    eventSource.onmessage = (event) => {
      setLogLines((prev) => [...prev, event.data]);
      setError(null);
    };

    eventSource.onerror = () => {
      setError('Connection lost — retrying...');
    };

    return () => {
      eventSource.close();
    };
  }, [jobId, jobStatus]);

  // Auto-scroll to bottom on new log lines
  useEffect(() => {
    const container = logContainerRef.current;
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  }, [logLines.length]);

  const handleClear = () => {
    setLogLines([]);
  };

  return (
    <div className="etl-log-panel">
      <div className="etl-log-header">
        <h4>Logs</h4>
        <button className="clear-btn" onClick={handleClear} type="button">
          Clear
        </button>
      </div>

      {error && <div className="etl-log-error">{error}</div>}

      <div className="etl-log-container" ref={logContainerRef}>
        {jobStatus === 'Pending' ? (
          <div className="etl-log-placeholder">
            No logs available yet — job is pending.
          </div>
        ) : logLines.length === 0 ? (
          <div className="etl-log-placeholder">Waiting for log output…</div>
        ) : (
          logLines.map((line, i) => (
            <pre key={i} className="etl-log-line">{line}</pre>
          ))
        )}
      </div>
    </div>
  );
}

export default EtlLogPanel;
