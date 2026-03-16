/**
 * Unit tests for log parser utilities.
 * Tests detectLogFormat, parseLogInput, and flattenObject functions.
 */

import { describe, it, expect } from 'vitest';
import { 
  detectLogFormat, 
  parseLogInput, 
  flattenObject,
} from './logParser';

describe('detectLogFormat', () => {
  describe('JSON detection', () => {
    it('detects JSON object', () => {
      const input = '{"key": "value", "number": 42}';
      expect(detectLogFormat(input)).toBe('json');
    });

    it('detects JSON array', () => {
      const input = '[{"key": "value"}, {"key": "value2"}]';
      expect(detectLogFormat(input)).toBe('json');
    });

    it('detects JSON with whitespace', () => {
      const input = '  { "key": "value" }  ';
      expect(detectLogFormat(input)).toBe('json');
    });

    it('detects nested JSON', () => {
      const input = '{"outer": {"inner": {"deep": "value"}}}';
      expect(detectLogFormat(input)).toBe('json');
    });
  });

  describe('CSV detection', () => {
    it('detects CSV with header and data', () => {
      const input = 'name,age,city\nJohn,30,NYC';
      expect(detectLogFormat(input)).toBe('csv');
    });

    it('detects CSV with multiple data rows', () => {
      const input = 'col1,col2,col3\nval1,val2,val3\nval4,val5,val6';
      expect(detectLogFormat(input)).toBe('csv');
    });

    it('does not detect single line as CSV', () => {
      const input = 'name,age,city';
      expect(detectLogFormat(input)).not.toBe('csv');
    });

    it('does not detect inconsistent comma counts as CSV', () => {
      const input = 'name,age,city\nJohn,30';
      expect(detectLogFormat(input)).not.toBe('csv');
    });
  });

  describe('Syslog detection', () => {
    it('detects BSD syslog format', () => {
      const input = 'Jan  1 00:00:00 hostname process[123]: message';
      expect(detectLogFormat(input)).toBe('syslog');
    });

    it('detects BSD syslog with double-digit day', () => {
      const input = 'Dec 15 14:30:45 server sshd[1234]: Connection from 192.168.1.1';
      expect(detectLogFormat(input)).toBe('syslog');
    });

    it('detects ISO 8601 syslog format', () => {
      const input = '2024-01-15T14:30:45Z hostname process: message';
      expect(detectLogFormat(input)).toBe('syslog');
    });

    it('detects ISO 8601 with timezone offset', () => {
      const input = '2024-01-15T14:30:45+05:00 hostname process: message';
      expect(detectLogFormat(input)).toBe('syslog');
    });
  });

  describe('Key-value detection', () => {
    it('detects simple key=value format', () => {
      const input = 'user=admin action=login status=success';
      expect(detectLogFormat(input)).toBe('kv');
    });

    it('detects key=value with quoted values', () => {
      const input = 'user="john doe" action=login';
      expect(detectLogFormat(input)).toBe('kv');
    });
  });

  describe('Default fallback', () => {
    it('returns json for empty input', () => {
      expect(detectLogFormat('')).toBe('json');
    });

    it('returns json for whitespace-only input', () => {
      expect(detectLogFormat('   ')).toBe('json');
    });

    it('returns json for unrecognized format', () => {
      const input = 'some random text without patterns';
      expect(detectLogFormat(input)).toBe('json');
    });
  });
});

