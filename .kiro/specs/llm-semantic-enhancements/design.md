# Design Document: LLM-Friendly Semantic Layer Enhancements

## Overview

This design extends the OCSF Semantic Layer with metadata specifically designed to help agentic LLMs generate accurate SQL queries for security analytics. The enhancements include synonyms, business glossary, sample values, entity relationships, and pre-built query templates for common DNS security use cases.

## Architecture

The enhanced semantic layer adds new metadata sections to the existing YAML model format:

```
┌─────────────────────────────────────────────────────────────┐
│                    Semantic Model YAML                       │
├─────────────────────────────────────────────────────────────┤
│  entities:                                                   │
│    - attributes with synonyms, descriptions, samples         │
│  metrics:                                                    │
│    - existing metric definitions                             │
│  relationships:           ← NEW                              │
│    - entity connections with join paths                      │
│  computed_fields:         ← NEW                              │
│    - derived threat indicators                               │
│  query_templates:         ← NEW                              │
│    - pre-built security queries with MITRE mapping           │
│  threat_intel_joins:      ← NEW                              │
│    - standard IOC matching patterns                          │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Enhanced Attribute Definition

```rust
pub struct EnhancedAttribute {
    // Existing fields
    pub name: String,
    pub caption: String,
    pub data_type: DataType,
    pub ocsf_mapping: OcsfMapping,
    pub is_dimension: bool,
    
    // New LLM-friendly fields
    pub synonyms: Vec<String>,
    pub description: String,
    pub security_context: Option<String>,
    pub sample_values: Vec<String>,
    pub value_pattern: Option<String>,
    pub is_observable: bool,
    pub threat_relevance: Option<ThreatRelevance>,
}

pub struct ThreatRelevance {
    pub use_cases: Vec<String>,
    pub mitre_techniques: Vec<String>,
}
```

### Entity Relationship Definition

```rust
pub struct EntityRelationship {
    pub name: String,
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: RelationshipType,
    pub cardinality: Cardinality,
    pub join_condition: String,
    pub description: String,
}

pub enum RelationshipType {
    OriginatedFrom,
    TargetedTo,
    PerformedBy,
    Contains,
    ReferencedIn,
}

pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}
```

### Query Template Definition

```rust
pub struct QueryTemplate {
    pub name: String,
    pub intent: String,                    // Natural language description
    pub natural_language_patterns: Vec<String>, // Example questions
    pub sql_template: String,              // Parameterized SQL
    pub parameters: Vec<QueryParameter>,
    pub mitre_attack: Option<MitreMapping>,
    pub category: QueryCategory,
    pub severity: Option<Severity>,
}

pub struct MitreMapping {
    pub technique_id: String,      // e.g., "T1071.004"
    pub technique_name: String,    // e.g., "Application Layer Protocol: DNS"
    pub tactic: String,            // e.g., "Command and Control"
}

