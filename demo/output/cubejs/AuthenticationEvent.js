cube(`AuthenticationEvent`, {
  sql: `SELECT * FROM ocsf_authentication_event`,

  title: `Authentication Event`,
  description: `User authentication attempts across all systems`,

  measures: {
    count: {
      type: `count`,
    },
    auth_attempts: {
      type: `count`,
      sql: `metadata.uid`,
      description: `Count of authentication attempts`,
    },
    failed_auth_rate: {
      type: `avg`,
      sql: `CASE WHEN status_id != 1 THEN 1.0 ELSE 0.0 END`,
      description: `Percentage of failed authentication attempts`,
    },
  },

  dimensions: {
    id: {
      sql: `metadata_uid`,
      type: `string`,
      primaryKey: true,
    },
    user_email: {
      sql: `actor.user.email_addr`,
      type: `string`,
    },
    auth_result: {
      sql: `CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END`,
      type: `string`,
    },
    source_ip: {
      sql: `src_endpoint.ip`,
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