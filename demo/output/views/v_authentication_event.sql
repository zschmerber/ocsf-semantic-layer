CREATE OR REPLACE VIEW v_authentication_event AS
SELECT
    metadata_uid,
    class_uid,
    category_uid,
    time,
    severity_id,
    status_id,
    raw_data:actor.user.email_addr AS "user_email",
    CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END AS "auth_result",
    raw_data:src_endpoint.ip AS "source_ip",
    time AS "event_time"
FROM ocsf_class_3002
WHERE class_uid = 3002;