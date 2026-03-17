//! Semantic model store.
//!
//! This module contains the semantic model store for managing
//! entity and metric definitions with YAML persistence.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::entity::SemanticEntity;
use crate::metric::SemanticMetric;

/// A physical data source abstraction decoupled from semantic definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dataset {
    pub name: String,
    pub dialect: String,
    pub table: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
}

/// Configuration for observable extraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableConfig {
    /// Whether to extract observables to a dedicated table.
    #[serde(default)]
    pub extract_to_table: bool,

    /// Name of the observables table.
    #[serde(default = "default_table_name")]
    pub table_name: String,

    /// Observable type_ids to include in extraction.
    #[serde(default)]
    pub include_types: Vec<u32>,
}

fn default_table_name() -> String {
    "ocsf_observables".to_string()
}

impl Default for ObservableConfig {
    fn default() -> Self {
        Self {
            extract_to_table: false,
            table_name: default_table_name(),
            include_types: Vec::new(),
        }
    }
}

/// A semantic model definition.
///
/// This is the top-level structure for a semantic model that can be
/// serialized to/from YAML format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticModel {
    /// Model version.
    #[serde(default = "default_version")]
    pub version: String,

    /// OCSF schema version this model is based on.
    #[serde(default)]
    pub ocsf_version: String,

    /// Model name.
    pub name: String,

    /// Model description.
    #[serde(default)]
    pub description: String,

    /// Semantic entities.
    #[serde(default)]
    pub entities: Vec<SemanticEntity>,

    /// Semantic metrics.
    #[serde(default)]
    pub metrics: Vec<SemanticMetric>,

    /// Observable extraction configuration.
    #[serde(default)]
    pub observable_config: ObservableConfig,

    /// Physical data source definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub datasets: Vec<Dataset>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Default for SemanticModel {
    fn default() -> Self {
        Self {
            version: default_version(),
            ocsf_version: String::new(),
            name: String::new(),
            description: String::new(),
            entities: Vec::new(),
            metrics: Vec::new(),
            observable_config: ObservableConfig::default(),
            datasets: Vec::new(),
        }
    }
}

impl SemanticModel {
    /// Creates a new semantic model with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Sets the model version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the OCSF version.
    pub fn with_ocsf_version(mut self, version: impl Into<String>) -> Self {
        self.ocsf_version = version.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds an entity.
    pub fn add_entity(mut self, entity: SemanticEntity) -> Self {
        self.entities.push(entity);
        self
    }

    /// Sets the entities.
    pub fn with_entities(mut self, entities: Vec<SemanticEntity>) -> Self {
        self.entities = entities;
        self
    }

    /// Adds a metric.
    pub fn add_metric(mut self, metric: SemanticMetric) -> Self {
        self.metrics.push(metric);
        self
    }

    /// Sets the metrics.
    pub fn with_metrics(mut self, metrics: Vec<SemanticMetric>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Sets the observable configuration.
    pub fn with_observable_config(mut self, config: ObservableConfig) -> Self {
        self.observable_config = config;
        self
    }

    /// Adds a dataset.
    pub fn add_dataset(mut self, dataset: Dataset) -> Self {
        self.datasets.push(dataset);
        self
    }

    /// Sets the datasets.
    pub fn with_datasets(mut self, datasets: Vec<Dataset>) -> Self {
        self.datasets = datasets;
        self
    }

    /// Gets an entity by name.
    pub fn get_entity(&self, name: &str) -> Option<&SemanticEntity> {
        self.entities.iter().find(|e| e.name == name)
    }

    /// Gets a mutable entity by name.
    pub fn get_entity_mut(&mut self, name: &str) -> Option<&mut SemanticEntity> {
        self.entities.iter_mut().find(|e| e.name == name)
    }

    /// Gets a metric by name.
    pub fn get_metric(&self, name: &str) -> Option<&SemanticMetric> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Gets a mutable metric by name.
    pub fn get_metric_mut(&mut self, name: &str) -> Option<&mut SemanticMetric> {
        self.metrics.iter_mut().find(|m| m.name == name)
    }

    /// Returns all entity names.
    pub fn entity_names(&self) -> impl Iterator<Item = &str> {
        self.entities.iter().map(|e| e.name.as_str())
    }

    /// Returns all metric names.
    pub fn metric_names(&self) -> impl Iterator<Item = &str> {
        self.metrics.iter().map(|m| m.name.as_str())
    }

    /// Serializes the model to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize semantic model to YAML")
    }

    /// Deserializes a model from YAML.
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse semantic model from YAML")
    }

