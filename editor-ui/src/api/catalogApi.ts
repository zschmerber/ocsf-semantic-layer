/**
 * Catalog API client functions.
 */

import { get, post, put, del } from './client';
import type {
  CatalogEntry,
  PluginInfo,
  PushResult,
  PullResult,
  SyncDiff,
  SyncResult,
} from '../types';

export function listEntries(ocsfVersion?: string): Promise<CatalogEntry[]> {
  const q = ocsfVersion ? `?ocsf_version=${encodeURIComponent(ocsfVersion)}` : '';
  return get<CatalogEntry[]>(`/catalog/entries${q}`);
}

export function getEntry(id: number): Promise<CatalogEntry> {
  return get<CatalogEntry>(`/catalog/entries/${id}`);
}

export function createEntry(entry: CatalogEntry): Promise<{ entry: CatalogEntry }> {
  return post<{ entry: CatalogEntry }, CatalogEntry>('/catalog/entries', entry);
}

export function updateEntry(id: number, entry: CatalogEntry): Promise<CatalogEntry> {
  return put<CatalogEntry, CatalogEntry>(`/catalog/entries/${id}`, entry);
}

export function deleteEntry(id: number): Promise<void> {
  return del<void>(`/catalog/entries/${id}`);
}

export function listVersions(): Promise<string[]> {
  return get<string[]>('/catalog/versions');
}

export function listPlugins(): Promise<PluginInfo[]> {
  return get<PluginInfo[]>('/catalog/plugins');
}

export function pushSync(engine: string): Promise<PushResult> {
  return post<PushResult, Record<string, never>>(`/catalog/plugins/${engine}/push`, {});
}

export function pullSync(engine: string): Promise<PullResult> {
  return post<PullResult, Record<string, never>>(`/catalog/plugins/${engine}/pull`, {});
}

export function getDiff(engine: string): Promise<SyncDiff> {
  return get<SyncDiff>(`/catalog/plugins/${engine}/diff`);
}

export function fullSync(engine: string): Promise<SyncResult> {
  return post<SyncResult, Record<string, never>>(`/catalog/plugins/${engine}/sync`, {});
}

export function semanticSearch(q: string, topK = 5): Promise<CatalogEntry[]> {
  return get<CatalogEntry[]>(`/catalog/search?q=${encodeURIComponent(q)}&top_k=${topK}`);
}

export function fieldMappingSuggestions(field: string, topK = 5): Promise<CatalogEntry[]> {
  return get<CatalogEntry[]>(`/catalog/suggest?field=${encodeURIComponent(field)}&top_k=${topK}`);
}
