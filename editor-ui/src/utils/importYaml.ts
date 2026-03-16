/**
 * Utility functions for importing semantic models from YAML files.
 * 
 * Requirements:
 * - 6.2: THE Editor SHALL provide an import button that loads a semantic model from a YAML file
 * - 6.3: WHEN importing a model, THE Editor SHALL validate the YAML structure before loading
 * - 6.4: IF the imported YAML is invalid, THEN THE Editor SHALL display parsing errors with line numbers
 */

import yaml from 'js-yaml';
import type { 
  SemanticModel, 
  SemanticEntity, 
  SemanticMetric, 
  SemanticAttribute,
  EntityRelationship,
  ObservableConfig,
  OCSFMapping,
  ThreatRelevance,
} from '../types';

/**
 * Result of a YAML import operation.
 */
export interface ImportResult {
  success: boolean;
  model?: SemanticModel;
  errors: ImportError[];
}

/**
 * An error encountered during import.
 */
export interface ImportError {
  line?: number;
  column?: number;
  message: string;
  path?: string;
}

/**
 * Parses a YAML string and returns the parsed object or errors.
 * 
 * @param yamlContent - The YAML string to parse
 * @returns The parsed object or null with errors
 */
function parseYaml(yamlContent: string): { data: unknown; error?: ImportError } {
  try {
    const data = yaml.load(yamlContent);
    return { data };
  } catch (e) {
    if (e instanceof yaml.YAMLException) {
      return {
        data: null,
        error: {
          line: e.mark?.line !== undefined ? e.mark.line + 1 : undefined,
          column: e.mark?.column !== undefined ? e.mark.column + 1 : undefined,
          message: e.reason || e.message,
        },
      };
    }
    return {
      data: null,
      error: {
        message: e instanceof Error ? e.message : 'Unknown parsing error',
      },
    };
  }
}

/**
 * Validates that a value is a non-empty string.
 */
function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.length > 0;
}

/**
 * Validates that a value is a string (can be empty).
 */
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

/**
 * Validates that a value is an array.
 */
function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

/**
 * Validates that a value is an object (not null, not array).
 */
function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * Validates the ObservableConfig structure.
 */
function validateObservableConfig(config: unknown, errors: ImportError[]): config is ObservableConfig {
  if (!isObject(config)) {
    errors.push({ message: 'observable_config must be an object', path: 'observable_config' });
    return false;
  }

  if (typeof config.extract_to_table !== 'boolean') {
    errors.push({ message: 'observable_config.extract_to_table must be a boolean', path: 'observable_config.extract_to_table' });
  }

  if (!isString(config.table_name)) {
    errors.push({ message: 'observable_config.table_name must be a string', path: 'observable_config.table_name' });
  }

  if (!isArray(config.include_types)) {
    errors.push({ message: 'observable_config.include_types must be an array', path: 'observable_config.include_types' });
  }

  return errors.length === 0;
}

/**
 * Validates a SemanticAttribute structure.
 */
function validateAttribute(attr: unknown, entityName: string, index: number, errors: ImportError[]): boolean {
  const basePath = `entities[${entityName}].attributes[${index}]`;
  
  if (!isObject(attr)) {
    errors.push({ message: `Attribute at index ${index} must be an object`, path: basePath });
    return false;
  }

  if (!isNonEmptyString(attr.name)) {
    errors.push({ message: 'Attribute name is required and must be a non-empty string', path: `${basePath}.name` });
  }

  if (!isString(attr.caption)) {
    errors.push({ message: 'Attribute caption must be a string', path: `${basePath}.caption` });
  }

  if (!isString(attr.description)) {
    errors.push({ message: 'Attribute description must be a string', path: `${basePath}.description` });
  }

  // ocsf_mapping is optional but if present must be an object
  if (attr.ocsf_mapping !== undefined && !isObject(attr.ocsf_mapping)) {
    errors.push({ message: 'Attribute ocsf_mapping must be an object', path: `${basePath}.ocsf_mapping` });
  }

  return true;
}

/**
 * Validates a SemanticEntity structure.
 */