    /// Saves the model to a YAML file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = self.to_yaml()?;
        std::fs::write(path.as_ref(), yaml)
            .with_context(|| format!("Failed to write semantic model to {:?}", path.as_ref()))?;
        Ok(())
    }

    /// Loads a model from a YAML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let yaml = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read semantic model from {:?}", path.as_ref()))?;
        Self::from_yaml(&yaml)
    }
}

/// A semantic model store for managing multiple models.
///
/// Provides entity and metric management with persistence support.
#[derive(Debug, Clone, Default)]
pub struct SemanticModelStore {
    /// The current semantic model.
    model: SemanticModel,

    /// Entity index for fast lookup.
    entity_index: HashMap<String, usize>,

    /// Metric index for fast lookup.
    metric_index: HashMap<String, usize>,

    /// Change history.
    history: Vec<ChangeRecord>,
}

/// A record of a change to the semantic model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeRecord {
    /// Timestamp of the change.
    pub timestamp: String,

    /// Type of change.
    pub change_type: ChangeType,

    /// Description of the change.
    pub description: String,
}

/// Type of change to the semantic model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Entity was added.
    EntityAdded,
    /// Entity was modified.
    EntityModified,
    /// Entity was removed.
    EntityRemoved,
    /// Metric was added.
    MetricAdded,
    /// Metric was modified.
    MetricModified,
    /// Metric was removed.
    MetricRemoved,
    /// Model was loaded.
    ModelLoaded,
    /// Model was saved.
    ModelSaved,
}

impl SemanticModelStore {
    /// Creates a new empty model store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a model store from an existing model.
    pub fn from_model(model: SemanticModel) -> Self {
        let mut store = Self {
            model,
            entity_index: HashMap::new(),
            metric_index: HashMap::new(),
            history: Vec::new(),
        };
        store.rebuild_indexes();
        store
    }

    /// Rebuilds the internal indexes.
    fn rebuild_indexes(&mut self) {
        self.entity_index.clear();
        for (i, entity) in self.model.entities.iter().enumerate() {
            self.entity_index.insert(entity.name.clone(), i);
        }

        self.metric_index.clear();
        for (i, metric) in self.model.metrics.iter().enumerate() {
            self.metric_index.insert(metric.name.clone(), i);
        }
    }

    /// Records a change to the history.
    fn record_change(&mut self, change_type: ChangeType, description: impl Into<String>) {
        self.history.push(ChangeRecord {
            timestamp: chrono_lite_timestamp(),
            change_type,
            description: description.into(),
        });
    }

    /// Returns the underlying model.
    pub fn model(&self) -> &SemanticModel {
        &self.model
    }

    /// Returns a mutable reference to the underlying model.
    pub fn model_mut(&mut self) -> &mut SemanticModel {
        &mut self.model
    }

    /// Defines (adds or updates) an entity.
    pub fn define_entity(&mut self, entity: SemanticEntity) {
        let name = entity.name.clone();
        if let Some(&idx) = self.entity_index.get(&name) {
            self.model.entities[idx] = entity;
            self.record_change(ChangeType::EntityModified, format!("Modified entity: {}", name));
        } else {
            let idx = self.model.entities.len();
            self.model.entities.push(entity);
            self.entity_index.insert(name.clone(), idx);
            self.record_change(ChangeType::EntityAdded, format!("Added entity: {}", name));
        }
    }

    /// Gets an entity by name.
    pub fn get_entity(&self, name: &str) -> Option<&SemanticEntity> {
        self.entity_index
            .get(name)
            .map(|&idx| &self.model.entities[idx])
    }

    /// Removes an entity by name.
    pub fn remove_entity(&mut self, name: &str) -> Option<SemanticEntity> {
        if let Some(&idx) = self.entity_index.get(name) {
            let entity = self.model.entities.remove(idx);
            self.rebuild_indexes();
            self.record_change(ChangeType::EntityRemoved, format!("Removed entity: {}", name));
            Some(entity)
        } else {
            None
        }
    }

    /// Lists all entities.
    pub fn list_entities(&self) -> &[SemanticEntity] {
        &self.model.entities
    }

    /// Defines (adds or updates) a metric.
    pub fn define_metric(&mut self, metric: SemanticMetric) {
        let name = metric.name.clone();
        if let Some(&idx) = self.metric_index.get(&name) {
            self.model.metrics[idx] = metric;
            self.record_change(ChangeType::MetricModified, format!("Modified metric: {}", name));
        } else {
            let idx = self.model.metrics.len();
            self.model.metrics.push(metric);
            self.metric_index.insert(name.clone(), idx);
            self.record_change(ChangeType::MetricAdded, format!("Added metric: {}", name));
        }
    }

    /// Gets a metric by name.
    pub fn get_metric(&self, name: &str) -> Option<&SemanticMetric> {
        self.metric_index
            .get(name)
            .map(|&idx| &self.model.metrics[idx])
    }

