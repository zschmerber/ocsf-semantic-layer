# OCSF Semantic Layer Dashboard - Access Guide

## 🌐 Web Server Running

**Main Dashboard:** http://localhost:8888/viz/  
**Test Page:** http://localhost:8888/test-load.html

The web server is running from the `demo/` directory on port 8888.

## 📊 Available Data

### DNS Test Event
- **File:** `test-dns-event.json`
- **URL:** http://localhost:8888/test-dns-event.json
- **Content:** AWS Route 53 DNS Activity event (OCSF class 4003)
  - Query: `ip-127-0-0-62.alert.firewall.canary.`
  - Source IP: `10.200.21.100`
  - 4 observables (hostnames, IPs)
  - Firewall alert disposition

### Enhanced DNS Semantic Model
- **File:** `dns-semantic-model-enhanced.yaml`
- **URL:** http://localhost:8888/semantic-model.yaml
- **Features:**
  - 27 synonym mappings for natural language queries
  - 12 pre-built DNS security query templates
  - 4 computed fields for threat detection
  - Hot/cold path analytics configuration

### Graph Visualization Data
- **File:** `output/graph.json`
- **URL:** http://localhost:8888/output/graph.json
- **Content:** 3 nodes, 0 edges
  - DNS Event semantic entity
  - Network Activity category
  - DNS Activity class (4003)

### Generated Warehouse Artifacts
All accessible under http://localhost:8888/output/

#### dbt Artifacts
- `dbt/semantic_manifest.yml` - dbt semantic layer definition
- `dbt/sources.yml` - Source table definitions
- `dbt/models/ocsf_dns_event.sql` - dbt model for DNS events

#### Cube.js Schemas
- `cubejs/DnsEvent.js` - Cube.js schema for DNS analytics
- `cubejs/AuthenticationEvent.js` - Cube.js schema for auth events

#### SQL Views
- `views/v_dns_event.sql` - DNS event view
- `views/v_authentication_event.sql` - Authentication event view
- `views/all_views.sql` - All views combined

#### Table Schemas
- `tables/ocsf_dns.sql` - DNS table schema
- `tables/ocsf_authentication.sql` - Authentication table schema
- `tables/all_tables.sql` - All tables combined

#### Observables & ETL
- `observables/observables_table.sql` - Observable extraction table
- `etl/extract_observables.sql` - ETL pipeline for observables

### LLM Context Exports
- `output/llm-context.json` - Full export (19 KB, ~4,760 tokens)
- `output/llm-context-minimal.json` - Minimal export (14 KB, ~3,556 tokens)

## 🎨 Dashboard Features

### 1. About Tab
- Project overview with architecture diagrams
- System architecture (Mermaid diagram)
- DNS threat detection flow (sequence diagram)
- Query translation visualization
- Feature cards and workflow steps
- CLI commands and examples

### 2. Graph Tab
- Interactive D3.js force-directed graph
- View modes:
  - **All** - Complete semantic ↔ physical interchange
  - **Semantic** - Business entities only
  - **Physical** - OCSF schema only
  - **Hot/Cold** - Data flow paths
- Node statistics and legend
- Click nodes for details
- Export to SVG

### 3. Model Tab
- Semantic entities with attributes
- Metrics and dimensions
- Observable configuration
- Entity relationships

### 4. Artifacts Tab
- File tree browser
- Syntax-highlighted code viewer
- Browse all generated artifacts
- SQL, YAML, JavaScript files

### 5. Query Tab
- Interactive query translator
- Semantic query → SQL translation
- Multiple dialect support:
  - Snowflake
  - Databricks
  - BigQuery
  - PostgreSQL

## 🔧 Troubleshooting

### If the dashboard doesn't show data:

1. **Check the web server is running:**
   ```bash
   curl http://localhost:8888/viz/
   ```

2. **Verify data files are accessible:**
   ```bash
   curl http://localhost:8888/output/graph.json
   curl http://localhost:8888/semantic-model.yaml
   curl http://localhost:8888/test-dns-event.json
   ```

3. **Open the test page to verify data loading:**
   http://localhost:8888/test-load.html

4. **Check browser console for errors:**
   - Open browser DevTools (F12)
   - Check Console tab for JavaScript errors
   - Check Network tab for failed requests

### Common Issues:

- **CORS errors:** The server should allow all origins since it's Python's http.server
- **404 errors:** Make sure you're accessing `/viz/` not `/viz/index.html`
- **Empty panels:** Data loads asynchronously - wait a few seconds after page load
- **Graph not rendering:** Click the "Graph" tab to trigger initialization

## 📝 Next Steps

1. **Open the dashboard:** http://localhost:8888/viz/
2. **Explore the About tab** to understand the system
3. **View the Graph tab** to see the semantic layer visualization
4. **Check the Model tab** to see DNS entities and metrics
5. **Browse Artifacts** to see generated warehouse code
6. **Try the Query tab** to translate semantic queries to SQL

## 🛑 Stop the Server

To stop the web server, use the Kiro process management:
```
Stop process ID 6
```

Or manually:
```bash
lsof -ti:8888 | xargs kill
```