function validateEntity(entity: unknown, index: number, errors: ImportError[]): boolean {
  const basePath = `entities[${index}]`;
  
  if (!isObject(entity)) {
    errors.push({ message: `Entity at index ${index} must be an object`, path: basePath });
    return false;
  }

  if (!isNonEmptyString(entity.name)) {
    errors.push({ message: 'Entity name is required and must be a non-empty string', path: `${basePath}.name` });
  }

  if (!isString(entity.caption)) {
    errors.push({ message: 'Entity caption must be a string', path: `${basePath}.caption` });
  }

  if (!isString(entity.description)) {
    errors.push({ message: 'Entity description must be a string', path: `${basePath}.description` });
  }

  if (!isArray(entity.source_event_classes)) {
    errors.push({ message: 'Entity source_event_classes must be an array', path: `${basePath}.source_event_classes` });
  }

  if (!isArray(entity.attributes)) {
    errors.push({ message: 'Entity attributes must be an array', path: `${basePath}.attributes` });
  } else {
    const entityName = isNonEmptyString(entity.name) ? entity.name : `index_${index}`;
    entity.attributes.forEach((attr, attrIndex) => {
      validateAttribute(attr, entityName, attrIndex, errors);
    });
  }

  // relationships is optional but if present must be an array
  if (entity.relationships !== undefined && !isArray(entity.relationships)) {
    errors.push({ message: 'Entity relationships must be an array', path: `${basePath}.relationships` });
  }

  // covers_observables is optional but if present must be an array
  if (entity.covers_observables !== undefined && !isArray(entity.covers_observables)) {
    errors.push({ message: 'Entity covers_observables must be an array', path: `${basePath}.covers_observables` });
  }

  return true;
}

/**
 * Validates a SemanticMetric structure.
 */
function validateMetric(metric: unknown, index: number, errors: ImportError[]): boolean {
  const basePath = `metrics[${index}]`;
  
  if (!isObject(metric)) {
    errors.push({ message: `Metric at index ${index} must be an object`, path: basePath });
    return false;
  }

  if (!isNonEmptyString(metric.name)) {
    errors.push({ message: 'Metric name is required and must be a non-empty string', path: `${basePath}.name` });
  }

  if (!isString(metric.caption)) {
    errors.push({ message: 'Metric caption must be a string', path: `${basePath}.caption` });
  }

  if (!isString(metric.description)) {
    errors.push({ message: 'Metric description must be a string', path: `${basePath}.description` });
  }

  const validAggregations = ['count', 'sum', 'avg', 'min', 'max', 'count_distinct'];
  if (!isString(metric.aggregation) || !validAggregations.includes(metric.aggregation)) {
    errors.push({ 
      message: `Metric aggregation must be one of: ${validAggregations.join(', ')}`, 
      path: `${basePath}.aggregation` 
    });
  }

  if (!isArray(metric.dimensions)) {
    errors.push({ message: 'Metric dimensions must be an array', path: `${basePath}.dimensions` });
  }

  if (!isArray(metric.time_granularities)) {
    errors.push({ message: 'Metric time_granularities must be an array', path: `${basePath}.time_granularities` });
  }

  return true;
}

/**
 * Validates the structure of a parsed YAML object to ensure it conforms to SemanticModel.
 * 
 * @param data - The parsed YAML data
 * @returns Array of validation errors
 */
function validateSemanticModelStructure(data: unknown): ImportError[] {
  const errors: ImportError[] = [];

  if (!isObject(data)) {
    errors.push({ message: 'Root element must be an object' });
    return errors;
  }

  // Required string fields
  if (!isString(data.version)) {
    errors.push({ message: 'version is required and must be a string', path: 'version' });
  }

  if (!isString(data.ocsf_version)) {
    errors.push({ message: 'ocsf_version is required and must be a string', path: 'ocsf_version' });
  }

  if (!isString(data.name)) {
    errors.push({ message: 'name is required and must be a string', path: 'name' });
  }

  if (!isString(data.description)) {
    errors.push({ message: 'description is required and must be a string', path: 'description' });
  }

  // Entities array
  if (!isArray(data.entities)) {
    errors.push({ message: 'entities is required and must be an array', path: 'entities' });
  } else {
    data.entities.forEach((entity, index) => {
      validateEntity(entity, index, errors);
    });
  }

  // Metrics array
  if (!isArray(data.metrics)) {
    errors.push({ message: 'metrics is required and must be an array', path: 'metrics' });
  } else {
    data.metrics.forEach((metric, index) => {
      validateMetric(metric, index, errors);
    });
  }

  // Observable config
  if (!isObject(data.observable_config)) {
    errors.push({ message: 'observable_config is required and must be an object', path: 'observable_config' });
  } else {
    validateObservableConfig(data.observable_config, errors);
  }

  return errors;
}

/**
 * Normalizes the parsed data to ensure all optional fields have default values.
 * This function assumes the data has already been validated.
 */
