-- Authentication events
CREATE TABLE "ocsf_authentication" (
    "metadata_uid" VARCHAR NOT NULL,
    "metadata_version" VARCHAR,
    "metadata_product" VARIANT,
    "class_uid" INTEGER NOT NULL,
    "class_name" VARCHAR,
    "category_uid" INTEGER NOT NULL,
    "category_name" VARCHAR,
    "type_uid" INTEGER,
    "type_name" VARCHAR,
    "activity_id" INTEGER,
    "activity_name" VARCHAR,
    "time" TIMESTAMP_NTZ NOT NULL,
    "time_dt" TIMESTAMP_NTZ,
    "severity_id" INTEGER,
    "severity" VARCHAR,
    "status_id" INTEGER,
    "status" VARCHAR,
    "status_code" VARCHAR,
    "status_detail" VARCHAR,
    "message" VARCHAR,
    "raw_data" VARCHAR,
    "observables" ARRAY,
    "src_endpoint" VARCHAR,
    "metadata" VARCHAR NOT NULL,
    "status_id" INTEGER NOT NULL,
    "actor" VARCHAR NOT NULL,
    "time" TIMESTAMP_NTZ NOT NULL,
    PRIMARY KEY (metadata_uid)
)
CLUSTER BY (class_uid, category_uid);