    /// Removes a metric by name.
    pub fn remove_metric(&mut self, name: &str) -> Option<SemanticMetric> {
        if let Some(&idx) = self.metric_index.get(name) {
            let metric = self.model.metrics.remove(idx);
            self.rebuild_indexes();
            self.record_change(ChangeType::MetricRemoved, format!("Removed metric: {}", name));
            Some(metric)
        } else {
            None
        }
    }

    /// Lists all metrics.
    pub fn list_metrics(&self) -> &[SemanticMetric] {
        &self.model.metrics
    }

    /// Saves the model to a YAML file.
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.model.save(path.as_ref())?;
        self.record_change(
            ChangeType::ModelSaved,
            format!("Saved model to {:?}", path.as_ref()),
        );
        Ok(())
    }

    /// Loads a model from a YAML file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let model = SemanticModel::load(path.as_ref())?;
        let mut store = Self::from_model(model);
        store.record_change(
            ChangeType::ModelLoaded,
            format!("Loaded model from {:?}", path.as_ref()),
        );
        Ok(store)
    }

    /// Exports the model to YAML string.
    pub fn export_yaml(&self) -> Result<String> {
        self.model.to_yaml()
    }

    /// Returns the model version.
    pub fn get_version(&self) -> &str {
        &self.model.version
    }

    /// Returns the change history.
    pub fn get_history(&self) -> &[ChangeRecord] {
        &self.history
    }
}

