/**
 * Log parser utilities for detecting and parsing various log formats.
 * Supports JSON, CSV, syslog, and key-value formats.
 * 
 * Requirements: 14.2, 14.3
 */

import type { ParsedLogField } from '../types';

/**
 * Supported log formats.
 */
export type LogFormat = 'json' | 'csv' | 'syslog' | 'kv';

/**
 * Detects the format of the input log data.
 * 
 * Detection priority:
 * 1. JSON - starts with '{' or '[' and is valid JSON
 * 2. CSV - has commas with consistent column count across lines
 * 3. Syslog - matches syslog timestamp patterns
 * 4. Key-value - has key=value patterns
 * 5. Default fallback to JSON
 * 
 * @param input - Raw log input string
 * @returns Detected log format
 */
export function detectLogFormat(input: string): LogFormat {
  const trimmed = input.trim();
  
  if (!trimmed) {
    return 'json'; // Default fallback for empty input
  }
  
  // JSON detection - starts with '{' or '[' and is valid JSON
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      JSON.parse(trimmed);
      return 'json';
    } catch {
      // Not valid JSON, continue checking other formats
    }
  }
  
  // CSV detection - has commas and consistent column count across lines
  const lines = trimmed.split('\n').filter(line => line.trim());
  if (lines.length > 1) {
    const firstLineCommas = (lines[0].match(/,/g) || []).length;
    const secondLineCommas = (lines[1].match(/,/g) || []).length;
    if (firstLineCommas > 0 && firstLineCommas === secondLineCommas) {
      return 'csv';
    }
  }
  
  // Syslog detection - starts with timestamp pattern
  // BSD syslog: "Jan  1 00:00:00" or "Jan 01 00:00:00"
  // ISO 8601: "2024-01-01T00:00:00"
  if (/^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}/.test(trimmed) ||
      /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/.test(trimmed)) {
    return 'syslog';
  }
  
  // Key-value detection - has key=value patterns
  if (/\w+=\S+/.test(trimmed)) {
    return 'kv';
  }
  
  return 'json'; // Default fallback
}

/**
 * Parses log input based on the specified format.
 * 
 * @param input - Raw log input string
 * @param format - Log format to use for parsing
 * @returns Array of parsed log fields
 */
export function parseLogInput(input: string, format: LogFormat): ParsedLogField[] {
  switch (format) {
    case 'json':
      return parseJsonLog(input);
    case 'csv':
      return parseCsvLog(input);
    case 'syslog':
      return parseSyslogLog(input);
    case 'kv':
      return parseKvLog(input);
  }
}

/**
 * Parses JSON log input.
 * 
 * @param input - JSON string
 * @returns Array of parsed log fields
 */
function parseJsonLog(input: string): ParsedLogField[] {
  const trimmed = input.trim();
  if (!trimmed) {
    return [];
  }
  
  const obj = JSON.parse(trimmed);
  return flattenObject(obj);
}

/**
 * Flattens a nested object into an array of ParsedLogField entries.
 * Each leaf value becomes a separate field with its full path.
 * 
 * @param obj - Object to flatten
 * @param prefix - Current path prefix for nested objects
 * @returns Array of parsed log fields
 */
export function flattenObject(obj: unknown, prefix = ''): ParsedLogField[] {
  const fields: ParsedLogField[] = [];
  
  if (obj === null) {
    fields.push({ 
      name: prefix || 'root', 
      value: 'null', 
      type: 'null', 
      path: prefix || 'root' 
    });
  } else if (Array.isArray(obj)) {
    // Add the array itself as a field
    fields.push({ 
      name: prefix || 'root', 
      value: JSON.stringify(obj), 
      type: 'array', 
      path: prefix || 'root' 
    });
    // Recursively flatten array elements
    obj.forEach((item, i) => {
      fields.push(...flattenObject(item, `${prefix}[${i}]`));
    });
  } else if (typeof obj === 'object') {
    // Recursively flatten object properties
    for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
      const path = prefix ? `${prefix}.${key}` : key;
      fields.push(...flattenObject(value, path));
    }
  } else {
    // Primitive value (string, number, boolean)
    const type = typeof obj as 'string' | 'number' | 'boolean';
    fields.push({ 
      name: prefix, 
      value: String(obj), 
      type, 
      path: prefix 
    });
  }
  
  return fields;
}

/**
 * Parses CSV log input.
 * First line is treated as headers, subsequent lines as data.
 * 
 * @param input - CSV string
 * @returns Array of parsed log fields
 */
function parseCsvLog(input: string): ParsedLogField[] {
  const lines = input.trim().split('\n').filter(line => line.trim());
  
  if (lines.length === 0) {
    return [];
  }
  
  const fields: ParsedLogField[] = [];
  
  // Parse header line
  const headers = parseCSVLine(lines[0]);
  
  // If only header line, return headers as fields with empty values
  if (lines.length === 1) {
    headers.forEach((header, i) => {
      fields.push({
        name: header || `column_${i}`,
        value: '',
        type: 'string',
        path: header || `column_${i}`,
      });
    });
    return fields;
  }
  
  // Parse first data line to get sample values
  const values = parseCSVLine(lines[1]);
  
  headers.forEach((header, i) => {
    const value = values[i] || '';
    const fieldName = header || `column_${i}`;
    
    fields.push({
      name: fieldName,
      value,
      type: inferType(value),
      path: fieldName,
    });
  });
  
  return fields;
}

