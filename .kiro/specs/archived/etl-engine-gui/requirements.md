# Requirements Document

## Introduction

The ETL Engine GUI adds a React-based dashboard to the existing OCSF Semantic Model Editor (editor-ui). It provides a visual interface for managing ETL jobs that transform raw security log data into OCSF-normalized Delta Lake tables. The GUI communicates with the ETL Engine backend (Rust/Axum on port 3030) and integrates as a new "ETL" tab alongside the existing Schema, Entities, Metrics, Validation, Index, Architecture, and Catalog tabs.

## Glossary

- **ETL_Dashboard**: The top-level React component rendered when the "ETL" tab is active, containing the job list, job creation, and job detail views.
- **Job_List**: The component that displays all ETL jobs in a table with status badges and auto-refresh.
- **Job_Creation_Form**: The form component for submitting a new ETL job, including plugin name, OCSF class UID, source configuration, field mappings, and Delta Lake URI.
- **Job_Detail_View**: The component that displays a single job's metadata, lifecycle action buttons, generated artifacts, and streaming logs.
- **Mapping_Builder**: The sub-component within Job_Creation_Form that allows adding, removing, and editing source-to-OCSF field mapping rows with confidence sliders and optional transforms.
- **Artifact_Viewer**: The component that displays generated artifact files (Go, SQL, YAML) with syntax highlighting.
- **Log_Panel**: The terminal-style component that streams job logs via Server-Sent Events (SSE).
- **Health_Indicator**: A small status badge in the header showing whether the ETL Engine backend is reachable.
- **Lifecycle_Actions**: The set of buttons (Generate, Compile, Test, Run, Stop) that trigger job state transitions via POST endpoints.
- **ETL_API_Client**: The module containing fetch functions for all ETL Engine REST endpoints, with a configurable base URL.
- **Source_Config_Selector**: The form sub-component for choosing and configuring the source type (File, TCP, SQS, Kafka).

## Requirements

### Requirement 1: ETL Tab Integration

**User Story:** As a security data engineer, I want an "ETL" tab in the editor-ui navigation, so that I can access ETL job management alongside other OCSF tools.

#### Acceptance Criteria

1. THE ETL_Dashboard SHALL render as a new tab labeled "ETL" in the App.tsx tab navigation bar.
2. WHEN the user clicks the "ETL" tab, THE ETL_Dashboard SHALL display the Job_List as the default view.
3. THE ETL_Dashboard SHALL follow the existing editor-ui component conventions: dedicated folder with ComponentName.tsx, ComponentName.css, and index.ts files.
4. THE ETL_Dashboard SHALL use CSS modules (plain CSS files) consistent with the existing editor-ui styling approach.

### Requirement 2: ETL API Client

**User Story:** As a developer, I want a configurable API client for the ETL Engine backend, so that the GUI can communicate with the ETL server running on a separate port.

#### Acceptance Criteria

1. THE ETL_API_Client SHALL use a configurable base URL defaulting to `http://localhost:3030`.
2. THE ETL_API_Client SHALL expose typed fetch functions for each ETL Engine endpoint: health check, list jobs, create job, get job detail, generate, compile, test, run, stop, get logs, list catalog entries, and create job from catalog.
3. THE ETL_API_Client SHALL provide TanStack React Query hooks wrapping each fetch function with appropriate cache keys and stale times.
4. WHEN the ETL Engine backend returns an HTTP error status, THE ETL_API_Client SHALL parse the `error` field from the JSON response body and throw a descriptive Error.
5. THE ETL_API_Client SHALL set the `Content-Type` header to `application/json` for all POST requests that include a JSON body.

### Requirement 3: Health Indicator

**User Story:** As a security data engineer, I want to see whether the ETL Engine backend is reachable, so that I know if the ETL features are available.

#### Acceptance Criteria

1. WHEN the "ETL" tab is active, THE Health_Indicator SHALL poll the GET `/api/health` endpoint every 30 seconds.
2. WHEN the health check returns HTTP 200, THE Health_Indicator SHALL display a green connected status.
3. WHEN the health check fails or times out, THE Health_Indicator SHALL display a red disconnected status.
4. THE Health_Indicator SHALL render within the ETL_Dashboard header area.

### Requirement 4: Job List Dashboard