describe('parseLogInput', () => {
  describe('JSON parsing', () => {
    it('parses simple JSON object', () => {
      const input = '{"name": "test", "value": 42}';
      const fields = parseLogInput(input, 'json');
      
      expect(fields).toContainEqual({
        name: 'name',
        value: 'test',
        type: 'string',
        path: 'name',
      });
      expect(fields).toContainEqual({
        name: 'value',
        value: '42',
        type: 'number',
        path: 'value',
      });
    });

    it('parses nested JSON object', () => {
      const input = '{"outer": {"inner": "value"}}';
      const fields = parseLogInput(input, 'json');
      
      expect(fields).toContainEqual({
        name: 'outer.inner',
        value: 'value',
        type: 'string',
        path: 'outer.inner',
      });
    });

    it('parses JSON with null value', () => {
      const input = '{"key": null}';
      const fields = parseLogInput(input, 'json');
      
      expect(fields).toContainEqual({
        name: 'key',
        value: 'null',
        type: 'null',
        path: 'key',
      });
    });

    it('parses JSON with boolean values', () => {
      const input = '{"active": true, "deleted": false}';
      const fields = parseLogInput(input, 'json');
      
      expect(fields).toContainEqual({
        name: 'active',
        value: 'true',
        type: 'boolean',
        path: 'active',
      });
      expect(fields).toContainEqual({
        name: 'deleted',
        value: 'false',
        type: 'boolean',
        path: 'deleted',
      });
    });

    it('parses JSON with array', () => {
      const input = '{"items": ["a", "b"]}';
      const fields = parseLogInput(input, 'json');
      
      // Should have the array field
      expect(fields).toContainEqual({
        name: 'items',
        value: '["a","b"]',
        type: 'array',
        path: 'items',
      });
      // Should have array elements
      expect(fields).toContainEqual({
        name: 'items[0]',
        value: 'a',
        type: 'string',
        path: 'items[0]',
      });
      expect(fields).toContainEqual({
        name: 'items[1]',
        value: 'b',
        type: 'string',
        path: 'items[1]',
      });
    });

    it('returns empty array for empty input', () => {
      const fields = parseLogInput('', 'json');
      expect(fields).toEqual([]);
    });
  });

  describe('CSV parsing', () => {
    it('parses CSV with header and data', () => {
      const input = 'name,age,city\nJohn,30,NYC';
      const fields = parseLogInput(input, 'csv');
      
      expect(fields).toContainEqual({
        name: 'name',
        value: 'John',
        type: 'string',
        path: 'name',
      });
      expect(fields).toContainEqual({
        name: 'age',
        value: '30',
        type: 'number',
        path: 'age',
      });
      expect(fields).toContainEqual({
        name: 'city',
        value: 'NYC',
        type: 'string',
        path: 'city',
      });
    });

    it('parses CSV with quoted values', () => {
      const input = 'name,description\nTest,"A value, with comma"';
      const fields = parseLogInput(input, 'csv');
      
      expect(fields).toContainEqual({
        name: 'description',
        value: 'A value, with comma',
        type: 'string',
        path: 'description',
      });
    });

    it('parses CSV header-only', () => {
      const input = 'col1,col2,col3';
      const fields = parseLogInput(input, 'csv');
      
      expect(fields).toHaveLength(3);
      expect(fields[0].name).toBe('col1');
      expect(fields[0].value).toBe('');
    });

    it('returns empty array for empty input', () => {
      const fields = parseLogInput('', 'csv');
      expect(fields).toEqual([]);
    });
  });

  describe('Syslog parsing', () => {
    it('parses BSD syslog format', () => {
      const input = 'Jan  1 12:30:45 myhost sshd[1234]: Connection from 192.168.1.1';
      const fields = parseLogInput(input, 'syslog');
      
      expect(fields).toContainEqual({
        name: 'timestamp',
        value: 'Jan  1 12:30:45',
        type: 'string',
        path: 'timestamp',
      });
      expect(fields).toContainEqual({
        name: 'hostname',
        value: 'myhost',
        type: 'string',
        path: 'hostname',
      });
      expect(fields).toContainEqual({
        name: 'process',
        value: 'sshd',
        type: 'string',
        path: 'process',
      });
      expect(fields).toContainEqual({
        name: 'pid',
        value: '1234',
        type: 'number',
        path: 'pid',
      });
    });

    it('parses ISO 8601 syslog format', () => {
      const input = '2024-01-15T14:30:45Z server nginx: GET /api/health';
      const fields = parseLogInput(input, 'syslog');
      
      expect(fields).toContainEqual({
        name: 'timestamp',
        value: '2024-01-15T14:30:45Z',
        type: 'string',
        path: 'timestamp',
      });
      expect(fields).toContainEqual({
        name: 'hostname',
        value: 'server',
        type: 'string',
        path: 'hostname',
      });
    });

    it('extracts key-value pairs from syslog message', () => {
      const input = 'Jan  1 12:30:45 host app[123]: user=admin action=login';
      const fields = parseLogInput(input, 'syslog');
      
      expect(fields).toContainEqual({
        name: 'user',
        value: 'admin',
        type: 'string',
        path: 'message.user',
      });
      expect(fields).toContainEqual({
        name: 'action',
        value: 'login',
        type: 'string',
        path: 'message.action',
      });
    });

    it('returns raw field for unrecognized syslog', () => {
      const input = 'some unstructured log message';
      const fields = parseLogInput(input, 'syslog');
      
      expect(fields).toContainEqual({
        name: 'raw',
        value: 'some unstructured log message',
        type: 'string',
        path: 'raw',
      });
    });
  });

  describe('Key-value parsing', () => {
    it('parses simple key=value pairs', () => {
      const input = 'user=admin action=login status=success';
      const fields = parseLogInput(input, 'kv');
      
      expect(fields).toContainEqual({
        name: 'user',
        value: 'admin',
        type: 'string',
        path: 'user',
      });
      expect(fields).toContainEqual({
        name: 'action',
        value: 'login',
        type: 'string',
        path: 'action',
      });
      expect(fields).toContainEqual({
        name: 'status',
        value: 'success',
        type: 'string',
        path: 'status',
      });
    });

    it('parses double-quoted values', () => {
      const input = 'user="john doe" message="hello world"';
      const fields = parseLogInput(input, 'kv');
      
      expect(fields).toContainEqual({
        name: 'user',
        value: 'john doe',
        type: 'string',
        path: 'user',
      });
    });

    it('parses single-quoted values', () => {
      const input = "user='john doe' action='log in'";
      const fields = parseLogInput(input, 'kv');
      
      expect(fields).toContainEqual({
        name: 'user',
        value: 'john doe',
        type: 'string',
        path: 'user',
      });
    });

    it('infers numeric types', () => {
      const input = 'count=42 rate=3.14';
      const fields = parseLogInput(input, 'kv');
      
      expect(fields).toContainEqual({
        name: 'count',
        value: '42',
        type: 'number',
        path: 'count',
      });
      expect(fields).toContainEqual({
        name: 'rate',
        value: '3.14',
        type: 'number',
        path: 'rate',
      });
    });

    it('infers boolean types', () => {
      const input = 'active=true deleted=false';
      const fields = parseLogInput(input, 'kv');
      
      expect(fields).toContainEqual({
        name: 'active',
        value: 'true',
        type: 'boolean',
        path: 'active',
      });
      expect(fields).toContainEqual({
        name: 'deleted',
        value: 'false',
        type: 'boolean',
        path: 'deleted',
      });
    });

    it('returns empty array for empty input', () => {
      const fields = parseLogInput('', 'kv');
      expect(fields).toEqual([]);
    });
  });
});

