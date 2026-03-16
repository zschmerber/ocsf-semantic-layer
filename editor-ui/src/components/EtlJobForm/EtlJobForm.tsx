/**
 * EtlJobForm — Job creation form with source config selector and mapping builder.
 *
 * Requirements: 5.1–5.11, 6.1–6.5
 */

import { useState } from 'react';
import { useCreateEtlJob } from '../../api/etlClient';
import type {
  SourceConfig,
  SourceConfigType,
  FieldMapping,
  TransformType,
  CastType,
  EtlJob,
} from '../../types/etl';
import { generateJobId, validateJobForm } from '../../utils/etlHelpers';
import './EtlJobForm.css';

interface EtlJobFormProps {
  onCreated: (jobId: string) => void;
  onCancel: () => void;
}

const SOURCE_TYPES: SourceConfigType[] = ['File', 'Tcp', 'Sqs', 'Kafka'];
const TRANSFORM_OPTIONS = ['None', 'Lower', 'Upper', 'Trim', 'Cast'] as const;
const CAST_TYPES: CastType[] = [
  'Integer', 'BigInt', 'Float', 'Double', 'Boolean', 'String', 'Timestamp', 'Date',
];

type UiTransform = typeof TRANSFORM_OPTIONS[number];

function buildTransformType(ui: UiTransform, castType: CastType): TransformType {
  switch (ui) {
    case 'None': return null;
    case 'Lower': return 'Lower';
    case 'Upper': return 'Upper';
    case 'Trim': return 'Trim';
    case 'Cast': return { Cast: castType };
  }
}

function buildSourceConfig(
  sourceType: SourceConfigType,
  filePath: string,
  bindAddress: string,
  queueUrl: string,
  sqsRegion: string,
  brokers: string,
  topic: string,
): SourceConfig {
  switch (sourceType) {
    case 'File': return { type: 'File', path: filePath };
    case 'Tcp': return { type: 'Tcp', bind_address: bindAddress };
    case 'Sqs': return { type: 'Sqs', queue_url: queueUrl, region: sqsRegion };
    case 'Kafka': return {
      type: 'Kafka',
      brokers: brokers.split(',').map((b) => b.trim()).filter(Boolean),
      topic,
    };
  }
}

interface MappingRow {
  source_field: string;
  target_ocsf_path: string;
  uiTransform: UiTransform;
  castType: CastType;
  confidence: number;
}

function newMappingRow(): MappingRow {
  return {
    source_field: '',
    target_ocsf_path: '',
    uiTransform: 'None',
    castType: 'String',
    confidence: 0.95,
  };
}

function toFieldMapping(row: MappingRow): FieldMapping {
  return {
    source_field: row.source_field,
    target_ocsf_path: row.target_ocsf_path,
    transformation: buildTransformType(row.uiTransform, row.castType),
    confidence: row.confidence,
  };
}