/// Simple timestamp function (avoids chrono dependency).
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Cardinality, EntityRelationship, OCSFMapping, SemanticAttribute, SemanticType};
    use crate::metric::{Aggregation, TimeGranularity};

    fn create_test_entity() -> SemanticEntity {
        SemanticEntity::new("authentication_event")
            .with_caption("Authentication Event")
            .with_description("User authentication attempts across all systems")
            .with_source_event_classes(vec![3002, 3003])
            .with_covers_observables(vec![5, 10])
            .add_attribute(
                SemanticAttribute::new("user_email")
                    .with_caption("User Email")
                    .with_type(SemanticType::String)
                    .with_field_mapping("actor.user.email_addr")
                    .as_dimension()
                    .with_sample_values(vec!["user@example.com".to_string()]),
            )
            .add_attribute(
                SemanticAttribute::new("auth_result")
                    .with_caption("Authentication Result")
                    .with_type(SemanticType::String)
                    .with_expression_mapping("CASE WHEN status_id = 1 THEN 'success' ELSE 'failure' END")
                    .as_dimension(),
            )
            .add_relationship(
                EntityRelationship::new(
                    "performed_by",
                    "user",
                    "authentication_event.user_email = user.email",
                )
                .with_cardinality(Cardinality::ManyToMany),
            )
    }

    fn create_test_metric() -> SemanticMetric {
        SemanticMetric::new("auth_attempts")
            .with_caption("Authentication Attempts")
            .with_description("Count of authentication attempts")
            .with_aggregation(Aggregation::Count)
            .with_field_measure("metadata.uid")
            .with_dimensions(vec![
                "user_email".to_string(),
                "auth_result".to_string(),
                "source_ip".to_string(),
            ])
            .with_time_granularities(vec![
                TimeGranularity::Minute,
                TimeGranularity::Hour,
                TimeGranularity::Day,
            ])
    }

    #[test]
    fn test_semantic_model_builder() {
        let model = SemanticModel::new("security-analytics")
            .with_version("1.0")
            .with_ocsf_version("1.4.0")
            .with_description("Semantic layer for security analytics")
            .add_entity(create_test_entity())
            .add_metric(create_test_metric());

        assert_eq!(model.name, "security-analytics");
        assert_eq!(model.version, "1.0");
        assert_eq!(model.ocsf_version, "1.4.0");
        assert_eq!(model.entities.len(), 1);
        assert_eq!(model.metrics.len(), 1);
    }

    #[test]
    fn test_semantic_model_yaml_roundtrip() {
        let model = SemanticModel::new("security-analytics")
            .with_version("1.0")
            .with_ocsf_version("1.4.0")
            .with_description("Semantic layer for security analytics")
            .add_entity(create_test_entity())
            .add_metric(create_test_metric())
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5, 10, 22, 30],
            });

        let yaml = model.to_yaml().unwrap();
        let deserialized = SemanticModel::from_yaml(&yaml).unwrap();

        assert_eq!(model.name, deserialized.name);
        assert_eq!(model.version, deserialized.version);
        assert_eq!(model.ocsf_version, deserialized.ocsf_version);
        assert_eq!(model.entities.len(), deserialized.entities.len());
        assert_eq!(model.metrics.len(), deserialized.metrics.len());
        assert_eq!(model.observable_config, deserialized.observable_config);
    }

    #[test]
    fn test_semantic_model_store_entity_operations() {
        let mut store = SemanticModelStore::new();

        // Add entity
        store.define_entity(create_test_entity());
        assert!(store.get_entity("authentication_event").is_some());
        assert_eq!(store.list_entities().len(), 1);

        // Update entity
        let updated = SemanticEntity::new("authentication_event")
            .with_caption("Updated Caption");
        store.define_entity(updated);
        assert_eq!(
            store.get_entity("authentication_event").unwrap().caption,
            "Updated Caption"
        );
        assert_eq!(store.list_entities().len(), 1);

        // Remove entity
        let removed = store.remove_entity("authentication_event");
        assert!(removed.is_some());
        assert!(store.get_entity("authentication_event").is_none());
        assert_eq!(store.list_entities().len(), 0);
    }

    #[test]
    fn test_semantic_model_store_metric_operations() {
        let mut store = SemanticModelStore::new();

        // Add metric
        store.define_metric(create_test_metric());
        assert!(store.get_metric("auth_attempts").is_some());
        assert_eq!(store.list_metrics().len(), 1);

        // Update metric
        let updated = SemanticMetric::new("auth_attempts")
            .with_caption("Updated Caption");
        store.define_metric(updated);
        assert_eq!(
            store.get_metric("auth_attempts").unwrap().caption,
            "Updated Caption"
        );
        assert_eq!(store.list_metrics().len(), 1);

        // Remove metric
        let removed = store.remove_metric("auth_attempts");
        assert!(removed.is_some());
        assert!(store.get_metric("auth_attempts").is_none());
        assert_eq!(store.list_metrics().len(), 0);
    }

    #[test]
    fn test_semantic_model_store_history() {
        let mut store = SemanticModelStore::new();

        store.define_entity(create_test_entity());
        store.define_metric(create_test_metric());

        let history = store.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].change_type, ChangeType::EntityAdded);
        assert_eq!(history[1].change_type, ChangeType::MetricAdded);
    }

    #[test]
    fn test_semantic_model_store_from_model() {
        let model = SemanticModel::new("test")
            .add_entity(create_test_entity())
            .add_metric(create_test_metric());

        let store = SemanticModelStore::from_model(model);

        assert!(store.get_entity("authentication_event").is_some());
        assert!(store.get_metric("auth_attempts").is_some());
    }

    #[test]
    fn test_yaml_format_matches_design() {
        // Test that the YAML format matches the design document example
        let model = SemanticModel::new("security-analytics")
            .with_version("1.0")
            .with_ocsf_version("1.4.0")
            .with_description("Semantic layer for security analytics on OCSF data")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_caption("Authentication Event")
                    .with_description("User authentication attempts across all systems")
                    .with_source_event_classes(vec![3002, 3003])
                    .with_covers_observables(vec![5, 10])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_caption("User Email")
                            .with_type(SemanticType::String)
                            .with_mapping(OCSFMapping::from_field("actor.user.email_addr"))
                            .as_dimension()
                            .with_sample_values(vec!["user@example.com".to_string()]),
                    ),
            )
            .add_metric(
                SemanticMetric::new("auth_attempts")
                    .with_caption("Authentication Attempts")
                    .with_description("Count of authentication attempts")
                    .with_aggregation(Aggregation::Count)
                    .with_field_measure("metadata.uid")
                    .with_dimensions(vec![
                        "user_email".to_string(),
                        "auth_result".to_string(),
                        "source_ip".to_string(),
                    ])
                    .with_time_granularities(vec![
                        TimeGranularity::Minute,
                        TimeGranularity::Hour,
                        TimeGranularity::Day,
                    ]),
            )
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5, 10, 22, 30],
            });

        let yaml = model.to_yaml().unwrap();

        // Verify key fields are present in YAML
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("ocsf_version:"));
        assert!(yaml.contains("name: security-analytics"));
        assert!(yaml.contains("entities:"));
        assert!(yaml.contains("metrics:"));
        assert!(yaml.contains("observable_config:"));
        assert!(yaml.contains("source_event_classes:"));
        assert!(yaml.contains("covers_observables:"));
        assert!(yaml.contains("ocsf_mapping:"));
        assert!(yaml.contains("is_dimension: true"));
        assert!(yaml.contains("aggregation: count"));
        assert!(yaml.contains("time_granularities:"));
    }

    #[test]
    fn test_observable_config_defaults() {
        let config = ObservableConfig::default();
        assert!(!config.extract_to_table);
        assert_eq!(config.table_name, "ocsf_observables");
        assert!(config.include_types.is_empty());
    }
}
