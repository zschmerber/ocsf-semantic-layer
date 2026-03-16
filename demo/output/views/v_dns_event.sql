CREATE OR REPLACE VIEW v_dns_event AS
SELECT
    metadata_uid,
    class_uid,
    category_uid,
    time,
    severity_id,
    status_id,
    raw_data:query.hostname AS "query_hostname",
    raw_data:query.type AS "query_type",
    raw_data:src_endpoint.ip AS "source_ip",
    raw_data:src_endpoint.port AS "source_port",
    rcode AS "response_code",
    action AS "action",
    severity AS "severity",
    raw_data:cloud.provider AS "cloud_provider",
    raw_data:cloud.region AS "cloud_region",
    time AS "event_time"
FROM ocsf_class_4003
WHERE class_uid = 4003;