//! Field lineage tracking for the OCSF Semantic Index.
//!
//! This module provides types for tracking how raw source fields map to OCSF fields,
//! including transformations. This enables understanding data transformations and
//! debugging mapping issues.

use serde::{Deserialize, Serialize};

use crate::backend::IndexRecord;
use crate::types::{LineageId, RecordId};

// ============================================================================
// FieldLineageRecord
// ============================================================================

/// Field lineage record tracking how source fields map to OCSF fields.
///
/// A `FieldLineageRecord` represents a single field-level mapping from a source
/// field to a target OCSF field, optionally including the transformation expression
/// used. This enables tracking of field-level data transformations and supports
/// both one-to-many and many-to-one mappings.
///
/// # Example
///
/// ```rust
/// use ocsf_index::field_lineage::FieldLineageRecord;
/// use ocsf_index::types::LineageId;
///
/// // Simple direct mapping
/// let lineage = FieldLineageRecord::new(
///     LineageId::new(1),
///     "src_ip",
///     "src_endpoint.ip"
/// );
///
/// assert_eq!(lineage.source_field, "src_ip");
/// assert_eq!(lineage.target_field, "src_endpoint.ip");
/// assert!(lineage.transformation.is_none());
///
/// // Mapping with transformation
/// let lineage_with_transform = FieldLineageRecord::new(
///     LineageId::new(1),
///     "timestamp_str",
///     "time"
/// )
/// .with_transformation("CAST(timestamp_str AS TIMESTAMP)")
/// .with_ocsf_version("1.3.0");
///
/// assert_eq!(lineage_with_transform.transformation, Some("CAST(timestamp_str AS TIMESTAMP)".to_string()));
/// assert_eq!(lineage_with_transform.ocsf_version, Some("1.3.0".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldLineageRecord {
    /// Unique identifier for this field lineage record (assigned by backend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<LineageId>,

    /// The ID of the source lineage record this field mapping belongs to.
    pub source_lineage_id: LineageId,

    /// The source field path (e.g., "src_ip", "event.user.name").
    pub source_field: String,

    /// The target OCSF field path (e.g., "src_endpoint.ip", "actor.user.name").
    pub target_field: String,

    /// Optional SQL transformation expression used to convert the source to target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,

    /// Optional OCSF schema version for version-specific field paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocsf_version: Option<String>,
}

impl FieldLineageRecord {
    /// Creates a new FieldLineageRecord with the given source lineage ID and field paths.
    ///
    /// The record is created with:
    /// - `transformation`: None
    /// - `ocsf_version`: None
    ///
    /// # Arguments
    ///
    /// * `source_lineage_id` - The ID of the parent source lineage record
    /// * `source_field` - The source field path
    /// * `target_field` - The target OCSF field path
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::field_lineage::FieldLineageRecord;
    /// use ocsf_index::types::LineageId;
    ///
    /// let lineage = FieldLineageRecord::new(
    ///     LineageId::new(1),
    ///     "src_ip",
    ///     "src_endpoint.ip"
    /// );
    ///
    /// assert_eq!(lineage.source_lineage_id, LineageId::new(1));
    /// assert_eq!(lineage.source_field, "src_ip");
    /// assert_eq!(lineage.target_field, "src_endpoint.ip");
    /// assert!(lineage.transformation.is_none());
    /// assert!(lineage.ocsf_version.is_none());
    /// ```
    pub fn new(
        source_lineage_id: LineageId,
        source_field: impl Into<String>,
        target_field: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            source_lineage_id,
            source_field: source_field.into(),
            target_field: target_field.into(),
            transformation: None,
            ocsf_version: None,
        }
    }

    /// Sets the transformation expression for this field mapping.
    ///
    /// # Arguments
    ///
    /// * `expr` - The SQL transformation expression
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::field_lineage::FieldLineageRecord;
    /// use ocsf_index::types::LineageId;
    ///
    /// let lineage = FieldLineageRecord::new(
    ///     LineageId::new(1),
    ///     "timestamp_str",
    ///     "time"
    /// )
    /// .with_transformation("CAST(timestamp_str AS TIMESTAMP)");
    ///
    /// assert_eq!(lineage.transformation, Some("CAST(timestamp_str AS TIMESTAMP)".to_string()));
    /// ```
    pub fn with_transformation(mut self, expr: impl Into<String>) -> Self {
        self.transformation = Some(expr.into());
        self
    }

    /// Sets the OCSF version for this field mapping.
    ///
    /// This is useful for tracking version-specific field paths when field names
    /// change between OCSF versions.
    ///
    /// # Arguments
    ///
    /// * `version` - The OCSF schema version (e.g., "1.3.0")
    ///
    /// # Example
    ///
    /// ```rust
    /// use ocsf_index::field_lineage::FieldLineageRecord;
    /// use ocsf_index::types::LineageId;
    ///
    /// let lineage = FieldLineageRecord::new(
    ///     LineageId::new(1),
    ///     "src_ip",
    ///     "src_endpoint.ip"
    /// )
    /// .with_ocsf_version("1.3.0");
    ///
    /// assert_eq!(lineage.ocsf_version, Some("1.3.0".to_string()));
    /// ```
    pub fn with_ocsf_version(mut self, version: impl Into<String>) -> Self {
        self.ocsf_version = Some(version.into());
        self
    }

    /// Sets the field lineage ID (typically called by the backend after persistence).
    ///
    /// # Arguments
    ///
    /// * `id` - The lineage ID
    pub fn with_id(mut self, id: LineageId) -> Self {
        self.id = Some(id);
        self
    }
}

