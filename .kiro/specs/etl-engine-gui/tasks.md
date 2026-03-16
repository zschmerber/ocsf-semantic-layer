# Implementation Plan: ETL Engine GUI

## Overview

Add an "ETL" tab to the existing editor-ui React application that provides a visual dashboard for managing ETL jobs. The implementation proceeds bottom-up: types → API client → pure helpers → leaf components → composite components → App.tsx integration. Each step builds on the previous, with no orphaned code.

## Tasks

- [x] 1. Define TypeScript types and install fast-check
  - [x] 1.1 Create `editor-ui/src/types/etl.ts` with all ETL type definitions
    - Define `HealthResponse`, `EtlJobSummary`, `EtlJobDetail`, `JobActionResponse`, `EtlCatalogEntry`
    - Define `SourceConfig` discriminated union (`FileSourceConfig`, `TcpSourceConfig`, `SqsSourceConfig`, `KafkaSourceConfig`)
    - Define `TransformType`, `CastType`, `FieldMapping`, `CatalogRef`, `EtlJob`, `S3Config`
    - Define UI state types: `EtlViewId`, `JobStatus`
    - Export the `getStatusBadgeClass`, `getMonacoLanguage`, `getEnabledActions`, `generateJobId`, and `validateJobForm` pure helper functions from a new `editor-ui/src/utils/etlHelpers.ts` file
    - _Requirements: 4.3, 5.8, 5.11, 7.2, 8.2–8.6, 9.3_

  - [x] 1.2 Install `fast-check` as a dev dependency
    - Run `cd editor-ui && npm install --save-dev fast-check`
    - _Requirements: (testing infrastructure)_

- [x] 2. Implement ETL API client with React Query hooks
  - [x] 2.1 Create `editor-ui/src/api/etlClient.ts`
    - Implement `etlGet<T>(path)` and `etlPost<T>(path, body?)` fetch helpers with base URL `http://localhost:3030`
    - Parse JSON `error` field on non-2xx responses and throw descriptive `Error`
    - Catch network errors and throw "Cannot connect to ETL Engine" error
    - Set `Content-Type: application/json` header on POST requests with body
    - Implement React Query hooks: `useEtlHealth` (30s refetch), `useEtlJobs` (5s refetch), `useEtlJob(jobId)` (3s refetch), `useCreateEtlJob`, `useJobAction(jobId, action)`, `useEtlCatalogEntries` (30s stale), `useCreateJobFromCatalog`
    - Use query key structure: `['etl', 'health']`, `['etl', 'jobs']`, `['etl', 'job', jobId]`, `['etl', 'catalog']`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [ ]* 2.2 Write property tests for API client error handling
    - **Property 1: Error response parsing** — Generate random HTTP status codes (400–599) and error message strings, mock fetch, verify thrown error includes the message
    - **Validates: Requirements 2.4**

  - [ ]* 2.3 Write property test for Content-Type header on POST
    - **Property 2: Content-Type header on POST requests** — Generate random paths and body objects, call `etlPost`, verify `Content-Type: application/json` header is set
    - **Validates: Requirements 2.5**

- [x] 3. Implement pure helper functions and their property tests
  - [x] 3.1 Create `editor-ui/src/utils/etlHelpers.ts` with pure helper functions
    - `getStatusBadgeClass(status: string): string` — maps status to CSS class per design table
    - `getMonacoLanguage(filePath: string): string` — maps file extension to Monaco language ID
    - `getEnabledActions(status: string): Set<string>` — returns enabled lifecycle action names for a given status
    - `generateJobId(): string` — generates a UUID v4 string
    - `validateJobForm(form): boolean` — validates required fields (plugin name, mappings, source config, delta URI)
    - _Requirements: 4.3, 5.8, 5.11, 7.2, 8.2–8.6, 9.3_

  - [ ]* 3.2 Write property tests for status badge mapping
    - **Property 3: Status badge color mapping** — Generate statuses from known set plus random "Failed: <msg>" strings, verify `getStatusBadgeClass` returns correct CSS class
    - **Validates: Requirements 4.3, 7.2**

  - [ ]* 3.3 Write property test for form validation
    - **Property 4: Form validation rejects incomplete submissions** — Generate random partial form states with randomly omitted required fields, verify `validateJobForm` returns false
    - **Validates: Requirements 5.11**

  - [ ]* 3.4 Write property test for UUID generation
    - **Property 5: Client-side UUID generation** — Generate 100+ UUIDs via `generateJobId`, verify each matches UUID v4 regex
    - **Validates: Requirements 5.8**

  - [ ]* 3.5 Write property test for lifecycle button enablement
    - **Property 9: Lifecycle button enablement follows status state machine** — Generate random statuses, verify `getEnabledActions` returns the correct set of enabled buttons
    - **Validates: Requirements 8.2, 8.3, 8.4, 8.5, 8.6**

  - [ ]* 3.6 Write property test for Monaco language mapping
    - **Property 10: File extension to Monaco language mapping** — Generate random file paths with known and unknown extensions, verify `getMonacoLanguage` returns correct language ID
    - **Validates: Requirements 9.3**

