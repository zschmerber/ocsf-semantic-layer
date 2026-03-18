/**
 * API client for the OCSF Semantic Model Editor.
 * 
 * Provides a centralized HTTP client with error handling and retry logic.
 * 
 * Requirements: 7.1, 7.2, 7.3, 7.4
 */

// ============================================
// Configuration
// ============================================

/** Base URL for API requests - uses Vite proxy in development */
export const API_BASE_URL = '/api';

/** Default timeout for API requests (30 seconds) */
export const DEFAULT_TIMEOUT = 30000;

/** Default retry count for failed requests */
export const DEFAULT_RETRY_COUNT = 3;

/** Delay between retries (exponential backoff base) */
export const RETRY_DELAY_BASE = 1000;

// ============================================
// Error Types
// ============================================

/**
 * API error response structure from the backend.
 */
export interface APIErrorResponse {
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

/**
 * Custom error class for API errors with typed error information.
 */
export class APIError extends Error {
  public readonly status: number;
  public readonly code: string;
  public readonly details?: Record<string, unknown>;

  constructor(
    message: string,
    status: number,
    code: string = 'UNKNOWN_ERROR',
    details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'APIError';
    this.status = status;
    this.code = code;
    this.details = details;
  }

  /**
   * Check if this is a network error (no response from server).
   */
  get isNetworkError(): boolean {
    return this.status === 0;
  }

  /**
   * Check if this is a client error (4xx status).
   */
  get isClientError(): boolean {
    return this.status >= 400 && this.status < 500;
  }

  /**
   * Check if this is a server error (5xx status).
   */
  get isServerError(): boolean {
    return this.status >= 500;
  }

