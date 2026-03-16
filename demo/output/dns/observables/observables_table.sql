-- Dedicated observables table for hot path analytics and threat intel matching
CREATE TABLE "ocsf_observables" (
    "observable_id" VARCHAR NOT NULL,
    "type_id" INTEGER NOT NULL,
    "type_name" VARCHAR NOT NULL,
    "value" VARCHAR,
    "event_uid" VARCHAR NOT NULL,
    "event_class_uid" INTEGER NOT NULL,
    "event_time" TIMESTAMP_NTZ NOT NULL,
    "attribute_path" VARCHAR NOT NULL,
    "ingestion_time" TIMESTAMP_NTZ,
    PRIMARY KEY (observable_id)
)
CLUSTER BY (type_id, value);

-- Snowflake: Consider CLUSTER BY (type_id, value) for index-like performance

-- Snowflake: Consider CLUSTER BY (event_uid) for index-like performance

-- Snowflake: Consider CLUSTER BY (event_time) for index-like performance