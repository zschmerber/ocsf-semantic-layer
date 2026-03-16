/**
 * CatalogEntryForm — create or edit a CatalogEntry.
 */

import { useState } from 'react';
import { useCreateCatalogEntry, useUpdateCatalogEntry } from '../../api/catalogHooks';
import type { CatalogEntry } from '../../types';
import './CatalogEntryForm.css';

interface CatalogEntryFormProps {
  initial?: CatalogEntry;
  onSuccess?: (entry: CatalogEntry) => void;
  onCancel?: () => void;
}

function emptyEntry(): CatalogEntry {
  return {
    entity_name: '',
    caption: '',
    description: '',
    ocsf_version: '1.3.0',
    source_event_classes: [],
    attributes: [],
    relationships: [],
    covers_observables: [],
    source_lineage: [],
    field_lineage: [],
    catalog_version: 0,
    updated_at: new Date().toISOString(),
  };
}

export function CatalogEntryForm({ initial, onSuccess, onCancel }: CatalogEntryFormProps) {
  const [form, setForm] = useState<CatalogEntry>(initial ?? emptyEntry());
  const [classInput, setClassInput] = useState('');
  const [error, setError] = useState<string | null>(null);

  const createMutation = useCreateCatalogEntry();
  const updateMutation = useUpdateCatalogEntry();
  const isPending = createMutation.isPending || updateMutation.isPending;

  const set = (field: keyof CatalogEntry, value: unknown) =>
    setForm((f) => ({ ...f, [field]: value }));

  const addEventClass = () => {
    const uid = parseInt(classInput.trim(), 10);
    if (!isNaN(uid) && !form.source_event_classes.includes(uid)) {
      set('source_event_classes', [...form.source_event_classes, uid]);
    }
    setClassInput('');
  };

  const removeEventClass = (uid: number) =>
    set('source_event_classes', form.source_event_classes.filter((c) => c !== uid));

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!form.entity_name.trim()) { setError('Entity name is required.'); return; }
    if (!form.ocsf_version.trim()) { setError('OCSF version is required.'); return; }

    const entryId = typeof form.id === 'number' ? form.id : null;

    if (entryId != null) {
      updateMutation.mutate(
        { id: entryId, entry: form },
        { onSuccess: (updated) => onSuccess?.(updated), onError: (e) => setError(e.message) }
      );
    } else {
      createMutation.mutate(form, {
        onSuccess: (res) => onSuccess?.(res.entry),
        onError: (e) => setError(e.message),
      });
    }
  };

  return (
    <form className="catalog-entry-form" onSubmit={handleSubmit}>
      <div className="form-row">
        <label>Entity Name *</label>
        <input
          value={form.entity_name}
          onChange={(e) => set('entity_name', e.target.value)}
          placeholder="e.g. dns_event"
          required
        />
      </div>
      <div className="form-row">
        <label>Caption</label>
        <input
          value={form.caption}
          onChange={(e) => set('caption', e.target.value)}
          placeholder="Human-readable name"
        />
      </div>
      <div className="form-row">
        <label>Description</label>
        <textarea
          value={form.description}
          onChange={(e) => set('description', e.target.value)}
          rows={3}
          placeholder="Describe this entity…"
        />
      </div>
      <div className="form-row">
        <label>OCSF Version *</label>
        <input
          value={form.ocsf_version}
          onChange={(e) => set('ocsf_version', e.target.value)}
          placeholder="e.g. 1.3.0"
          required
        />
      </div>
      <div className="form-row">
        <label>Source Event Classes</label>
        <div className="tag-input">
          {form.source_event_classes.map((uid) => (
            <span key={uid} className="tag">
              {uid}
              <button type="button" onClick={() => removeEventClass(uid)}>×</button>
            </span>
          ))}
          <input
            value={classInput}
            onChange={(e) => setClassInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), addEventClass())}
            placeholder="Add class UID…"
          />
          <button type="button" className="btn-add" onClick={addEventClass}>+</button>
        </div>
      </div>

      {error && <div className="form-error">{error}</div>}

      <div className="form-actions">
        <button type="submit" className="btn primary" disabled={isPending}>
          {isPending ? 'Saving…' : (form.id != null ? 'Update' : 'Create')}
        </button>
        {onCancel && (
          <button type="button" className="btn secondary" onClick={onCancel}>
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}

export default CatalogEntryForm;
