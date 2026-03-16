/**
 * GranularitySelector component for selecting time granularities.
 * 
 * Checkbox group for time granularities (minute, hour, day, week, month).
 * 
 * Requirements: 3.5
 */

import { useCallback } from 'react';
import type { TimeGranularity } from '../../types';

// ============================================
// Types
// ============================================

export interface GranularitySelectorProps {
  selectedGranularities: TimeGranularity[];
  onChange: (granularities: TimeGranularity[]) => void;
}

interface GranularityOption {
  value: TimeGranularity;
  label: string;
  description: string;
}

const GRANULARITY_OPTIONS: GranularityOption[] = [
  { value: 'minute', label: 'Minute', description: 'Per-minute aggregation' },
  { value: 'hour', label: 'Hour', description: 'Hourly aggregation' },
  { value: 'day', label: 'Day', description: 'Daily aggregation' },
  { value: 'week', label: 'Week', description: 'Weekly aggregation' },
  { value: 'month', label: 'Month', description: 'Monthly aggregation' },
];

// ============================================
// Component
// ============================================

export function GranularitySelector({
  selectedGranularities,
  onChange,
}: GranularitySelectorProps) {
  const handleToggle = useCallback(
    (granularity: TimeGranularity) => {
      if (selectedGranularities.includes(granularity)) {
        onChange(selectedGranularities.filter((g) => g !== granularity));
      } else {
        onChange([...selectedGranularities, granularity]);
      }
    },
    [selectedGranularities, onChange]
  );

  const handleSelectAll = useCallback(() => {
    if (selectedGranularities.length === GRANULARITY_OPTIONS.length) {
      onChange([]);
    } else {
      onChange(GRANULARITY_OPTIONS.map((opt) => opt.value));
    }
  }, [selectedGranularities, onChange]);

  const allSelected = selectedGranularities.length === GRANULARITY_OPTIONS.length;
  const someSelected = selectedGranularities.length > 0 && !allSelected;

  return (
    <div className="granularity-selector">
      <div className="granularity-header">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={allSelected}
            ref={(el) => {
              if (el) el.indeterminate = someSelected;
            }}
            onChange={handleSelectAll}
          />
          <span>Select All</span>
        </label>
      </div>
      <div className="granularity-options">
        {GRANULARITY_OPTIONS.map((opt) => (
          <label key={opt.value} className="granularity-item">
            <input
              type="checkbox"
              checked={selectedGranularities.includes(opt.value)}
              onChange={() => handleToggle(opt.value)}
            />
            <span className="granularity-item-label">{opt.label}</span>
            <span className="granularity-item-desc">{opt.description}</span>
          </label>
        ))}
      </div>
    </div>
  );
}

export default GranularitySelector;
