/**
 * DimensionSelector component for selecting metric dimensions.
 * 
 * Multi-select for dimensions from entity attributes marked as dimensions.
 * 
 * Requirements: 3.4
 */

import { useMemo, useCallback } from 'react';
import type { SemanticEntity } from '../../types';

// ============================================
// Types
// ============================================

export interface DimensionSelectorProps {
  entities: SemanticEntity[];
  selectedDimensions: string[];
  onChange: (dimensions: string[]) => void;
}

interface DimensionOption {
  id: string;
  entityName: string;
  entityCaption: string;
  attributeName: string;
  attributeCaption: string;
  type: string;
}

// ============================================
// Component
// ============================================

export function DimensionSelector({
  entities,
  selectedDimensions,
  onChange,
}: DimensionSelectorProps) {
  // Get all dimension attributes from all entities - Requirement 3.4
  const dimensionOptions = useMemo((): DimensionOption[] => {
    const options: DimensionOption[] = [];
    
    for (const entity of entities) {
      for (const attr of entity.attributes) {
        if (attr.is_dimension) {
          options.push({
            id: `${entity.name}.${attr.name}`,
            entityName: entity.name,
            entityCaption: entity.caption || entity.name,
            attributeName: attr.name,
            attributeCaption: attr.caption || attr.name,
            type: typeof attr.attr_type === 'string' ? attr.attr_type : 'array',
          });
        }
      }
    }
    
    return options;
  }, [entities]);

  // Group dimensions by entity
  const groupedDimensions = useMemo(() => {
    const groups: Record<string, DimensionOption[]> = {};
    
    for (const opt of dimensionOptions) {
      if (!groups[opt.entityName]) {
        groups[opt.entityName] = [];
      }
      groups[opt.entityName].push(opt);
    }
    
    return groups;
  }, [dimensionOptions]);

  const handleToggle = useCallback(
    (dimensionId: string) => {
      if (selectedDimensions.includes(dimensionId)) {
        onChange(selectedDimensions.filter((d) => d !== dimensionId));
      } else {
        onChange([...selectedDimensions, dimensionId]);
      }
    },
    [selectedDimensions, onChange]
  );

  const handleSelectAll = useCallback(
    (entityName: string) => {
      const entityDimensions = groupedDimensions[entityName]?.map((d) => d.id) ?? [];
      const allSelected = entityDimensions.every((d) => selectedDimensions.includes(d));
      
      if (allSelected) {
        // Deselect all from this entity
        onChange(selectedDimensions.filter((d) => !entityDimensions.includes(d)));
      } else {
        // Select all from this entity
        const newSelection = new Set([...selectedDimensions, ...entityDimensions]);
        onChange(Array.from(newSelection));
      }
    },
    [groupedDimensions, selectedDimensions, onChange]
  );

  if (dimensionOptions.length === 0) {
    return (
      <div className="dimension-selector-empty">
        <p className="text-muted">
          No dimension attributes available. Mark entity attributes as dimensions to use them here.
        </p>
      </div>
    );
  }

  return (
    <div className="dimension-selector">
      {Object.entries(groupedDimensions).map(([entityName, dimensions]) => {
        const entityCaption = dimensions[0]?.entityCaption ?? entityName;
        const allSelected = dimensions.every((d) => selectedDimensions.includes(d.id));
        const someSelected = dimensions.some((d) => selectedDimensions.includes(d.id));
        
        return (
          <div key={entityName} className="dimension-group">
            <div className="dimension-group-header">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={allSelected}
                  ref={(el) => {
                    if (el) el.indeterminate = someSelected && !allSelected;
                  }}
                  onChange={() => handleSelectAll(entityName)}
                />
                <span className="dimension-group-name">{entityCaption}</span>
              </label>
            </div>
            <div className="dimension-group-items">
              {dimensions.map((dim) => (
                <label key={dim.id} className="dimension-item">
                  <input
                    type="checkbox"
                    checked={selectedDimensions.includes(dim.id)}
                    onChange={() => handleToggle(dim.id)}
                  />
                  <span className="dimension-item-name">{dim.attributeCaption}</span>
                  <span className="dimension-item-type">{dim.type}</span>
                </label>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default DimensionSelector;
