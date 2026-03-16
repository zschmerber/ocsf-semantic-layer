{{ config(materialized='view') }}

SELECT
    actor.user.email_addr AS user_email,
    CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END AS auth_result,
    src_endpoint.ip AS source_ip,
    time AS event_time,
    time,
    metadata_uid,
    class_uid
FROM {{ source('ocsf', 'ocsf_class_3002') }}
