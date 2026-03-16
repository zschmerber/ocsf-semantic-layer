# DNS Semantic Layer - Detailed Diagram Guide

## 🌐 Access the Diagram

**URL:** http://localhost:8888/dns-diagram.html

## 📊 What's Included

The detailed diagram page provides a comprehensive visual representation of the DNS semantic layer with:

### 1. Complete Entity Diagram (Mermaid Class Diagram)
- **DNS_Event** semantic entity with all 10 attributes
- **OCSF_DNS_Activity** physical schema (class 4003)
- **Threat_Intel_Domains** external threat intelligence
- **Threat_Intel_IPs** external threat intelligence
- **DNS_Observables** hot path table for fast lookups
- Relationships and mappings between all entities

### 2. Attribute Details Grid
Each of the 10 attributes is shown in a card with:
- **Name and Type** (string, integer, timestamp)
- **Description** - What the attribute represents
- **OCSF Mapping** - Exact field path in OCSF schema
- **Synonyms** - Alternative names for natural language queries (27 total)
- **Threat Tags** - Security use cases and MITRE ATT&CK techniques
- **Dimension/Observable** indicators

#### Attributes Covered:
1. **query_hostname** - FQDN being queried (DGA detection, DNS tunneling, C2 beaconing)
2. **query_type** - DNS record type (DNS tunneling, data exfiltration)
3. **source_ip** - Client IP address (compromised host detection)
4. **source_port** - Client port number
5. **response_code** - DNS response (NOERROR, NXDOMAIN, etc.)
6. **action** - Action taken (Allowed, Blocked, Dropped)
7. **severity** - Event severity level
8. **cloud_provider** - Cloud service provider (AWS, Azure, GCP)
9. **cloud_region** - Cloud region (us-east-1, eu-west-1, etc.)
10. **event_time** - Query timestamp

### 3. Security Metrics
Four pre-built metrics with formulas:
- **dns_query_count** - Total DNS queries
- **blocked_dns_rate** - Percentage of blocked queries
- **dns_by_severity** - Events grouped by severity
- **nxdomain_rate** - NXDOMAIN percentage (DGA indicator)

### 4. Entity Relationships Diagram
Visual flow showing:
- DNS Event → Threat Intel Domains (hostname matching)
- DNS Event → Threat Intel IPs (source IP matching)
- DNS Event → DNS Observables (extraction for hot path)
- Threat context enrichment
- Fast threat matching capabilities

### 5. Hot/Cold Path Architecture
Explanation of the two-tier query architecture:
- **Hot Path**: Fast observable lookup table for IOC matching
- **Cold Path**: Full event records for detailed investigation

### 6. MITRE ATT&CK Coverage (Mindmap)
Visual representation of threat detection capabilities:
- **T1071.004** - Application Layer Protocol: DNS
- **T1568.002** - Dynamic Resolution: Domain Generation Algorithms
- **T1048.003** - Exfiltration Over Alternative Protocol
- Compromised host detection
- Mapping of attributes to techniques

### 7. Usage Examples
Natural language queries and their SQL translations:
- "Show me blocked domains from suspicious sources"
- "What's the NXDOMAIN rate by region?"

## 🎨 Visual Features

- **Dark theme** matching the main dashboard
- **Color-coded elements**:
  - Blue (#4a90d9) - Semantic entities
  - Green (#7ee787) - OCSF classes
  - Red (#f85149) - Threat intelligence
  - Orange (#f0883e) - Hot path tables
- **Interactive Mermaid diagrams** that render in the browser
- **Responsive grid layout** for attribute cards
- **Syntax-highlighted code** for OCSF mappings

## 📋 Diagram Types

1. **Class Diagram** - Shows entity structure and relationships
2. **Graph Diagram** - Shows data flow and connections
3. **Mindmap** - Shows MITRE ATT&CK technique coverage

## 🔍 Key Insights

### Synonym Mapping (27 total)
The model includes natural language synonyms for each attribute, enabling queries like:
- "domain" → query_hostname
- "client_ip" → source_ip
- "rcode" → response_code
- "verdict" → action

### Threat Detection Focus
Every security-relevant attribute includes:
- **Security context** - Why it matters for threat detection
- **Use cases** - Specific detection scenarios
- **MITRE techniques** - Mapped ATT&CK techniques

### Observable Extraction
Two observable types are automatically extracted:
- **Type 1**: Hostnames (query_hostname)
- **Type 2**: IP Addresses (source_ip)

These populate the hot path table for fast threat intelligence matching.

## 🚀 Navigation

From the diagram page, you can:
- **← Back to Dashboard** - Return to the main visualization
- **View Status Page** - Check system status and file availability

## 💡 Use Cases

This diagram is useful for:
1. **Understanding the semantic model** - See all attributes and mappings at once
2. **Query development** - Know which synonyms to use
3. **Threat detection** - Understand security context for each field
4. **Data engineering** - See OCSF field paths for ETL pipelines
5. **Documentation** - Share with team members and stakeholders
6. **Training** - Teach analysts about the semantic layer

## 🔧 Technical Details

- **Mermaid.js** for diagram rendering
- **Responsive CSS Grid** for attribute cards
- **Dark theme** optimized for readability
- **No external dependencies** except Mermaid CDN
- **Static HTML** - no server-side processing required

## 📝 Next Steps

1. Open http://localhost:8888/dns-diagram.html
2. Scroll through the complete entity diagram
3. Review each attribute card for details
4. Check the security metrics and their formulas
5. Understand the entity relationships
6. Review MITRE ATT&CK coverage
7. Use the examples to understand query translation

The diagram provides a complete reference for the DNS semantic layer, showing how business-friendly attributes map to OCSF fields and support threat detection use cases.