pub enum QueryCategory {
    ThreatDetection,
    Anomaly,
    Investigation,
    Compliance,
    Operational,
}
```

### Computed Field Definition

```rust
pub struct ComputedField {
    pub name: String,
    pub caption: String,
    pub description: String,
    pub sql_expression: String,
    pub data_type: DataType,
    pub dependencies: Vec<String>,  // Base fields required
}
```

## Data Models

### Enhanced DNS Semantic Model Example

```yaml
entities:
  - name: dns_event
    caption: DNS Event
    description: DNS queries and responses from network monitoring
    source_event_classes: [4003]
    
    attributes:
      - name: query_hostname
        caption: Query Hostname
        type: string
        ocsf_mapping:
          field: query.hostname
        is_dimension: true
        is_observable: true
        synonyms:
          - "domain"
          - "dns name"
          - "queried domain"
          - "lookup hostname"
          - "dns query"
          - "fqdn"
          - "fully qualified domain name"
          - "requested domain"
        description: "The fully qualified domain name being queried in the DNS request"
        security_context: "Primary indicator for threat detection. Check against threat intel feeds, analyze for DGA patterns (high entropy, random characters), and monitor for known malicious domains."
        sample_values:
          - "www.example.com"
          - "api.internal.corp"
          - "xn7kd9a.malware.cc"  # DGA example
          - "data.exfil.attacker.com"
        value_pattern: "^[a-zA-Z0-9][a-zA-Z0-9-_.]+$"
        threat_relevance:
          use_cases:
            - "DGA detection"
            - "DNS tunneling"
            - "C2 communication"
            - "Data exfiltration"
          mitre_techniques:
            - "T1071.004"
            - "T1568.002"

      - name: source_ip
        caption: Source IP Address
        type: string
        ocsf_mapping:
          field: src_endpoint.ip
        is_dimension: true
        is_observable: true
        synonyms:
          - "client IP"
          - "origin IP"
          - "sender IP"
          - "src IP"
          - "requesting IP"
          - "source address"
          - "client address"
          - "originating IP"
        description: "IP address of the host that initiated the DNS query"
        security_context: "Identifies the potentially compromised host. High query volumes from a single IP may indicate malware beaconing or data exfiltration."
        sample_values:
          - "10.0.0.50"
          - "192.168.1.100"
          - "172.16.0.25"
        value_pattern: "^(?:[0-9]{1,3}\\.){3}[0-9]{1,3}$"

      - name: action
        caption: Action
        type: string
        ocsf_mapping:
          field: action
        is_dimension: true
        synonyms:
          - "result"
          - "outcome"
          - "disposition"
          - "dns action"
          - "query result"
          - "allowed or blocked"
        description: "The action taken on the DNS query (Allowed, Blocked, etc.)"
        security_context: "Blocked queries indicate policy enforcement. High blocked rates from a host may indicate malware attempting C2 communication."
        sample_values:
          - "Allowed"
          - "Blocked"
          - "Dropped"
          - "Quarantined"

      - name: response_code
        caption: Response Code
        type: string
        ocsf_mapping:
          field: rcode
        is_dimension: true
        synonyms:
          - "rcode"
          - "dns response"
          - "query status"
          - "resolution status"
          - "dns result code"
        description: "DNS response code indicating query success or failure type"
        security_context: "NXDOMAIN spikes may indicate DGA activity. SERVFAIL patterns could indicate DNS infrastructure issues or attacks."
        sample_values:
          - "NoError"
          - "NXDOMAIN"
          - "SERVFAIL"
          - "REFUSED"

      - name: query_type
        caption: Query Type
        type: string
        ocsf_mapping:
          field: query.type
        is_dimension: true
        synonyms:
          - "record type"
          - "dns type"
          - "dns record type"
          - "query record type"
        description: "The DNS record type being requested (A, AAAA, TXT, MX, etc.)"
        security_context: "TXT queries are commonly used for DNS tunneling. Unusual record types may indicate covert channels."
        sample_values:
          - "A"
          - "AAAA"
          - "TXT"
          - "MX"
          - "CNAME"
          - "NS"

      - name: severity
        caption: Severity
        type: string
        ocsf_mapping:
          field: severity
        is_dimension: true
        synonyms:
          - "alert level"
          - "risk level"
          - "threat level"
          - "priority"
        description: "Severity level assigned to the DNS event"
        sample_values:
          - "Informational"
          - "Low"
          - "Medium"
          - "High"
          - "Critical"

      - name: cloud_provider
        caption: Cloud Provider
        type: string
        ocsf_mapping:
          field: cloud.provider
        is_dimension: true
        synonyms:
          - "cloud"
          - "provider"
          - "cloud platform"
          - "infrastructure provider"
        description: "Cloud provider where the DNS query was logged"
        sample_values:
          - "AWS"
          - "Azure"
          - "GCP"

      - name: cloud_region
        caption: Cloud Region
        type: string
        ocsf_mapping:
          field: cloud.region
        is_dimension: true
        synonyms:
          - "region"
          - "aws region"
          - "location"
          - "data center"
        description: "Geographic region of the cloud infrastructure"
        sample_values:
          - "us-east-1"
          - "us-west-2"
          - "eu-west-1"

