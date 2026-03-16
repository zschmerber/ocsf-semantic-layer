# Databricks Semantic Layer - Complete Example

## 📊 Overview

This guide shows how the OCSF semantic layer works in Databricks, from raw DNS logs to business-friendly queries.

## 🔄 Complete Data Flow

```
Raw DNS Log (JSON)
    ↓
Physical Layer (OCSF Table)
    ↓
Semantic Layer (View)
    ↓
Business Query
```

---

## 1️⃣ Raw DNS Log (Original Data)

### AWS Route 53 DNS Query Log
**File:** `s3://my-bucket/dns-logs/2022/10/13/dns-query.json`

```json
{
    "action": "Allowed",
    "action_id": 1,
    "activity_id": 6,
    "activity_name": "Traffic",
    "answers": [
        {
            "class": "IN",
            "rdata": "127.0.0.62",
            "type": "A"
        }
    ],
    "category_name": "Network Activity",
    "category_uid": 4,
    "class_name": "DNS Activity",
    "class_uid": 4003,
    "cloud": {
        "account": {
            "uid": "123456789012"
        },
        "provider": "AWS",
        "region": "us-east-1"
    },
    "connection_info": {
        "direction": "Unknown",
        "direction_id": 0,
        "protocol_name": "UDP"
    },
    "disposition": "Alert",
    "dst_endpoint": {
        "instance_uid": "rslvr-in-0000000000000000",
        "interface_uid": "rni-0000000000000000"
    },
    "firewall_rule": {
        "uid": "rslvr-frg-000000000000000"
    },
    "metadata": {
        "product": {
            "feature": {
                "name": "Resolver Query Logs"
            },
            "name": "Route 53",
            "vendor_name": "AWS",
            "version": "1.100000"
        },
        "profiles": [
            "cloud",
            "security_control",
            "datetime"
        ],
        "version": "1.5.0",
        "uid": "dns-event-12345"
    },
    "observables": [
        {
            "name": "answers[].rdata",
            "type": "IP Address",
            "type_id": 2,
            "value": "127.0.0.62"
        },
        {
            "name": "src_endpoint.ip",
            "type": "IP Address",
            "type_id": 2,
            "value": "10.200.21.100"
        },
        {
            "name": "query.hostname",
            "type": "Hostname",
            "type_id": 1,
            "value": "ip-127-0-0-62.alert.firewall.canary."
        }
    ],
    "query": {
        "class": "IN",
        "hostname": "ip-127-0-0-62.alert.firewall.canary.",
        "type": "A"
    },
    "rcode": "NoError",
    "rcode_id": 0,
    "severity": "Informational",
    "severity_id": 1,
    "src_endpoint": {
        "ip": "10.200.21.100",
        "port": 15083,
        "vpc_uid": "vpc-00000000000000000"
    },
    "time": 1665694956000,
    "time_dt": "2022-10-13T17:02:36.000-04:00",
    "type_name": "DNS Activity: Traffic",
    "type_uid": 400306
}
```

---

## 2️⃣ Physical Layer (OCSF Table in Databricks)

### Table: `ocsf_class_4003`
**Location:** Delta Lake table in Databricks

```sql
CREATE TABLE `ocsf_class_4003` (
    -- Core OCSF fields
    `metadata_uid` STRING NOT NULL,
    `class_uid` BIGINT NOT NULL,
    `category_uid` BIGINT NOT NULL,
    `time` TIMESTAMP NOT NULL,
    `severity_id` BIGINT,
    `severity` STRING,
    `action` STRING,
    `rcode` STRING,
    
    -- Raw nested JSON stored as STRING for complex objects
    `raw_data` STRING,
    
    -- Extracted observables array
    `observables` ARRAY<STRUCT<
        type: STRING,
        type_id: BIGINT,
        value: STRING,
        name: STRING
    >>,
    
    PRIMARY KEY (metadata_uid)
)
PARTITIONED BY (DATE(time))
LOCATION 's3://my-databricks-bucket/ocsf/class_4003/';
```

### Sample Row in Physical Table

| metadata_uid | class_uid | time | action | rcode | raw_data | observables |
|---|---|---|---|---|---|---|
| dns-event-12345 | 4003 | 2022-10-13 17:02:36 | Allowed | NoError | {"query":{"hostname":"ip-127-0-0-62.alert.firewall.canary.","type":"A"},"src_endpoint":{"ip":"10.200.21.100","port":15083},"cloud":{"provider":"AWS","region":"us-east-1"},...} | [{"type":"Hostname","type_id":1,"value":"ip-127-0-0-62.alert.firewall.canary."},{"type":"IP Address","type_id":2,"value":"10.200.21.100"}] |

