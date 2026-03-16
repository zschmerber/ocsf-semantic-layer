-- Observable extraction from ocsf_events to ocsf_observables
INSERT INTO ocsf_observables
(
    observable_id,
    type_id,
    type_name,
    value,
    event_uid,
    event_class_uid,
    event_time,
    attribute_path,
    ingestion_time
)
SELECT
    metadata_uid || '_' || 'src_endpoint_ip' AS observable_id,
    2 AS type_id,
    'IP Address' AS type_name,
    raw_data:src_endpoint.ip AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'src_endpoint.ip' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:src_endpoint.ip IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'dst_endpoint_ip' AS observable_id,
    2 AS type_id,
    'IP Address' AS type_name,
    raw_data:dst_endpoint.ip AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'dst_endpoint.ip' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:dst_endpoint.ip IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'actor_user_ip' AS observable_id,
    2 AS type_id,
    'IP Address' AS type_name,
    raw_data:actor.user.ip AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'actor.user.ip' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:actor.user.ip IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'actor_user_email_addr' AS observable_id,
    5 AS type_id,
    'Email Address' AS type_name,
    raw_data:actor.user.email_addr AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'actor.user.email_addr' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:actor.user.email_addr IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'user_email_addr' AS observable_id,
    5 AS type_id,
    'Email Address' AS type_name,
    raw_data:user.email_addr AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'user.email_addr' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:user.email_addr IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'actor_user_name' AS observable_id,
    10 AS type_id,
    'User Name' AS type_name,
    raw_data:actor.user.name AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'actor.user.name' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:actor.user.name IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'user_name' AS observable_id,
    10 AS type_id,
    'User Name' AS type_name,
    raw_data:user.name AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'user.name' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:user.name IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'src_endpoint_hostname' AS observable_id,
    22 AS type_id,
    'Hostname' AS type_name,
    raw_data:src_endpoint.hostname AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'src_endpoint.hostname' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:src_endpoint.hostname IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'dst_endpoint_hostname' AS observable_id,
    22 AS type_id,
    'Hostname' AS type_name,
    raw_data:dst_endpoint.hostname AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'dst_endpoint.hostname' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:dst_endpoint.hostname IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'file_hashes_md5' AS observable_id,
    30 AS type_id,
    'File Hash' AS type_name,
    raw_data:file.hashes.md5 AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'file.hashes.md5' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:file.hashes.md5 IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'file_hashes_sha1' AS observable_id,
    30 AS type_id,
    'File Hash' AS type_name,
    raw_data:file.hashes.sha1 AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'file.hashes.sha1' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:file.hashes.sha1 IS NOT NULL
UNION ALL
SELECT
    metadata_uid || '_' || 'file_hashes_sha256' AS observable_id,
    30 AS type_id,
    'File Hash' AS type_name,
    raw_data:file.hashes.sha256 AS value,
    metadata_uid AS event_uid,
    class_uid AS event_class_uid,
    time AS event_time,
    'file.hashes.sha256' AS attribute_path,
    CURRENT_TIMESTAMP() AS ingestion_time
FROM ocsf_events
WHERE raw_data:file.hashes.sha256 IS NOT NULL;