function normalizeModel(data: Record<string, unknown>): SemanticModel {
  const entities: SemanticEntity[] = (data.entities as unknown[]).map((e) => {
    const entity = e as Record<string, unknown>;
    const attributes: SemanticAttribute[] = ((entity.attributes as unknown[]) || []).map((a) => {
      const attr = a as Record<string, unknown>;
      return {
        name: attr.name as string,
        caption: (attr.caption as string) || '',
        description: (attr.description as string) || '',
        attr_type: (attr.attr_type as SemanticAttribute['attr_type']) || 'string',
        ocsf_mapping: (attr.ocsf_mapping as OCSFMapping) || {},
        is_dimension: Boolean(attr.is_dimension),
        sample_values: (attr.sample_values as string[]) || [],
        synonyms: (attr.synonyms as string[]) || [],
        security_context: attr.security_context as string | undefined,
        value_pattern: attr.value_pattern as string | undefined,
        is_observable: Boolean(attr.is_observable),
        threat_relevance: attr.threat_relevance as ThreatRelevance | undefined,
      };
    });

    const relationships: EntityRelationship[] = ((entity.relationships as unknown[]) || []).map((r) => {
      const rel = r as Record<string, unknown>;
      return {
        name: (rel.name as string) || '',
        target_entity: (rel.target_entity as string) || '',
        relationship_type: rel.relationship_type as EntityRelationship['relationship_type'],
        cardinality: (rel.cardinality as EntityRelationship['cardinality']) || 'one-to-one',
        join_condition: (rel.join_condition as string) || '',
        description: (rel.description as string) || '',
      };
    });

    return {
      name: entity.name as string,
      caption: (entity.caption as string) || '',
      description: (entity.description as string) || '',
      source_event_classes: (entity.source_event_classes as number[]) || [],
      attributes,
      relationships,
      covers_observables: (entity.covers_observables as number[]) || [],
    };
  });

  const metrics: SemanticMetric[] = (data.metrics as unknown[]).map((m) => {
    const metric = m as Record<string, unknown>;
    return {
      name: metric.name as string,
      caption: (metric.caption as string) || '',
      description: (metric.description as string) || '',
      aggregation: (metric.aggregation as SemanticMetric['aggregation']) || 'count',
      measure: (metric.measure as OCSFMapping) || {},
      dimensions: (metric.dimensions as string[]) || [],
      time_granularities: (metric.time_granularities as SemanticMetric['time_granularities']) || [],
      is_hot_path: Boolean(metric.is_hot_path),
      observable_type_id: metric.observable_type_id as number | undefined,
    };
  });

  const observableConfig = data.observable_config as Record<string, unknown>;

  return {
    version: data.version as string,
    ocsf_version: data.ocsf_version as string,
    name: data.name as string,
    description: data.description as string,
    entities,
    metrics,
    observable_config: {
      extract_to_table: Boolean(observableConfig.extract_to_table),
      table_name: (observableConfig.table_name as string) || 'ocsf_observables',
      include_types: (observableConfig.include_types as number[]) || [],
    },
  };
}

/**
 * Imports a semantic model from a YAML string.
 * Validates the YAML structure before returning the model.
 * 
 * Requirements:
 * - 6.3: Validate YAML structure before loading
 * - 6.4: Display parsing errors with line numbers
 * 
 * @param yamlContent - The YAML string to import
 * @returns ImportResult with the model or errors
 */
export function importModelFromYaml(yamlContent: string): ImportResult {
  // Step 1: Parse the YAML
  const { data, error: parseError } = parseYaml(yamlContent);
  
  if (parseError) {
    return {
      success: false,
      errors: [parseError],
    };
  }

  // Step 2: Validate the structure
  const validationErrors = validateSemanticModelStructure(data);
  
  if (validationErrors.length > 0) {
    return {
      success: false,
      errors: validationErrors,
    };
  }

  // Step 3: Normalize and return the model
  const model = normalizeModel(data as Record<string, unknown>);
  
  return {
    success: true,
    model,
    errors: [],
  };
}

/**
 * Opens a file picker dialog and reads the selected YAML file.
 * 
 * @returns Promise that resolves with the file content or null if cancelled
 */
export function openFilePicker(): Promise<{ content: string; filename: string } | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.yaml,.yml';
    
    input.onchange = (event) => {
      const file = (event.target as HTMLInputElement).files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      
      const reader = new FileReader();
      reader.onload = (e) => {
        const content = e.target?.result as string;
        resolve({ content, filename: file.name });
      };
      reader.onerror = () => {
        resolve(null);
      };
      reader.readAsText(file);
    };
    
    input.oncancel = () => {
      resolve(null);
    };
    
    // Trigger the file picker
    input.click();
  });
}

/**
 * Formats import errors for display to the user.
 * 
 * @param errors - Array of import errors
 * @returns Formatted error message string
 */
export function formatImportErrors(errors: ImportError[]): string {
  return errors.map((error) => {
    let message = '';
    if (error.line !== undefined) {
      message += `Line ${error.line}`;
      if (error.column !== undefined) {
        message += `, Column ${error.column}`;
      }
      message += ': ';
    }
    if (error.path) {
      message += `[${error.path}] `;
    }
    message += error.message;
    return message;
  }).join('\n');
}