**Key Points:**
- Complex nested JSON stored in `raw_data` column
- Requires knowledge of OCSF schema to query
- Field paths like `query.hostname` are nested in JSON
- Not user-friendly for analysts

---

## 3️⃣ Semantic Layer (View in Databricks)

### View: `v_dns_event`
**Generated SQL:**

```sql
CREATE OR REPLACE VIEW v_dns_event AS
SELECT
    metadata_uid,
    class_uid,
    category_uid,
    time,
    severity_id,
    
    -- Semantic attributes with friendly names
    raw_data:query.hostname AS `query_hostname`,
    raw_data:query.type AS `query_type`,
    raw_data:src_endpoint.ip AS `source_ip`,
    raw_data:src_endpoint.port AS `source_port`,
    rcode AS `response_code`,
    action AS `action`,
    severity AS `severity`,
    raw_data:cloud.provider AS `cloud_provider`,
    raw_data:cloud.region AS `cloud_region`,
    time AS `event_time`
    
FROM ocsf_class_4003
WHERE class_uid = 4003;
```

### Sample Row in Semantic View

| query_hostname | query_type | source_ip | source_port | response_code | action | severity | cloud_provider | cloud_region | event_time |
|---|---|---|---|---|---|---|---|---|---|
| ip-127-0-0-62.alert.firewall.canary. | A | 10.200.21.100 | 15083 | NoError | Allowed | Informational | AWS | us-east-1 | 2022-10-13 17:02:36 |

**Key Points:**
- Flattened, business-friendly column names
- No need to know OCSF schema structure
- Hides complexity of nested JSON
- Analysts can query using familiar terms

---

## 4️⃣ Hot Path (Observables Table)

### Table: `ocsf_observables`
**Purpose:** Fast threat intelligence matching

```sql
CREATE TABLE `ocsf_observables` (
    `observable_id` STRING NOT NULL,
    `type_id` BIGINT NOT NULL,
    `type_name` STRING NOT NULL,
    `value` STRING,
    `event_uid` STRING NOT NULL,
    `event_class_uid` BIGINT NOT NULL,
    `event_time` TIMESTAMP NOT NULL,
    `attribute_path` STRING NOT NULL,
    `ingestion_time` TIMESTAMP,
    PRIMARY KEY (observable_id)
)
PARTITIONED BY (event_time)
ZORDER BY (type_id, value);
```

### Sample Rows from DNS Event

| observable_id | type_id | type_name | value | event_uid | event_class_uid | attribute_path |
|---|---|---|---|---|---|---|
| obs-001 | 1 | Hostname | ip-127-0-0-62.alert.firewall.canary. | dns-event-12345 | 4003 | query.hostname |
| obs-002 | 2 | IP Address | 10.200.21.100 | dns-event-12345 | 4003 | src_endpoint.ip |
| obs-003 | 2 | IP Address | 127.0.0.62 | dns-event-12345 | 4003 | answers[0].rdata |

**Key Points:**
- Extracted observables for fast lookup
- Optimized with ZORDER for threat intel matching
- Enables hot path queries (fast IOC matching)
- Links back to full event via `event_uid`

---

## 5️⃣ Business Queries

### Query 1: Find Blocked DNS Queries (Semantic Layer)

```sql
-- Analyst-friendly query using semantic view
SELECT 
    query_hostname,
    source_ip,
    cloud_region,
    event_time
FROM v_dns_event
WHERE action = 'Blocked'
  AND event_time >= CURRENT_DATE - INTERVAL 7 DAYS
ORDER BY event_time DESC;
```

**What happens behind the scenes:**
1. Query uses friendly names (`query_hostname`, `source_ip`)
2. View translates to physical layer (`raw_data:query.hostname`, `raw_data:src_endpoint.ip`)
3. Databricks executes against `ocsf_class_4003` table
4. Results returned with semantic column names

### Query 2: NXDOMAIN Rate by Source IP (Metric)

