/**
 * DatasetPanel component for CRUD management of Dataset entries.
 *
 * Requirements: 8.6
 */

import { useState, useCallback } from 'react';
import type { Dataset } from '../../types';
import './DatasetPanel.css';

// ============================================
// Types
// ============================================

export interface DatasetPanelProps {
  datasets: Dataset[];
  onAdd: (dataset: Dataset) => void;
  onUpdate: (name: string, dataset: Dataset) => void;
  onRemove: (name: string) => void;
}

interface FormState {
  name: string;
  dialect: string;
  table: string;
  schema_name: string;
  connection: string;
}

const EMPTY_FORM: FormState = {
  name: '',
  dialect: '',
  table: '',
  schema_name: '',
  connection: '',
};

// ============================================
// Component
// ============================================

export function DatasetPanel({ datasets, onAdd, onUpdate, onRemove }: DatasetPanelProps) {
  const [showForm, setShowForm] = useState(false);
  const [editingName, setEditingName] = useState<string | null>(null);
  const [form, setForm] = useState<FormState>(EMPTY_FORM);

  const resetForm = useCallback(() => {
    setForm(EMPTY_FORM);
    setShowForm(false);
    setEditingName(null);
  }, []);

  const handleAdd = useCallback(() => {
    setForm(EMPTY_FORM);
    setEditingName(null);
    setShowForm(true);
  }, []);

  const handleEdit = useCallback((ds: Dataset) => {
    setForm({
      name: ds.name,
      dialect: ds.dialect,
      table: ds.table,
      schema_name: ds.schema_name ?? '',
      connection: ds.connection ?? '',
    });
    setEditingName(ds.name);
    setShowForm(true);
  }, []);

  const handleSubmit = useCallback(() => {
    if (!form.name || !form.dialect || !form.table) return;

    const dataset: Dataset = {
      name: form.name,
      dialect: form.dialect,
      table: form.table,
      ...(form.schema_name ? { schema_name: form.schema_name } : {}),
      ...(form.connection ? { connection: form.connection } : {}),
    };

    if (editingName) {
      onUpdate(editingName, dataset);
    } else {
      onAdd(dataset);
    }
    resetForm();
  }, [form, editingName, onAdd, onUpdate, resetForm]);

  const handleFieldChange = useCallback(
    (field: keyof FormState) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({ ...prev, [field]: e.target.value }));
    },
    []
  );

  const isValid = form.name.trim() !== '' && form.dialect.trim() !== '' && form.table.trim() !== '';

  return (
    <div className="dataset-panel">
      <div className="dataset-panel-header">
        <span className="dataset-panel-title">Datasets</span>
        {!showForm && (
          <button className="btn primary" onClick={handleAdd}>
            Add Dataset
          </button>
        )}
      </div>

      <div className="dataset-panel-body">
        {datasets.length === 0 && !showForm && (
          <div className="dataset-list-empty">No datasets configured</div>
        )}

        {datasets.length > 0 && (
          <div className="dataset-list">
            {datasets.map((ds) => (
              <div key={ds.name} className="dataset-item">
                <div className="dataset-item-info">
                  <div className="dataset-item-name">{ds.name}</div>
                  <div className="dataset-item-meta">
                    <span>dialect: <code>{ds.dialect}</code></span>
                    <span>table: <code>{ds.table}</code></span>
                    {ds.schema_name && <span>schema: <code>{ds.schema_name}</code></span>}
                    {ds.connection && <span>connection: <code>{ds.connection}</code></span>}
                  </div>
                </div>
                <div className="dataset-item-actions">
                  <button
                    className="btn-icon"
                    onClick={() => handleEdit(ds)}
                    title="Edit dataset"
                  >
                    ✏️
                  </button>
                  <button
                    className="btn-icon danger"
                    onClick={() => onRemove(ds.name)}
                    title="Remove dataset"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        {showForm && (
          <div className="dataset-form">
            <div className="dataset-form-title">
              {editingName ? 'Edit Dataset' : 'New Dataset'}
            </div>

            <div className="form-group">
              <label className="form-label" htmlFor="ds-name">
                Name <span className="required">*</span>
              </label>
              <input
                id="ds-name"
                type="text"
                className="form-input"
                value={form.name}
                onChange={handleFieldChange('name')}
                placeholder="e.g., prod_warehouse"
              />
            </div>

            <div className="form-row">
              <div className="form-group half">
                <label className="form-label" htmlFor="ds-dialect">
                  Dialect <span className="required">*</span>
                </label>
                <input
                  id="ds-dialect"
                  type="text"
                  className="form-input"
                  value={form.dialect}
                  onChange={handleFieldChange('dialect')}
                  placeholder="e.g., snowflake"
                />
              </div>
              <div className="form-group half">
                <label className="form-label" htmlFor="ds-table">
                  Table <span className="required">*</span>
                </label>
                <input
                  id="ds-table"
                  type="text"
                  className="form-input"
                  value={form.table}
                  onChange={handleFieldChange('table')}
                  placeholder="e.g., ocsf_events"
                />
              </div>
            </div>

            <div className="form-row">
              <div className="form-group half">
                <label className="form-label" htmlFor="ds-schema">
                  Schema Name
                </label>
                <input
                  id="ds-schema"
                  type="text"
                  className="form-input"
                  value={form.schema_name}
                  onChange={handleFieldChange('schema_name')}
                  placeholder="e.g., security"
                />
              </div>
              <div className="form-group half">
                <label className="form-label" htmlFor="ds-connection">
                  Connection
                </label>
                <input
                  id="ds-connection"
                  type="text"
                  className="form-input"
                  value={form.connection}
                  onChange={handleFieldChange('connection')}
                  placeholder="e.g., snowflake://prod"
                />
              </div>
            </div>

            <div className="dataset-form-actions">
              <button className="btn" onClick={resetForm}>
                Cancel
              </button>
              <button
                className="btn primary"
                onClick={handleSubmit}
                disabled={!isValid}
              >
                {editingName ? 'Update' : 'Add'}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