computed_fields:
  - name: domain_entropy
    caption: Domain Entropy
    description: "Shannon entropy of the query hostname - high values (>3.5) may indicate DGA"
    sql_expression: |
      -SUM(
        (LEN(query_hostname) - LEN(REPLACE(query_hostname, char, ''))) / LEN(query_hostname) *
        LOG(2, (LEN(query_hostname) - LEN(REPLACE(query_hostname, char, ''))) / LEN(query_hostname))
      )
    data_type: float
    dependencies: [query_hostname]
    
  - name: subdomain_depth
    caption: Subdomain Depth
    description: "Number of subdomain levels - deep nesting may indicate tunneling"
    sql_expression: "LENGTH(query_hostname) - LENGTH(REPLACE(query_hostname, '.', '')) - 1"
    data_type: integer
    dependencies: [query_hostname]
    
  - name: query_length
    caption: Query Length
    description: "Character length of hostname - long queries may indicate data encoding"
    sql_expression: "LENGTH(query_hostname)"
    data_type: integer
    dependencies: [query_hostname]

  - name: is_numeric_heavy
    caption: Is Numeric Heavy
    description: "True if hostname contains >40% numeric characters - DGA indicator"
    sql_expression: |
      CASE WHEN (LENGTH(REGEXP_REPLACE(query_hostname, '[^0-9]', '')) * 1.0 / LENGTH(query_hostname)) > 0.4 
      THEN true ELSE false END
    data_type: boolean
    dependencies: [query_hostname]

relationships:
  - name: dns_to_source_endpoint
    from_entity: dns_event
    to_entity: endpoint
    relationship_type: OriginatedFrom
    cardinality: ManyToOne
    join_condition: "dns_event.source_ip = endpoint.ip_address"
    description: "DNS query originated from this endpoint/host"

  - name: dns_to_user
    from_entity: dns_event
    to_entity: user
    relationship_type: PerformedBy
    cardinality: ManyToOne
    join_condition: "dns_event.actor_user_uid = user.uid"
    description: "User account associated with the DNS query"

  - name: dns_to_threat_intel
    from_entity: dns_event
    to_entity: threat_intel_ioc
    relationship_type: ReferencedIn
    cardinality: ManyToMany
    join_condition: "dns_event.query_hostname = threat_intel_ioc.indicator OR dns_event.source_ip = threat_intel_ioc.indicator"
    description: "Match DNS observables against threat intelligence IOCs"