  /**
   * Check if this error is retryable.
   */
  get isRetryable(): boolean {
    // Retry on network errors and server errors, but not client errors
    return this.isNetworkError || this.isServerError;
  }
}

// ============================================
// Request Helpers
// ============================================

/**
 * Sleep for a specified duration.
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Calculate exponential backoff delay.
 */
function getBackoffDelay(attempt: number, baseDelay: number = RETRY_DELAY_BASE): number {
  // Exponential backoff with jitter
  const exponentialDelay = baseDelay * Math.pow(2, attempt);
  const jitter = Math.random() * 0.3 * exponentialDelay;
  return Math.min(exponentialDelay + jitter, 30000); // Cap at 30 seconds
}

/**
 * Parse error response from the API.
 */
async function parseErrorResponse(response: Response): Promise<APIError> {
  try {
    const data: APIErrorResponse = await response.json();
    return new APIError(
      data.error?.message || response.statusText,
      response.status,
      data.error?.code || 'UNKNOWN_ERROR',
      data.error?.details
    );
  } catch {
    // If we can't parse the error response, create a generic error
    return new APIError(
      response.statusText || 'Request failed',
      response.status,
      'PARSE_ERROR'
    );
  }
}

// ============================================
// API Client
// ============================================

export interface RequestOptions extends RequestInit {
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Number of retry attempts for failed requests */
  retries?: number;
  /** Whether to skip retry logic */
  skipRetry?: boolean;
}

/**
 * Make an API request with error handling and retry logic.
 * 
 * @param endpoint - API endpoint path (without base URL)
 * @param options - Fetch options with additional configuration
 * @returns Parsed JSON response
 * @throws APIError on failure
 */
export async function apiRequest<T>(
  endpoint: string,
  options: RequestOptions = {}
): Promise<T> {
  const {
    timeout = DEFAULT_TIMEOUT,
    retries = DEFAULT_RETRY_COUNT,
    skipRetry = false,
    ...fetchOptions
  } = options;

  const url = `${API_BASE_URL}${endpoint}`;
  
  // Set default headers
  const headers = new Headers(fetchOptions.headers);
  if (!headers.has('Content-Type') && fetchOptions.body) {
    headers.set('Content-Type', 'application/json');
  }
  headers.set('Accept', 'application/json');

  const config: RequestInit = {
    ...fetchOptions,
    headers,
  };

  let lastError: APIError | null = null;
  const maxAttempts = skipRetry ? 1 : retries;

  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      // Create abort controller for timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);

      const response = await fetch(url, {
        ...config,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      // Check for successful response
      if (response.ok) {
        // Handle empty responses
        const contentType = response.headers.get('Content-Type');
        if (contentType?.includes('application/json')) {
          return await response.json();
        }
        return {} as T;
      }

      // Parse error response
      lastError = await parseErrorResponse(response);

      // Don't retry client errors (4xx)
      if (lastError.isClientError) {
        throw lastError;
      }

      // Log retry attempt
      if (attempt < maxAttempts - 1) {
        console.warn(
          `API request failed (attempt ${attempt + 1}/${maxAttempts}):`,
          lastError.message
        );
        await sleep(getBackoffDelay(attempt));
      }
    } catch (error) {
      // Handle abort/timeout errors
      if (error instanceof DOMException && error.name === 'AbortError') {
        lastError = new APIError('Request timeout', 0, 'TIMEOUT');
      } else if (error instanceof APIError) {
        lastError = error;
        // Don't retry client errors
        if (error.isClientError) {
          throw error;
        }
      } else if (error instanceof TypeError) {
        // Network error (e.g., server not reachable)
        lastError = new APIError(
          'Network error: Unable to reach server',
          0,
          'NETWORK_ERROR'
        );
      } else {
        lastError = new APIError(
          error instanceof Error ? error.message : 'Unknown error',
          0,
          'UNKNOWN_ERROR'
        );
      }

      // Log retry attempt
      if (attempt < maxAttempts - 1 && lastError.isRetryable) {
        console.warn(
          `API request failed (attempt ${attempt + 1}/${maxAttempts}):`,
          lastError.message
        );
        await sleep(getBackoffDelay(attempt));
      }
    }
  }

  // All retries exhausted
  throw lastError || new APIError('Request failed after retries', 0, 'RETRY_EXHAUSTED');
}

// ============================================
// Convenience Methods
// ============================================

/**
 * Make a GET request.
 */
export function get<T>(endpoint: string, options?: RequestOptions): Promise<T> {
  return apiRequest<T>(endpoint, { ...options, method: 'GET' });
}

/**
 * Make a POST request with JSON body.
 */
export function post<T, B = unknown>(
  endpoint: string,
  body: B,
  options?: RequestOptions
): Promise<T> {
  return apiRequest<T>(endpoint, {
    ...options,
    method: 'POST',
    body: JSON.stringify(body),
  });
}

/**
 * Make a PUT request with JSON body.
 */
export function put<T, B = unknown>(
  endpoint: string,
  body: B,
  options?: RequestOptions
): Promise<T> {
  return apiRequest<T>(endpoint, {
    ...options,
    method: 'PUT',
    body: JSON.stringify(body),
  });
}

/**
 * Make a DELETE request.
 */
export function del<T>(endpoint: string, options?: RequestOptions): Promise<T> {
  return apiRequest<T>(endpoint, { ...options, method: 'DELETE' });
}


// ============================================
// LLM Configuration API
// ============================================

import type { LLMConfigResponse, LLMConfigRequest } from '../types';
import type { InterpretedMapping } from '../types/referenceEvent';

/**
 * Get the current LLM configuration status.
 */
export function getLLMConfig(): Promise<LLMConfigResponse> {
  return get<LLMConfigResponse>('/llm/config');
}

/**
 * Configure the LLM service with an API key.
 */
export function setLLMConfig(config: LLMConfigRequest): Promise<LLMConfigResponse> {
  return post<LLMConfigResponse, LLMConfigRequest>('/llm/config', config);
}

// ============================================
// LLM Mapping Interpretation API
// ============================================

/** Timeout for LLM calls (60 seconds — LLM inference can be slow) */
const LLM_TIMEOUT = 60000;

/**
 * Backend response shape from POST /llm/interpret-mapping.
 * Uses snake_case to match the Rust serde serialization.
 */
interface InterpretMappingBackendResponse {
  mappings: Array<{
    raw_field: string;
    ocsf_field: string;
    transformation: string | null;
    confidence: 'high' | 'medium' | 'low';
    explanation: string | null;
  }>;
  source_system: {
    log_type: string | null;
    vendor: string | null;
  };
  issues: string[];
}

/**
 * Send a reference event and mapping artifact to the LLM for interpretation.
 * Returns a normalized InterpretedMapping with camelCase fields and timestamps.
 *
 * Requirements: 15.1, 15.12
 */
export async function interpretMapping(body: {
  event_json: string;
  mapping_text: string;
}): Promise<InterpretedMapping> {
  const raw = await post<InterpretMappingBackendResponse>(
    '/llm/interpret-mapping',
    body,
    { timeout: LLM_TIMEOUT, skipRetry: true },
  );

  // Normalize snake_case backend response → camelCase frontend type
  return {
    entries: raw.mappings.map((m) => ({
      rawField: m.raw_field,
      ocsfField: m.ocsf_field,
      transformation: m.transformation,
      confidence: (m.confidence.charAt(0).toUpperCase() + m.confidence.slice(1)) as 'High' | 'Medium' | 'Low',
      explanation: m.explanation,
      verificationStatus: 'Unverified' as const, // computed later by the store
      conflictDetail: null,
    })),
    sourceSystem: {
      logType: raw.source_system.log_type,
      vendor: raw.source_system.vendor,
    },
    issues: raw.issues,
    interpretedAt: new Date().toISOString(),
  };
}
