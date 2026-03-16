-- Dedicated observables table for hot path analytics and threat intel matching
CREATE TABLE `ocsf_observables` (
    `observable_id` STRING NOT NULL,
    `type_id` BIGINT NOT NULL,
    `type_name` STRING NOT NULL,
    `value` STRING,
    `event_uid` STRING NOT NULL,
    `event_class_uid` BIGINT NOT NULL,
    `event_time` TIMESTAMP NOT NULL,
    `attribute_path` STRING NOT NULL,
    `ingestion_time` TIMESTAMP,
    PRIMARY KEY (observable_id)
)
PARTITIONED BY (event_time);

-- Databricks: Consider ZORDER BY (type_id, value) for index-like performance

-- Databricks: Consider ZORDER BY (event_uid) for index-like performance

-- Databricks: Consider ZORDER BY (event_time) for index-like performance