query_templates:
  # ============================================
  # DNS TUNNELING DETECTION
  # ============================================
  - name: dns_tunneling_high_volume
    intent: "Detect DNS tunneling by identifying hosts with unusually high query volumes"
    natural_language_patterns:
      - "Find DNS tunneling"
      - "Show hosts with high DNS query volume"
      - "Detect data exfiltration via DNS"
      - "Which hosts are making too many DNS queries"
      - "Find suspicious DNS activity"
    sql_template: |
      SELECT 
        source_ip,
        COUNT(*) as query_count,
        COUNT(DISTINCT query_hostname) as unique_domains,
        AVG(LENGTH(query_hostname)) as avg_query_length
      FROM dns_event
      WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      GROUP BY source_ip
      HAVING query_count > {{threshold}}
      ORDER BY query_count DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
        description: "Time window in hours"
      - name: threshold
        type: integer
        default: 10000
        description: "Minimum query count to flag"
      - name: limit
        type: integer
        default: 100
    mitre_attack:
      technique_id: "T1071.004"
      technique_name: "Application Layer Protocol: DNS"
      tactic: "Command and Control"
    category: ThreatDetection
    severity: High

  - name: dns_tunneling_long_queries
    intent: "Detect DNS tunneling by finding unusually long DNS queries (data encoded in subdomain)"
    natural_language_patterns:
      - "Find long DNS queries"
      - "Detect encoded data in DNS"
      - "Show DNS queries with long hostnames"
      - "Find DNS exfiltration attempts"
    sql_template: |
      SELECT 
        source_ip,
        query_hostname,
        LENGTH(query_hostname) as query_length,
        time
      FROM dns_event
      WHERE LENGTH(query_hostname) > {{min_length}}
        AND time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      ORDER BY query_length DESC
      LIMIT {{limit}}
    parameters:
      - name: min_length
        type: integer
        default: 50
        description: "Minimum hostname length to flag"
      - name: hours
        type: integer
        default: 24
      - name: limit
        type: integer
        default: 100
    mitre_attack:
      technique_id: "T1048.003"
      technique_name: "Exfiltration Over Alternative Protocol"
      tactic: "Exfiltration"
    category: ThreatDetection
    severity: High

  - name: dns_txt_record_abuse
    intent: "Detect potential DNS tunneling via TXT record queries"
    natural_language_patterns:
      - "Find TXT record queries"
      - "Show DNS TXT lookups"
      - "Detect DNS tunneling via TXT"
      - "Which hosts are querying TXT records"
    sql_template: |
      SELECT 
        source_ip,
        query_hostname,
        COUNT(*) as txt_query_count,
        MIN(time) as first_seen,
        MAX(time) as last_seen
      FROM dns_event
      WHERE query_type = 'TXT'
        AND time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      GROUP BY source_ip, query_hostname
      HAVING txt_query_count > {{threshold}}
      ORDER BY txt_query_count DESC
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: threshold
        type: integer
        default: 10
    mitre_attack:
      technique_id: "T1071.004"
      technique_name: "Application Layer Protocol: DNS"
      tactic: "Command and Control"
    category: ThreatDetection
    severity: Medium

  # ============================================
  # DGA (DOMAIN GENERATION ALGORITHM) DETECTION
  # ============================================
  - name: dga_nxdomain_spike
    intent: "Detect DGA activity by finding hosts with high NXDOMAIN response rates"
    natural_language_patterns:
      - "Find DGA domains"
      - "Detect domain generation algorithm"
      - "Show NXDOMAIN spikes"
      - "Which hosts have failed DNS lookups"
      - "Find malware beaconing"
      - "Detect algorithmically generated domains"
    sql_template: |
      SELECT 
        source_ip,
        COUNT(*) as total_queries,
        SUM(CASE WHEN response_code = 'NXDOMAIN' THEN 1 ELSE 0 END) as nxdomain_count,
        ROUND(100.0 * SUM(CASE WHEN response_code = 'NXDOMAIN' THEN 1 ELSE 0 END) / COUNT(*), 2) as nxdomain_rate
      FROM dns_event
      WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      GROUP BY source_ip
      HAVING nxdomain_rate > {{nxdomain_threshold}}
        AND total_queries > {{min_queries}}
      ORDER BY nxdomain_count DESC
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: nxdomain_threshold
        type: float
        default: 50.0
        description: "Minimum NXDOMAIN percentage to flag"
      - name: min_queries
        type: integer
        default: 100
        description: "Minimum total queries for statistical significance"
    mitre_attack:
      technique_id: "T1568.002"
      technique_name: "Dynamic Resolution: Domain Generation Algorithms"
      tactic: "Command and Control"
    category: ThreatDetection
    severity: High

  - name: dga_random_domains
    intent: "Find domains with high entropy (randomness) indicating DGA"
    natural_language_patterns:
      - "Find random looking domains"
      - "Detect high entropy domains"
      - "Show suspicious domain names"
      - "Find gibberish domains"
    sql_template: |
      WITH domain_stats AS (
        SELECT 
          query_hostname,
          source_ip,
          COUNT(*) as query_count,
          -- Approximate entropy via character diversity
          LENGTH(query_hostname) as domain_length,
          LENGTH(REGEXP_REPLACE(query_hostname, '[^0-9]', '')) as numeric_chars,
          LENGTH(REGEXP_REPLACE(query_hostname, '[^a-z]', '')) as alpha_chars
        FROM dns_event
        WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
          AND LENGTH(query_hostname) > 10
        GROUP BY query_hostname, source_ip
      )
      SELECT *,
        ROUND(100.0 * numeric_chars / domain_length, 2) as numeric_ratio
      FROM domain_stats
      WHERE numeric_chars > domain_length * 0.3  -- >30% numeric = suspicious
        OR domain_length > {{max_normal_length}}
      ORDER BY query_count DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: max_normal_length
        type: integer
        default: 40
      - name: limit
        type: integer
        default: 100
    mitre_attack:
      technique_id: "T1568.002"
      technique_name: "Dynamic Resolution: Domain Generation Algorithms"
      tactic: "Command and Control"
    category: ThreatDetection
    severity: High

  # ============================================
  # FAST FLUX DETECTION
  # ============================================
  - name: fast_flux_detection
    intent: "Detect fast flux DNS by finding domains resolving to many different IPs"
    natural_language_patterns:
      - "Find fast flux domains"
      - "Detect domains with changing IPs"
      - "Show bulletproof hosting indicators"
      - "Find domains with multiple IP addresses"
    sql_template: |
      SELECT 
        query_hostname,
        COUNT(DISTINCT answers_rdata) as unique_ips,
        COUNT(*) as query_count,
        ARRAY_AGG(DISTINCT answers_rdata) as ip_addresses
      FROM dns_event
      WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
        AND answers_rdata IS NOT NULL
      GROUP BY query_hostname
      HAVING unique_ips > {{ip_threshold}}
      ORDER BY unique_ips DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: ip_threshold
        type: integer
        default: 10
        description: "Minimum unique IPs to flag as fast flux"
      - name: limit
        type: integer
        default: 50
    mitre_attack:
      technique_id: "T1568.001"
      technique_name: "Dynamic Resolution: Fast Flux DNS"
      tactic: "Command and Control"
    category: ThreatDetection
    severity: High

  # ============================================
  # C2 BEACONING DETECTION
  # ============================================
  - name: c2_beaconing_regular_intervals
    intent: "Detect C2 beaconing by finding DNS queries at regular time intervals"
    natural_language_patterns:
      - "Find beaconing activity"
      - "Detect C2 communication"
      - "Show regular DNS patterns"
      - "Find malware callbacks"
      - "Detect command and control"
    sql_template: |
      WITH query_intervals AS (
        SELECT 
          source_ip,
          query_hostname,
          time,
          LAG(time) OVER (PARTITION BY source_ip, query_hostname ORDER BY time) as prev_time,
          DATEDIFF(second, LAG(time) OVER (PARTITION BY source_ip, query_hostname ORDER BY time), time) as interval_seconds
        FROM dns_event
        WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      )
      SELECT 
        source_ip,
        query_hostname,
        COUNT(*) as beacon_count,
        AVG(interval_seconds) as avg_interval,
        STDDEV(interval_seconds) as interval_stddev,
        MIN(time) as first_seen,
        MAX(time) as last_seen
      FROM query_intervals
      WHERE interval_seconds IS NOT NULL
      GROUP BY source_ip, query_hostname
      HAVING beacon_count > {{min_beacons}}
        AND interval_stddev < avg_interval * {{jitter_threshold}}  -- Low variance = regular beaconing
      ORDER BY beacon_count DESC
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: min_beacons
        type: integer
        default: 10
      - name: jitter_threshold
        type: float
        default: 0.2
        description: "Max stddev as fraction of mean (0.2 = 20% jitter)"
    mitre_attack:
      technique_id: "T1071.004"
      technique_name: "Application Layer Protocol: DNS"
      tactic: "Command and Control"
    category: ThreatDetection
    severity: Critical

  # ============================================
  # THREAT INTELLIGENCE MATCHING
  # ============================================
  - name: ioc_domain_match
    intent: "Match DNS queries against known malicious domains from threat intel"
    natural_language_patterns:
      - "Find known bad domains"
      - "Match against threat intel"
      - "Check IOC matches"
      - "Find malicious domain lookups"
      - "Show threat intel hits"
    sql_template: |
      SELECT 
        d.source_ip,
        d.query_hostname,
        d.time,
        d.action,
        t.threat_type,
        t.confidence,
        t.source as intel_source
      FROM dns_event d
      INNER JOIN threat_intel_ioc t 
        ON d.query_hostname = t.indicator
        OR d.query_hostname LIKE '%.' || t.indicator
      WHERE d.time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
        AND t.indicator_type = 'domain'
      ORDER BY d.time DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: limit
        type: integer
        default: 1000
    category: ThreatDetection
    severity: Critical

  # ============================================
  # BLOCKED DNS ANALYSIS
  # ============================================
  - name: blocked_dns_by_host
    intent: "Find hosts with high rates of blocked DNS queries"
    natural_language_patterns:
      - "Show blocked DNS queries"
      - "Which hosts are being blocked"
      - "Find policy violations"
      - "Show DNS blocks by host"
    sql_template: |
      SELECT 
        source_ip,
        COUNT(*) as total_queries,
        SUM(CASE WHEN action = 'Blocked' THEN 1 ELSE 0 END) as blocked_count,
        ROUND(100.0 * SUM(CASE WHEN action = 'Blocked' THEN 1 ELSE 0 END) / COUNT(*), 2) as blocked_rate,
        ARRAY_AGG(DISTINCT query_hostname) FILTER (WHERE action = 'Blocked') as blocked_domains
      FROM dns_event
      WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      GROUP BY source_ip
      HAVING blocked_count > {{min_blocked}}
      ORDER BY blocked_count DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: min_blocked
        type: integer
        default: 10
      - name: limit
        type: integer
        default: 100
    category: Investigation
    severity: Medium

  # ============================================
  # OPERATIONAL QUERIES
  # ============================================
  - name: top_queried_domains
    intent: "Show the most frequently queried domains"
    natural_language_patterns:
      - "Top DNS queries"
      - "Most queried domains"
      - "Popular domains"
      - "DNS query statistics"
    sql_template: |
      SELECT 
        query_hostname,
        COUNT(*) as query_count,
        COUNT(DISTINCT source_ip) as unique_clients,
        MIN(time) as first_seen,
        MAX(time) as last_seen
      FROM dns_event
      WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      GROUP BY query_hostname
      ORDER BY query_count DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: limit
        type: integer
        default: 100
    category: Operational

  - name: dns_by_region
    intent: "Show DNS query distribution by cloud region"
    natural_language_patterns:
      - "DNS queries by region"
      - "Regional DNS activity"
      - "Which regions have DNS traffic"
    sql_template: |
      SELECT 
        cloud_region,
        COUNT(*) as query_count,
        COUNT(DISTINCT source_ip) as unique_clients,
        COUNT(DISTINCT query_hostname) as unique_domains
      FROM dns_event
      WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      GROUP BY cloud_region
      ORDER BY query_count DESC
    parameters:
      - name: hours
        type: integer
        default: 24
    category: Operational

  - name: new_domains_first_seen
    intent: "Find domains queried for the first time recently"
    natural_language_patterns:
      - "New domains"
      - "First time domains"
      - "Recently seen domains"
      - "Newly observed domains"
    sql_template: |
      WITH domain_history AS (
        SELECT 
          query_hostname,
          MIN(time) as first_seen,
          COUNT(*) as total_queries
        FROM dns_event
        GROUP BY query_hostname
      )
      SELECT 
        query_hostname,
        first_seen,
        total_queries
      FROM domain_history
      WHERE first_seen >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
      ORDER BY first_seen DESC
      LIMIT {{limit}}
    parameters:
      - name: hours
        type: integer
        default: 24
      - name: limit
        type: integer
        default: 100
    category: Investigation
    severity: Low