export function EtlJobForm({ onCreated, onCancel }: EtlJobFormProps) {
  // Core fields
  const [pluginName, setPluginName] = useState('');
  const [ocsfClassUid, setOcsfClassUid] = useState<number>(0);
  const [ocsfVersion, setOcsfVersion] = useState('1.3.0');
  const [deltaTableUri, setDeltaTableUri] = useState('');

  // Source config
  const [sourceType, setSourceType] = useState<SourceConfigType>('File');
  const [filePath, setFilePath] = useState('');
  const [bindAddress, setBindAddress] = useState('');
  const [queueUrl, setQueueUrl] = useState('');
  const [sqsRegion, setSqsRegion] = useState('');
  const [brokers, setBrokers] = useState('');
  const [topic, setTopic] = useState('');

  // Mappings
  const [mappings, setMappings] = useState<MappingRow[]>([]);

  // UI state
  const [error, setError] = useState<string | null>(null);
  const createEtlJob = useCreateEtlJob();

  const addMapping = () => setMappings((prev) => [...prev, newMappingRow()]);

  const removeMapping = (index: number) =>
    setMappings((prev) => prev.filter((_, i) => i !== index));

  const updateMapping = (index: number, field: keyof MappingRow, value: unknown) =>
    setMappings((prev) => prev.map((m, i) => (i === index ? { ...m, [field]: value } : m)));

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const sourceConfig = buildSourceConfig(
      sourceType, filePath, bindAddress, queueUrl, sqsRegion, brokers, topic,
    );
    const fieldMappings = mappings.map(toFieldMapping);

    const formData = {
      plugin_name: pluginName,
      mappings: fieldMappings,
      source_config: sourceConfig,
      delta_table_uri: deltaTableUri,
    };

    if (!validateJobForm(formData)) {
      setError('Please fill in all required fields: plugin name, at least one mapping, source configuration, and Delta Lake URI.');
      return;
    }

    const jobId = generateJobId();
    const job: EtlJob = {
      job_id: jobId,
      plugin_name: pluginName,
      ocsf_class_uid: ocsfClassUid,
      ocsf_version: ocsfVersion,
      mappings: fieldMappings,
      source_config: sourceConfig,
      delta_table_uri: deltaTableUri,
      s3_config: null,
      catalog_ref: null,
    };

    createEtlJob.mutate(job, {
      onSuccess: () => onCreated(jobId),
      onError: (err) => setError(err.message),
    });
  };

  return (
    <form className="etl-job-form" onSubmit={handleSubmit}>
      <h3>Create ETL Job</h3>

      {/* Core Fields */}
      <div className="form-row">
        <label>Plugin Name *</label>
        <input
          type="text"
          value={pluginName}
          onChange={(e) => setPluginName(e.target.value)}
          placeholder="e.g. cloudtrail_parser"
        />
      </div>
      <div className="form-row">
        <label>OCSF Class UID *</label>
        <input
          type="number"
          value={ocsfClassUid}
          onChange={(e) => setOcsfClassUid(parseInt(e.target.value, 10) || 0)}
          placeholder="e.g. 4001"
        />
      </div>
      <div className="form-row">
        <label>OCSF Version</label>
        <input
          type="text"
          value={ocsfVersion}
          onChange={(e) => setOcsfVersion(e.target.value)}
          placeholder="1.3.0"
        />
      </div>
      <div className="form-row">
        <label>Delta Lake URI *</label>
        <input
          type="text"
          value={deltaTableUri}
          onChange={(e) => setDeltaTableUri(e.target.value)}
          placeholder="s3://bucket/path/to/table"
        />
      </div>

      {/* Source Config Selector */}
      <fieldset className="source-config-section">
        <legend>Source Configuration</legend>
        <div className="form-row">
          <label>Source Type</label>
          <select
            value={sourceType}
            onChange={(e) => setSourceType(e.target.value as SourceConfigType)}
          >
            {SOURCE_TYPES.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </div>

        {sourceType === 'File' && (
          <div className="form-row">
            <label>File Path</label>
            <input
              type="text"
              value={filePath}
              onChange={(e) => setFilePath(e.target.value)}
              placeholder="/var/log/source.json"
            />
          </div>
        )}
        {sourceType === 'Tcp' && (
          <div className="form-row">
            <label>Bind Address</label>
            <input
              type="text"
              value={bindAddress}
              onChange={(e) => setBindAddress(e.target.value)}
              placeholder="0.0.0.0:5140"
            />
          </div>
        )}
        {sourceType === 'Sqs' && (
          <>
            <div className="form-row">
              <label>Queue URL</label>
              <input
                type="text"
                value={queueUrl}
                onChange={(e) => setQueueUrl(e.target.value)}
                placeholder="https://sqs.us-east-1.amazonaws.com/..."
              />
            </div>
            <div className="form-row">
              <label>Region</label>
              <input
                type="text"
                value={sqsRegion}
                onChange={(e) => setSqsRegion(e.target.value)}
                placeholder="us-east-1"
              />
            </div>
          </>
        )}
        {sourceType === 'Kafka' && (
          <>
            <div className="form-row">
              <label>Brokers (comma-separated)</label>
              <input
                type="text"
                value={brokers}
                onChange={(e) => setBrokers(e.target.value)}
                placeholder="broker1:9092, broker2:9092"
              />
            </div>
            <div className="form-row">
              <label>Topic</label>
              <input
                type="text"
                value={topic}
                onChange={(e) => setTopic(e.target.value)}
                placeholder="security-events"
              />
            </div>
          </>
        )}
      </fieldset>

      {/* Mapping Builder */}
      <fieldset className="mapping-section">
        <legend>Field Mappings *</legend>
        {mappings.length === 0 && (
          <p className="mapping-empty">No mappings yet. Add at least one mapping.</p>
        )}
        {mappings.map((row, i) => (
          <div key={i} className="mapping-row">
            <input
              type="text"
              value={row.source_field}
              onChange={(e) => updateMapping(i, 'source_field', e.target.value)}
              placeholder="Source field"
              className="mapping-input"
            />
            <input
              type="text"
              value={row.target_ocsf_path}
              onChange={(e) => updateMapping(i, 'target_ocsf_path', e.target.value)}
              placeholder="Target OCSF path"
              className="mapping-input"
            />
            <div className="mapping-confidence">
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={row.confidence}
                onChange={(e) => updateMapping(i, 'confidence', parseFloat(e.target.value))}
              />
              <span className="confidence-value">{row.confidence.toFixed(2)}</span>
            </div>
            <select
              value={row.uiTransform}
              onChange={(e) => updateMapping(i, 'uiTransform', e.target.value)}
              className="mapping-select"
            >
              {TRANSFORM_OPTIONS.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
            {row.uiTransform === 'Cast' && (
              <select
                value={row.castType}
                onChange={(e) => updateMapping(i, 'castType', e.target.value)}
                className="mapping-select"
              >
                {CAST_TYPES.map((ct) => (
                  <option key={ct} value={ct}>{ct}</option>
                ))}
              </select>
            )}
            <button
              type="button"
              className="btn-remove"
              onClick={() => removeMapping(i)}
              aria-label={`Remove mapping ${i + 1}`}
            >
              ×
            </button>
          </div>
        ))}
        <button type="button" className="btn secondary" onClick={addMapping}>
          Add Mapping
        </button>
      </fieldset>

      {/* Error display */}
      {error && <div className="form-error">{error}</div>}

      {/* Form actions */}
      <div className="form-actions">
        <button type="submit" className="btn primary" disabled={createEtlJob.isPending}>
          {createEtlJob.isPending ? 'Creating…' : 'Create Job'}
        </button>
        <button type="button" className="btn secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </form>
  );
}