impl IndexRecord for FieldLineageRecord {
    fn record_type() -> &'static str {
        "field_lineage"
    }

    fn id(&self) -> Option<RecordId> {
        self.id.map(|id| RecordId::new(id.0))
    }

    fn set_id(&mut self, id: RecordId) {
        self.id = Some(LineageId::new(id.0));
    }
}

// ============================================================================
// FieldMapping (for visualization)
// ============================================================================

/// A field mapping for graph visualization.
///
/// This struct represents a source-to-target field relationship in a format
/// suitable for rendering field lineage graphs in the UI.
///
/// # Example
///
/// ```rust
/// use ocsf_index::field_lineage::{FieldLineageRecord, FieldMapping};
/// use ocsf_index::types::LineageId;
///
/// let record = FieldLineageRecord::new(
///     LineageId::new(1),
///     "src_ip",
///     "src_endpoint.ip"
/// )
/// .with_transformation("LOWER(src_ip)");
///
/// let mapping: FieldMapping = record.into();
///
/// assert_eq!(mapping.source_path, "src_ip");
/// assert_eq!(mapping.target_path, "src_endpoint.ip");
/// assert_eq!(mapping.transformation, Some("LOWER(src_ip)".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldMapping {
    /// The source field path.
    pub source_path: String,

    /// The target OCSF field path.
    pub target_path: String,

    /// Optional transformation expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,
}

impl From<FieldLineageRecord> for FieldMapping {
    fn from(record: FieldLineageRecord) -> Self {
        Self {
            source_path: record.source_field,
            target_path: record.target_field,
            transformation: record.transformation,
        }
    }
}

impl From<&FieldLineageRecord> for FieldMapping {
    fn from(record: &FieldLineageRecord) -> Self {
        Self {
            source_path: record.source_field.clone(),
            target_path: record.target_field.clone(),
            transformation: record.transformation.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // FieldLineageRecord Tests
    // ========================================================================

    #[test]
    fn test_field_lineage_record_new() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");

        assert!(lineage.id.is_none());
        assert_eq!(lineage.source_lineage_id, LineageId::new(1));
        assert_eq!(lineage.source_field, "src_ip");
        assert_eq!(lineage.target_field, "src_endpoint.ip");
        assert!(lineage.transformation.is_none());
        assert!(lineage.ocsf_version.is_none());
    }

    #[test]
    fn test_field_lineage_record_with_transformation() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "timestamp_str", "time")
            .with_transformation("CAST(timestamp_str AS TIMESTAMP)");

