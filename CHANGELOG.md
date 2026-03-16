# Changelog

All notable changes to the OCSF Semantic Layer project.

## [0.1.0] - 2024-01-22

### Added - LLM-Friendly Semantic Layer Enhancements

#### Core Features
- **Synonym Resolution** - Natural language query support with 27+ synonym mappings
- **Query Templates** - 12 pre-built DNS security query templates with natural language patterns
- **Computed Fields** - 4 threat detection computed fields (domain_entropy, subdomain_depth, query_length, is_numeric_heavy)
- **Threat Intel Integration** - Pre-built IOC matching joins for domains and IPs
- **LLM Export Format** - Compact JSON/YAML export optimized for AI agent context windows
- **Security Context** - Threat relevance metadata and MITRE ATT&CK technique mappings

#### New Modules
- `ocsf-semantic/src/llm_export.rs` - LLM-optimized export format with configurable sections
- `ocsf-semantic/src/query_templates.rs` - Query template types with parameter substitution
- `ocsf-semantic/src/dns_templates.rs` - 12 DNS security query templates
- `ocsf-semantic/src/computed_fields.rs` - Computed field definitions
- `ocsf-semantic/src/threat_intel.rs` - Threat intelligence join types
- `ocsf-semantic/src/synonym_resolver.rs` - Case-insensitive synonym resolution

#### CLI Commands
- `export-llm-context` - Export semantic models for LLM agents
  - Multiple output formats (JSON, JSON-compact, YAML)
  - Configurable sections (entities, synonyms, templates, computed, threatintel, samples, security)
  - Minimal mode for smaller context windows (26% size reduction)
  - DNS template integration

#### Enhanced Data Types
- `SemanticAttribute` - Added LLM metadata fields:
  - `synonyms: Vec<String>` - Alternative names for natural language queries
  - `security_context: Option<String>` - Threat detection context
  - `value_pattern: Option<String>` - Regex validation patterns
  - `is_observable: bool` - IOC-relevant field marker
  - `threat_relevance: Option<ThreatRelevance>` - Use cases and MITRE mappings

- `ThreatRelevance` - New struct for security metadata:
  - `use_cases: Vec<String>` - Security use cases
  - `mitre_techniques: Vec<String>` - MITRE ATT&CK technique IDs

- `EntityRelationship` - Enhanced with:
  - `relationship_type: Option<RelationshipType>` - Semantic relationship type
  - `description: String` - Human-readable description
  - `to_join_sql()` method - SQL JOIN generation

- `RelationshipType` - New enum:
  - `OriginatedFrom` - Entity originated from another
  - `TargetedTo` - Entity targeted another
  - `PerformedBy` - Action performed by entity
  - `Contains` - Entity contains another
  - `ReferencedIn` - Entity referenced in another

- `Cardinality` - Added `ManyToOne` variant

#### DNS Security Query Templates
1. **DNS Tunneling Detection**
   - `dns_tunneling_high_volume` - Detect high query volumes
   - `dns_tunneling_long_queries` - Find unusually long queries
   - `dns_txt_record_abuse` - Detect TXT record abuse

2. **DGA Detection**
   - `dga_nxdomain_spike` - Find NXDOMAIN spikes
   - `dga_random_domains` - Detect random domain patterns

3. **C2 and Fast Flux**
   - `fast_flux_detection` - Identify fast flux networks
   - `c2_beaconing_regular_intervals` - Find C2 beaconing

4. **Threat Intelligence**
   - `ioc_domain_match` - Match threat intel domains
   - `blocked_dns_by_host` - Track blocked queries per host

5. **Operational**
   - `top_queried_domains` - Most queried domains
   - `dns_by_region` - Regional DNS activity
   - `new_domains_first_seen` - Track new domain appearances

#### Example Models
- `demo/dns-semantic-model-enhanced.yaml` - Complete DNS analytics model with:
  - 10 attributes with full LLM metadata
  - 27 synonym mappings
  - Entity relationships for threat intel
  - Sample values and value patterns
  - Security context for each attribute
  - MITRE ATT&CK technique mappings

#### Documentation
- `README.md` - Comprehensive project documentation
- `FEATURES.md` - Feature overview and use cases
- `demo/ARCHITECTURE.md` - System architecture and data flow
- `demo/LLM-EXPORT-README.md` - Detailed LLM export guide
- `CHANGELOG.md` - This file

#### Testing
- Added 8 new tests (281 total, up from 273)
- All property-based tests passing
- Round-trip serialization tests for new types
- LLM export format validation tests

#### Performance
- LLM export generation: <100ms
- Full export size: 18.6 KB (~4,760 tokens)
- Minimal export size: 13.9 KB (~3,556 tokens)
- Size reduction with minimal mode: 26%

### Changed
- Enhanced `SemanticAttribute` with LLM-friendly metadata fields
- Enhanced `EntityRelationship` with relationship types and descriptions
- Updated `Cardinality` enum with `ManyToOne` variant
- Updated demo scripts to include LLM export testing

### Technical Details

#### Module Structure
```
ocsf-semantic/src/
├── entity.rs              # Enhanced with LLM fields
├── llm_export.rs          # NEW: LLM export format
├── query_templates.rs     # NEW: Query template types
├── dns_templates.rs       # NEW: DNS security templates
├── computed_fields.rs     # NEW: Computed field types
├── threat_intel.rs        # NEW: Threat intel integration
└── synonym_resolver.rs    # NEW: Synonym resolution
```

#### Export Statistics
- **Entities**: 1 (dns_event with 10 attributes)
- **Synonyms**: 27 mappings
- **Query Templates**: 12 DNS security templates
- **Computed Fields**: 4 threat detection fields
- **Threat Intel Joins**: 2 pre-built joins

#### MITRE ATT&CK Coverage
- T1071.004 - Application Layer Protocol: DNS
- T1568.002 - Dynamic Resolution: Domain Generation Algorithms
- T1048.003 - Exfiltration Over Alternative Protocol

### Compatibility
- Rust 1.75+
- OCSF v1.6.0
- All existing features remain backward compatible

### Migration Guide
No breaking changes. Existing semantic models continue to work without modification.

To use new LLM features:
1. Add `synonyms` to attributes for natural language support
2. Add `security_context` for threat detection context
3. Use `export-llm-context` command to generate LLM-friendly exports

### Contributors
- OCSF Semantic Layer Team

---

## [0.0.1] - Initial Release

### Added
- Core semantic modeling framework
- OCSF schema validation
- Warehouse artifact generation (dbt, Cube.js, SQL)
- Observable hot path analytics
- Visual representation
- Vector-based field mapping
- Query translation
- Schema-driven generation
- CLI tool (ocsf-cli)

### Supported Platforms
- Snowflake
- Databricks
- Google BigQuery
- PostgreSQL

### Test Coverage
- 273 tests passing
- Property-based testing with proptest
- Round-trip serialization tests
- SQL generation validation
