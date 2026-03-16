-- Authentication events
CREATE TABLE `ocsf_authentication` (
    `metadata_uid` STRING NOT NULL,
    `metadata_version` STRING,
    `metadata_product` STRING,
    `class_uid` BIGINT NOT NULL,
    `class_name` STRING,
    `category_uid` BIGINT NOT NULL,
    `category_name` STRING,
    `type_uid` BIGINT,
    `type_name` STRING,
    `activity_id` BIGINT,
    `activity_name` STRING,
    `time` TIMESTAMP NOT NULL,
    `time_dt` TIMESTAMP,
    `severity_id` BIGINT,
    `severity` STRING,
    `status_id` BIGINT,
    `status` STRING,
    `status_code` STRING,
    `status_detail` STRING,
    `message` STRING,
    `raw_data` STRING,
    `observables` ARRAY<STRING>,
    `src_endpoint` STRING,
    `metadata` STRING NOT NULL,
    `status_id` BIGINT NOT NULL,
    `time` TIMESTAMP NOT NULL,
    `actor` STRING NOT NULL,
    PRIMARY KEY (metadata_uid)
)
PARTITIONED BY (time);

