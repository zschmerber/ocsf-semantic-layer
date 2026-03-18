/**
 * Class UID Detector — resolves class_uid and category_uid from a parsed OCSF event
 * against the loaded schema tree.
 *
 * Requirements: 2.1, 2.2, 2.3, 2.5, 2.6, 14.2
 */

import type { ClassDetectionResult } from '../types/referenceEvent';
import type { SchemaTree } from '../types';

/**
 * Detect and resolve class_uid and category_uid from a parsed OCSF event
 * using the loaded schema tree.
 *
 * - If class_uid is present and a valid integer, looks it up in the schema tree.
 * - If class_uid is present but not a valid integer, returns an error.
 * - If class_uid is absent, returns Low confidence with no error.
 * - Similarly resolves category_uid to a category name when present.
 * - Never throws.
 */
export function detectClassUid(
  parsedEvent: Record<string, unknown>,
  schemaTree: SchemaTree,
): ClassDetectionResult {
  const result: ClassDetectionResult = {
    classUid: null,
    categoryUid: null,
    className: null,
    categoryName: null,
    confidence: 'Low',
    error: null,
  };

  // --- Resolve category_uid ---
  if ('category_uid' in parsedEvent && parsedEvent.category_uid !== undefined) {
    const raw = parsedEvent.category_uid;
    if (typeof raw === 'number' && Number.isInteger(raw)) {
      result.categoryUid = raw;
      const category = schemaTree.categories.find((c) => c.uid === raw);
      if (category) {
        result.categoryName = category.name;
      }
    }
    // Non-integer category_uid is silently ignored (no error for category)
  }

  // --- Resolve class_uid ---
  if (!('class_uid' in parsedEvent) || parsedEvent.class_uid === undefined) {
    // class_uid absent — Low confidence, no error
    return result;
  }

  const rawClassUid = parsedEvent.class_uid;

  // Validate that class_uid is a valid integer
  if (typeof rawClassUid !== 'number' || !Number.isInteger(rawClassUid)) {
    result.confidence = 'Low';
    result.error = `class_uid must be an integer, got: ${typeof rawClassUid}`;
    return result;
  }

  // class_uid is a valid integer — Definitive confidence
  result.classUid = rawClassUid;
  result.confidence = 'Definitive';

  // Look up the class in the schema tree
  for (const category of schemaTree.categories) {
    const cls = category.classes.find((c) => c.uid === rawClassUid);
    if (cls) {
      result.className = cls.name;
      // Also set category from the class's parent if not already resolved
      if (result.categoryUid === null) {
        result.categoryUid = category.uid;
      }
      if (result.categoryName === null) {
        result.categoryName = category.name;
      }
      return result;
    }
  }

  // class_uid not found in schema
  result.className = null;
  result.error = `class_uid ${rawClassUid} not found in loaded schema`;
  return result;
}
