{{ config(materialized='view') }}

SELECT
    query.hostname AS query_hostname,
    query.type AS query_type,
    src_endpoint.ip AS source_ip,
    src_endpoint.port AS source_port,
    rcode AS response_code,
    action AS action,
    severity AS severity,
    cloud.provider AS cloud_provider,
    cloud.region AS cloud_region,
    time AS event_time,
    time,
    metadata_uid,
    class_uid
FROM {{ source('ocsf', 'ocsf_class_4003') }}
