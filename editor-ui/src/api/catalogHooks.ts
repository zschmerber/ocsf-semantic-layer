/**
 * TanStack Query hooks for the Catalog API.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as catalogApi from './catalogApi';
import type { CatalogEntry } from '../types';

export const catalogKeys = {
  all: ['catalog'] as const,
  entries: (version?: string) => ['catalog', 'entries', version] as const,
  entry: (id: number) => ['catalog', 'entry', id] as const,
  versions: () => ['catalog', 'versions'] as const,
  plugins: () => ['catalog', 'plugins'] as const,
  search: (q: string, topK: number) => ['catalog', 'search', q, topK] as const,
};

export function useCatalogEntries(ocsfVersion?: string) {
  return useQuery({
    queryKey: catalogKeys.entries(ocsfVersion),
    queryFn: () => catalogApi.listEntries(ocsfVersion),
    staleTime: 30_000,
  });
}

export function useCatalogVersions() {
  return useQuery({
    queryKey: catalogKeys.versions(),
    queryFn: catalogApi.listVersions,
    staleTime: 60_000,
  });
}

export function useCatalogPlugins() {
  return useQuery({
    queryKey: catalogKeys.plugins(),
    queryFn: catalogApi.listPlugins,
    staleTime: 30_000,
  });
}

export function useCreateCatalogEntry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (entry: CatalogEntry) => catalogApi.createEntry(entry),
    onSuccess: () => qc.invalidateQueries({ queryKey: catalogKeys.all }),
  });
}

export function useUpdateCatalogEntry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, entry }: { id: number; entry: CatalogEntry }) =>
      catalogApi.updateEntry(id, entry),
    onSuccess: () => qc.invalidateQueries({ queryKey: catalogKeys.all }),
  });
}

export function useDeleteCatalogEntry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => catalogApi.deleteEntry(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: catalogKeys.all }),
  });
}

export function usePushSync() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (engine: string) => catalogApi.pushSync(engine),
    onSuccess: () => qc.invalidateQueries({ queryKey: catalogKeys.plugins() }),
  });
}

export function usePullSync() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (engine: string) => catalogApi.pullSync(engine),
    onSuccess: () => qc.invalidateQueries({ queryKey: catalogKeys.all }),
  });
}

export function useGetDiff() {
  return useMutation({
    mutationFn: (engine: string) => catalogApi.getDiff(engine),
  });
}

export function useFullSync() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (engine: string) => catalogApi.fullSync(engine),
    onSuccess: () => qc.invalidateQueries({ queryKey: catalogKeys.all }),
  });
}

export function useCatalogSearch(q: string, topK = 5, enabled = true) {
  return useQuery({
    queryKey: catalogKeys.search(q, topK),
    queryFn: () => catalogApi.semanticSearch(q, topK),
    enabled: enabled && q.trim().length > 0,
    staleTime: 30_000,
  });
}
