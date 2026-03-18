/**
 * EventPastePanel — primary entry point for the data-first workflow.
 *
 * Accepts pasted OCSF JSON events (or file uploads) and an optional
 * mapping artifact. Displays a summary of parsed fields, detected class,
 * observables, type mismatches, and mapping interpretation results.
 *
 * Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.3, 2.4, 2.5, 2.6,
 *   4.3, 12.1, 13.1, 13.3, 14.1, 14.4, 14.5, 15.1, 15.3, 15.4, 15.5,
 *   15.6, 15.7, 15.9, 15.11, 15.13, 16.5, 16.7, 17.2
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { useReferenceEventStore } from '../../store/referenceEventStore';
import { useEditorStore } from '../../store/editorStore';
import { extractFields } from '../../utils/eventFieldExtractor';
import { getLLMConfig, interpretMapping } from '../../api/client';
import type { InterpretedMapping } from '../../types/referenceEvent';
import './EventPastePanel.css';

// ============================================
// Props
// ============================================

export interface EventPastePanelProps {
  onSkip?: () => void;
}

// ============================================
// Component
// ============================================

export function EventPastePanel({ onSkip }: EventPastePanelProps) {
  // Store state
  const {
    rawJson,
    parsedFields,
    classDetection,
    observableFlags,
    typeMismatches,
    mappingRawText,
    interpretedMapping,
    mappingLoading,
    mappingError,
    mappingCoverage,
    setReferenceEvent,
    clearReferenceEvent,
    setMappingRawText,
    setInterpretedMapping,
    setMappingLoading,
    setMappingError,
  } = useReferenceEventStore();

  const schema = useEditorStore((s) => s.schema);

  // Local state
  const [eventText, setEventText] = useState(rawJson ?? '');
  const [parseError, setParseError] = useState<string | null>(null);
  const [parseWarning, setParseWarning] = useState<string | null>(null);
  const [mappingExpanded, setMappingExpanded] = useState(false);
  const [llmConfigured, setLlmConfigured] = useState<boolean | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Check LLM config on mount
  useEffect(() => {
    getLLMConfig()
      .then((config) => setLlmConfigured(config.configured))
      .catch(() => setLlmConfigured(false));
  }, []);

  // ============================================
  // Event Paste / Upload Handling
  // ============================================

  const processEventJson = useCallback(
    (json: string) => {
      setEventText(json);
      setParseError(null);
      setParseWarning(null);

      if (!json.trim()) {
        return;
      }

      // Pre-validate before calling store
      const { error, warning } = extractFields(json);
      if (error) {
        setParseError(error);
        return;
      }
      if (warning) {
        setParseWarning(warning);
      }

      if (!schema) {
        setParseError('Schema tree not loaded yet. Please wait and try again.');
        return;
      }

      setReferenceEvent(json, schema);
    },
    [schema, setReferenceEvent],
  );

  const handlePaste = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      processEventJson(e.target.value);
    },
    [processEventJson],
  );

  const handleFileUpload = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (ev) => {
        const content = ev.target?.result as string;
        processEventJson(content);
      };
      reader.onerror = () => {
        setParseError('Failed to read file');
      };
      reader.readAsText(file);

      // Reset input so the same file can be re-uploaded
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    },
    [processEventJson],
  );

  const handleClear = useCallback(() => {
    setEventText('');
    setParseError(null);
    setParseWarning(null);
    clearReferenceEvent();
  }, [clearReferenceEvent]);

  // ============================================
  // Mapping Handling
  // ============================================

  const handleMappingChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const text = e.target.value;
      setMappingRawText(text || null);
    },
    [setMappingRawText],
  );

  const handleMappingSubmit = useCallback(async () => {
    if (!mappingRawText || !rawJson) return;

    setMappingLoading(true);
    setMappingError(null);

    try {
      const result = await interpretMapping({ event_json: rawJson, mapping_text: mappingRawText });
      setInterpretedMapping(result);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to interpret mapping';
      setMappingError(message);
    } finally {
      setMappingLoading(false);
    }
  }, [mappingRawText, rawJson, setMappingLoading, setMappingError, setInterpretedMapping]);

  // Auto-submit mapping when event is loaded and mapping text exists
  useEffect(() => {
    if (rawJson && mappingRawText && !interpretedMapping && !mappingLoading && llmConfigured) {
      handleMappingSubmit();
    }
    // Only trigger when rawJson changes (event loaded) with existing mapping text
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rawJson]);

  const hasEvent = rawJson !== null && parsedFields.length > 0;

  // ============================================
  // Render
  // ============================================

  return (
    <div className="event-paste-panel">
      {/* Single-event MVP banner (Req 13.1) */}
      {hasEvent && (
        <div className="sample-event-banner" title="Multi-event support is planned for a future version">
          ℹ️ Working from a single sample event — field presence and types may vary across your full dataset
        </div>
      )}

      {/* Header */}
      <div className="event-paste-header">
        <h3>Paste OCSF Event</h3>
        <div className="event-paste-actions">
          <label className="file-upload-btn">
            <input
              ref={fileInputRef}
              type="file"
              accept=".json"
              onChange={handleFileUpload}
            />
            Upload JSON
          </label>
          {hasEvent && (
            <button className="clear-btn" onClick={handleClear} type="button">
              Clear
            </button>
          )}
        </div>
      </div>

      {/* Skip link (Req 1.6, 12.1) */}
      <p className="skip-hint">
        This step is optional.{' '}
        <button className="skip-link" onClick={onSkip} type="button">
          Skip — browse schema manually
        </button>
      </p>

      {/* Primary textarea */}
      <textarea
        className="event-input"
        placeholder={'Paste a transformed OCSF JSON event here, e.g.:\n{\n  "class_uid": 4001,\n  "category_uid": 4,\n  "activity_id": 1,\n  "src_endpoint": { "ip": "10.0.0.1" },\n  ...\n}'}
        value={eventText}
        onChange={handlePaste}
        rows={10}
        spellCheck={false}
      />

      {/* Parse error (Req 1.4, 14.5) */}
      {parseError && (
        <div className="parse-error">
          <span className="error-icon">⚠</span>
          {parseError}
        </div>
      )}

      {/* Parse warning (Req 14.1) */}
      {parseWarning && !parseError && (
        <div className="parse-warning">
          <span className="warning-icon">⚠</span>
          {parseWarning}
        </div>
      )}

      {/* Summary panel (Req 1.5) */}
      {hasEvent && classDetection && (
        <EventSummary
          fieldCount={parsedFields.length}
          classDetection={classDetection}
          observableCount={observableFlags.length}
          mismatchCount={typeMismatches.length}
          interpretedMapping={interpretedMapping}
          mappingCoverage={mappingCoverage}
        />
      )}

      {/* Mapping section (Req 15.1, 15.5, 15.11) */}
      <div className="mapping-section">
        <button
          className="mapping-toggle"
          onClick={() => setMappingExpanded(!mappingExpanded)}
          type="button"
        >
          {mappingExpanded ? '▾' : '▸'} Mapping (optional — any format)
        </button>

        {mappingExpanded && (
          <div className="mapping-content">
            {llmConfigured === false ? (
              <div className="mapping-disabled-message">
                Configure an LLM provider (Anthropic or OpenAI) in Settings to enable mapping interpretation.
              </div>
            ) : !hasEvent ? (
              <>
                <textarea
                  className="mapping-input"
                  placeholder="Paste an OCSF event first — the mapping will be interpreted together with the event"
                  value={mappingRawText ?? ''}
                  onChange={handleMappingChange}
                  rows={6}
                  spellCheck={false}
                />
                <p className="mapping-deferred-hint">
                  Paste an OCSF event first — the mapping will be interpreted together with the event.
                </p>
              </>
            ) : (
              <>
                <textarea
                  className="mapping-input"
                  placeholder="Paste your mapping artifact here (Logstash config, Cribl pack, Splunk transforms, Python script, dbt SQL, YAML, etc.)"
                  value={mappingRawText ?? ''}
                  onChange={handleMappingChange}
                  rows={6}
                  spellCheck={false}
                />
                {mappingRawText && !interpretedMapping && !mappingLoading && (
                  <button
                    className="btn primary mapping-submit-btn"
                    onClick={handleMappingSubmit}
                    type="button"
                  >
                    Interpret Mapping
                  </button>
                )}
              </>
            )}

            {/* Loading indicator (Req 15.4) */}
            {mappingLoading && (
              <div className="mapping-loading">
                <span className="spinner" /> Interpreting mapping via LLM…
              </div>
            )}

            {/* Mapping error with retry (Req 15.6) */}
            {mappingError && (
              <div className="mapping-error">
                <span className="error-icon">⚠</span>
                {mappingError}
                <button
                  className="retry-btn"
                  onClick={handleMappingSubmit}
                  type="button"
                >
                  Retry
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}


// ============================================
// EventSummary Sub-component
// ============================================

interface EventSummaryProps {
  fieldCount: number;
  classDetection: {
    classUid: number | null;
    className: string | null;
    categoryName: string | null;
    confidence: 'Definitive' | 'Low';
    error: string | null;
  };
  observableCount: number;
  mismatchCount: number;
  interpretedMapping: InterpretedMapping | null;
  mappingCoverage: import('../../types/referenceEvent').MappingCoverage | null;
}

function EventSummary({
  fieldCount,
  classDetection,
  observableCount,
  mismatchCount,
  interpretedMapping,
  mappingCoverage,
}: EventSummaryProps) {
  return (
    <div className="event-summary">
      <div className="summary-row">
        <div className="summary-item">
          <span className="summary-label">Fields</span>
          <span className="summary-value">{fieldCount}</span>
        </div>

        <div className="summary-item">
          <span className="summary-label">Event Class</span>
          <span className="summary-value">
            {classDetection.className ?? 'Unknown'}
            <span className={`confidence-badge confidence-${classDetection.confidence.toLowerCase()}`}>
              {classDetection.confidence}
            </span>
          </span>
          {classDetection.error && (
            <span className="summary-error">{classDetection.error}</span>
          )}
        </div>

        <div className="summary-item">
          <span className="summary-label">Observables</span>
          <span className="summary-value">{observableCount}</span>
        </div>

        {mismatchCount > 0 && (
          <div className="summary-item summary-warning">
            <span className="summary-label">Type Mismatches</span>
            <span className="summary-value">{mismatchCount}</span>
          </div>
        )}
      </div>

      {/* Mapping summary (Req 15.3, 15.13) */}
      {interpretedMapping && (
        <MappingSummary mapping={interpretedMapping} coverage={mappingCoverage} />
      )}
    </div>
  );
}

// ============================================
// MappingSummary Sub-component
// ============================================

interface MappingSummaryProps {
  mapping: InterpretedMapping;
  coverage: import('../../types/referenceEvent').MappingCoverage | null;
}

function MappingSummary({ mapping, coverage }: MappingSummaryProps) {
  const highCount = mapping.entries.filter((e) => e.confidence === 'High').length;
  const mediumCount = mapping.entries.filter((e) => e.confidence === 'Medium').length;
  const lowCount = mapping.entries.filter((e) => e.confidence === 'Low').length;

  const sourceLabel = [mapping.sourceSystem.vendor, mapping.sourceSystem.logType]
    .filter(Boolean)
    .join(' — ');

  return (
    <div className="mapping-summary">
      <div className="summary-row">
        <div className="summary-item">
          <span className="summary-label">Mappings</span>
          <span className="summary-value">{mapping.entries.length}</span>
        </div>

        {sourceLabel && (
          <div className="summary-item">
            <span className="summary-label">Source</span>
            <span className="summary-value">{sourceLabel}</span>
          </div>
        )}

        <div className="summary-item">
          <span className="summary-label">Confidence</span>
          <span className="summary-value confidence-distribution">
            {highCount > 0 && <span className="confidence-badge confidence-definitive">{highCount} High</span>}
            {mediumCount > 0 && <span className="confidence-badge confidence-medium">{mediumCount} Med</span>}
            {lowCount > 0 && <span className="confidence-badge confidence-low">{lowCount} Low</span>}
          </span>
        </div>
      </div>

      {/* Coverage metrics (Req 17.2) */}
      {coverage && (
        <div className="coverage-row">
          <div className="coverage-item">
            <span className="coverage-label">Event fields mapped</span>
            <div className="coverage-bar">
              <div
                className="coverage-fill"
                style={{ width: `${coverage.percentEventFieldsMapped}%` }}
              />
            </div>
            <span className="coverage-pct">{Math.round(coverage.percentEventFieldsMapped)}%</span>
          </div>
          <div className="coverage-item">
            <span className="coverage-label">Unmapped fields</span>
            <span className="coverage-pct">{Math.round(coverage.percentEventFieldsUnmapped)}%</span>
          </div>
          <div className="coverage-item">
            <span className="coverage-label">Unobserved mappings</span>
            <span className="coverage-pct">{Math.round(coverage.percentMappingFieldsUnobserved)}%</span>
          </div>
        </div>
      )}
    </div>
  );
}

export default EventPastePanel;
