# Requirements Document

## Introduction

This feature enhances the OCSF Semantic Layer with LLM-friendly metadata including synonyms, business glossary, sample values, relationships, and pre-built security query templates. The goal is to enable agentic LLMs to generate accurate SQL queries when security analysts ask natural language questions about DNS activity and other OCSF events.

## Glossary

- **Semantic_Layer**: The abstraction layer that maps OCSF physical schema to business-friendly concepts
- **Synonym**: Alternative names or phrases that users might use to refer to the same field or concept
- **Query_Template**: Pre-defined SQL patterns for common security questions
- **Business_Glossary**: Rich descriptions explaining fields in security analyst context
- **Entity_Relationship**: Defined connections between semantic entities
- **MITRE_ATT&CK**: Framework for categorizing adversary tactics and techniques
- **DGA**: Domain Generation Algorithm - malware technique using random domain names
- **DNS_Tunneling**: Technique to exfiltrate data or establish C2 via DNS queries
- **Fast_Flux**: Technique rapidly changing DNS records to hide malicious infrastructure
- **C2**: Command and Control - attacker infrastructure for controlling compromised hosts
- **IOC**: Indicator of Compromise - observable artifact suggesting intrusion

## Requirements

### Requirement 1: Synonym Support

**User Story:** As a security analyst, I want to ask questions using natural language terms like "client IP" or "origin address", so that the LLM can correctly map my intent to the actual OCSF field `src_endpoint.ip`.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support a `synonyms` array for each attribute definition
2. WHEN an attribute has synonyms defined, THE Query_Translator SHALL recognize all synonym terms as equivalent to the canonical field name
3. THE Synonym list SHALL include common security analyst terminology, abbreviations, and alternative phrasings
4. WHEN generating SQL, THE Query_Translator SHALL use the canonical OCSF field path regardless of which synonym was used in the query

### Requirement 2: Business Glossary Descriptions

**User Story:** As an LLM agent, I want rich contextual descriptions for each field, so that I can understand when to use each field and generate appropriate queries.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support a `description` field with detailed business context for each attribute
2. THE Semantic_Layer SHALL support a `security_context` field explaining the field's relevance to threat detection
3. WHEN a field is commonly used for specific threat detection scenarios, THE description SHALL mention those use cases
4. THE descriptions SHALL be written for LLM consumption with clear, unambiguous language

### Requirement 3: Sample Values and Patterns

**User Story:** As an LLM agent, I want to see example values for each field, so that I can understand the data format and generate valid filter conditions.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support a `sample_values` array showing representative values for each attribute
2. THE Semantic_Layer SHALL support a `value_pattern` regex for validating expected formats
3. WHEN an attribute has enumerated values, THE sample_values SHALL include all valid enum options
4. THE sample_values SHALL include both normal and suspicious/malicious examples where applicable

### Requirement 4: Entity Relationships

**User Story:** As an LLM agent, I want to understand how entities relate to each other, so that I can generate correct JOIN conditions when queries span multiple entities.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support a `relationships` section defining connections between entities
2. WHEN a relationship is defined, THE definition SHALL include the join path or condition
3. THE relationship definition SHALL include cardinality (one-to-one, one-to-many, many-to-many)
4. THE relationship definition SHALL include a human-readable description of the relationship

### Requirement 5: Pre-built Security Query Templates

**User Story:** As a security analyst, I want common threat detection queries pre-defined, so that I can quickly investigate DNS-based threats without writing SQL from scratch.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support a `query_templates` section with pre-built queries
2. WHEN a query template is defined, THE definition SHALL include natural language intent descriptions
3. THE query templates SHALL cover common DNS security use cases including:
   - DNS tunneling detection
   - DGA domain detection
   - Data exfiltration indicators
   - Fast flux detection
   - C2 beaconing patterns
   - NXDOMAIN spike analysis
   - Threat intel IOC matching
4. WHEN a query template maps to MITRE ATT&CK, THE definition SHALL include the technique ID
5. THE query templates SHALL include parameterized placeholders for customization

### Requirement 6: MITRE ATT&CK Mapping

**User Story:** As a security analyst, I want queries tagged with MITRE ATT&CK technique IDs, so that I can correlate findings with the threat framework my SOC uses.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support `mitre_attack` annotations on query templates
2. WHEN a query detects a specific technique, THE annotation SHALL include the technique ID (e.g., T1071.004)
3. THE annotation SHALL include the tactic category (e.g., Command and Control, Exfiltration)
4. THE Semantic_Layer SHALL support filtering query templates by MITRE technique

### Requirement 7: Computed Fields for Threat Detection

**User Story:** As a security analyst, I want pre-computed threat indicators like domain entropy and subdomain depth, so that I can detect DGA and tunneling without complex SQL.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support `computed_fields` that derive values from base attributes
2. THE computed fields SHALL include:
   - `domain_entropy`: Shannon entropy of the queried domain name
   - `subdomain_depth`: Count of subdomain levels
   - `query_length`: Character length of the full query hostname
   - `is_numeric_subdomain`: Boolean indicating numeric-heavy subdomain
3. WHEN a computed field is defined, THE definition SHALL include the SQL expression
4. THE computed fields SHALL be usable in filters and aggregations

### Requirement 8: Threat Intelligence Integration Points

**User Story:** As a security analyst, I want to easily join DNS queries against threat intel feeds, so that I can identify known malicious domains and IPs.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL define standard join points for threat intelligence tables
2. THE join points SHALL support matching on:
   - Domain/hostname
   - IP address
   - Hash values (for related file events)
3. WHEN threat intel integration is defined, THE definition SHALL include the expected schema of the threat intel table
4. THE Semantic_Layer SHALL include example queries for IOC matching

### Requirement 9: Time-based Analysis Support

**User Story:** As a security analyst, I want to easily analyze DNS patterns over time, so that I can detect beaconing, spikes, and anomalies.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL define time-based aggregation patterns
2. THE patterns SHALL include:
   - Hourly/daily query volume trends
   - Time-between-queries for beaconing detection
   - First-seen/last-seen timestamps per domain
3. WHEN time analysis is needed, THE Query_Translator SHALL generate appropriate window functions
4. THE time patterns SHALL support configurable time windows

### Requirement 10: Output Format for LLM Consumption

**User Story:** As an LLM agent, I want the semantic layer exported in a format optimized for my context window, so that I can efficiently understand the schema and generate queries.

#### Acceptance Criteria

1. THE Semantic_Layer SHALL support export to a compact LLM-friendly format
2. THE export format SHALL prioritize information density over human readability
3. THE export SHALL include all synonyms, descriptions, and query templates
4. THE export format SHALL be configurable to include/exclude sections based on context needs
