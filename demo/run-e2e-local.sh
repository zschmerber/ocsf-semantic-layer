#!/bin/bash
# End-to-end demo script for OCSF Semantic Layer (local schema)
set -e

echo "=========================================="
echo "OCSF Semantic Layer - End-to-End Demo"
echo "=========================================="

# Create output directory
mkdir -p demo/output
mkdir -p demo/schema

echo ""
echo "Step 1: Create a local mock OCSF schema"
echo "----------------------------------------"

# Create version.json
cat > demo/schema/version.json << 'EOF'
{"version": "1.4.0"}
EOF

# Create dictionary.json
cat > demo/schema/dictionary.json << 'EOF'
{
    "attributes": {
        "user_name": {
            "type": "string_t",
            "caption": "User Name",
            "description": "The name of the user",
            "requirement": "required",
            "observable": 10
        },
        "email_addr": {
            "type": "string_t",
            "caption": "Email Address",
            "description": "An email address",
            "observable": 5
        },
        "ip": {
            "type": "string_t",
            "caption": "IP Address",
            "description": "An IP address",
            "observable": 2
        },
        "status_id": {
            "type": "integer_t",
            "caption": "Status ID",
            "description": "The status identifier"
        },
        "time": {
            "type": "timestamp_t",
            "caption": "Event Time",
            "description": "The event timestamp"
        }
    }
}
EOF

# Create categories.json
cat > demo/schema/categories.json << 'EOF'
{
    "attributes": {
        "iam": {
            "uid": 3,
            "caption": "Identity & Access Management",
            "description": "IAM events"
        },
        "system": {
            "uid": 1,
            "caption": "System Activity",
            "description": "System events"
        }
    }
}
EOF

# Create objects directory
mkdir -p demo/schema/objects

cat > demo/schema/objects/user.json << 'EOF'
{
    "name": "user",
    "caption": "User",
    "description": "The user object",
    "attributes": {
        "name": {
            "type": "string_t",
            "caption": "Name",
            "requirement": "required"
        },
        "email_addr": {
            "type": "string_t",
            "caption": "Email Address",
            "observable": 5
        }
    }
}
EOF

cat > demo/schema/objects/endpoint.json << 'EOF'
{
    "name": "endpoint",
    "caption": "Endpoint",
    "description": "The endpoint object",
    "attributes": {
        "ip": {
            "type": "string_t",
            "caption": "IP Address",
            "observable": 2
        },
        "hostname": {
            "type": "string_t",
            "caption": "Hostname"
        }
    }
}
EOF

cat > demo/schema/objects/actor.json << 'EOF'
{
    "name": "actor",
    "caption": "Actor",
    "description": "The actor object",
    "attributes": {
        "user": {
            "type": "object_t",
            "object_type": "user",
            "caption": "User"
        }
    }
}
EOF

cat > demo/schema/objects/metadata.json << 'EOF'
{
    "name": "metadata",
    "caption": "Metadata",
    "description": "Event metadata",
    "attributes": {
        "uid": {
            "type": "string_t",
            "caption": "Unique ID",
            "requirement": "required"
        }
    }
}
EOF

# Create events directory
mkdir -p demo/schema/events/iam

cat > demo/schema/events/iam/authentication.json << 'EOF'
{
    "uid": 3002,
    "name": "authentication",
    "category": "3",
    "caption": "Authentication",
    "description": "Authentication events",
    "attributes": {
        "actor": {
            "requirement": "required",
            "group": "primary"
        },
        "src_endpoint": {
            "requirement": "recommended"
        },
        "status_id": {
            "requirement": "required"
        },
        "time": {
            "requirement": "required"
        },
        "metadata": {
            "requirement": "required"
        }
    },
    "observables": [
        {
            "type_id": 10,
            "type_name": "User Name",
            "type": "by_path",
            "path": "actor.user.name"
        },
        {
            "type_id": 5,
            "type_name": "Email Address",
            "type": "by_path",
            "path": "actor.user.email_addr"
        },
        {
            "type_id": 2,
            "type_name": "IP Address",
            "type": "by_path",
            "path": "src_endpoint.ip"
        }
    ]
}
EOF

echo "Local schema created in demo/schema/"
ls -la demo/schema/
echo ""

echo "Step 2: Create a semantic model"
echo "--------------------------------"
cat > ./demo/semantic-model.yaml << 'EOF'
version: "1.0"
ocsf_version: "1.4.0"
name: security-analytics
description: Semantic layer for security analytics on OCSF data

entities:
  - name: authentication_event
    caption: Authentication Event
    description: User authentication attempts across all systems
    source_event_classes:
      - 3002
    covers_observables:
      - 5
      - 10
    attributes:
      - name: user_email
        caption: User Email
        type: string
        ocsf_mapping:
          field: actor.user.email_addr
        is_dimension: true
      - name: auth_result
        caption: Authentication Result
        type: string
        ocsf_mapping:
          expression: "CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END"
        is_dimension: true
      - name: source_ip
        caption: Source IP Address
        type: string
        ocsf_mapping:
          field: src_endpoint.ip
        is_dimension: true
      - name: event_time
        caption: Event Time
        type: timestamp
        ocsf_mapping:
          field: time

metrics:
  - name: auth_attempts
    caption: Authentication Attempts
    description: Count of authentication attempts
    aggregation: count
    measure:
      field: metadata.uid
    dimensions:
      - user_email
      - auth_result
    time_granularities:
      - hour
      - day
    is_hot_path: false

  - name: failed_auth_rate
    caption: Failed Authentication Rate
    description: Percentage of failed authentication attempts
    aggregation: avg
    measure:
      expression: "CASE WHEN status_id != 1 THEN 1.0 ELSE 0.0 END"
    dimensions:
      - user_email
      - source_ip
    time_granularities:
      - hour
      - day

  - name: threat_intel_matches
    caption: Threat Intel Matches
    description: Count of observables matching threat intelligence
    aggregation: count
    measure:
      field: observable_value
    dimensions: []
    is_hot_path: true
    observable_type_id: 2

