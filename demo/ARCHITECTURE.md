# OCSF Semantic Layer Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Interface                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │  CLI Tool    │  │  LLM Agent   │  │  BI Tools    │              │
│  │  (ocsf-cli)  │  │  (via API)   │  │  (Tableau)   │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                      Semantic Layer Core                             │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Semantic Model (YAML)                                      │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │    │
│  │  │   Entities   │  │   Metrics    │  │  Templates   │     │    │
│  │  │              │  │              │  │              │     │    │
│  │  │ • dns_event  │  │ • query_cnt  │  │ • tunneling  │     │    │
│  │  │ • auth_event │  │ • failed_pct │  │ • dga_detect │     │    │
│  │  │ • user       │  │ • threat_mtc │  │ • c2_beacon  │     │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │    │
│  └────────────────────────────────────────────────────────────┘    │
│                              ↓                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Processing Modules                                         │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │    │
│  │  │  Validator   │  │  Generator   │  │  Translator  │     │    │
│  │  │              │  │              │  │              │     │    │
│  │  │ • Schema     │  │ • dbt        │  │ • Query→SQL  │     │    │
│  │  │   validation │  │ • Cube.js    │  │ • Synonym    │     │    │
│  │  │ • Field      │  │ • SQL views  │  │   resolution │     │    │
│  │  │   mapping    │  │ • ETL        │  │ • Template   │     │    │
│  │  │              │  │              │  │   matching   │     │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │    │
│  └────────────────────────────────────────────────────────────┘    │
│                              ↓                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  LLM Export Module (NEW!)                                   │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │    │
│  │  │  Synonyms    │  │  Templates   │  │  Security    │     │    │
│  │  │              │  │              │  │  Context     │     │    │
│  │  │ domain→      │  │ • 12 DNS     │  │ • MITRE      │     │    │
│  │  │ query_host   │  │   templates  │  │   ATT&CK     │     │    │
│  │  │ client_ip→   │  │ • Natural    │  │ • Threat     │     │    │
│  │  │ source_ip    │  │   language   │  │   relevance  │     │    │
│  │  │              │  │   patterns   │  │ • Use cases  │     │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │    │
│  │                                                              │    │
│  │  Output: 18.6 KB JSON (~4,760 tokens)                      │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                      OCSF Physical Layer                             │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  OCSF Schema v1.6.0                                         │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │    │
│  │  │ Event Class  │  │ Event Class  │  │ Event Class  │     │    │
│  │  │    4003      │  │    3002      │  │    1001      │     │    │
│  │  │              │  │              │  │              │     │    │
│  │  │ DNS Activity │  │ Auth Event   │  │ Process      │     │    │
│  │  │              │  │              │  │ Activity     │     │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │    │
│  └────────────────────────────────────────────────────────────┘    │
│                              ↓                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Observable Extraction (Hot Path)                           │    │
│  │  ┌──────────────────────────────────────────────────────┐  │    │
│  │  │  ocsf_observables table                               │  │    │
│  │  │  ┌────────────┬──────────┬──────────┬─────────────┐  │  │    │
│  │  │  │ type_id    │ value    │ event_id │ timestamp   │  │  │    │
│  │  │  ├────────────┼──────────┼──────────┼─────────────┤  │  │    │
│  │  │  │ 1 (Host)   │ evil.com │ evt-123  │ 2024-01-22  │  │  │    │
│  │  │  │ 2 (IP)     │ 1.2.3.4  │ evt-124  │ 2024-01-22  │  │  │    │
│  │  │  └────────────┴──────────┴──────────┴─────────────┘  │  │    │
│  │  │                                                        │  │    │
│  │  │  Benefits: 10-100x faster threat intel lookups       │  │    │
│  │  └──────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                      Data Warehouse Layer                            │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Generated Artifacts                                        │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │    │
│  │  │  dbt Models  │  │  Cube.js     │  │  SQL Views   │     │    │
│  │  │              │  │  Schemas     │  │              │     │    │
│  │  │ • Semantic   │  │              │  │ • Materialize│     │    │
│  │  │   models     │  │ • Dimensions │  │   views      │     │    │
│  │  │ • Metrics    │  │ • Measures   │  │ • Partitions │     │    │
│  │  │ • Tests      │  │ • Pre-aggs   │  │ • Indexes    │     │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │    │
│  └────────────────────────────────────────────────────────────┘    │
│                              ↓                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  Target Platforms                                           │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │    │
│  │  │  Snowflake   │  │  Databricks  │  │  BigQuery    │     │    │
│  │  └──────────────┘  └──────────────┘  └──────────────┘     │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Flow

### 1. Model Definition
```yaml
# User defines semantic model
entities:
  - name: dns_event
    attributes:
      - name: query_hostname
        synonyms: [domain, fqdn]
        security_context: "DGA detection"
```

### 2. Validation
```
Semantic Model → Validator → OCSF Schema
                    ↓
              ✓ Field mappings valid
              ✓ Event classes exist
              ✓ Observable types valid
```

### 3. Generation
```
Semantic Model → Generator → Warehouse Artifacts
                    ↓
              • dbt/semantic_manifest.yml
              • cubejs/dns_event.js
              • views/dns_event.sql
              • tables/ocsf_observables.sql
```

