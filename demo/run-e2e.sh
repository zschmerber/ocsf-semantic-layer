#!/bin/bash
# End-to-end demo script for OCSF Semantic Layer
set -e

echo "=========================================="
echo "OCSF Semantic Layer - End-to-End Demo"
echo "=========================================="

# Create output directory
mkdir -p demo/output

echo ""
echo "Step 1: Initialize a semantic model"
echo "------------------------------------"
cargo run -p ocsf-cli -- init -o ./demo/semantic-model.yaml -n "security-analytics" -v
echo ""
echo "Generated model:"
cat ./demo/semantic-model.yaml
echo ""

echo "Step 2: Add entities and metrics to the model"
echo "----------------------------------------------"
# Create a more complete model for testing
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

echo "Updated model:"
cat ./demo/semantic-model.yaml
echo ""

echo "Step 3: Validate the model"
echo "--------------------------"
cargo run -p ocsf-cli -- validate -m ./demo/semantic-model.yaml --json -v 2>&1 || true
echo ""

echo "Step 4: Generate warehouse artifacts"
echo "-------------------------------------"
cargo run -p ocsf-cli -- generate -m ./demo/semantic-model.yaml -o ./demo/output -d snowflake -v
echo ""
echo "Generated files:"
find ./demo/output -type f | head -20
echo ""

echo "Step 5: View generated dbt semantic manifest"
echo "---------------------------------------------"
if [ -f ./demo/output/dbt/semantic_manifest.yml ]; then
    cat ./demo/output/dbt/semantic_manifest.yml
fi
echo ""

echo "Step 6: View generated SQL views"
echo "---------------------------------"
if [ -f ./demo/output/views/authentication_event.sql ]; then
    cat ./demo/output/views/authentication_event.sql
fi
echo ""

echo "Step 7: View generated observables table"
echo "-----------------------------------------"
if [ -f ./demo/output/tables/ocsf_observables.sql ]; then
    cat ./demo/output/tables/ocsf_observables.sql
fi
echo ""

echo "Step 8: Visualize the semantic layer (JSON graph)"
echo "--------------------------------------------------"
cargo run -p ocsf-cli -- visualize -m ./demo/semantic-model.yaml -o ./demo/output/graph.json -f json -v
echo ""
echo "Graph data (first 100 lines):"
head -100 ./demo/output/graph.json
echo ""

echo "Step 9: Translate semantic queries to SQL"
echo "------------------------------------------"
echo "Query 1: Simple attribute selection"
cargo run -p ocsf-cli -- query -m ./demo/semantic-model.yaml "authentication_event.user_email,auth_result" -f json
echo ""

echo "Query 2: With metric aggregation"
cargo run -p ocsf-cli -- query -m ./demo/semantic-model.yaml '{"entity":"authentication_event","select":["user_email"],"metrics":["auth_attempts"],"group_by":["user_email"]}' -f json
echo ""

echo "Query 3: Hot path query (threat intel)"
cargo run -p ocsf-cli -- query -m ./demo/semantic-model.yaml '{"entity":"authentication_event","metrics":["threat_intel_matches"],"path_preference":"hot"}' -f json
echo ""

echo "=========================================="
echo "End-to-end demo complete!"
echo "=========================================="
echo ""
echo "Output files are in ./demo/output/"
echo "  - dbt/           : dbt semantic layer artifacts"
echo "  - cubejs/        : Cube.js schema files"
echo "  - views/         : SQL view definitions"
echo "  - tables/        : Table DDL statements"
echo "  - etl/           : ETL pipeline definitions"
echo "  - graph.json     : Visualization graph data"
EOF