```sql
-- Using pre-defined metric from semantic layer
SELECT 
    source_ip,
    cloud_region,
    COUNT(*) as total_queries,
    SUM(CASE WHEN response_code = 'NXDOMAIN' THEN 1 ELSE 0 END) as nxdomain_count,
    AVG(CASE WHEN response_code = 'NXDOMAIN' THEN 1.0 ELSE 0.0 END) as nxdomain_rate
FROM v_dns_event
WHERE event_time >= CURRENT_DATE - INTERVAL 1 DAYS
GROUP BY source_ip, cloud_region
HAVING nxdomain_rate > 0.3  -- High NXDOMAIN rate indicates DGA
ORDER BY nxdomain_rate DESC;
```

### Query 3: Threat Intel Matching (Hot Path)

```sql
-- Fast IOC matching using observables table
SELECT 
    o.value as malicious_domain,
    o.event_uid,
    ti.threat_type,
    ti.confidence,
    e.source_ip,
    e.event_time
FROM ocsf_observables o
INNER JOIN threat_intel_domains ti 
    ON o.value = ti.domain
INNER JOIN v_dns_event e 
    ON o.event_uid = e.metadata_uid
WHERE o.type_id = 1  -- Hostname
  AND o.event_time >= CURRENT_DATE - INTERVAL 1 DAYS
ORDER BY ti.confidence DESC, e.event_time DESC;
```

**Performance:**
- Hot path query on `ocsf_observables` (ZORDER optimized)
- Fast IOC matching against threat intel
- Reverse lookup to full event for context

---

## 6️⃣ Complete Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Raw DNS Logs (S3)                       │
│  s3://my-bucket/dns-logs/2022/10/13/dns-query.json         │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    [Databricks Ingestion]
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Physical Layer (Delta Lake)                    │
│                                                             │
│  Table: ocsf_class_4003                                    │
│  ┌───────────────────────────────────────────────────┐    │
│  │ metadata_uid │ class_uid │ time │ raw_data │ ... │    │
│  │ dns-event-12345 │ 4003 │ 2022-10-13 │ {...} │    │    │
│  └───────────────────────────────────────────────────┘    │
│                                                             │
│  - Nested JSON in raw_data column                         │
│  - Requires OCSF schema knowledge                         │
│  - Partitioned by date                                    │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    [ETL: Extract Observables]
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Hot Path (Observables Table)                   │
│                                                             │
│  Table: ocsf_observables                                   │
│  ┌───────────────────────────────────────────────────┐    │
│  │ type_id │ value │ event_uid │ event_time │ ...   │    │
│  │ 1 │ ip-127-0-0-62.alert... │ dns-event-12345 │   │    │
│  │ 2 │ 10.200.21.100 │ dns-event-12345 │ ...       │    │
│  └───────────────────────────────────────────────────┘    │
│                                                             │
│  - Fast IOC matching (ZORDER optimized)                   │
│  - Links back to full events                              │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    [Semantic View Definition]
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Semantic Layer (View)                          │
│                                                             │
│  View: v_dns_event                                         │
│  ┌───────────────────────────────────────────────────┐    │
│  │ query_hostname │ source_ip │ action │ severity │  │    │
│  │ ip-127-0-0-62... │ 10.200.21.100 │ Allowed │ ... │    │
│  └───────────────────────────────────────────────────┘    │
│                                                             │
│  - Business-friendly column names                          │
│  - Flattened structure                                     │
│  - Hides OCSF complexity                                   │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    [Analyst Queries]
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                   Business Queries                          │
│                                                             │
│  SELECT query_hostname, source_ip                          │
│  FROM v_dns_event                                          │
│  WHERE action = 'Blocked'                                  │
│                                                             │
│  - Natural language-like SQL                               │
│  - No OCSF schema knowledge needed                         │
│  - Synonym support (domain = query_hostname)               │
└─────────────────────────────────────────────────────────────┘
```

---

## 7️⃣ Synonym Support

The semantic layer supports natural language synonyms:

```sql
-- All these queries work the same way:

-- Using semantic name
SELECT query_hostname FROM v_dns_event;

-- Using synonym "domain"
SELECT query_hostname as domain FROM v_dns_event;

-- Using synonym "fqdn"
SELECT query_hostname as fqdn FROM v_dns_event;