/**
 * Parses a single CSV line, handling quoted values.
 * 
 * @param line - CSV line to parse
 * @returns Array of field values
 */
function parseCSVLine(line: string): string[] {
  const result: string[] = [];
  let current = '';
  let inQuotes = false;
  
  for (let i = 0; i < line.length; i++) {
    const char = line[i];
    const nextChar = line[i + 1];
    
    if (inQuotes) {
      if (char === '"') {
        if (nextChar === '"') {
          // Escaped quote
          current += '"';
          i++;
        } else {
          // End of quoted field
          inQuotes = false;
        }
      } else {
        current += char;
      }
    } else {
      if (char === '"') {
        inQuotes = true;
      } else if (char === ',') {
        result.push(current.trim());
        current = '';
      } else {
        current += char;
      }
    }
  }
  
  // Add the last field
  result.push(current.trim());
  
  return result;
}

/**
 * Parses syslog format log input.
 * Supports BSD syslog and ISO 8601 timestamp formats.
 * 
 * @param input - Syslog string
 * @returns Array of parsed log fields
 */
function parseSyslogLog(input: string): ParsedLogField[] {
  const trimmed = input.trim();
  
  if (!trimmed) {
    return [];
  }
  
  const fields: ParsedLogField[] = [];
  
  // Try BSD syslog format: "Jan  1 00:00:00 hostname process[pid]: message"
  const bsdMatch = trimmed.match(
    /^([A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(\S+)\s+(\S+?)(?:\[(\d+)\])?:\s*(.*)$/
  );
  
  if (bsdMatch) {
    const [, timestamp, hostname, process, pid, message] = bsdMatch;
    
    fields.push(
      { name: 'timestamp', value: timestamp, type: 'string', path: 'timestamp' },
      { name: 'hostname', value: hostname, type: 'string', path: 'hostname' },
      { name: 'process', value: process, type: 'string', path: 'process' }
    );
    
    if (pid) {
      fields.push({ name: 'pid', value: pid, type: 'number', path: 'pid' });
    }
    
    fields.push({ name: 'message', value: message, type: 'string', path: 'message' });
    
    // Try to parse key-value pairs from the message
    const kvFields = parseKvString(message);
    kvFields.forEach(kv => {
      fields.push({
        ...kv,
        path: `message.${kv.path}`,
      });
    });
    
    return fields;
  }
  
  // Try ISO 8601 format: "2024-01-01T00:00:00Z hostname process: message"
  const isoMatch = trimmed.match(
    /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)\s+(\S+)\s+(\S+?)(?:\[(\d+)\])?:\s*(.*)$/
  );
  
  if (isoMatch) {
    const [, timestamp, hostname, process, pid, message] = isoMatch;
    
    fields.push(
      { name: 'timestamp', value: timestamp, type: 'string', path: 'timestamp' },
      { name: 'hostname', value: hostname, type: 'string', path: 'hostname' },
      { name: 'process', value: process, type: 'string', path: 'process' }
    );
    
    if (pid) {
      fields.push({ name: 'pid', value: pid, type: 'number', path: 'pid' });
    }
    
    fields.push({ name: 'message', value: message, type: 'string', path: 'message' });
    
    // Try to parse key-value pairs from the message
    const kvFields = parseKvString(message);
    kvFields.forEach(kv => {
      fields.push({
        ...kv,
        path: `message.${kv.path}`,
      });
    });
    
    return fields;
  }
  
  // Fallback: just return the whole line as a message field
  fields.push({ name: 'raw', value: trimmed, type: 'string', path: 'raw' });
  
  return fields;
}

/**
 * Parses key-value format log input.
 * Supports formats like: key1=value1 key2="quoted value" key3=value3
 * 
 * @param input - Key-value string
 * @returns Array of parsed log fields
 */
function parseKvLog(input: string): ParsedLogField[] {
  const trimmed = input.trim();
  
  if (!trimmed) {
    return [];
  }
  
  return parseKvString(trimmed);
}

/**
 * Parses a key-value string into fields.
 * Handles quoted values and various separators.
 * 
 * @param input - Key-value string
 * @returns Array of parsed log fields
 */
function parseKvString(input: string): ParsedLogField[] {
  const fields: ParsedLogField[] = [];
  
  // Match key=value patterns, handling quoted values
  // Supports: key=value, key="quoted value", key='quoted value'
  const kvRegex = /(\w+)=(?:"([^"]*?)"|'([^']*?)'|(\S+))/g;
  
  let match;
  while ((match = kvRegex.exec(input)) !== null) {
    const key = match[1];
    // Value is in one of the capture groups: quoted double, quoted single, or unquoted
    const value = match[2] ?? match[3] ?? match[4] ?? '';
    
    fields.push({
      name: key,
      value,
      type: inferType(value),
      path: key,
    });
  }
  
  return fields;
}

/**
 * Infers the type of a string value.
 * 
 * @param value - String value to analyze
 * @returns Inferred type
 */
function inferType(value: string): 'string' | 'number' | 'boolean' | 'null' {
  if (value === '' || value === 'null' || value === 'NULL') {
    return 'null';
  }
  
  if (value === 'true' || value === 'false' || value === 'TRUE' || value === 'FALSE') {
    return 'boolean';
  }
  
  // Check if it's a number (integer or float)
  if (/^-?\d+(\.\d+)?$/.test(value)) {
    return 'number';
  }
  
  return 'string';
}