describe('flattenObject', () => {
  it('flattens simple object', () => {
    const obj = { a: 'value', b: 42 };
    const fields = flattenObject(obj);
    
    expect(fields).toContainEqual({
      name: 'a',
      value: 'value',
      type: 'string',
      path: 'a',
    });
    expect(fields).toContainEqual({
      name: 'b',
      value: '42',
      type: 'number',
      path: 'b',
    });
  });

  it('flattens nested object', () => {
    const obj = { outer: { inner: 'deep' } };
    const fields = flattenObject(obj);
    
    expect(fields).toContainEqual({
      name: 'outer.inner',
      value: 'deep',
      type: 'string',
      path: 'outer.inner',
    });
  });

  it('flattens deeply nested object', () => {
    const obj = { a: { b: { c: { d: 'value' } } } };
    const fields = flattenObject(obj);
    
    expect(fields).toContainEqual({
      name: 'a.b.c.d',
      value: 'value',
      type: 'string',
      path: 'a.b.c.d',
    });
  });

  it('handles null value', () => {
    const fields = flattenObject(null);
    
    expect(fields).toContainEqual({
      name: 'root',
      value: 'null',
      type: 'null',
      path: 'root',
    });
  });

  it('handles array', () => {
    const obj = ['a', 'b', 'c'];
    const fields = flattenObject(obj);
    
    // Should have the array itself
    expect(fields).toContainEqual({
      name: 'root',
      value: '["a","b","c"]',
      type: 'array',
      path: 'root',
    });
    // Should have array elements
    expect(fields).toContainEqual({
      name: '[0]',
      value: 'a',
      type: 'string',
      path: '[0]',
    });
  });

  it('handles nested array in object', () => {
    const obj = { items: [1, 2, 3] };
    const fields = flattenObject(obj);
    
    expect(fields).toContainEqual({
      name: 'items',
      value: '[1,2,3]',
      type: 'array',
      path: 'items',
    });
    expect(fields).toContainEqual({
      name: 'items[0]',
      value: '1',
      type: 'number',
      path: 'items[0]',
    });
  });

  it('handles boolean values', () => {
    const obj = { active: true, deleted: false };
    const fields = flattenObject(obj);
    
    expect(fields).toContainEqual({
      name: 'active',
      value: 'true',
      type: 'boolean',
      path: 'active',
    });
  });

  it('uses prefix for nested paths', () => {
    const obj = { key: 'value' };
    const fields = flattenObject(obj, 'prefix');
    
    expect(fields).toContainEqual({
      name: 'prefix.key',
      value: 'value',
      type: 'string',
      path: 'prefix.key',
    });
  });
});
