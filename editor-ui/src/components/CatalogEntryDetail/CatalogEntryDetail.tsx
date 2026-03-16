/**
 * CatalogEntryDetail — read-only detail view for a CatalogEntry.
 */

import type { CatalogEntry } from '../../types';
import './CatalogEntryDetail.css';

interface CatalogEntryDetailProps {
  entry: CatalogEntry;
  onClose?: () => void;
}

export function CatalogEntryDetail({ entry, onClose }: CatalogEntryDetailProps) {
  return (
    <div className="catalog-detail">
      <div className="catalog-detail-header">
        <div>
          <h3 className="catalog-detail-name">{entry.entity_name}</h3>
          {entry.caption && <p className="catalog-detail-caption">{entry.caption}</p>}
        </div>
        {onClose && (
          <button className="catalog-detail-close" onClick={onClose} aria-label="Close">×</button>
        )}
      </div>

      {entry.description && (
        <p className="catalog-detail-description">{entry.description}</p>
      )}

      <div className="catalog-detail-meta">
        <span className="meta-badge">v{entry.ocsf_version}</span>
        <span className="meta-badge">catalog v{entry.catalog_version}</span>
        <span className="meta-badge muted">
          Updated {new Date(entry.updated_at).toLocaleDateString()}
        </span>
      </div>

      <section className="catalog-detail-section">
        <h4>Source Event Classes</h4>
        {entry.source_event_classes.length === 0 ? (
          <span className="muted">None</span>
        ) : (
          <div className="tag-list">
            {entry.source_event_classes.map((uid) => (
              <code key={uid} className="tag-code">{uid}</code>
            ))}
          </div>
        )}
      </section>

      <section className="catalog-detail-section">
        <h4>Attributes <span className="count">({entry.attributes.length})</span></h4>
        {entry.attributes.length === 0 ? (
          <span className="muted">None defined</span>
        ) : (
          <pre className="catalog-detail-json">
            {JSON.stringify(entry.attributes, null, 2)}
          </pre>
        )}
      </section>

      {entry.detection_coverage && (
        <section className="catalog-detail-section">
          <h4>Detection Coverage</h4>
          <div className="coverage-grid">
            <div>
              <span className="coverage-label">MITRE Techniques</span>
              <div className="tag-list">
                {entry.detection_coverage.mitre_techniques.length === 0
                  ? <span className="muted">None</span>
                  : entry.detection_coverage.mitre_techniques.map((t) => (
                    <code key={t} className="tag-code">{t}</code>
                  ))}
              </div>
            </div>
            <div>
              <span className="coverage-label">MITRE Tactics</span>
              <div className="tag-list">
                {entry.detection_coverage.mitre_tactics.length === 0
                  ? <span className="muted">None</span>
                  : entry.detection_coverage.mitre_tactics.map((t) => (
                    <code key={t} className="tag-code">{t}</code>
                  ))}
              </div>
            </div>
            <div>
              <span className="coverage-label">Data Sources</span>
              <div className="tag-list">
                {entry.detection_coverage.data_sources.length === 0
                  ? <span className="muted">None</span>
                  : entry.detection_coverage.data_sources.map((s) => (
                    <span key={s} className="tag-code">{s}</span>
                  ))}
              </div>
            </div>
          </div>
        </section>
      )}

      {entry.field_lineage.length > 0 && (
        <section className="catalog-detail-section">
          <h4>Field Lineage <span className="count">({entry.field_lineage.length})</span></h4>
          <table className="lineage-table">
            <thead>
              <tr><th>Source Field</th><th>OCSF Field</th><th>Transformation</th></tr>
            </thead>
            <tbody>
              {entry.field_lineage.map((fl, i) => (
                <tr key={i}>
                  <td><code>{fl.source_field}</code></td>
                  <td><code>{fl.target_ocsf_field}</code></td>
                  <td>{fl.transformation ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}

      {entry.source_lineage.length > 0 && (
        <section className="catalog-detail-section">
          <h4>Source Lineage</h4>
          {entry.source_lineage.map((sl, i) => (
            <div key={i} className="lineage-record">
              <code>{sl.source_system}</code> / <code>{sl.source_table}</code>
              <span className="muted"> — {new Date(sl.ingestion_timestamp).toLocaleDateString()}</span>
            </div>
          ))}
        </section>
      )}
    </div>
  );
}

export default CatalogEntryDetail;
