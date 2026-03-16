# OCSF Semantic Layer - Feature Overview

## 🎯 Core Features

### 1. Semantic Modeling
Define business-level security concepts that map to OCSF event classes.

**Benefits:**
- Business-friendly terminology
- Reusable entity definitions
- Consistent metric calculations
- Cross-platform compatibility

**Example:**
```yaml
entities:
  - name: dns_event
    caption: DNS Event
    source_event_classes: [4003]
    attributes:
      - name: query_hostname
        caption: Query Hostname
        synonyms: [domain, fqdn, hostname]
        security_context: "Critical for DGA detection"
```

---

### 2. LLM-Friendly Export 🤖 NEW!

Export semantic models in a compact format optimized for AI agents and natural language queries.

**Features:**
- Synonym mappings for natural language resolution
- Pre-built query templates with natural language patterns
- Security context and threat relevance
- MITRE ATT&CK technique mappings
- Configurable sections and minimal mode

**Output Statistics:**
- Full export: 18.6 KB (~4,760 tokens)
- Minimal export: 13.9 KB (~3,556 tokens)
- 26% size reduction with minimal mode

**Use Cases:**
- Natural language query translation
- AI-assisted threat hunting
- Automated security analytics
- Context for LLM agents

**Example:**
```bash
# Export with DNS security templates
ocsf export-llm-context \
  -m dns-model.yaml \
  -o llm-context.json \
  --include-dns-templates

# Output includes:
# - 27 synonym mappings
# - 12 DNS security query templates
# - 4 computed fields for threat detection
# - MITRE ATT&CK mappings
```

---

### 3. Warehouse Artifact Generation

Generate production-ready data warehouse artifacts for multiple platforms.

**Supported Platforms:**
- Snowflake
- Databricks
- Google BigQuery
- PostgreSQL

**Generated Artifacts:**
- **dbt** - Semantic models, metrics, dimensions, tests
- **Cube.js** - Data cubes with pre-aggregations
- **SQL Views** - Materialized views with partitioning
- **ETL Pipelines** - Observable extraction logic

**Example:**
```bash
ocsf generate \
  -m semantic-model.yaml \
  -o ./output \
  -d snowflake \
  --artifacts dbt,cubejs,views
```

---

### 4. Observable Hot Path Analytics

Extract observables (IOCs) to a dedicated table for fast threat intelligence queries.

**Performance:**
- 10-100x faster than full event scans
- Efficient JOINs with threat feeds
- Reduced query costs

**Supported Observable Types:**
- Type 1: Hostname
- Type 2: IP Address
- Type 5: Email Address
- Type 10: User Name
- Type 22: Hash
- Type 30: URL

**Example:**
```yaml
observable_config:
  extract_to_table: true
  table_name: dns_observables
  include_types: [1, 2]  # Hostname, IP
```

---

### 5. Visual Representation

Visualize the semantic layer interchange between business concepts and OCSF schema.

**View Modes:**
- **Semantic Only** - Business entities and relationships
- **Physical Only** - OCSF event classes and fields
- **Interchange** - Both layers with mappings
- **Hot/Cold Path** - Data flow visualization

**Export Formats:**
- SVG (vector graphics)
- PNG (raster image)
- JSON (graph data)

**Example:**
```bash
ocsf visualize \
  -m semantic-model.yaml \
  -o graph.svg \
  --view interchange \
  --show-observables
```

---

### 6. Vector-Based Field Mapping

AI-assisted field mapping using semantic embeddings.

**Features:**
- Automatic field similarity detection
- Cross-schema mapping suggestions
- Incremental embedding updates
- Multiple embedding model support

**Example:**
```rust
// Find similar fields
let similar = store.find_similar("user_email", 5)?;
// Returns: ["actor.user.email_addr", "src_endpoint.user.email", ...]
```

---

### 7. Query Translation

Translate semantic queries to SQL with automatic synonym resolution.

**Features:**
- Simple attribute selection
- Metric aggregation
- Hot/cold path routing
- Template-based queries

**Example:**
```bash
# Simple query
ocsf query -m model.yaml "dns_event.query_hostname,source_ip"

# With metrics
ocsf query -m model.yaml \
  '{"entity":"dns_event","metrics":["dns_query_count"],"group_by":["source_ip"]}'
```

---

### 8. Schema-Driven Generation

Auto-generate semantic models from compiled OCSF schemas.

**Features:**
- Category and class filtering
- Automatic metric suggestions
- Dimension inference
- Configurable nesting depth

**Example:**
```bash
ocsf generate-from-schema \
  -s ocsf-compiled-v1.6.0.json \
  -o generated-model.yaml \
  --categories network,iam \
  --include-metrics
```

---

## 🔒 Security Features

### Pre-built DNS Security Query Templates

12 production-ready query templates for DNS threat detection:

1. **DNS Tunneling Detection**
   - High volume queries
   - Unusually long queries
   - TXT record abuse

2. **DGA Detection**
   - NXDOMAIN spikes
   - Random domain patterns