threat_intel_joins:
  - name: domain_ioc_lookup
    description: "Join DNS queries with domain-based threat intelligence"
    target_table: threat_intel_ioc
    expected_schema:
      - name: indicator
        type: string
        description: "The IOC value (domain, IP, hash)"
      - name: indicator_type
        type: string
        description: "Type of indicator (domain, ip, hash)"
      - name: threat_type
        type: string
        description: "Category of threat (malware, phishing, c2)"
      - name: confidence
        type: integer
        description: "Confidence score 0-100"
      - name: source
        type: string
        description: "Threat intel feed source"
      - name: first_seen
        type: timestamp
      - name: last_seen
        type: timestamp
    join_conditions:
      - "dns_event.query_hostname = threat_intel_ioc.indicator"
      - "dns_event.query_hostname LIKE '%.' || threat_intel_ioc.indicator"

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system.*

### Property 1: Synonym Resolution Consistency
*For any* attribute with synonyms defined, querying using any synonym term SHALL produce the same SQL output as querying with the canonical field name.
**Validates: Requirements 1.2, 1.4**

### Property 2: Query Template Parameter Substitution
*For any* query template with parameters, substituting valid parameter values SHALL produce syntactically valid SQL.
**Validates: Requirements 5.2, 5.5**

### Property 3: Computed Field Expression Validity
*For any* computed field definition, the SQL expression SHALL be valid for the target warehouse dialect.
**Validates: Requirements 7.3**

### Property 4: Relationship Join Correctness
*For any* defined relationship, the join condition SHALL produce valid SQL when entities are combined.
**Validates: Requirements 4.2, 4.3**

## Error Handling

- Invalid synonym references should produce clear error messages
- Query template parameter type mismatches should be caught at validation time
- Missing required parameters should fail with descriptive errors
- Computed field expressions should be validated against the target SQL dialect

## Testing Strategy

### Unit Tests
- Synonym lookup and resolution
- Query template parameter substitution
- Computed field SQL generation
- Relationship join condition generation

### Property-Based Tests
- Synonym resolution consistency across all defined synonyms
- Query template validity for random valid parameter combinations
- Computed field expression parsing

### Integration Tests
- End-to-end query generation from natural language
- Threat intel join execution
- Time-based analysis queries