observable_config:
  extract_to_table: true
  table_name: ocsf_observables
  include_types:
    - 2
    - 5
    - 10
EOF

echo "Semantic model:"
cat ./demo/semantic-model.yaml
echo ""

echo "Step 3: Validate the model against local schema"
echo "------------------------------------------------"
cargo run -p ocsf-cli -- validate -m ./demo/semantic-model.yaml -s ./demo/schema --json -v 2>&1 || true
echo ""

echo "Step 4: Generate warehouse artifacts"
echo "-------------------------------------"
cargo run -p ocsf-cli -- generate -m ./demo/semantic-model.yaml -o ./demo/output -d snowflake --schema ./demo/schema -v
echo ""
echo "Generated files:"
find ./demo/output -type f 2>/dev/null | head -20
echo ""

echo "Step 5: View generated dbt semantic manifest"
echo "---------------------------------------------"
if [ -f ./demo/output/dbt/semantic_manifest.yml ]; then
    echo "=== dbt/semantic_manifest.yml ==="
    cat ./demo/output/dbt/semantic_manifest.yml
fi
echo ""

echo "Step 6: View generated SQL views"
echo "---------------------------------"
if [ -f ./demo/output/views/authentication_event.sql ]; then
    echo "=== views/authentication_event.sql ==="
    cat ./demo/output/views/authentication_event.sql
fi
echo ""

echo "Step 7: View generated observables table"
echo "-----------------------------------------"
if [ -f ./demo/output/tables/ocsf_observables.sql ]; then
    echo "=== tables/ocsf_observables.sql ==="
    cat ./demo/output/tables/ocsf_observables.sql
fi
echo ""

echo "Step 8: View generated Cube.js schema"
echo "--------------------------------------"
if [ -f ./demo/output/cubejs/authentication_event.js ]; then
    echo "=== cubejs/authentication_event.js ==="
    cat ./demo/output/cubejs/authentication_event.js
fi
echo ""

echo "Step 9: Visualize the semantic layer (JSON graph)"
echo "--------------------------------------------------"
cargo run -p ocsf-cli -- visualize -m ./demo/semantic-model.yaml -o ./demo/output/graph.json -f json --schema ./demo/schema -v
echo ""
echo "=== Graph data (first 80 lines) ==="
head -80 ./demo/output/graph.json
echo ""

echo "Step 10: Translate semantic queries to SQL"
echo "-------------------------------------------"
echo ""
echo "Query 1: Simple attribute selection"
echo "------------------------------------"
cargo run -p ocsf-cli -- query -m ./demo/semantic-model.yaml "authentication_event.user_email,auth_result" -f json
echo ""

echo "Query 2: With metric aggregation"
echo "---------------------------------"
cargo run -p ocsf-cli -- query -m ./demo/semantic-model.yaml '{"entity":"authentication_event","select":["user_email"],"metrics":["auth_attempts"],"group_by":["user_email"]}' -f json
echo ""

echo "Query 3: Hot path query (threat intel)"
echo "---------------------------------------"
cargo run -p ocsf-cli -- query -m ./demo/semantic-model.yaml '{"entity":"authentication_event","metrics":["threat_intel_matches"],"path_preference":"hot"}' -f json
echo ""

echo "Step 11: Export LLM-friendly context from enhanced DNS model"
echo "--------------------------------------------------------------"
if [ -f ./demo/dns-semantic-model-enhanced.yaml ]; then
    echo "Exporting LLM context from enhanced DNS model..."
    cargo run -p ocsf-cli -- export-llm-context \
        -m ./demo/dns-semantic-model-enhanced.yaml \
        -o ./demo/output/llm-context.json \
        --include-dns-templates \
        -v
    echo ""
    echo "LLM context export (first 100 lines):"
    head -100 ./demo/output/llm-context.json
    echo ""
    echo "Export minimal version for smaller context windows..."
    cargo run -p ocsf-cli -- export-llm-context \
        -m ./demo/dns-semantic-model-enhanced.yaml \
        -o ./demo/output/llm-context-minimal.json \
        --minimal \
        --include-dns-templates \
        -v
else
    echo "Enhanced DNS model not found, skipping LLM export"
fi
echo ""

echo "Step 12: Generate semantic model from compiled OCSF schema"
echo "-----------------------------------------------------------"
if [ -f ./demo/schema/ocsf-compiled-v1.6.0.json ]; then
    echo "Using compiled OCSF schema v1.6.0..."
    cargo run -p ocsf-cli -- generate-from-schema \
        -s ./demo/schema/ocsf-compiled-v1.6.0.json \
        -o ./demo/output/generated-from-schema.yaml \
        --include-metrics \
        -v
    echo ""
    echo "Generated model summary (first 50 lines):"
    head -50 ./demo/output/generated-from-schema.yaml
else
    echo "Compiled schema not found, skipping schema-driven generation"
fi
echo ""

echo "=========================================="
echo "End-to-end demo complete!"
echo "=========================================="
echo ""
echo "Output files are in ./demo/output/"
ls -la ./demo/output/
echo ""
echo "Subdirectories:"
for dir in ./demo/output/*/; do
    echo "  $dir"
    ls -la "$dir" 2>/dev/null | head -5
done
EOF
