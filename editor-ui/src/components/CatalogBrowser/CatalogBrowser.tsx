/**
 * CatalogBrowser — browse, filter, and delete catalog entries.
 */

import { useState } from 'react';
import {
  useCatalogEntries,
  useCatalogVersions,
  useDeleteCatalogEntry,
} from '../../api/catalogHooks';
import type { CatalogEntry } from '../../types';
import './CatalogBrowser.css';

interface CatalogBrowserProps {
  onSelect?: (entry: CatalogEntry) => void;
  onEdit?: (entry: CatalogEntry) => void;
}

export function CatalogBrowser({ onSelect, onEdit }: CatalogBrowserProps) {
  const [versionFilter, setVersionFilter] = useState<string>('');
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null);

  const { data: versions = [] } = useCatalogVersions();
  const { data: entries = [], isLoading, error } = useCatalogEntries(versionFilter || undefined);
  const deleteMutation = useDeleteCatalogEntry();

  const entryId = (e: CatalogEntry): number | null =>
    e.id != null ? (typeof e.id === 'number' ? e.id : null) : null;

  const handleDelete = (id: number) => {
    deleteMutation.mutate(id, { onSuccess: () => setConfirmDeleteId(null) });
  };

  if (isLoading) return <div className="catalog-loading">Loading catalog…</div>;
  if (error) return <div className="catalog-error">Failed to load catalog entries.</div>;

  return (
    <div className="catalog-browser">
      <div className="catalog-browser-toolbar">
        <label htmlFor="version-filter">OCSF Version</label>
        <select
          id="version-filter"
          value={versionFilter}
          onChange={(e) => setVersionFilter(e.target.value)}
        >
          <option value="">All versions</option>
          {versions.map((v) => (
            <option key={v} value={v}>{v}</option>
          ))}
        </select>
        <span className="catalog-count">{entries.length} entries</span>
      </div>

      {entries.length === 0 ? (
        <div className="catalog-empty">No catalog entries found.</div>
      ) : (
        <table className="catalog-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Version</th>
              <th>Event Classes</th>
              <th>Attributes</th>
              <th>Coverage</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((entry) => {
              const id = entryId(entry);
              return (
                <tr
                  key={id ?? entry.entity_name}
                  className="catalog-row"
                  onClick={() => onSelect?.(entry)}
                >
                  <td>
                    <span className="entry-name">{entry.entity_name}</span>
                    {entry.caption && (
                      <span className="entry-caption">{entry.caption}</span>
                    )}
                  </td>
                  <td><code>{entry.ocsf_version}</code></td>
                  <td>{entry.source_event_classes.length}</td>
                  <td>{entry.attributes.length}</td>
                  <td>
                    {entry.detection_coverage
                      ? `${entry.detection_coverage.mitre_techniques.length} techniques`
                      : '—'}
                  </td>
                  <td className="catalog-actions" onClick={(e) => e.stopPropagation()}>
                    {onEdit && (
                      <button
                        className="btn-icon"
                        title="Edit"
                        onClick={() => onEdit(entry)}
                      >✏️</button>
                    )}
                    {id != null && (
                      <button
                        className="btn-icon danger"
                        title="Delete"
                        onClick={() => setConfirmDeleteId(id)}
                      >🗑️</button>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      {confirmDeleteId != null && (
        <div className="catalog-confirm-overlay">
          <div className="catalog-confirm-dialog">
            <p>Delete this catalog entry?</p>
            <div className="confirm-actions">
              <button
                className="btn danger"
                onClick={() => handleDelete(confirmDeleteId)}
                disabled={deleteMutation.isPending}
              >
                {deleteMutation.isPending ? 'Deleting…' : 'Delete'}
              </button>
              <button className="btn secondary" onClick={() => setConfirmDeleteId(null)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default CatalogBrowser;
