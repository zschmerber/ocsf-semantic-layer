import type { ParsedField, MappingEntry, MappingCoverage } from '../types/referenceEvent';

/**
 * Computes mapping coverage metrics between event fields and mapping entries.
 *
 * - percentEventFieldsMapped: % of event fields that have a matching ocsfField in entries
 * - percentEventFieldsUnmapped: % of event fields with no matching ocsfField (sums to 100 with mapped)
 * - percentMappingFieldsUnobserved: % of mapping entries whose ocsfField is NOT in event fields
 * - unmappedFieldPaths: event field paths with no mapping entry
 * - unobservedMappingFields: mapping ocsfFields not present in event fields
 */
export function computeCoverage(
  fields: ParsedField[],
  entries: MappingEntry[],
): MappingCoverage {
  const fieldPaths = new Set(fields.map((f) => f.path));
  const mappedOcsfFields = new Set(entries.map((e) => e.ocsfField));

  const totalFields = fieldPaths.size;
  const totalEntries = entries.length;

  const unmappedFieldPaths = fields
    .map((f) => f.path)
    .filter((p) => !mappedOcsfFields.has(p));

  const unobservedMappingFields = entries
    .map((e) => e.ocsfField)
    .filter((f) => !fieldPaths.has(f));

  const mappedCount = totalFields - unmappedFieldPaths.length;

  const percentEventFieldsMapped =
    totalFields === 0 ? 0 : (mappedCount / totalFields) * 100;

  const percentEventFieldsUnmapped =
    totalFields === 0 ? 0 : (unmappedFieldPaths.length / totalFields) * 100;

  const percentMappingFieldsUnobserved =
    totalEntries === 0 ? 0 : (unobservedMappingFields.length / totalEntries) * 100;

  return {
    percentEventFieldsMapped,
    percentMappingFieldsUnobserved,
    percentEventFieldsUnmapped,
    unmappedFieldPaths,
    unobservedMappingFields,
  };
}
