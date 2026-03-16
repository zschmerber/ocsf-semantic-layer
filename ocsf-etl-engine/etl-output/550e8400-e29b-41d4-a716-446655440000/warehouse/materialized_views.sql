CREATE MATERIALIZED VIEW IF NOT EXISTS mv_dns_activity_hot AS
SELECT
    query_hostname,
    src_endpoint_ip,
    dst_endpoint_ip,
    COUNT(*) AS event_count
FROM ocsf_4003
GROUP BY query_hostname, src_endpoint_ip, dst_endpoint_ip
;

