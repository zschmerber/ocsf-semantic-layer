/**
 * ETL Engine API client and React Query hooks.
 *
 * Communicates directly with the ETL Engine backend on port 3030,
 * separate from the editor backend proxy on port 8080.
 *
 * Requirements: 2.1, 2.2, 2.3, 2.4, 2.5
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type {
  HealthResponse,
  EtlJobSummary,
  EtlJobDetail,
  JobActionResponse,
  EtlCatalogEntry,
  EtlJob,
} from '../types/etl';

// ============================================
// Configuration
// ============================================

const ETL_BASE_URL = 'http://localhost:3030';

// ============================================
// Fetch Helpers
// ============================================

/**
 * GET request to the ETL Engine backend.
 * Parses JSON error field on non-2xx responses.
 * Throws "Cannot connect to ETL Engine" on network failure.
 */
export async function etlGet<T>(path: string): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${ETL_BASE_URL}${path}`);
  } catch {
    throw new Error('Cannot connect to ETL Engine');
  }

  if (!response.ok) {
    let message = response.statusText;
    try {
      const body = await response.json();
      if (body.error) {
        message = typeof body.error === 'string' ? body.error : JSON.stringify(body.error);
      }
    } catch {
      // use statusText fallback
    }
    throw new Error(message);
  }

  return response.json();
}

/**
 * POST request to the ETL Engine backend.
 * Sets Content-Type: application/json when body is provided.
 * Parses JSON error field on non-2xx responses.
 * Throws "Cannot connect to ETL Engine" on network failure.
 */
export async function etlPost<T>(path: string, body?: unknown): Promise<T> {
  const options: RequestInit = { method: 'POST' };
  if (body !== undefined) {
    options.headers = { 'Content-Type': 'application/json' };
    options.body = JSON.stringify(body);
  }

  let response: Response;
  try {
    response = await fetch(`${ETL_BASE_URL}${path}`, options);
  } catch {
    throw new Error('Cannot connect to ETL Engine');
  }

  if (!response.ok) {
    let message = response.statusText;
    try {
      const body = await response.json();
      if (body.error) {
        message = typeof body.error === 'string' ? body.error : JSON.stringify(body.error);
      }
    } catch {
      // use statusText fallback
    }
    throw new Error(message);
  }

  return response.json();
}

// ============================================
// Query Keys
// ============================================

export const etlKeys = {
  health: ['etl', 'health'] as const,
  jobs: ['etl', 'jobs'] as const,
  job: (jobId: string) => ['etl', 'job', jobId] as const,
  catalog: ['etl', 'catalog'] as const,
};

// ============================================
// React Query Hooks
// ============================================

/** Poll ETL Engine health every 30s. */
export function useEtlHealth() {
  return useQuery({
    queryKey: etlKeys.health,
    queryFn: () => etlGet<HealthResponse>('/api/health'),
    staleTime: 30_000,
    refetchInterval: 30_000,
  });
}

/** Poll all ETL jobs every 5s. */
export function useEtlJobs() {
  return useQuery({
    queryKey: etlKeys.jobs,
    queryFn: () => etlGet<EtlJobSummary[]>('/api/jobs'),
    refetchInterval: 5_000,
  });
}

/** Poll a single ETL job every 3s. */
export function useEtlJob(jobId: string) {
  return useQuery({
    queryKey: etlKeys.job(jobId),
    queryFn: () => etlGet<EtlJobDetail>(`/api/jobs/${jobId}`),
    refetchInterval: 3_000,
  });
}

/** Create a new ETL job. Invalidates the jobs list on success. */
export function useCreateEtlJob() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (job: EtlJob) => etlPost<EtlJobDetail>('/api/jobs', job),
    onSuccess: () => qc.invalidateQueries({ queryKey: etlKeys.jobs }),
  });
}

/** Trigger a lifecycle action on a job. Invalidates job detail and jobs list on success. */
export function useJobAction(jobId: string, action: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => etlPost<JobActionResponse>(`/api/jobs/${jobId}/${action}`),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: etlKeys.job(jobId) });
      qc.invalidateQueries({ queryKey: etlKeys.jobs });
    },
  });
}

/** Fetch ETL catalog entries with 30s stale time. */
export function useEtlCatalogEntries() {
  return useQuery({
    queryKey: etlKeys.catalog,
    queryFn: () => etlGet<EtlCatalogEntry[]>('/api/catalog/entries'),
    staleTime: 30_000,
  });
}

/** Create a job from a catalog entry. Invalidates the jobs list on success. */
export function useCreateJobFromCatalog() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (entryId: number) =>
      etlPost<EtlJobDetail>(`/api/jobs/from-catalog/${entryId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: etlKeys.jobs }),
  });
}
