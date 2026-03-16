CREATE OR REPLACE VIEW v_dns_activity AS
SELECT
    query_hostname AS query_hostname,
    src_endpoint_ip AS src_endpoint_ip,
    dst_endpoint_ip AS dst_endpoint_ip
FROM ocsf_4003;

