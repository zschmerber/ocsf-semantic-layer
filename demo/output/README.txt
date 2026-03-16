OCSF Semantic Layer - Demo Output
==================================

Web Server Running: http://localhost:8888

The visualization dashboard is now accessible at the URL above.

Available Tabs:
---------------
1. About - Comprehensive overview with architecture diagrams
2. Graph - Interactive visualization of semantic layer
3. Model - View entities and metrics
4. Artifacts - Browse generated warehouse artifacts
5. Query - Translate semantic queries to SQL

Demo Data:
----------
- DNS test event: ../test-dns-event.json
- Enhanced DNS model: ../dns-semantic-model-enhanced.yaml
- Graph visualization: graph.json
- LLM context export: llm-context.json (19 KB, ~4,760 tokens)
- LLM minimal export: llm-context-minimal.json (14 KB, ~3,556 tokens)

Generated Artifacts:
--------------------
- dbt/ - dbt semantic layer manifests
- cubejs/ - Cube.js schema definitions
- views/ - SQL view definitions
- tables/ - Table schema definitions
- observables/ - Observable extraction tables
- etl/ - ETL pipeline SQL

Features Demonstrated:
----------------------
✓ Schema-driven generation (93 entities from OCSF v1.6.0)
✓ LLM-friendly context export (27 synonyms, 12 query templates)
✓ DNS threat detection with hot/cold path analytics
✓ Observable extraction for threat intelligence
✓ Warehouse artifact generation (Snowflake, dbt, Cube.js)
✓ Interactive visualization with multiple view modes
✓ Query translation from semantic terms to SQL

Test Results:
-------------
✓ 576 tests passing across all crates
✓ All CLI commands working
✓ End-to-end demo completed successfully
