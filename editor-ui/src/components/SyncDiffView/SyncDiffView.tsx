/**
 * SyncDiffView — displays added/modified/removed entries from a plugin diff.
 */

import type { SyncDiff, CatalogEntry } from '../../types';
import './SyncDiffView.css';

interface SyncDiffViewProps {
  diff: SyncDiff;
  onClose?: () => void;
}

function EntryRow({ entry }: { entry: CatalogEntry }) {
  return (
    <div className="diff-entry-row">
      <span className="diff-entry-name">{entry.entity_name}</span>
      <code className="diff-entry-version">{entry.ocsf_version}</code>
    </div>
  );
}

function DiffSection({
  label,
  entries,
  variant,
}: {
  label: string;
  entries: CatalogEntry[];
  variant: 'added' | 'modified' | 'removed';
}) {
  if (entries.length === 0) return null;
  return (
    <div className={`diff-section diff-${variant}`}>
      <div className="diff-section-header">
        <span className="diff-section-label">{label}</span>
        <span className="diff-section-count">{entries.length}</span>
      </div>
      <div className="diff-section-entries">
        {entries.map((e, i) => <EntryRow key={i} entry={e} />)}
      </div>
    </div>
  );
}

export function SyncDiffView({ diff, onClose }: SyncDiffViewProps) {
  const total = diff.added.length + diff.modified.length + diff.removed.length;

  return (
    <div className="sync-diff-view">
      <div className="sync-diff-header">
        <span className="sync-diff-title">
          Diff — <code>{diff.engine}</code>
          <span className="sync-diff-total"> ({total} change{total !== 1 ? 's' : ''})</span>
        </span>
        {onClose && (
          <button className="sync-diff-close" onClick={onClose} aria-label="Close">×</button>
        )}
      </div>

      {total === 0 ? (
        <div className="sync-diff-empty">No differences — catalog is in sync.</div>
      ) : (
        <div className="sync-diff-sections">
          <DiffSection label="Added" entries={diff.added} variant="added" />
          <DiffSection label="Modified" entries={diff.modified} variant="modified" />
          <DiffSection label="Removed" entries={diff.removed} variant="removed" />
        </div>
      )}
    </div>
  );
}

export default SyncDiffView;
