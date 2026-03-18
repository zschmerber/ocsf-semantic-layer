import type { ParsedField, ObservableFlag, ObservableType } from '../types/referenceEvent';

// ============================================
// Value pattern regexes
// ============================================

const IPV4_RE = /^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/;
const IPV6_RE = /^([0-9a-fA-F]{1,4}:){2,7}[0-9a-fA-F]{1,4}$/;
const HOSTNAME_RE =
  /^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)+$/;
const MD5_RE = /^[a-fA-F0-9]{32}$/;
const SHA1_RE = /^[a-fA-F0-9]{40}$/;
const SHA256_RE = /^[a-fA-F0-9]{64}$/;
const URL_RE = /^https?:\/\//;
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const MAC_RE = /^([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}$/;

// ============================================
// Name heuristic helpers
// ============================================

/** Word-boundary-ish check for "ip" — avoids matching "description", "script", "tip", etc. */
function nameContainsIp(segment: string): boolean {
  return /(?:^|[_.-])ip(?:$|[_.-]|\d)/i.test(segment) || /^ip$/i.test(segment);
}

function nameContainsMac(segment: string): boolean {
  // Match "mac" as a word, not "machine"
  return /(?:^|[_.-])mac(?:$|[_.-])/i.test(segment);
}

function detectNameHeuristic(segment: string): ObservableType | null {
  const lower = segment.toLowerCase();

  if (nameContainsIp(segment)) return 'ip';
  if (lower.includes('host')) return 'hostname';
  if (lower.includes('hash')) return 'hash';
  if (lower.includes('url') || lower.includes('uri')) return 'url';
  if (lower.includes('email')) return 'email';
  if (nameContainsMac(segment)) return 'mac';
  if (lower.includes('process')) return 'process';

  return null;
}

// ============================================
// Value pattern matching
// ============================================

function detectValuePattern(value: string): ObservableType | null {
  if (IPV4_RE.test(value)) return 'ip';
  if (IPV6_RE.test(value)) return 'ip';
  if (URL_RE.test(value)) return 'url';
  if (EMAIL_RE.test(value)) return 'email';
  if (MAC_RE.test(value)) return 'mac';
  if (MD5_RE.test(value) || SHA1_RE.test(value) || SHA256_RE.test(value)) return 'hash';
  if (HOSTNAME_RE.test(value)) return 'hostname';

  return null;
}

// ============================================
// Public API
// ============================================

/**
 * Scan parsed fields and flag those that look like observables
 * based on field name heuristics and value pattern matching.
 */
export function flagObservables(fields: ParsedField[]): ObservableFlag[] {
  const flags: ObservableFlag[] = [];

  for (const field of fields) {
    // Use the last segment of the dot-notation path for name heuristic
    const segments = field.path.split('.');
    const lastSegment = segments[segments.length - 1];

    const nameType = detectNameHeuristic(lastSegment);

    // Value pattern matching only applies to string-type fields
    const valueType =
      field.type === 'string' && typeof field.rawValue === 'string'
        ? detectValuePattern(field.rawValue)
        : null;

    if (nameType && valueType) {
      // Both matched — pick the name heuristic type (it's the primary signal)
      flags.push({
        fieldPath: field.path,
        observableType: nameType,
        confidence: 'High',
        matchedBy: 'both',
      });
    } else if (nameType) {
      flags.push({
        fieldPath: field.path,
        observableType: nameType,
        confidence: 'High',
        matchedBy: 'name',
      });
    } else if (valueType) {
      flags.push({
        fieldPath: field.path,
        observableType: valueType,
        confidence: 'Medium',
        matchedBy: 'value',
      });
    }
    // Neither matched — don't flag
  }

  return flags;
}
