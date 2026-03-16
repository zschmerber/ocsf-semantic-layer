-- OCSF DNS Activity Table Schema
-- Generated for Snowflake

CREATE TABLE IF NOT EXISTS ocsf_dns_event (
    -- Core OCSF fields
    metadata_uid VARCHAR(255) PRIMARY KEY,
    class_uid INTEGER NOT NULL DEFAULT 4003,
    category_uid INTEGER NOT NULL DEFAULT 4,
    time TIMESTAMP_NTZ NOT NULL,
    time_dt TIMESTAMP_NTZ,
    severity_id INTEGER,
    severity VARCHAR(50),
    status_id INTEGER,
    
    -- DNS Activity specific fields
    activity_id INTEGER,
    activity_name VARCHAR(100),
    action VARCHAR(50),
    action_id INTEGER,
    rcode VARCHAR(50),
    rcode_id INTEGER,
    disposition VARCHAR(50),
    
    -- Query object (flattened)
    query_hostname VARCHAR(500),
    query_type VARCHAR(50),
    query_class VARCHAR(50),
    
    -- Source endpoint
    src_endpoint_ip VARCHAR(45),
    src_endpoint_port INTEGER,
    src_endpoint_vpc_uid VARCHAR(255),
    
    -- Destination endpoint
    dst_endpoint_instance_uid VARCHAR(255),
    dst_endpoint_interface_uid VARCHAR(255),
    
    -- Cloud context
    cloud_provider VARCHAR(50),
    cloud_region VARCHAR(100),
    cloud_account_uid VARCHAR(255),
    
    -- Connection info
    connection_info_protocol_name VARCHAR(50),
    connection_info_direction VARCHAR(50),
    
    -- Raw data for nested objects
    raw_data VARIANT,
    
    -- Metadata
    created_at TIMESTAMP_NTZ DEFAULT CURRENT_TIMESTAMP()
);

-- Index for common query patterns
CREATE INDEX IF NOT EXISTS idx_dns_time ON ocsf_dns_event(time);
CREATE INDEX IF NOT EXISTS idx_dns_src_ip ON ocsf_dns_event(src_endpoint_ip);
CREATE INDEX IF NOT EXISTS idx_dns_hostname ON ocsf_dns_event(query_hostname);
CREATE INDEX IF NOT EXISTS idx_dns_action ON ocsf_dns_event(action);
