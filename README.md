# OCSF Semantic Layer

A semantic abstraction layer for the [Open Cybersecurity Schema Framework (OCSF)](https://schema.ocsf.io/) that bridges physical schema and business-level security concepts, with a visual editor and semantic index for data lineage tracking.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue)
![React](https://img.shields.io/badge/react-18%2B-61dafb)
![License](https://img.shields.io/badge/license-MIT-blue)

## Overview

The OCSF Semantic Layer provides a business-friendly abstraction on top of OCSF event data, enabling:

- **Visual Editor** - Web-based UI for building semantic models with drag-and-drop field mapping
- **Semantic Index** - Track table registrations, data lineage, and detection coverage
- **Entity Definitions** - Map business concepts (User, Device, Threat) to OCSF event classes
- **Vector Embeddings** - AI-assisted field mapping using semantic similarity
- **Observable Analytics** - Hot path analytics for threat intelligence
- **Warehouse Artifacts** - Generate dbt, Cube.js, and SQL views
- **LLM-Friendly Exports** - Optimized context for AI agents and natural language queries

## Quick Start

### Prerequisites

- Rust 1.75+ ([install](https://rustup.rs/))
- Node.js 18+ ([install](https://nodejs.org/))
- Optional: OpenAI/Anthropic API key for LLM features

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ocsf-semantic-layer.git
cd ocsf-semantic-layer

# Build the Rust backend
cargo build --workspace --release

# Install frontend dependencies
cd editor-ui
npm install
```

### Running the Application

**Terminal 1 - Start the backend server:**
```bash
cargo run -p ocsf-editor
```

**Terminal 2 - Start the frontend dev server:**
```bash
cd editor-ui
npm run dev
```

Open http://localhost:5173 in your browser.

### Load the Example

Click the **Example** button in the header to load a complete DNS Analytics example with:
- Semantic model with entities, attributes, and metrics
- Sample index data with tables and lineage
- Detection coverage with MITRE ATT&CK mappings

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Editor UI (React)                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Schema  │ │ Entities │ │ Metrics  │ │Validation│ │  Index   │  │
│  │ Browser  │ │  Editor  │ │ Builder  │ │  Panel   │ │  Views   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                   │
                              REST API
                                   │
┌─────────────────────────────────────────────────────────────────────┐
│                      Backend Server (Rust)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ ocsf-editor  │  │  ocsf-index  │  │ocsf-semantic │              │
│  │   (Axum)     │  │  (Storage)   │  │   (Model)    │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
                                   │
┌─────────────────────────────────────────────────────────────────────┐
│                         Storage Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │   SQLite     │  │   In-Memory  │  │  OCSF Schema │              │
│  │   Backend    │  │   Backend    │  │   (GitHub)   │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
```

## Features

### 🎨 Visual Editor

The web-based editor provides a complete workflow for building semantic models:

| Tab | Description |
|-----|-------------|
| **Schema** | Browse OCSF schema categories, classes, and attributes |
| **Entities** | Define semantic entities with attributes and relationships |
| **Metrics** | Create aggregation metrics with dimensions and time granularities |
| **Validation** | Real-time validation against OCSF schema |
| **Index** | Manage table registry, lineage, and detection coverage |
| **Architecture** | View system architecture diagram |

### � Index Workflow

The Index tab provides a complete data onboarding workflow:

1. **Import** - Load raw log samples (JSON, CSV, syslog, key-value)
2. **Mapping** - Map source fields to OCSF target fields with transformations
3. **Builder** - Register tables with detection coverage metadata
4. **Tables** - Browse and manage registered tables
5. **Lineage** - Visualize data lineage from source to target
6. **Coverage** - View MITRE ATT&CK detection coverage matrix
7. **Statistics** - Analyze table column statistics

### 📤 Export Options

Export your work in multiple formats:

| Format | Contents |
|--------|----------|
| **Semantic Model (YAML)** | Entities, attributes, metrics, observable config |
| **Index Model (JSON)** | Table registry, source lineage, field lineage |
| **Combined (JSON)** | Both semantic and index data together |

### 🔗 Data Lineage Tracking

Track data flow from source systems to OCSF-normalized tables:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Source    │────▶│   ETL       │────▶│   OCSF      │
│   System    │     │  Pipeline   │     │   Table     │
│  (Zeek)     │     │             │     │(dns_activity)│
└─────────────┘     └─────────────┘     └─────────────┘
       │                                       │
       └───────── Field Lineage ───────────────┘
         query_name → query.hostname
         src_ip → src_endpoint.ip
```

### 🛡️ Detection Coverage

Map your data sources to MITRE ATT&CK techniques:

- **Techniques** - T1071.004 (DNS), T1568.002 (DGA), T1048.003 (Exfiltration)
- **Tactics** - Command & Control, Exfiltration, Initial Access
- **Data Sources** - DNS queries, network traffic, authentication logs
- **Kill Chain Phases** - Reconnaissance through Actions on Objectives

### 🤖 LLM Integration

Configure an LLM provider (Anthropic or OpenAI) to enable:

- Auto-generate entity descriptions
- Suggest synonyms for natural language queries
- Generate security context for attributes
- Recommend sample values

## Project Structure

```
.
├── editor-ui/              # React frontend
│   ├── src/
│   │   ├── components/     # UI components
│   │   │   ├── SchemaBrowser/
│   │   │   ├── EntityEditor/
│   │   │   ├── MetricBuilder/
│   │   │   ├── ValidationPanel/
│   │   │   ├── LineageVisualization/
│   │   │   ├── DetectionCoverage/
│   │   │   ├── TableRegistry/
│   │   │   ├── StatisticsViewer/
│   │   │   ├── LogImport/
│   │   │   ├── MappingBuilder/
│   │   │   └── IndexBuilder/
│   │   ├── api/            # API client
│   │   ├── store/          # Zustand state management
│   │   └── utils/          # Utilities
│   └── package.json
│
├── ocsf-core/              # OCSF schema types and parsing
├── ocsf-semantic/          # Semantic model definitions
├── ocsf-vector/            # Embeddings and vector store
├── ocsf-warehouse/         # Warehouse artifact generation
├── ocsf-viz/               # Visualization engine
├── ocsf-index/             # Semantic index (NEW)
│   ├── src/
│   │   ├── backend/        # Storage backends (SQLite, Memory)
│   │   ├── table_registry.rs
│   │   ├── source_lineage.rs
│   │   ├── field_lineage.rs
│   │   ├── detection_coverage.rs
│   │   └── table_statistics.rs
│   └── Cargo.toml
│
├── ocsf-editor/            # Backend server (Axum)
│   └── src/
│       ├── api/            # REST API handlers
│       └── main.rs
│
├── ocsf-cli/               # Command-line interface
└── demo/                   # Example models and scripts
```

## CLI Commands

### Editor Server

```bash
# Start the editor server (default port 3000)
cargo run -p ocsf-editor

# With custom port
OCSF_EDITOR_PORT=8080 cargo run -p ocsf-editor

# With SQLite persistence
OCSF_INDEX_BACKEND=sqlite OCSF_INDEX_PATH=./index.db cargo run -p ocsf-editor
```

### Schema Operations

```bash
# Ingest OCSF schema from GitHub
cargo run -p ocsf-cli -- ingest --source github --ocsf-version v1.6.0

# Validate semantic model
cargo run -p ocsf-cli -- validate -m model.yaml

# Generate warehouse artifacts
cargo run -p ocsf-cli -- generate -m model.yaml -o ./output -d snowflake
```

### LLM Export

```bash
# Full export with DNS templates
cargo run -p ocsf-cli -- export-llm-context \
  -m demo/dns-semantic-model-enhanced.yaml \
  -o llm-context.json \
  --include-dns-templates

# Minimal export for smaller context windows
cargo run -p ocsf-cli -- export-llm-context \
  -m model.yaml -o llm-context.json --minimal
```

### Visualization

```bash
# Generate SVG visualization
cargo run -p ocsf-cli -- visualize \
  -m semantic-model.yaml \
  -o graph.svg \
  --view interchange \
  --show-observables
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OCSF_EDITOR_PORT` | Backend server port | `3000` |
| `OCSF_INDEX_BACKEND` | Storage backend (`sqlite` or `memory`) | `memory` |
| `OCSF_INDEX_PATH` | SQLite database path | `./ocsf-index.db` |
| `ANTHROPIC_API_KEY` | Anthropic API key for LLM features | - |
| `OPENAI_API_KEY` | OpenAI API key for LLM features | - |

### Index Backend Configuration

**In-Memory (default):**
- Data is not persisted between restarts
- Good for development and testing

**SQLite:**
```bash
OCSF_INDEX_BACKEND=sqlite OCSF_INDEX_PATH=/path/to/index.db cargo run -p ocsf-editor
```
- Data persists across restarts
- Recommended for production use

## API Reference

### Semantic Model API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/schema` | GET | Get OCSF schema tree |
| `/api/validate` | POST | Validate semantic model |
| `/api/generate` | POST | Generate warehouse artifacts |
| `/api/llm/research` | POST | LLM-assisted research |

### Index API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/index/tables` | GET | List registered tables |
| `/api/index/tables` | POST | Register a new table |
| `/api/index/tables/:id` | PUT | Update a table |
| `/api/index/tables/:id` | DELETE | Deactivate a table |
| `/api/index/lineage/source` | GET | Get source lineage records |
| `/api/index/lineage/source` | POST | Record source lineage |
| `/api/index/lineage/field` | GET | Get field lineage records |
| `/api/index/lineage/field` | POST | Record field lineage |
| `/api/index/lineage/graph` | GET | Get lineage as graph |
| `/api/index/coverage/summary` | GET | Get detection coverage summary |
| `/api/index/coverage/matrix` | GET | Get MITRE ATT&CK matrix |
| `/api/index/statistics/:table` | GET | Get table statistics |
| `/api/index/export` | GET | Export index model |
| `/api/index/import` | POST | Import index model |

## Development

### Running Tests

```bash
# Run all Rust tests
cargo test --workspace

# Run frontend tests
cd editor-ui && npm test

# Run property-based tests (may take longer)
cargo test --workspace -- --include-ignored

# Check formatting
cargo fmt --check
cd editor-ui && npm run lint
```

### Building for Production

```bash
# Build Rust backend
cargo build --workspace --release

# Build frontend
cd editor-ui && npm run build

# The built frontend is in editor-ui/dist/
```

## Examples

### DNS Security Analytics

The included example demonstrates a complete DNS analytics semantic model:

**Entities:**
- `dns_event` - DNS queries and responses with threat detection

**Attributes:**
- `query_hostname` - Domain being queried (observable)
- `query_type` - DNS record type (A, AAAA, TXT, etc.)
- `source_ip` - Client IP address (observable)
- `response_code` - DNS response (NOERROR, NXDOMAIN, etc.)

**Metrics:**
- `dns_query_count` - Total DNS queries
- `blocked_dns_rate` - Percentage of blocked queries
- `nxdomain_rate` - NXDOMAIN rate (DGA indicator)

**Detection Coverage:**
- T1071.004 - Application Layer Protocol: DNS
- T1568.002 - Dynamic Resolution: Domain Generation Algorithms
- T1048.003 - Exfiltration Over Alternative Protocol

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test --workspace && cd editor-ui && npm test`
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [OCSF Project](https://schema.ocsf.io/) - Open Cybersecurity Schema Framework
- [MITRE ATT&CK](https://attack.mitre.org/) - Adversarial Tactics, Techniques & Common Knowledge
- Built with Rust 🦀 and React ⚛️