**User Story:** As a security data engineer, I want to see all ETL jobs with their current statuses, so that I can monitor pipeline progress at a glance.

#### Acceptance Criteria

1. THE Job_List SHALL display a table with columns: Plugin Name, OCSF Class UID, Status, and Actions.
2. THE Job_List SHALL auto-refresh the job list by polling GET `/api/jobs` every 5 seconds.
3. THE Job_List SHALL display the job status using color-coded badges: gray for Pending, blue for Generating/Compiling/Testing, green for Running/Completed, and red for Failed.
4. WHEN the user clicks a job row, THE Job_List SHALL navigate to the Job_Detail_View for that job.
5. WHEN no jobs exist, THE Job_List SHALL display an empty state message with a prompt to create a new job.
6. THE Job_List SHALL include a "New Job" button that opens the Job_Creation_Form.

### Requirement 5: Job Creation Form

**User Story:** As a security data engineer, I want to create new ETL jobs by specifying plugin name, OCSF class, source configuration, field mappings, and Delta Lake URI, so that I can set up new data pipelines.

#### Acceptance Criteria

1. THE Job_Creation_Form SHALL include input fields for: plugin name (text), OCSF class UID (number), OCSF version (text, defaulting to "1.3.0"), and Delta Lake URI (text).
2. THE Job_Creation_Form SHALL include the Source_Config_Selector for choosing source type (File, TCP, SQS, Kafka) and entering type-specific parameters.
3. WHEN the source type is "File", THE Source_Config_Selector SHALL display a path input field.
4. WHEN the source type is "TCP", THE Source_Config_Selector SHALL display a bind address input field.
5. WHEN the source type is "SQS", THE Source_Config_Selector SHALL display queue URL and region input fields.
6. WHEN the source type is "Kafka", THE Source_Config_Selector SHALL display brokers (comma-separated) and topic input fields.
7. THE Job_Creation_Form SHALL include the Mapping_Builder for defining field mappings.
8. THE Job_Creation_Form SHALL generate a UUID for the `job_id` field on the client side before submission.
9. WHEN the user submits a valid form, THE Job_Creation_Form SHALL POST the EtlJob JSON to `/api/jobs` and navigate to the Job_Detail_View for the created job.
10. WHEN the POST request fails, THE Job_Creation_Form SHALL display the error message returned by the ETL_API_Client.
11. THE Job_Creation_Form SHALL validate that plugin name, at least one mapping, source configuration, and Delta Lake URI are provided before allowing submission.

### Requirement 6: Mapping Builder

**User Story:** As a security data engineer, I want to visually build source-to-OCSF field mappings with confidence scores and optional transforms, so that I can define how raw fields map to OCSF paths.

#### Acceptance Criteria

1. THE Mapping_Builder SHALL display each mapping as a row with: source field (text input), target OCSF path (text input), confidence (range slider 0.0–1.0 with numeric display), and optional transform (dropdown: None, Lower, Upper, Trim, Cast).
2. WHEN the user selects "Cast" as the transform, THE Mapping_Builder SHALL display a secondary dropdown for the SQL type (Integer, BigInt, Float, Double, Boolean, String, Timestamp, Date).
3. THE Mapping_Builder SHALL include an "Add Mapping" button that appends a new empty mapping row.
4. THE Mapping_Builder SHALL include a remove button on each mapping row that deletes that row.
5. THE Mapping_Builder SHALL default the confidence slider to 0.95 for new mapping rows.

### Requirement 7: Job Detail View

**User Story:** As a security data engineer, I want to see full details of an ETL job including its status, artifacts, and logs, so that I can monitor and control the pipeline lifecycle.

#### Acceptance Criteria

1. THE Job_Detail_View SHALL display the job metadata: job ID, plugin name, OCSF class UID, OCSF version, source configuration summary, and Delta Lake URI.
2. THE Job_Detail_View SHALL display the current job status with the same color-coded badge used in the Job_List.
3. THE Job_Detail_View SHALL poll GET `/api/jobs/:id` every 3 seconds to refresh status and artifact list.
4. THE Job_Detail_View SHALL include a "Back to Jobs" button that returns to the Job_List.

### Requirement 8: Lifecycle Action Buttons

