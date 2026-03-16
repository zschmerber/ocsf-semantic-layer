cube(`DnsEvent`, {
  sql: `SELECT * FROM ocsf_dns_event`,

  title: `DNS Event`,
  description: `DNS queries and responses with threat detection capabilities`,

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
    nxdomain_rate: {
      type: `avg`,
      sql: `CASE WHEN response_code = 'NXDOMAIN' THEN 1.0 ELSE 0.0 END`,
      description: `Percentage of DNS queries resulting in NXDOMAIN (potential DGA indicator)`,
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
      description: `The fully qualified domain name being queried`,
    },
    query_type: {
      sql: `query.type`,
      type: `string`,
      description: `DNS record type being queried`,
    },
    source_ip: {
      sql: `src_endpoint.ip`,
      type: `string`,
      description: `IP address of the client making the DNS query`,
    },
    source_port: {
      sql: `src_endpoint.port`,
      type: `number`,
      description: `Source port number of the DNS query`,
    },
    response_code: {
      sql: `rcode`,
      type: `string`,
      description: `DNS response code (NOERROR, NXDOMAIN, SERVFAIL, etc.)`,
    },
    action: {
      sql: `action`,
      type: `string`,
      description: `Action taken on the DNS query (Allowed, Blocked, etc.)`,
    },
    severity: {
      sql: `severity`,
      type: `string`,
      description: `Event severity level`,
    },
    cloud_provider: {
      sql: `cloud.provider`,
      type: `string`,
      description: `Cloud service provider hosting the DNS infrastructure`,
    },
    cloud_region: {
      sql: `cloud.region`,
      type: `string`,
      description: `Cloud region where the DNS query originated`,
    },
    event_time: {
      sql: `time`,
      type: `time`,
      description: `Timestamp when the DNS query occurred`,
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