        assert_eq!(
            lineage.transformation,
            Some("CAST(timestamp_str AS TIMESTAMP)".to_string())
        );
    }

    #[test]
    fn test_field_lineage_record_with_ocsf_version() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_ocsf_version("1.3.0");

        assert_eq!(lineage.ocsf_version, Some("1.3.0".to_string()));
    }

    #[test]
    fn test_field_lineage_record_with_id() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_id(LineageId::new(42));

        assert_eq!(lineage.id, Some(LineageId::new(42)));
    }

    #[test]
    fn test_field_lineage_record_builder_chaining() {
        let lineage =
            FieldLineageRecord::new(LineageId::new(5), "event.user.name", "actor.user.name")
                .with_transformation("UPPER(event.user.name)")
                .with_ocsf_version("1.4.0")
                .with_id(LineageId::new(100));

        assert_eq!(lineage.id, Some(LineageId::new(100)));
        assert_eq!(lineage.source_lineage_id, LineageId::new(5));
        assert_eq!(lineage.source_field, "event.user.name");
        assert_eq!(lineage.target_field, "actor.user.name");
        assert_eq!(
            lineage.transformation,
            Some("UPPER(event.user.name)".to_string())
        );
        assert_eq!(lineage.ocsf_version, Some("1.4.0".to_string()));
    }

    #[test]
    fn test_field_lineage_record_json_serialization() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)")
            .with_ocsf_version("1.3.0");

        let json = serde_json::to_string(&lineage).unwrap();
        let deserialized: FieldLineageRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(lineage.source_lineage_id, deserialized.source_lineage_id);
        assert_eq!(lineage.source_field, deserialized.source_field);
        assert_eq!(lineage.target_field, deserialized.target_field);
        assert_eq!(lineage.transformation, deserialized.transformation);
        assert_eq!(lineage.ocsf_version, deserialized.ocsf_version);
    }

    #[test]
    fn test_field_lineage_record_json_none_fields_skipped() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");

        let json = serde_json::to_string(&lineage).unwrap();

        // None fields should be skipped in serialization
        assert!(!json.contains("\"id\""));
        assert!(!json.contains("transformation"));
        assert!(!json.contains("ocsf_version"));
    }

    #[test]
    fn test_field_lineage_record_json_deserialization_with_defaults() {
        // JSON with only required fields should deserialize with defaults
        let json = r#"{
            "source_lineage_id": 1,
            "source_field": "src_ip",
            "target_field": "src_endpoint.ip"
        }"#;
        let lineage: FieldLineageRecord = serde_json::from_str(json).unwrap();

        assert_eq!(lineage.source_lineage_id, LineageId::new(1));
        assert_eq!(lineage.source_field, "src_ip");
        assert_eq!(lineage.target_field, "src_endpoint.ip");
        assert!(lineage.id.is_none());
        assert!(lineage.transformation.is_none());
        assert!(lineage.ocsf_version.is_none());
    }

    #[test]
    fn test_field_lineage_record_equality() {
        let lineage1 = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");

        let lineage2 = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");

        let lineage3 = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("UPPER(src_ip)");

        assert_eq!(lineage1, lineage2);
        assert_ne!(lineage1, lineage3);
    }

    #[test]
    fn test_field_lineage_record_clone() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)")
            .with_ocsf_version("1.3.0");

        let cloned = lineage.clone();
        assert_eq!(lineage, cloned);
    }

    // ========================================================================
    // IndexRecord Trait Tests
    // ========================================================================

    #[test]
    fn test_field_lineage_record_record_type() {
        assert_eq!(FieldLineageRecord::record_type(), "field_lineage");
    }

    #[test]
    fn test_field_lineage_record_id_none() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");
        assert!(lineage.id().is_none());
    }

    #[test]
    fn test_field_lineage_record_id_some() {
        let lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_id(LineageId::new(42));
        assert_eq!(lineage.id(), Some(RecordId::new(42)));
    }

    #[test]
    fn test_field_lineage_record_set_id() {
        let mut lineage = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");
        assert!(lineage.id.is_none());

        lineage.set_id(RecordId::new(99));
        assert_eq!(lineage.id, Some(LineageId::new(99)));
    }

    // ========================================================================
    // FieldMapping Tests
    // ========================================================================

    #[test]
    fn test_field_mapping_from_field_lineage_record() {
        let record = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");

        let mapping: FieldMapping = record.clone().into();

        assert_eq!(mapping.source_path, "src_ip");
        assert_eq!(mapping.target_path, "src_endpoint.ip");
        assert_eq!(mapping.transformation, Some("LOWER(src_ip)".to_string()));
    }

    #[test]
    fn test_field_mapping_from_field_lineage_record_ref() {
        let record =
            FieldLineageRecord::new(LineageId::new(1), "event.user.name", "actor.user.name")
                .with_transformation("UPPER(event.user.name)");

        let mapping: FieldMapping = (&record).into();

        assert_eq!(mapping.source_path, "event.user.name");
        assert_eq!(mapping.target_path, "actor.user.name");
        assert_eq!(
            mapping.transformation,
            Some("UPPER(event.user.name)".to_string())
        );
    }

    #[test]
    fn test_field_mapping_from_field_lineage_record_no_transform() {
        let record = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");

        let mapping: FieldMapping = record.into();

        assert_eq!(mapping.source_path, "src_ip");
        assert_eq!(mapping.target_path, "src_endpoint.ip");
        assert!(mapping.transformation.is_none());
    }

    #[test]
    fn test_field_mapping_json_serialization() {
        let record = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip")
            .with_transformation("LOWER(src_ip)");

        let mapping: FieldMapping = record.into();

        let json = serde_json::to_string(&mapping).unwrap();
        let deserialized: FieldMapping = serde_json::from_str(&json).unwrap();

        assert_eq!(mapping, deserialized);
    }

    #[test]
    fn test_field_mapping_json_none_transformation_skipped() {
        let record = FieldLineageRecord::new(LineageId::new(1), "src_ip", "src_endpoint.ip");

        let mapping: FieldMapping = record.into();

        let json = serde_json::to_string(&mapping).unwrap();

        // None transformation should be skipped in serialization
        assert!(!json.contains("transformation"));
    }

    #[test]
    fn test_field_mapping_equality() {
        let mapping1 = FieldMapping {
            source_path: "src_ip".to_string(),
            target_path: "src_endpoint.ip".to_string(),
            transformation: Some("LOWER(src_ip)".to_string()),
        };

        let mapping2 = FieldMapping {
            source_path: "src_ip".to_string(),
            target_path: "src_endpoint.ip".to_string(),
            transformation: Some("LOWER(src_ip)".to_string()),
        };

        let mapping3 = FieldMapping {
            source_path: "src_ip".to_string(),
            target_path: "src_endpoint.ip".to_string(),
            transformation: Some("UPPER(src_ip)".to_string()),
        };

        assert_eq!(mapping1, mapping2);
        assert_ne!(mapping1, mapping3);
    }

    #[test]
    fn test_field_mapping_clone() {
        let mapping = FieldMapping {
            source_path: "src_ip".to_string(),
            target_path: "src_endpoint.ip".to_string(),
            transformation: Some("LOWER(src_ip)".to_string()),
        };

        let cloned = mapping.clone();
        assert_eq!(mapping, cloned);
    }

    // ========================================================================
    // One-to-Many and Many-to-One Mapping Tests
    // ========================================================================

    #[test]
    fn test_one_to_many_mapping() {
        // One source field maps to multiple OCSF fields
        let source_lineage_id = LineageId::new(1);

        let mapping1 = FieldLineageRecord::new(source_lineage_id, "ip_address", "src_endpoint.ip");

        let mapping2 = FieldLineageRecord::new(source_lineage_id, "ip_address", "dst_endpoint.ip");

        // Both mappings share the same source field
        assert_eq!(mapping1.source_field, mapping2.source_field);
        // But have different target fields
        assert_ne!(mapping1.target_field, mapping2.target_field);
    }

    #[test]
    fn test_many_to_one_mapping() {
        // Multiple source fields combine into one OCSF field
        let source_lineage_id = LineageId::new(1);

        let mapping1 =
            FieldLineageRecord::new(source_lineage_id, "first_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");

        let mapping2 =
            FieldLineageRecord::new(source_lineage_id, "last_name", "actor.user.full_name")
                .with_transformation("CONCAT(first_name, ' ', last_name)");

        // Both mappings have different source fields
        assert_ne!(mapping1.source_field, mapping2.source_field);
        // But share the same target field
        assert_eq!(mapping1.target_field, mapping2.target_field);
    }
}
