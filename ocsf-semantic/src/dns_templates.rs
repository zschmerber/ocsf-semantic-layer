//! Pre-built DNS security query templates.
//!
//! This module provides ready-to-use query templates for common DNS
//! security analytics use cases including tunneling detection, DGA
//! detection, fast flux detection, and C2 beaconing detection.

use crate::query_templates::{
    MitreMapping, ParameterType, QueryCategory, QueryParameter, QueryTemplate, Severity,
};

/// DNS tunneling detection templates.
pub mod tunneling {
    use super::*;

    /// Detect DNS tunneling by identifying hosts with unusually high query volumes.
    pub fn high_volume() -> QueryTemplate {
        QueryTemplate::new(
            "dns_tunneling_high_volume",
            "Detect DNS tunneling by identifying hosts with unusually high query volumes",
            r#"SELECT 
  source_ip,
  COUNT(*) as query_count,
  COUNT(DISTINCT query_hostname) as unique_domains,
  AVG(LENGTH(query_hostname)) as avg_query_length
FROM dns_event
WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
GROUP BY source_ip
HAVING query_count > {{threshold}}
ORDER BY query_count DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Find DNS tunneling".to_string(),
            "Show hosts with high DNS query volume".to_string(),
            "Detect data exfiltration via DNS".to_string(),
            "Which hosts are making too many DNS queries".to_string(),
            "Find suspicious DNS activity".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24").with_description("Time window in hours"))
        .add_parameter(QueryParameter::new("threshold").with_type(ParameterType::Integer).with_default("10000").with_description("Minimum query count to flag"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"))
        .with_mitre_attack(MitreMapping::new("T1071.004", "Application Layer Protocol: DNS", "Command and Control"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::High)
    }

    /// Detect DNS tunneling by finding unusually long DNS queries.
    pub fn long_queries() -> QueryTemplate {
        QueryTemplate::new(
            "dns_tunneling_long_queries",
            "Detect DNS tunneling by finding unusually long DNS queries (data encoded in subdomain)",
            r#"SELECT 
  source_ip,
  query_hostname,
  LENGTH(query_hostname) as query_length,
  time
FROM dns_event
WHERE LENGTH(query_hostname) > {{min_length}}
  AND time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
ORDER BY query_length DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Find long DNS queries".to_string(),
            "Detect encoded data in DNS".to_string(),
            "Show DNS queries with long hostnames".to_string(),
            "Find DNS exfiltration attempts".to_string(),
        ])
        .add_parameter(QueryParameter::new("min_length").with_type(ParameterType::Integer).with_default("50").with_description("Minimum hostname length to flag"))
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"))
        .with_mitre_attack(MitreMapping::new("T1048.003", "Exfiltration Over Alternative Protocol", "Exfiltration"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::High)
    }

    /// Detect potential DNS tunneling via TXT record queries.
    pub fn txt_record_abuse() -> QueryTemplate {
        QueryTemplate::new(
            "dns_txt_record_abuse",
            "Detect potential DNS tunneling via TXT record queries",
            r#"SELECT 
  source_ip,
  query_hostname,
  COUNT(*) as txt_query_count,
  MIN(time) as first_seen,
  MAX(time) as last_seen
FROM dns_event
WHERE query_type = 'TXT'
  AND time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
GROUP BY source_ip, query_hostname
HAVING txt_query_count > {{threshold}}
ORDER BY txt_query_count DESC"#
        )
        .with_patterns(vec![
            "Find TXT record queries".to_string(),
            "Show DNS TXT lookups".to_string(),
            "Detect DNS tunneling via TXT".to_string(),
            "Which hosts are querying TXT records".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("threshold").with_type(ParameterType::Integer).with_default("10"))
        .with_mitre_attack(MitreMapping::new("T1071.004", "Application Layer Protocol: DNS", "Command and Control"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::Medium)
    }
}

/// DGA (Domain Generation Algorithm) detection templates.
pub mod dga {
    use super::*;

    /// Detect DGA activity by finding hosts with high NXDOMAIN response rates.
    pub fn nxdomain_spike() -> QueryTemplate {
        QueryTemplate::new(
            "dga_nxdomain_spike",
            "Detect DGA activity by finding hosts with high NXDOMAIN response rates",
            r#"SELECT 
  source_ip,
  COUNT(*) as total_queries,
  SUM(CASE WHEN response_code = 'NXDOMAIN' THEN 1 ELSE 0 END) as nxdomain_count,
  ROUND(100.0 * SUM(CASE WHEN response_code = 'NXDOMAIN' THEN 1 ELSE 0 END) / COUNT(*), 2) as nxdomain_rate
FROM dns_event
WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
GROUP BY source_ip
HAVING nxdomain_rate > {{nxdomain_threshold}}
  AND total_queries > {{min_queries}}
ORDER BY nxdomain_count DESC"#
        )
        .with_patterns(vec![
            "Find DGA domains".to_string(),
            "Detect domain generation algorithm".to_string(),
            "Show NXDOMAIN spikes".to_string(),
            "Which hosts have failed DNS lookups".to_string(),
            "Find malware beaconing".to_string(),
            "Detect algorithmically generated domains".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("nxdomain_threshold").with_type(ParameterType::Float).with_default("50.0").with_description("Minimum NXDOMAIN percentage to flag"))
        .add_parameter(QueryParameter::new("min_queries").with_type(ParameterType::Integer).with_default("100").with_description("Minimum total queries for statistical significance"))
        .with_mitre_attack(MitreMapping::new("T1568.002", "Dynamic Resolution: Domain Generation Algorithms", "Command and Control"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::High)
    }

    /// Find domains with high entropy (randomness) indicating DGA.
    pub fn random_domains() -> QueryTemplate {
        QueryTemplate::new(
            "dga_random_domains",
            "Find domains with high entropy (randomness) indicating DGA",
            r#"WITH domain_stats AS (
  SELECT 
    query_hostname,
    source_ip,
    COUNT(*) as query_count,
    LENGTH(query_hostname) as domain_length,
    LENGTH(REGEXP_REPLACE(query_hostname, '[^0-9]', '')) as numeric_chars,
    LENGTH(REGEXP_REPLACE(query_hostname, '[^a-z]', '')) as alpha_chars
  FROM dns_event
  WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
    AND LENGTH(query_hostname) > 10
  GROUP BY query_hostname, source_ip
)
SELECT *,
  ROUND(100.0 * numeric_chars / domain_length, 2) as numeric_ratio
FROM domain_stats
WHERE numeric_chars > domain_length * 0.3
  OR domain_length > {{max_normal_length}}
ORDER BY query_count DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Find random looking domains".to_string(),
            "Detect high entropy domains".to_string(),
            "Show suspicious domain names".to_string(),
            "Find gibberish domains".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("max_normal_length").with_type(ParameterType::Integer).with_default("40"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"))
        .with_mitre_attack(MitreMapping::new("T1568.002", "Dynamic Resolution: Domain Generation Algorithms", "Command and Control"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::High)
    }
}

/// Fast flux and C2 detection templates.
pub mod c2 {
    use super::*;

    /// Detect fast flux DNS by finding domains resolving to many different IPs.
    pub fn fast_flux() -> QueryTemplate {
        QueryTemplate::new(
            "fast_flux_detection",
            "Detect fast flux DNS by finding domains resolving to many different IPs",
            r#"SELECT 
  query_hostname,
  COUNT(DISTINCT answers_rdata) as unique_ips,
  COUNT(*) as query_count,
  ARRAY_AGG(DISTINCT answers_rdata) as ip_addresses
FROM dns_event
WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
  AND answers_rdata IS NOT NULL
GROUP BY query_hostname
HAVING unique_ips > {{ip_threshold}}
ORDER BY unique_ips DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Find fast flux domains".to_string(),
            "Detect domains with changing IPs".to_string(),
            "Show bulletproof hosting indicators".to_string(),
            "Find domains with multiple IP addresses".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("ip_threshold").with_type(ParameterType::Integer).with_default("10").with_description("Minimum unique IPs to flag as fast flux"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("50"))
        .with_mitre_attack(MitreMapping::new("T1568.001", "Dynamic Resolution: Fast Flux DNS", "Command and Control"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::High)
    }

    /// Detect C2 beaconing by finding DNS queries at regular time intervals.
    pub fn beaconing() -> QueryTemplate {
        QueryTemplate::new(
            "c2_beaconing_regular_intervals",
            "Detect C2 beaconing by finding DNS queries at regular time intervals",
            r#"WITH query_intervals AS (
  SELECT 
    source_ip,
    query_hostname,
    time,
    LAG(time) OVER (PARTITION BY source_ip, query_hostname ORDER BY time) as prev_time,
    DATEDIFF(second, LAG(time) OVER (PARTITION BY source_ip, query_hostname ORDER BY time), time) as interval_seconds
  FROM dns_event
  WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
)
SELECT 
  source_ip,
  query_hostname,
  COUNT(*) as beacon_count,
  AVG(interval_seconds) as avg_interval,
  STDDEV(interval_seconds) as interval_stddev,
  MIN(time) as first_seen,
  MAX(time) as last_seen
FROM query_intervals
WHERE interval_seconds IS NOT NULL
GROUP BY source_ip, query_hostname
HAVING beacon_count > {{min_beacons}}
  AND interval_stddev < avg_interval * {{jitter_threshold}}
ORDER BY beacon_count DESC"#
        )
        .with_patterns(vec![
            "Find beaconing activity".to_string(),
            "Detect C2 communication".to_string(),
            "Show regular DNS patterns".to_string(),
            "Find malware callbacks".to_string(),
            "Detect command and control".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("min_beacons").with_type(ParameterType::Integer).with_default("10"))
        .add_parameter(QueryParameter::new("jitter_threshold").with_type(ParameterType::Float).with_default("0.2").with_description("Max stddev as fraction of mean (0.2 = 20% jitter)"))
        .with_mitre_attack(MitreMapping::new("T1071.004", "Application Layer Protocol: DNS", "Command and Control"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::Critical)
    }
}

/// Threat intelligence and IOC matching templates.
pub mod threat_intel {
    use super::*;

    /// Match DNS queries against known malicious domains from threat intel.
    pub fn ioc_domain_match() -> QueryTemplate {
        QueryTemplate::new(
            "ioc_domain_match",
            "Match DNS queries against known malicious domains from threat intel",
            r#"SELECT 
  d.source_ip,
  d.query_hostname,
  d.time,
  d.action,
  t.threat_type,
  t.confidence,
  t.source as intel_source
FROM dns_event d
INNER JOIN threat_intel_ioc t 
  ON d.query_hostname = t.indicator
  OR d.query_hostname LIKE '%.' || t.indicator
WHERE d.time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
  AND t.indicator_type = 'domain'
ORDER BY d.time DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Find known bad domains".to_string(),
            "Match against threat intel".to_string(),
            "Check IOC matches".to_string(),
            "Find malicious domain lookups".to_string(),
            "Show threat intel hits".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("1000"))
        .with_category(QueryCategory::ThreatDetection)
        .with_severity(Severity::Critical)
    }
}

/// Investigation and operational templates.
pub mod operational {
    use super::*;

    /// Find hosts with high rates of blocked DNS queries.
    pub fn blocked_dns_by_host() -> QueryTemplate {
        QueryTemplate::new(
            "blocked_dns_by_host",
            "Find hosts with high rates of blocked DNS queries",
            r#"SELECT 
  source_ip,
  COUNT(*) as total_queries,
  SUM(CASE WHEN action = 'Blocked' THEN 1 ELSE 0 END) as blocked_count,
  ROUND(100.0 * SUM(CASE WHEN action = 'Blocked' THEN 1 ELSE 0 END) / COUNT(*), 2) as blocked_rate,
  ARRAY_AGG(DISTINCT query_hostname) FILTER (WHERE action = 'Blocked') as blocked_domains
FROM dns_event
WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
GROUP BY source_ip
HAVING blocked_count > {{min_blocked}}
ORDER BY blocked_count DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Show blocked DNS queries".to_string(),
            "Which hosts are being blocked".to_string(),
            "Find policy violations".to_string(),
            "Show DNS blocks by host".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("min_blocked").with_type(ParameterType::Integer).with_default("10"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"))
        .with_category(QueryCategory::Investigation)
        .with_severity(Severity::Medium)
    }

    /// Show the most frequently queried domains.
    pub fn top_queried_domains() -> QueryTemplate {
        QueryTemplate::new(
            "top_queried_domains",
            "Show the most frequently queried domains",
            r#"SELECT 
  query_hostname,
  COUNT(*) as query_count,
  COUNT(DISTINCT source_ip) as unique_clients,
  MIN(time) as first_seen,
  MAX(time) as last_seen
FROM dns_event
WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
GROUP BY query_hostname
ORDER BY query_count DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "Top DNS queries".to_string(),
            "Most queried domains".to_string(),
            "Popular domains".to_string(),
            "DNS query statistics".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"))
        .with_category(QueryCategory::Operational)
    }

    /// Show DNS query distribution by cloud region.
    pub fn dns_by_region() -> QueryTemplate {
        QueryTemplate::new(
            "dns_by_region",
            "Show DNS query distribution by cloud region",
            r#"SELECT 
  cloud_region,
  COUNT(*) as query_count,
  COUNT(DISTINCT source_ip) as unique_clients,
  COUNT(DISTINCT query_hostname) as unique_domains
FROM dns_event
WHERE time >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
GROUP BY cloud_region
ORDER BY query_count DESC"#
        )
        .with_patterns(vec![
            "DNS queries by region".to_string(),
            "Regional DNS activity".to_string(),
            "Which regions have DNS traffic".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .with_category(QueryCategory::Operational)
    }

    /// Find domains queried for the first time recently.
    pub fn new_domains_first_seen() -> QueryTemplate {
        QueryTemplate::new(
            "new_domains_first_seen",
            "Find domains queried for the first time recently",
            r#"WITH domain_history AS (
  SELECT 
    query_hostname,
    MIN(time) as first_seen,
    COUNT(*) as total_queries
  FROM dns_event
  GROUP BY query_hostname
)
SELECT 
  query_hostname,
  first_seen,
  total_queries
FROM domain_history
WHERE first_seen >= DATEADD(hour, -{{hours}}, CURRENT_TIMESTAMP())
ORDER BY first_seen DESC
LIMIT {{limit}}"#
        )
        .with_patterns(vec![
            "New domains".to_string(),
            "First time domains".to_string(),
            "Recently seen domains".to_string(),
            "Newly observed domains".to_string(),
        ])
        .add_parameter(QueryParameter::new("hours").with_type(ParameterType::Integer).with_default("24"))
        .add_parameter(QueryParameter::new("limit").with_type(ParameterType::Integer).with_default("100"))
        .with_category(QueryCategory::Investigation)
        .with_severity(Severity::Low)
    }
}

/// Returns all DNS security query templates.
pub fn all_templates() -> Vec<QueryTemplate> {
    vec![
        // Tunneling detection
        tunneling::high_volume(),
        tunneling::long_queries(),
        tunneling::txt_record_abuse(),
        // DGA detection
        dga::nxdomain_spike(),
        dga::random_domains(),
        // C2 and fast flux
        c2::fast_flux(),
        c2::beaconing(),
        // Threat intel
        threat_intel::ioc_domain_match(),
        // Operational
        operational::blocked_dns_by_host(),
        operational::top_queried_domains(),
        operational::dns_by_region(),
        operational::new_domains_first_seen(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_all_templates_have_required_fields() {
        for template in all_templates() {
            assert!(!template.name.is_empty(), "Template must have a name");
            assert!(!template.intent.is_empty(), "Template must have an intent");
            assert!(!template.sql_template.is_empty(), "Template must have SQL");
        }
    }

    #[test]
    fn test_threat_detection_templates_have_mitre() {
        for template in all_templates() {
            if template.category == QueryCategory::ThreatDetection {
                // Most threat detection templates should have MITRE mapping
                // (ioc_domain_match is an exception as it's generic)
                if template.name != "ioc_domain_match" {
                    assert!(
                        template.mitre_attack.is_some(),
                        "Threat detection template '{}' should have MITRE mapping",
                        template.name
                    );
                }
            }
        }
    }

    #[test]
    fn test_templates_substitute_with_defaults() {
        for template in all_templates() {
            let result = template.substitute(&HashMap::new());
            assert!(
                result.is_ok(),
                "Template '{}' should substitute with defaults: {:?}",
                template.name,
                result.err()
            );
        }
    }

    #[test]
    fn test_tunneling_high_volume_template() {
        let template = tunneling::high_volume();
        assert_eq!(template.name, "dns_tunneling_high_volume");
        assert_eq!(template.category, QueryCategory::ThreatDetection);
        assert_eq!(template.severity, Some(Severity::High));
        
        let sql = template.substitute(&HashMap::new()).unwrap();
        assert!(sql.contains("source_ip"));
        assert!(sql.contains("query_count"));
    }

    #[test]
    fn test_beaconing_template() {
        let template = c2::beaconing();
        assert_eq!(template.name, "c2_beaconing_regular_intervals");
        assert_eq!(template.severity, Some(Severity::Critical));
        
        let mitre = template.mitre_attack.as_ref().unwrap();
        assert_eq!(mitre.technique_id, "T1071.004");
    }
}