**User Story:** As a security data engineer, I want action buttons that follow the job lifecycle state machine, so that I can advance the pipeline through Generate → Compile → Test → Run → Stop steps.

#### Acceptance Criteria

1. THE Lifecycle_Actions SHALL display buttons for: Generate, Compile, Test, Run, and Stop.
2. WHEN the job status is "Pending", THE Lifecycle_Actions SHALL enable only the "Generate" button.
3. WHEN the job status is "Compiling", THE Lifecycle_Actions SHALL enable only the "Compile" button.
4. WHEN the job status is "Testing", THE Lifecycle_Actions SHALL enable only the "Test" button.
5. WHEN the job status is "Running", THE Lifecycle_Actions SHALL enable only the "Stop" button.
6. WHEN the job status is "Completed" or starts with "Failed", THE Lifecycle_Actions SHALL disable all action buttons.
7. WHEN the user clicks an enabled action button, THE Lifecycle_Actions SHALL POST to the corresponding endpoint (`/api/jobs/:id/generate`, `/compile`, `/test`, `/run`, or `/stop`) and display a loading spinner on that button until the response arrives.
8. WHEN an action POST returns an error, THE Lifecycle_Actions SHALL display the error message below the action buttons.

### Requirement 9: Artifact Viewer

**User Story:** As a security data engineer, I want to preview generated artifacts with syntax highlighting, so that I can review the Go plugin code, SQL schemas, and YAML configurations before running the pipeline.

#### Acceptance Criteria

1. THE Artifact_Viewer SHALL list all artifact file paths returned in the `artifact_files` array from the job detail response.
2. WHEN the user clicks an artifact file name, THE Artifact_Viewer SHALL fetch the file content and display it in a read-only code panel.
3. THE Artifact_Viewer SHALL apply syntax highlighting based on file extension: `.go` as Go, `.sql` as SQL, `.yaml`/`.yml` as YAML, `.json` as JSON, and `.mod` as Go module.
4. WHEN no artifacts are available, THE Artifact_Viewer SHALL display a message indicating artifacts will appear after the Generate step.

### Requirement 10: Log Streaming Panel

**User Story:** As a security data engineer, I want to see real-time logs from the ETL pipeline in a terminal-like panel, so that I can debug issues and monitor progress.

#### Acceptance Criteria

1. THE Log_Panel SHALL connect to the SSE endpoint GET `/api/jobs/:id/logs` and display each received event as a new line.
2. THE Log_Panel SHALL render with a dark background, monospace font, and auto-scroll to the latest line.
3. WHEN the SSE connection fails or the endpoint returns an error, THE Log_Panel SHALL display a connection error message.
4. THE Log_Panel SHALL include a "Clear" button that clears the displayed log lines in the UI without affecting the server-side log buffer.
5. WHEN the job status is "Pending", THE Log_Panel SHALL display a placeholder message indicating no logs are available yet.

### Requirement 11: Catalog Integration

**User Story:** As a security data engineer, I want to create ETL jobs from existing catalog entries, so that I can quickly set up pipelines for pre-defined entity mappings.

#### Acceptance Criteria

1. THE Job_List SHALL include a "From Catalog" button that opens a catalog entry selector.
2. WHEN the user selects a catalog entry, THE ETL_Dashboard SHALL POST to `/api/jobs/from-catalog/:entry_id` to create a new job.
3. WHEN the catalog job creation succeeds, THE ETL_Dashboard SHALL navigate to the Job_Detail_View for the newly created job.
4. WHEN the catalog job creation fails, THE ETL_Dashboard SHALL display the error message from the response.
5. THE catalog entry selector SHALL fetch entries from GET `/api/catalog/entries` and display them in a selectable list with entry name and OCSF version.

### Requirement 12: Error Handling

**User Story:** As a security data engineer, I want clear error feedback when API calls fail, so that I can understand and resolve issues.

#### Acceptance Criteria

1. WHEN any ETL API request returns a non-2xx HTTP status, THE ETL_Dashboard SHALL display the error message in a visible error banner within the relevant component.
2. WHEN a network error occurs (backend unreachable), THE ETL_Dashboard SHALL display a "Cannot connect to ETL Engine" message.
3. IF the ETL Engine backend is unreachable on initial load, THEN THE ETL_Dashboard SHALL display a configuration hint showing the expected base URL.
