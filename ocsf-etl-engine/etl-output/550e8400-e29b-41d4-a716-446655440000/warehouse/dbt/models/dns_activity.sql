SELECT
    query_hostname AS query_hostname,
    src_endpoint_ip AS src_endpoint_ip,
    dst_endpoint_ip AS dst_endpoint_ip
FROM {{ source('ocsf', 'ocsf_4003') }}