3. **C2 Beaconing**
   - Regular interval queries
   - Consistent timing patterns

4. **Fast Flux Detection**
   - Rapid IP changes
   - Short TTL values

5. **Threat Intelligence**
   - IOC domain matching
   - IP reputation lookups

6. **Operational Queries**
   - Top queried domains
   - Regional DNS activity
   - New domain tracking

### Computed Fields for Threat Detection

4 computed fields for DNS threat indicators:

1. **domain_entropy** - Shannon entropy (randomness indicator)
2. **subdomain_depth** - Number of subdomain levels
3. **query_length** - Total query string length
4. **is_numeric_heavy** - Percentage of numeric characters

### MITRE ATT&CK Integration

All query templates mapped to MITRE ATT&CK techniques:

- T1071.004 - Application Layer Protocol: DNS
- T1568.002 - Dynamic Resolution: Domain Generation Algorithms
- T1048.003 - Exfiltration Over Alternative Protocol
- And more...

---

## 📊 Performance Characteristics

### Hot Path (Observable Table)
- **Query Time**: 10-100x faster
- **Storage**: 1-5% of full events
- **Use Case**: Threat intel lookups

### Cold Path (Full Events)
- **Query Time**: Full table scan
- **Storage**: 100% of events
- **Use Case**: Complex analytics

### LLM Export
- **Generation**: <100ms
- **Size**: 13.9-18.6 KB
- **Tokens**: 3,556-4,760

---

## 🧪 Testing & Quality

### Test Coverage
- **281 tests** in ocsf-semantic
- **82 tests** in ocsf-warehouse
- Property-based testing with proptest
- Round-trip serialization tests
- SQL generation validation

### Quality Assurance
- Comprehensive unit tests
- Property-based testing for universal correctness
- Integration tests for end-to-end workflows
- Continuous validation against OCSF schema

---

## 🚀 Getting Started

### Quick Start
```bash
# 1. Initialize a new model
ocsf init -o my-model.yaml -n my-analytics

# 2. Validate against OCSF schema
ocsf validate -m my-model.yaml

# 3. Generate warehouse artifacts
ocsf generate -m my-model.yaml -o ./output -d snowflake

# 4. Export for LLM agents
ocsf export-llm-context -m my-model.yaml -o llm-context.json
```

### Example Models
- `demo/dns-semantic-model-enhanced.yaml` - Full DNS analytics with LLM enhancements
- `demo/semantic-model.yaml` - Authentication analytics example

---

## 📚 Documentation

- [README.md](README.md) - Main documentation
- [LLM Export Guide](demo/LLM-EXPORT-README.md) - Detailed LLM export documentation
- [Architecture](demo/ARCHITECTURE.md) - System architecture and data flow
- [Requirements](.kiro/specs/llm-semantic-enhancements/requirements.md) - LLM enhancement requirements
- [Design](.kiro/specs/llm-semantic-enhancements/design.md) - LLM enhancement design

---

## 🎯 Use Cases

### For Security Data Engineers
✅ Map security data to OCSF standard  
✅ Generate warehouse artifacts automatically  
✅ Validate semantic models against OCSF schema  
✅ Create reusable entity definitions  

### For Security Analysts
✅ Query OCSF data using business terms  
✅ Use pre-built threat detection templates  
✅ Natural language query translation via LLM  
✅ Fast threat intelligence lookups  

### For Threat Intelligence Teams
✅ Fast observable lookups via hot path  
✅ IOC matching against threat feeds  
✅ MITRE ATT&CK technique mapping  
✅ Automated threat hunting queries  

### For Data Architects
✅ Design security data warehouses  
✅ Generate dbt and Cube.js artifacts  
✅ Visualize semantic layer architecture  
✅ Multi-platform deployment support  

---

## 🔄 Workflow

```
1. Define Semantic Model (YAML)
         ↓
2. Validate Against OCSF Schema
         ↓
3. Generate Warehouse Artifacts
         ↓
4. Export for LLM Agents (NEW!)
         ↓
5. Deploy to Data Warehouse
         ↓
6. Query Using Business Terms
```

---

## ✨ What's New in Latest Release

### LLM-Friendly Semantic Layer Enhancements

✅ **Synonym Resolution** - 27 synonym mappings for natural language queries  
✅ **Query Templates** - 12 pre-built DNS security query templates  
✅ **Computed Fields** - 4 threat detection computed fields  
✅ **Threat Intel Integration** - Pre-built IOC matching joins  
✅ **LLM Export Format** - Compact JSON/YAML optimized for AI agents  
✅ **Security Context** - Threat relevance and MITRE ATT&CK mappings  
✅ **CLI Command** - `export-llm-context` with multiple output formats  
✅ **Enhanced DNS Model** - Complete example with all LLM enhancements  

**Test Results:**
- All 281 tests passing ✅
- Full OCSF v1.6.0 support ✅
- Production ready ✅

---

## 🤝 Contributing

Contributions welcome! See [README.md](README.md) for guidelines.

## 📄 License

MIT License - see LICENSE file for details
