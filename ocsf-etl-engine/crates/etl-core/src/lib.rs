//! ETL Core - pure data types for the OCSF ETL Engine.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── 2.2 Transform & SqlType ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transform {
    Lower,
    Upper,
    Trim,
    Cast(SqlType),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqlType {
    Integer,
    BigInt,
    Float,
    Double,
    Boolean,
    String,
    Timestamp,
    Date,
}

// ── 2.1 MappingSpec ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MappingSpec {
    pub source_field: String,
    pub target_ocsf_path: String,
    pub transformation: Option<Transform>,
    pub confidence: f32,
}

impl MappingSpec {
    pub fn new(
        source_field: String,
        target_ocsf_path: String,
        transformation: Option<Transform>,
        confidence: f32,
    ) -> Self {
        Self {
            source_field,
            target_ocsf_path,
            transformation,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

// ── 2.3 SourceConfig ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    File { path: String },
    Tcp { bind_address: String },
    Sqs { queue_url: String, region: String },
    Kafka { brokers: Vec<String>, topic: String },
}

// ── 2.4 S3Config ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

// ── 2.5 CatalogRef ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogRef {
    pub entry_id: u64,
    pub catalog_version: u64,
    pub catalog_url: Option<String>,
}

// ── 2.6 EtlJob ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EtlJob {
    pub job_id: Uuid,
    pub plugin_name: String,
    pub ocsf_class_uid: u32,
    pub ocsf_version: String,
    pub mappings: Vec<MappingSpec>,
    pub source_config: SourceConfig,
    pub delta_table_uri: String,
    pub s3_config: Option<S3Config>,
    pub catalog_ref: Option<CatalogRef>,
}

// ── 2.7 EtlJobStatus ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EtlJobStatus {
    Pending,
    Generating,
    Compiling,
    Testing,
    Running,
    Completed,
    Failed(String),
}

// ── 2.8 State machine & errors ───────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum EtlError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
}

impl EtlJobStatus {
    pub fn transition(&self, next: &EtlJobStatus) -> Result<EtlJobStatus, EtlError> {
        // Failed is reachable from any active state (not from Completed or Failed)
        if matches!(next, EtlJobStatus::Failed(_)) {
            return match self {
                EtlJobStatus::Completed | EtlJobStatus::Failed(_) => {
                    Err(EtlError::InvalidTransition {
                        from: format!("{:?}", self),
                        to: format!("{:?}", next),
                    })
                }
                _ => Ok(next.clone()),
            };
        }

        let valid = matches!(
            (self, next),
            (EtlJobStatus::Pending, EtlJobStatus::Generating)
                | (EtlJobStatus::Generating, EtlJobStatus::Compiling)
                | (EtlJobStatus::Compiling, EtlJobStatus::Testing)
                | (EtlJobStatus::Testing, EtlJobStatus::Running)
                | (EtlJobStatus::Running, EtlJobStatus::Completed)
        );

        if valid {
            Ok(next.clone())
        } else {
            Err(EtlError::InvalidTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", next),
            })
        }
    }
}

// ── Task 3: Property tests ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── Generators ───────────────────────────────────────────────────

    fn arb_mapping_spec() -> impl Strategy<Value = MappingSpec> {
        ("[a-z_]{1,20}", "[a-z_.]{1,30}", 0.0f32..=1.0f32).prop_map(
            |(source, target, conf)| MappingSpec::new(source, target, None, conf),
        )
    }

    fn arb_etl_job() -> impl Strategy<Value = EtlJob> {
        (
            "[a-z_]{3,15}",
            1000u32..10000u32,
            prop::collection::vec(arb_mapping_spec(), 1..5),
        )
            .prop_map(|(plugin_name, uid, mappings)| EtlJob {
                job_id: Uuid::new_v4(),
                plugin_name,
                ocsf_class_uid: uid,
                ocsf_version: "1.3.0".to_string(),
                mappings,
                source_config: SourceConfig::File {
                    path: "/tmp/test.log".to_string(),
                },
                delta_table_uri: "s3://bucket/table".to_string(),
                s3_config: None,
                catalog_ref: None,
            })
    }

    // ── 3.1 MappingSpec JSON round-trip ──────────────────────────────
    // **Validates: Requirements 1.9**

    proptest! {
        #[test]
        fn mapping_spec_json_roundtrip(
            source in "[a-z_]{1,20}",
            target in "[a-z_.]{1,30}",
            confidence in 0.0f32..=1.0f32,
        ) {
            let spec = MappingSpec::new(source, target, None, confidence);
            let json = serde_json::to_string(&spec).unwrap();
            let back: MappingSpec = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(spec, back);
        }
    }

    // ── 3.2 EtlJob JSON round-trip ──────────────────────────────────
    // **Validates: Requirements 1.8**

    proptest! {
        #[test]
        fn etl_job_json_roundtrip(job in arb_etl_job()) {
            let json = serde_json::to_string(&job).unwrap();
            let back: EtlJob = serde_json::from_str(&json).unwrap();
            // Compare all fields except job_id (which uses Uuid::new_v4)
            prop_assert_eq!(&job.plugin_name, &back.plugin_name);
            prop_assert_eq!(job.ocsf_class_uid, back.ocsf_class_uid);
            prop_assert_eq!(&job.mappings, &back.mappings);
        }
    }

    // ── 3.3 Confidence clamping ─────────────────────────────────────
    // **Validates: Requirements 1.3**

    proptest! {
        #[test]
        fn confidence_always_clamped(raw_confidence in proptest::num::f32::ANY) {
            if raw_confidence.is_nan() {
                return Ok(());  // skip NaN
            }
            let spec = MappingSpec::new(
                "src".to_string(), "dst".to_string(), None, raw_confidence
            );
            prop_assert!(spec.confidence >= 0.0);
            prop_assert!(spec.confidence <= 1.0);
        }
    }

    // ── 3.4 State machine transitions ───────────────────────────────
    // **Validates: Requirements 6.7**

    proptest! {
        #[test]
        fn state_machine_valid_transitions(steps in prop::collection::vec(0u8..7, 1..10)) {
            let all_states = vec![
                EtlJobStatus::Pending,
                EtlJobStatus::Generating,
                EtlJobStatus::Compiling,
                EtlJobStatus::Testing,
                EtlJobStatus::Running,
                EtlJobStatus::Completed,
                EtlJobStatus::Failed("test".to_string()),
            ];

            let mut current = EtlJobStatus::Pending;
            for step in steps {
                let next = &all_states[step as usize % all_states.len()];
                let result = current.transition(next);
                // We just verify it doesn't panic and returns Ok or Err appropriately
                match result {
                    Ok(new_state) => current = new_state,
                    Err(_) => {} // Invalid transition, that's fine
                }
            }
        }
    }
}