### 4. LLM Export (NEW!)
```
Semantic Model → LLM Exporter → Compact JSON
                    ↓
              • Synonyms (27 mappings)
              • Templates (12 queries)
              • Security context
              • MITRE ATT&CK mappings
              
              Output: 18.6 KB (~4,760 tokens)
```

### 5. Query Translation
```
Natural Language → Synonym Resolver → Template Matcher → SQL Generator
                                                              ↓
"Find DNS tunneling"  →  dns_tunneling_high_volume  →  SELECT ...
```

## Module Architecture

### ocsf-core
- OCSF schema parsing
- Event class definitions
- Observable extraction logic

### ocsf-semantic
- Semantic entity types
- Metric definitions
- Query templates (12 DNS templates)
- Computed fields (4 threat detection fields)
- Threat intel integration
- Synonym resolution
- **LLM export format** (NEW!)

### ocsf-vector
- Embedding generation
- Vector similarity search
- Field mapping assistance

### ocsf-warehouse
- dbt artifact generation
- Cube.js schema generation
- SQL view generation
- ETL pipeline generation

### ocsf-viz
- Graph generation
- SVG/PNG export
- Hot/cold path visualization

### ocsf-cli
- Command-line interface
- Model validation
- Artifact generation
- Query translation
- **LLM context export** (NEW!)

## LLM Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      LLM Agent                               │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Context Window                                     │     │
│  │  ┌──────────────────────────────────────────────┐  │     │
│  │  │  System Prompt                                │  │     │
│  │  │  "You are a security analyst assistant..."   │  │     │
│  │  └──────────────────────────────────────────────┘  │     │
│  │  ┌──────────────────────────────────────────────┐  │     │
│  │  │  Semantic Model Context (4,760 tokens)       │  │     │
│  │  │  • Entities with synonyms                    │  │     │
│  │  │  • Query templates                           │  │     │
│  │  │  • Security context                          │  │     │
│  │  │  • MITRE ATT&CK mappings                     │  │     │
│  │  └──────────────────────────────────────────────┘  │     │
│  │  ┌──────────────────────────────────────────────┐  │     │
│  │  │  User Query                                   │  │     │
│  │  │  "Find DNS tunneling in the last 24 hours"  │  │     │
│  │  └──────────────────────────────────────────────┘  │     │
│  └────────────────────────────────────────────────────┘     │
│                         ↓                                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │  LLM Processing                                     │     │
│  │  1. Resolve synonyms: "DNS" → query_hostname       │     │
│  │  2. Match template: dns_tunneling_high_volume      │     │
│  │  3. Generate SQL with parameters                   │     │
│  └────────────────────────────────────────────────────┘     │
│                         ↓                                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Generated SQL                                      │     │
│  │  SELECT source_ip, COUNT(*) as query_count         │     │
│  │  FROM dns_event                                     │     │
│  │  WHERE time >= DATEADD(hour, -24, CURRENT_TS())    │     │
│  │  GROUP BY source_ip                                 │     │
│  │  HAVING query_count > 10000                         │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Performance Characteristics

### Hot Path (Observable Table)
- **Query Time**: 10-100x faster than full event scan
- **Use Case**: Threat intel lookups, IOC matching
- **Storage**: ~1-5% of full event data

### Cold Path (Full Events)
- **Query Time**: Full table scan
- **Use Case**: Complex analytics, historical analysis
- **Storage**: 100% of event data

### LLM Export
- **Generation Time**: <100ms
- **Output Size**: 13.9 KB (minimal) to 18.6 KB (full)
- **Token Count**: ~3,556 to ~4,760 tokens
- **Compression**: 26% reduction with minimal mode

## Security Features

### Threat Detection
- 12 pre-built DNS security query templates
- 4 computed fields for threat indicators
- MITRE ATT&CK technique mappings
- Threat intelligence join definitions

### Observable Types Supported
- Type 1: Hostname
- Type 2: IP Address
- Type 5: Email Address
- Type 10: User Name
- Type 22: Hash
- Type 30: URL

### Query Templates
1. DNS Tunneling (high volume, long queries, TXT abuse)
2. DGA Detection (NXDOMAIN spikes, random domains)
3. C2 Beaconing (regular intervals)
4. Fast Flux Detection
5. IOC Matching (domain, IP)
6. Operational Queries (top domains, regional activity)

## Extensibility

### Adding New Entities
1. Define entity in YAML
2. Map to OCSF event classes
3. Validate against schema
4. Generate artifacts

### Adding Query Templates
1. Define template in `dns_templates.rs`
2. Add natural language patterns
3. Map to MITRE ATT&CK
4. Include in LLM export

### Adding Computed Fields
1. Define field in `computed_fields.rs`
2. Specify SQL expression
3. List dependencies
4. Include in LLM export

### Custom Warehouse Dialects
1. Implement `WarehouseDialect` trait
2. Add SQL generation logic
3. Register in CLI

## Testing Strategy

### Unit Tests
- 281 tests in ocsf-semantic
- 82 tests in ocsf-warehouse
- Specific examples and edge cases

### Property-Based Tests
- Universal properties across all inputs
- Round-trip serialization
- SQL generation validity
- Synonym resolution consistency

### Integration Tests
- End-to-end model validation
- Artifact generation
- Query translation
- LLM export format
