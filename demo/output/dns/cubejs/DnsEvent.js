cube(`DnsEvent`, {
  sql: `SELECT * FROM ocsf_dns_event`,

  title: `DNS Event`,
  description: `DNS queries and responses`,

  measures: {
    count: {
      type: `count`,
    },
    dns_query_count: {
      type: `count`,
      sql: `metadata.uid`,
      description: `Total number of DNS queries`,
    },
    blocked_dns_rate: {
      type: `avg`,
      sql: `CASE WHEN action != 'Allowed' THEN 1.0 ELSE 0.0 END`,
      description: `Percentage of blocked DNS queries`,
    },
    dns_by_severity: {
      type: `count`,
      sql: `metadata.uid`,
      description: `Count of DNS events grouped by severity`,
    },
  },

  dimensions: {
    id: {
      sql: `metadata_uid`,
      type: `string`,
      primaryKey: true,
    },
    query_hostname: {
      sql: `query.hostname`,
      type: `string`,
    },
    query_type: {
      sql: `query.type`,
      type: `string`,
    },
    source_ip: {
      sql: `src_endpoint.ip`,
      type: `string`,
    },
    source_port: {
      sql: `src_endpoint.port`,
      type: `number`,
    },
    response_code: {
      sql: `rcode`,
      type: `string`,
    },
    action: {
      sql: `action`,
      type: `string`,
    },
    severity: {
      sql: `severity`,
      type: `string`,
    },
    cloud_provider: {
      sql: `cloud.provider`,
      type: `string`,
    },
    cloud_region: {
      sql: `cloud.region`,
      type: `string`,
    },
    event_time: {
      sql: `time`,
      type: `time`,
    },
    time: {
      sql: `time`,
      type: `time`,
    },
  },

  preAggregations: {
    // Define pre-aggregations here for better performance
  },
});