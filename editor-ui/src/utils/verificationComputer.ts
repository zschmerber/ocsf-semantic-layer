import type { MappingEntry, ParsedField, VerificationStatus } from '../types/referenceEvent';

/**
 * Cross-references mapping entries against parsed event fields
 * to compute a verification status for each mapping's ocsfField.
 *
 * - "Verified": the ocsfField matches a field path in the ParsedField array
 * - "Unverified": the ocsfField does not match any field path
 * - "Conflict": reserved for future type-contradiction detection
 */
export function computeVerification(
  entries: MappingEntry[],
  fields: ParsedField[],
): Map<string, VerificationStatus> {
  const fieldPaths = new Set(fields.map((f) => f.path));
  const result = new Map<string, VerificationStatus>();

  for (const entry of entries) {
    const status: VerificationStatus = fieldPaths.has(entry.ocsfField)
      ? 'Verified'
      : 'Unverified';
    result.set(entry.ocsfField, status);
  }

  return result;
}