-- Using synonym "client_ip" for source_ip
SELECT source_ip as client_ip FROM v_dns_event;
```

**27 Total Synonyms Available:**
- query_hostname: domain, fqdn, hostname, queried_domain
- query_type: record_type, dns_type, qtype
- source_ip: client_ip, src_ip, source_address
- source_port: client_port, src_port
- response_code: rcode, dns_response, response_status
- action: disposition, verdict, dns_action
- severity: severity_level, priority
- cloud_provider: csp, provider
- cloud_region: region, datacenter
- event_time: timestamp, time, query_time

---

## 8️⃣ Deployment Steps

### Step 1: Create Physical Table
```sql
-- Run in Databricks SQL Editor
CREATE TABLE `ocsf_class_4003` (
    `metadata_uid` STRING NOT NULL,
    `class_uid` BIGINT NOT NULL,
    `time` TIMESTAMP NOT NULL,
    `raw_data` STRING,
    `observables` ARRAY<STRUCT<type: STRING, type_id: BIGINT, value: STRING>>,
    PRIMARY KEY (metadata_uid)
)
PARTITIONED BY (DATE(time))
LOCATION 's3://my-databricks-bucket/ocsf/class_4003/';
```

### Step 2: Ingest Raw Logs
```python
# Databricks notebook
from pyspark.sql import SparkSession

# Read DNS logs from S3
df = spark.read.json("s3://my-bucket/dns-logs/2022/10/13/*.json")

# Write to Delta table
df.write.format("delta") \
  .mode("append") \
  .partitionBy("time") \
  .saveAsTable("ocsf_class_4003")
```

### Step 3: Create Semantic View
```sql
-- Run generated SQL from: demo/output/databricks/views/v_dns_event.sql
CREATE OR REPLACE VIEW v_dns_event AS
SELECT
    raw_data:query.hostname AS `query_hostname`,
    raw_data:src_endpoint.ip AS `source_ip`,
    -- ... (full SQL from generated file)
FROM ocsf_class_4003
WHERE class_uid = 4003;
```

### Step 4: Create Observables Table
```sql
-- Run generated SQL from: demo/output/databricks/observables/observables_table.sql
CREATE TABLE `ocsf_observables` (
    `observable_id` STRING NOT NULL,
    `type_id` BIGINT NOT NULL,
    `value` STRING,
    -- ... (full SQL from generated file)
)
PARTITIONED BY (event_time)
ZORDER BY (type_id, value);
```

### Step 5: Extract Observables (ETL)
```sql
-- Run generated SQL from: demo/output/databricks/etl/extract_observables.sql
INSERT INTO ocsf_observables
SELECT
    CONCAT(metadata_uid, '-', type_id, '-', value) as observable_id,
    type_id,
    type_name,
    value,
    metadata_uid as event_uid,
    class_uid as event_class_uid,
    time as event_time,
    name as attribute_path,
    CURRENT_TIMESTAMP() as ingestion_time
FROM ocsf_class_4003
LATERAL VIEW EXPLODE(observables) obs AS type_id, type_name, value, name
WHERE class_uid = 4003;
```

---

## 9️⃣ Benefits Summary

### For Data Engineers
- **Automated generation** - No manual view creation
- **Consistent naming** - Standard semantic layer across all OCSF classes
- **Performance optimization** - Hot path for fast queries
- **Easy maintenance** - Regenerate when schema changes

### For Security Analysts
- **No OCSF knowledge needed** - Query using business terms
- **Natural language synonyms** - Use familiar terminology
- **Pre-built metrics** - NXDOMAIN rate, blocked DNS rate, etc.
- **Threat context** - MITRE ATT&CK mappings included

### For Threat Hunters
- **Fast IOC matching** - Hot path observables table
- **Reverse lookup** - From observable to full event
- **Threat intel integration** - Join with external IOC feeds
- **Security metrics** - DGA detection, DNS tunneling indicators

---

## 🔟 Files Generated

All artifacts are in: `demo/output/databricks/`

- `views/v_dns_event.sql` - Semantic view definition
- `tables/all_tables.sql` - Physical table schemas
- `observables/observables_table.sql` - Hot path table
- `etl/extract_observables.sql` - Observable extraction ETL
- `dbt/semantic_manifest.yml` - dbt semantic layer config
- `cubejs/DnsEvent.js` - Cube.js schema for BI tools

---

## 📚 Additional Resources

- **Detailed Diagram:** http://localhost:8888/dns-diagram.html
- **Dashboard:** http://localhost:8888/viz/
- **Status Page:** http://localhost:8888/STATUS.html
- **Model File:** `demo/dns-semantic-model-enhanced.yaml`
- **Test Event:** `demo/test-dns-event.json`