- [x] 4. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement EtlLogPanel component
  - [x] 5.1 Create `editor-ui/src/components/EtlLogPanel/` (EtlLogPanel.tsx, EtlLogPanel.css, index.ts)
    - Connect to SSE endpoint `GET /api/jobs/:id/logs` via native `EventSource`
    - Render log lines in a dark background, monospace font, auto-scrolling container
    - Show placeholder message when `jobStatus` is `"Pending"`
    - Include "Clear" button that empties displayed log lines
    - Display connection error message on SSE failure
    - Close `EventSource` on unmount and on terminal job status
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

  - [ ]* 5.2 Write property test for add mapping operation
    - **Property 6: Add mapping appends row with default confidence** — Generate random mapping arrays (length 0–20), apply add, verify length +1 and last row has confidence 0.95
    - **Validates: Requirements 6.3, 6.5**

  - [ ]* 5.3 Write property test for remove mapping operation
    - **Property 7: Remove mapping deletes the targeted row** — Generate random mapping arrays (length 1–20) and random valid index, apply remove, verify length -1 and correct item removed
    - **Validates: Requirements 6.4**

- [x] 6. Implement EtlArtifactViewer component
  - [x] 6.1 Create `editor-ui/src/components/EtlArtifactViewer/` (EtlArtifactViewer.tsx, EtlArtifactViewer.css, index.ts)
    - List all artifact file paths from `artifactFiles` prop
    - On click, fetch file content and display in read-only Monaco Editor instance
    - Auto-detect Monaco language from file extension using `getMonacoLanguage` helper
    - Show "artifacts will appear after Generate step" message when list is empty
    - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 7. Implement EtlJobForm component
  - [x] 7.1 Create `editor-ui/src/components/EtlJobForm/` (EtlJobForm.tsx, EtlJobForm.css, index.ts)
    - Include input fields: plugin name (text), OCSF class UID (number), OCSF version (text, default "1.3.0"), Delta Lake URI (text)
    - Implement Source_Config_Selector: dropdown for source type (File, TCP, SQS, Kafka) with type-specific fields
      - File: path input; TCP: bind address input; SQS: queue URL + region inputs; Kafka: brokers + topic inputs
    - Implement Mapping_Builder inline: rows with source field, target OCSF path, confidence slider (0.0–1.0, default 0.95), transform dropdown (None, Lower, Upper, Trim, Cast), Cast sub-dropdown for SQL type
    - "Add Mapping" button appends new empty row; remove button on each row
    - Generate UUID v4 for `job_id` on submission using `generateJobId`
    - Validate with `validateJobForm` before submission; display validation errors
    - On successful POST to `/api/jobs`, call `onCreated(jobId)` to navigate to detail view
    - On POST failure, display error message below form actions
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11, 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 8. Implement EtlJobDetail component
  - [x] 8.1 Create `editor-ui/src/components/EtlJobDetail/` (EtlJobDetail.tsx, EtlJobDetail.css, index.ts)
    - Display job metadata: job ID, plugin name, OCSF class UID, OCSF version, source config summary, Delta Lake URI
    - Display status with color-coded badge using `getStatusBadgeClass`
    - Poll `GET /api/jobs/:id` every 3 seconds via `useEtlJob(jobId)` hook
    - Implement lifecycle action buttons (Generate, Compile, Test, Run, Stop) using `getEnabledActions` for enablement
    - Show loading spinner on active action button; display error below buttons on failure
    - Include "Back to Jobs" button calling `onBack`
    - Embed `EtlArtifactViewer` with `artifactFiles` from job detail
    - Embed `EtlLogPanel` with `jobId` and `jobStatus`
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

- [x] 9. Implement EtlJobList component
  - [x] 9.1 Create `editor-ui/src/components/EtlJobList/` (EtlJobList.tsx, EtlJobList.css, index.ts)
    - Render table with columns: Plugin Name, OCSF Class UID, Status (badge), Actions
    - Poll `GET /api/jobs` every 5 seconds via `useEtlJobs()` hook
    - On row click, call `onSelectJob(jobId)`
    - Display empty state message with prompt to create a new job when no jobs exist
    - Include "New Job" button calling `onNewJob`
    - Include "From Catalog" button calling `onFromCatalog`
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 10. Implement EtlDashboard and catalog selector
  - [x] 10.1 Create `editor-ui/src/components/EtlDashboard/` (EtlDashboard.tsx, EtlDashboard.css, index.ts)
    - Manage `etlView` state: `'list' | 'form' | 'detail'` and `selectedJobId`
    - Render Health_Indicator in header: poll `GET /api/health` every 30s via `useEtlHealth`, show green/red badge
    - Display configuration hint banner when health check fails (show expected base URL)
    - Render `EtlJobList`, `EtlJobForm`, or `EtlJobDetail` based on `etlView`
    - Pass navigation callbacks: `onSelectJob`, `onNewJob`, `onBack`, `onFromCatalog`
    - Implement catalog entry selector: fetch from `useEtlCatalogEntries`, display selectable list with name + OCSF version
    - On catalog entry selection, POST to `/api/jobs/from-catalog/:entry_id` via `useCreateJobFromCatalog`, navigate to detail on success, show error on failure
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 3.1, 3.2, 3.3, 3.4, 11.1, 11.2, 11.3, 11.4, 11.5, 12.2, 12.3_

- [x] 11. Integrate ETL tab into App.tsx
  - [x] 11.1 Wire EtlDashboard into App.tsx
    - Add `'etl'` to the `TabId` union type
    - Add `{ id: 'etl', label: 'ETL' }` to the `tabs` array
    - Import `EtlDashboard` and render when `activeTab === 'etl'`
    - Export `EtlDashboard` from `editor-ui/src/components/index.ts`
    - _Requirements: 1.1, 1.2_

- [x] 12. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests use `fast-check` with `vitest` (minimum 100 iterations per property)
- The ETL API client uses direct base URL `http://localhost:3030` (not the Vite proxy)
- Axum 0.7.9 uses `:id` path parameter syntax in endpoint references
- All components follow the existing folder convention: ComponentName/ComponentName.tsx + .css + index.ts
