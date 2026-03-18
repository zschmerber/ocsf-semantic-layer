import type { ParsedField } from '../types/referenceEvent';

const OCSF_MARKER_FIELDS = ['class_uid', 'category_uid', 'activity_id', 'type_uid', 'metadata'];

function inferType(value: unknown): string {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'array';
  if (typeof value === 'boolean') return 'boolean';
  if (typeof value === 'number') {
    return Number.isInteger(value) ? 'integer' : 'float';
  }
  if (typeof value === 'string') return 'string';
  return 'object';
}

function flattenObject(obj: Record<string, unknown>, prefix: string, fields: ParsedField[]): void {
  for (const key of Object.keys(obj)) {
    const path = prefix ? `${prefix}.${key}` : key;
    const value = obj[key];

    if (value === null) {
      fields.push({ path, type: 'null', value: 'null', rawValue: null });
    } else if (Array.isArray(value)) {
      fields.push({ path, type: 'array', value: JSON.stringify(value), rawValue: value });
    } else if (typeof value === 'object') {
      flattenObject(value as Record<string, unknown>, path, fields);
    } else {
      const type = inferType(value);
      const displayValue = typeof value === 'string' ? value : JSON.stringify(value);
      fields.push({ path, type, value: displayValue, rawValue: value });
    }
  }
}

function hasOcsfFields(obj: Record<string, unknown>): boolean {
  return OCSF_MARKER_FIELDS.some((field) => field in obj);
}

export function extractFields(json: string): {
  fields: ParsedField[];
  error: string | null;
  warning: string | null;
} {
  if (!json || !json.trim()) {
    return { fields: [], error: 'Input is empty', warning: null };
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : 'Invalid JSON';
    return { fields: [], error: message, warning: null };
  }

  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    return { fields: [], error: 'JSON must be an object', warning: null };
  }

  const obj = parsed as Record<string, unknown>;
  const fields: ParsedField[] = [];
  flattenObject(obj, '', fields);

  const warning = hasOcsfFields(obj)
    ? null
    : 'No recognizable OCSF fields found. Verify this is a transformed OCSF event.';

  return { fields, error: null, warning };
}
