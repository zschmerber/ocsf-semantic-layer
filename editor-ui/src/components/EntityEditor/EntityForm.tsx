/**
 * EntityForm component for creating and editing semantic entities.
 * 
 * Provides form fields for name, caption, description, and event class selection.
 * 
 * Requirements: 2.1, 2.2, 2.7, 2.8
 */

import { useState, useCallback, useMemo } from 'react';
import type { SemanticEntity, ClassNode, Dataset } from '../../types';
import { createDefaultEntity } from '../../types';

// ============================================
// Types
// ============================================

export interface EntityFormProps {
  entity?: SemanticEntity;
  availableClasses: ClassNode[];
  datasets?: Dataset[];
  onSave: (entity: SemanticEntity) => void;
  onCancel: () => void;
}

// ============================================
// Component
// ============================================

export function EntityForm({
  entity,
  availableClasses,
  datasets = [],
  onSave,
  onCancel,
}: EntityFormProps) {
  const isEditing = !!entity;
  
  // Form state
  const [name, setName] = useState(entity?.name ?? '');
  const [caption, setCaption] = useState(entity?.caption ?? '');
  const [description, setDescription] = useState(entity?.description ?? '');
  const [selectedClasses, setSelectedClasses] = useState<number[]>(
    entity?.source_event_classes ?? []
  );
  const [classSearchQuery, setClassSearchQuery] = useState('');
  const [showClassDropdown, setShowClassDropdown] = useState(false);
  const [datasetRef, setDatasetRef] = useState(entity?.dataset_ref ?? '');
  
  // Validation
  const [errors, setErrors] = useState<Record<string, string>>({});
  
  // Filter available classes based on search
  const filteredClasses = useMemo(() => {
    if (!classSearchQuery) return availableClasses;
    const query = classSearchQuery.toLowerCase();
    return availableClasses.filter(
      (cls) =>
        cls.name.toLowerCase().includes(query) ||
        cls.caption.toLowerCase().includes(query)
    );
  }, [availableClasses, classSearchQuery]);
  
  // Get selected class objects
  const selectedClassObjects = useMemo(() => {
    return availableClasses.filter((cls) => selectedClasses.includes(cls.uid));
  }, [availableClasses, selectedClasses]);
  
  const validateForm = useCallback((): boolean => {
    const newErrors: Record<string, string> = {};
    
    if (!name.trim()) {
      newErrors.name = 'Name is required';
    } else if (!/^[a-z][a-z0-9_]*$/.test(name)) {
      newErrors.name = 'Name must start with lowercase letter and contain only lowercase letters, numbers, and underscores';
    }
    
    if (!caption.trim()) {
      newErrors.caption = 'Caption is required';
    }
    
    if (selectedClasses.length === 0) {
      newErrors.classes = 'At least one event class must be selected';
    }
    
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [name, caption, selectedClasses]);
  
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      
      if (!validateForm()) return;
      
      const newEntity: SemanticEntity = entity
        ? {
            ...entity,
            name: name.trim(),
            caption: caption.trim(),
            description: description.trim(),
            source_event_classes: selectedClasses,
            dataset_ref: datasetRef || undefined,
          }
        : {
            ...createDefaultEntity(name.trim()),
            caption: caption.trim(),
            description: description.trim(),
            source_event_classes: selectedClasses,
            dataset_ref: datasetRef || undefined,
          };
      
      onSave(newEntity);
    },
    [entity, name, caption, description, selectedClasses, datasetRef, validateForm, onSave]
  );
  
  const handleClassToggle = useCallback((classUid: number) => {
    setSelectedClasses((prev) =>
      prev.includes(classUid)
        ? prev.filter((uid) => uid !== classUid)
        : [...prev, classUid]
    );
    // Clear class error when selection changes
    setErrors((prev) => {
      const { classes, ...rest } = prev;
      return rest;
    });
  }, []);
  
  const handleRemoveClass = useCallback((classUid: number) => {
    setSelectedClasses((prev) => prev.filter((uid) => uid !== classUid));
  }, []);

  return (
    <form className="entity-form" onSubmit={handleSubmit}>
      <div className="form-header">
        <h3>{isEditing ? 'Edit Entity' : 'Create Entity'}</h3>
      </div>
      
      <div className="form-body">
        {/* Name Field */}
        <div className="form-group">
          <label className="form-label" htmlFor="entity-name">
            Name <span className="required">*</span>
          </label>
          <input
            id="entity-name"
            type="text"
            className={`form-input ${errors.name ? 'error' : ''}`}
            value={name}
            onChange={(e) => {
              setName(e.target.value);
              if (errors.name) {
                setErrors((prev) => {
                  const { name, ...rest } = prev;
                  return rest;
                });
              }
            }}
            placeholder="e.g., dns_activity"
            disabled={isEditing}
          />
          {errors.name && <span className="form-error">{errors.name}</span>}
          {isEditing && (
            <span className="form-hint">Name cannot be changed after creation</span>
          )}
        </div>
        
        {/* Caption Field */}
        <div className="form-group">
          <label className="form-label" htmlFor="entity-caption">
            Caption <span className="required">*</span>
          </label>
          <input
            id="entity-caption"
            type="text"
            className={`form-input ${errors.caption ? 'error' : ''}`}
            value={caption}
            onChange={(e) => {
              setCaption(e.target.value);
              if (errors.caption) {
                setErrors((prev) => {
                  const { caption, ...rest } = prev;
                  return rest;
                });
              }
            }}
            placeholder="e.g., DNS Activity"
          />
          {errors.caption && <span className="form-error">{errors.caption}</span>}
        </div>
        
        {/* Description Field */}
        <div className="form-group">
          <label className="form-label" htmlFor="entity-description">
            Description
          </label>
          <textarea
            id="entity-description"
            className="form-textarea"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe the purpose and use of this entity..."
            rows={3}
          />
        </div>
        
        {/* Dataset Reference */}
        {datasets.length > 0 && (
          <div className="form-group">
            <label className="form-label" htmlFor="entity-dataset-ref">
              Dataset Reference
            </label>
            <select
              id="entity-dataset-ref"
              className="form-select"
              value={datasetRef}
              onChange={(e) => setDatasetRef(e.target.value)}
            >
              <option value="">None</option>
              {datasets.map((ds) => (
                <option key={ds.name} value={ds.name}>
                  {ds.name}
                </option>
              ))}
            </select>
          </div>
        )}
        
        {/* Event Class Multi-Select */}
        <div className="form-group">
          <label className="form-label">
            Source Event Classes <span className="required">*</span>
          </label>
          
          {/* Selected Classes Tags */}
          {selectedClassObjects.length > 0 && (
            <div className="selected-classes">
              {selectedClassObjects.map((cls) => (
                <span key={cls.uid} className="class-tag">
                  <span className="class-tag-name">{cls.caption}</span>
                  <button
                    type="button"
                    className="class-tag-remove"
                    onClick={() => handleRemoveClass(cls.uid)}
                    aria-label={`Remove ${cls.caption}`}
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
          )}
          
          {/* Class Dropdown */}
          <div className="class-dropdown-container">
            <div
              className={`class-dropdown-trigger ${errors.classes ? 'error' : ''}`}
              onClick={() => setShowClassDropdown(!showClassDropdown)}
            >
              <input
                type="text"
                className="class-search-input"
                placeholder="Search and select event classes..."
                value={classSearchQuery}
                onChange={(e) => {
                  setClassSearchQuery(e.target.value);
                  setShowClassDropdown(true);
                }}
                onFocus={() => setShowClassDropdown(true)}
              />
              <span className="dropdown-arrow">{showClassDropdown ? '▲' : '▼'}</span>
            </div>
            
            {showClassDropdown && (
              <div className="class-dropdown-menu">
                {filteredClasses.length === 0 ? (
                  <div className="class-dropdown-empty">
                    {classSearchQuery ? 'No matching classes' : 'No classes available'}
                  </div>
                ) : (
                  filteredClasses.map((cls) => (
                    <div
                      key={cls.uid}
                      className={`class-dropdown-item ${
                        selectedClasses.includes(cls.uid) ? 'selected' : ''
                      }`}
                      onClick={() => handleClassToggle(cls.uid)}
                    >
                      <span className="class-checkbox">
                        {selectedClasses.includes(cls.uid) ? '☑' : '☐'}
                      </span>
                      <span className="class-info">
                        <span className="class-caption">{cls.caption}</span>
                        <span className="class-name">{cls.name}</span>
                      </span>
                      <span className="class-uid">UID: {cls.uid}</span>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
          {errors.classes && <span className="form-error">{errors.classes}</span>}
        </div>
      </div>
      
      <div className="form-footer">
        <button type="button" className="btn" onClick={onCancel}>
          Cancel
        </button>
        <button type="submit" className="btn primary">
          {isEditing ? 'Save Changes' : 'Create Entity'}
        </button>
      </div>
    </form>
  );
}

export default EntityForm;
