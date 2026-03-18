/**
 * Schema Type Comparator — compares inferred field types from a parsed event
 * against OCSF schema-defined attribute types for the detected event class.
 *
 * Requirements: 4.1, 4.2
 */

import type { ParsedField, TypeMismatch } from '../types/referenceEvent';
import type { SchemaTree, AttributeNode } from '../types';

/**
 * OCSF schema type strings that map to each inferred JS type.
 * "array" matches any schema type (arrays are expected for repeated fields).
 */
const TYPE_COMPATIBILITY: Record<string, string[]> = {
  string: [
    'string_t', 'hostname_t', 'ip_t', 'email_t', 'url_t', 'mac_t',
    'subnet_t', 'path_t', 'process_name_t', 'username_t', 'datetime_t',
  ],
  integer: ['integer_t', 'long_t', 'timestamp_t'],
  float: ['float_t', 'double_t'],
  boolean: ['boolean_t'],
};

/**
 * Check if an inferred type is compatible with an OCSF schema type.
 */
function isTypeCompatible(inferredType: string, schemaType: string): boolean {
  // Arrays match any schema type
  if (inferredType === 'array') return true;

  const compatible = TYPE_COMPATIBILITY[inferredType];
  if (!compatible) return false;

  return compatible.includes(schemaType);
}

/**
 * Look up an attribute in a class's attribute tree by dot-notation field path.
 * Handles nested paths like "src_endpoint.ip" by traversing children.
 */
function findAttribute(
  attributes: AttributeNode[],
  pathParts: string[],
): AttributeNode | null {
  if (pathParts.length === 0) return null;

  const [head, ...rest] = pathParts;
  const attr = attributes.find((a) => a.name === head);
  if (!attr) return null;

  if (rest.length === 0) return attr;

  // Traverse into children for nested paths
  if (attr.children && attr.children.length > 0) {
    return findAttribute(attr.children, rest);
  }

  return null;
}

/**
 * Compare each parsed field's inferred type against the OCSF schema-defined type
 * for the given event class.
 *
 * - Finds the event class in the schema tree by classUid.
 * - If the class is not found, returns an empty array.
 * - For each field, looks up the corresponding schema attribute.
 * - Fields not found in the schema are silently skipped.
 * - When a type mismatch is detected, creates a TypeMismatch entry.
 */
export function compareTypes(
  fields: ParsedField[],
  classUid: number,
  schemaTree: SchemaTree,
): TypeMismatch[] {
  // Find the event class by classUid
  let classAttributes: AttributeNode[] | null = null;
  for (const category of schemaTree.categories) {
    const cls = category.classes.find((c) => c.uid === classUid);
    if (cls) {
      classAttributes = cls.attributes;
      break;
    }
  }

  if (!classAttributes) return [];

  const mismatches: TypeMismatch[] = [];

  for (const field of fields) {
    const pathParts = field.path.split('.');
    const attr = findAttribute(classAttributes, pathParts);

    // Skip fields not found in the schema
    if (!attr) continue;

    // Skip null/object types — they don't have meaningful type comparisons
    if (field.type === 'null' || field.type === 'object') continue;

    if (!isTypeCompatible(field.type, attr.type)) {
      mismatches.push({
        fieldPath: field.path,
        observedType: field.type,
        schemaType: attr.type,
      });
    }
  }

  return mismatches;
}
