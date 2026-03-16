/**
 * EtlArtifactViewer — Read-only Monaco-based code viewer for ETL job artifacts.
 *
 * Lists artifact file paths and displays selected file content with
 * syntax highlighting auto-detected from file extension.
 *
 * Requirements: 9.1, 9.2, 9.3, 9.4
 */

import { useState } from 'react';
import Editor from '@monaco-editor/react';
import { getMonacoLanguage } from '../../utils/etlHelpers';
import './EtlArtifactViewer.css';

interface EtlArtifactViewerProps {
  artifactFiles: string[];
  jobId: string;
}

const ETL_BASE_URL = 'http://localhost:3030';

export function EtlArtifactViewer({ artifactFiles, jobId }: EtlArtifactViewerProps) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileClick = async (filePath: string) => {
    setSelectedFile(filePath);
    setLoading(true);
    setError(null);
    setFileContent('');

    try {
      const url = `${ETL_BASE_URL}/api/jobs/${jobId}/artifacts/${encodeURIComponent(filePath)}`;
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch artifact: ${response.statusText}`);
      }
      const text = await response.text();
      setFileContent(text);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load artifact');
    } finally {
      setLoading(false);
    }
  };

  if (artifactFiles.length === 0) {
    return (
      <div className="etl-artifact-viewer">
        <h4>Artifacts</h4>
        <div className="etl-artifact-empty">
          Artifacts will appear after the Generate step
        </div>
      </div>
    );
  }

  return (
    <div className="etl-artifact-viewer">
      <h4>Artifacts</h4>
      <div className="etl-artifact-layout">
        <ul className="etl-artifact-list">
          {artifactFiles.map((file) => (
            <li
              key={file}
              className={`etl-artifact-item${selectedFile === file ? ' selected' : ''}`}
              onClick={() => handleFileClick(file)}
            >
              {file}
            </li>
          ))}
        </ul>

        <div className="etl-artifact-content">
          {loading && (
            <div className="etl-artifact-loading">
              <span className="spinner" /> Loading…
            </div>
          )}

          {error && <div className="etl-artifact-error">{error}</div>}

          {!loading && !error && selectedFile && (
            <Editor
              height="400px"
              language={getMonacoLanguage(selectedFile)}
              value={fileContent}
              options={{ readOnly: true, minimap: { enabled: false } }}
              theme="vs-dark"
            />
          )}

          {!loading && !error && !selectedFile && (
            <div className="etl-artifact-placeholder">
              Select a file to view its contents
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default EtlArtifactViewer;
