# LLM-Friendly Semantic Layer Export

This document describes the LLM-optimized export feature for OCSF semantic models.

## Overview

The `export-llm-context` command generates a compact, LLM-friendly representation of semantic models optimized for use in AI agent context windows. The export includes:

- **Entity definitions** with business-friendly names and descriptions
- **Synonym mappings** for natural language query resolution
- **Query templates** with natural language patterns and SQL
- **Computed fields** for threat detection
- **Threat intelligence joins** for IOC matching
- **Security context** explaining threat detection relevance
- **MITRE ATT&CK mappings** for security use cases

## Usage

### Basic Export

```bash
cargo run -p ocsf-cli -- export-llm-context \
  -m demo/dns-semantic-model-enhanced.yaml \
  -o llm-context.json \
  --include-dns-templates
```

### Minimal Export (Smaller Context Window)

```bash
cargo run -p ocsf-cli -- export-llm-context \
  -m demo/dns-semantic-model-enhanced.yaml \
  -o llm-context-minimal.json \
  --minimal \
  --include-dns-templates
```

### Custom Sections

```bash
cargo run -p ocsf-cli -- export-llm-context \
  -m demo/dns-semantic-model-enhanced.yaml \
  -o llm-context-custom.json \
  --sections entities,synonyms,templates
```

### YAML Output

```bash
cargo run -p ocsf-cli -- export-llm-context \
  -m demo/dns-semantic-model-enhanced.yaml \
  -o llm-context.yaml \
  -f yaml \
  --include-dns-templates
```

## Export Sections

Available sections (use with `--sections`):

- `entities` - Entity and attribute definitions with synonyms
- `synonyms` - Synonym lookup table (synonym → canonical name)
- `templates` - Query templates with natural language patterns
- `computed` - Computed field definitions
- `threatintel` - Threat intelligence join definitions
- `samples` - Sample values for attributes
- `security` - Security context descriptions

## Output Statistics

Using the enhanced DNS model:

| Export Type | Size | Estimated Tokens | Reduction |
|-------------|------|------------------|-----------|
| Full        | 18.6 KB | ~4,760 | - |
| Minimal     | 13.9 KB | ~3,556 | 26% |

## Export Structure

```json
{
  "model_name": "dns-analytics-enhanced",
  "version": "1.0",
  "entities": [
    {
      "name": "dns_event",
      "caption": "DNS Event",
      "description": "DNS queries and responses with threat detection capabilities",
      "attributes": [
        {
          "name": "query_hostname",
          "caption": "Query Hostname",
          "type": "string",
          "synonyms": ["domain", "fqdn", "hostname", "queried_domain"],
          "description": "The fully qualified domain name being queried",
          "security_context": "Critical for detecting DGA domains...",
          "sample_values": ["example.com", "mail.google.com"],
          "is_dimension": true,
          "is_observable": true
        }
      ]
    }
  ],
  "synonyms": {
    "domain": "query_hostname",
    "fqdn": "query_hostname",
    "client_ip": "source_ip"
  },
  "query_templates": [
    {
      "name": "dns_tunneling_high_volume",
      "intent": "Detect DNS tunneling by identifying hosts with unusually high query volumes",
      "patterns": [
        "Find DNS tunneling",
        "Show hosts with high DNS query volume"
      ],
      "sql": "SELECT source_ip, COUNT(*) as query_count...",
      "parameters": {
        "hours": "24",
        "threshold": "10000"
      },
      "mitre_technique": "T1071.004 - Application Layer Protocol: DNS",
      "category": "ThreatDetection"
    }
  ],
  "computed_fields": [
    {
      "name": "domain_entropy",
      "description": "Shannon entropy of domain name (higher = more random)",
      "sql": "...",
      "dependencies": ["query_hostname"]
    }
  ]
}
```

## Use Cases

### 1. Natural Language Query Translation

LLMs can use the synonym mappings to translate user queries:

```
User: "Show me blocked domains by client IP"
LLM resolves: blocked → action, domains → query_hostname, client IP → source_ip
```

### 2. Query Template Matching

LLMs can match user intent to pre-built templates:

```
User: "Find DNS tunneling activity"
LLM matches: dns_tunneling_high_volume template
LLM generates: SQL with appropriate parameters
```

### 3. Security Context Understanding

LLMs understand the security significance of fields:

```
Field: query_hostname
Security Context: "Critical for detecting DGA domains, DNS tunneling, and C2 beaconing"
MITRE: T1071.004, T1568.002
```

### 4. Computed Field Generation

LLMs can suggest computed fields for threat detection:

```
User: "Detect suspicious domains"
LLM suggests: domain_entropy, subdomain_depth, is_numeric_heavy
```

## Integration with LLM Agents

### Example Prompt

```
You are a security data analyst assistant. Use the following semantic model
to help users query DNS activity data.

<semantic_model>
{contents of llm-context.json}
</semantic_model>

When users ask questions:
1. Resolve synonyms to canonical field names
2. Match intent to query templates when possible
3. Consider security context for threat detection queries
4. Use computed fields for advanced analytics
```

### Example Interaction

```
User: "Show me domains with high entropy that were blocked"

LLM Response:
I'll query for blocked DNS events with high domain entropy, which can
indicate DGA (Domain Generation Algorithm) activity.

SELECT 
  query_hostname,
  domain_entropy,
  source_ip,
  time
FROM dns_event
WHERE action = 'Blocked'
  AND domain_entropy > 3.5
ORDER BY domain_entropy DESC
LIMIT 100

This query uses:
- query_hostname (synonym: domain)
- action (synonym: blocked)
- domain_entropy (computed field for randomness detection)
- Relates to MITRE T1568.002 (Domain Generation Algorithms)
```

## Pre-built DNS Templates

The enhanced DNS model includes 12 pre-built query templates:

1. **dns_tunneling_high_volume** - Detect high query volumes
2. **dns_tunneling_long_queries** - Find unusually long queries
3. **dns_txt_record_abuse** - Detect TXT record abuse
4. **dga_nxdomain_spike** - Find NXDOMAIN spikes
5. **dga_random_domains** - Detect random domain patterns
6. **fast_flux_detection** - Identify fast flux networks
7. **c2_beaconing_regular_intervals** - Find C2 beaconing
8. **ioc_domain_match** - Match threat intel domains
9. **blocked_dns_by_host** - Track blocked queries per host
10. **top_queried_domains** - Most queried domains
11. **dns_by_region** - Regional DNS activity
12. **new_domains_first_seen** - Track new domain appearances

## Performance Considerations

- **Token efficiency**: Minimal export reduces tokens by ~26%
- **Context window**: Full export fits in ~5K tokens
- **Selective sections**: Use `--sections` to include only needed data
- **Caching**: Export once, reuse for multiple queries

## Next Steps

1. Integrate export into your LLM agent workflow
2. Customize sections based on your use case
3. Add more query templates for your specific threats
4. Extend computed fields for custom analytics
5. Map additional MITRE ATT&CK techniques
