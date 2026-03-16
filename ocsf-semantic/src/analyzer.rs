//! Observable analyzer for semantic coverage analysis.
//!
//! This module provides functionality to analyze how OCSF observables relate
//! to semantic entities, determining coverage and identifying redundancies.

use std::collections::{HashMap, HashSet};

use ocsf_core::ObservableCatalog;

use crate::entity::SemanticEntity;
use crate::model::SemanticModel;

/// Status of an observable's coverage by the semantic layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStatus {
    /// Observable is fully covered by semantic entities.
    FullyCovered,
    /// Observable is partially covered by semantic entities.
    PartiallyCovered,
    /// Observable is not covered by any semantic entity.
    NotCovered,
    /// Observable is essential and should not be replaced.
    Essential,
}

/// Detail about an observable's coverage.
#[derive(Debug, Clone)]
pub struct ObservableCoverageDetail {
    /// The observable type_id.
    pub type_id: u32,
    /// The observable type name.
    pub type_name: String,
    /// Coverage status.
    pub status: CoverageStatus,
    /// Names of entities that cover this observable.
    pub covering_entities: Vec<String>,
    /// Coverage percentage (0.0 to 1.0).
    pub coverage_percentage: f64,
    /// Recommendation for this observable.
    pub recommendation: String,
}

/// Report of observable coverage by the semantic layer.
#[derive(Debug, Clone, Default)]
pub struct ObservableCoverageReport {
    /// Total number of unique observable type_ids.
    pub total_observables: usize,
    /// Number of observables fully covered by semantic layer.
    pub covered_by_semantic_layer: usize,
    /// Number of observables partially covered.
    pub partially_covered: usize,
    /// Number of observables not covered.
    pub not_covered: usize,
    /// Detailed coverage information for each observable.
    pub details: Vec<ObservableCoverageDetail>,
}

impl ObservableCoverageReport {
    /// Creates a new empty coverage report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the overall coverage percentage.
    pub fn overall_coverage_percentage(&self) -> f64 {
        if self.total_observables == 0 {
            return 0.0;
        }
        (self.covered_by_semantic_layer as f64 + self.partially_covered as f64 * 0.5)
            / self.total_observables as f64
    }

    /// Returns observables that are fully covered (redundant).
    pub fn redundant_observables(&self) -> impl Iterator<Item = &ObservableCoverageDetail> {
        self.details
            .iter()
            .filter(|d| d.status == CoverageStatus::FullyCovered)
    }

    /// Returns observables that are not covered.
    pub fn uncovered_observables(&self) -> impl Iterator<Item = &ObservableCoverageDetail> {
        self.details
            .iter()
            .filter(|d| d.status == CoverageStatus::NotCovered)
    }

    /// Returns observables that are partially covered.
    pub fn partial_observables(&self) -> impl Iterator<Item = &ObservableCoverageDetail> {
        self.details
            .iter()
            .filter(|d| d.status == CoverageStatus::PartiallyCovered)
    }
}

/// Compatibility report showing observable-semantic layer relationship.
#[derive(Debug, Clone, Default)]
pub struct CompatibilityReport {
    /// OCSF schema version.
    pub schema_version: String,
    /// Semantic model version.
    pub semantic_model_version: String,
    /// Observables that should be kept (essential).
    pub essential_observables: Vec<u32>,
    /// Observables that can be replaced by semantic layer.
    pub redundant_observables: Vec<u32>,
    /// Gaps in semantic coverage.
    pub coverage_gaps: Vec<CoverageGap>,
}

/// A gap in semantic coverage for an observable.
#[derive(Debug, Clone)]
pub struct CoverageGap {
    /// The observable type_id.
    pub observable_type_id: u32,
    /// The observable type name.
    pub type_name: String,
    /// Description of the gap.
    pub description: String,
    /// Suggested entity to create.
    pub suggested_entity_name: Option<String>,
}

/// Analyzer for observable-to-semantic-entity relationships.
#[derive(Debug)]
pub struct ObservableAnalyzer<'a> {
    /// The observable catalog from the schema.
    catalog: &'a ObservableCatalog,
    /// The semantic model.
    model: &'a SemanticModel,
    /// Mapping from observable type_id to covering entity names.
    coverage_map: HashMap<u32, Vec<String>>,
}

impl<'a> ObservableAnalyzer<'a> {
    /// Creates a new observable analyzer.
    pub fn new(catalog: &'a ObservableCatalog, model: &'a SemanticModel) -> Self {
        let mut analyzer = Self {
            catalog,
            model,
            coverage_map: HashMap::new(),
        };
        analyzer.build_coverage_map();
        analyzer
    }

    /// Builds the mapping from observable type_ids to covering entities.
    fn build_coverage_map(&mut self) {
        for entity in &self.model.entities {
            for &type_id in &entity.covers_observables {
                self.coverage_map
                    .entry(type_id)
                    .or_default()
                    .push(entity.name.clone());
            }
        }
    }

    /// Maps an observable type_id to the semantic entities that cover it.
    pub fn map_observable_to_entities(&self, type_id: u32) -> Vec<&SemanticEntity> {
        self.coverage_map
            .get(&type_id)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.model.get_entity(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns all observable type_ids that are covered by at least one entity.
    pub fn covered_observable_type_ids(&self) -> HashSet<u32> {
        self.coverage_map.keys().copied().collect()
    }

    /// Calculates the coverage percentage for a specific observable type_id.
    ///
    /// Returns a value between 0.0 (not covered) and 1.0 (fully covered).
    pub fn calculate_coverage_percentage(&self, type_id: u32) -> f64 {
        let covering_entities = self.map_observable_to_entities(type_id);
        if covering_entities.is_empty() {
            return 0.0;
        }

        // Check if any entity explicitly covers this observable
        // For now, explicit coverage means full coverage
        // In a more sophisticated implementation, we could check
        // if the entity's attributes actually cover all the observable's use cases
        1.0
    }

    /// Determines the coverage status for an observable type_id.
    pub fn get_coverage_status(&self, type_id: u32) -> CoverageStatus {
        let coverage = self.calculate_coverage_percentage(type_id);
        
        if coverage >= 1.0 {
            CoverageStatus::FullyCovered
        } else if coverage > 0.0 {
            CoverageStatus::PartiallyCovered
        } else {
            CoverageStatus::NotCovered
        }
    }

    /// Generates a recommendation for an observable based on its coverage.
    fn generate_recommendation(&self, type_id: u32, status: CoverageStatus) -> String {
        match status {
            CoverageStatus::FullyCovered => {
                "Observable is fully covered by semantic layer. Consider removing from direct queries.".to_string()
            }
            CoverageStatus::PartiallyCovered => {
                "Observable is partially covered. Review semantic entity definitions for completeness.".to_string()
            }
            CoverageStatus::NotCovered => {
                format!(
                    "Observable type_id {} is not covered. Consider creating a semantic entity to cover this observable.",
                    type_id
                )
            }
            CoverageStatus::Essential => {
                "Observable is marked as essential and should be retained.".to_string()
            }
        }
    }

    /// Analyzes semantic coverage for all observables in the catalog.
    pub fn analyze_coverage(&self) -> ObservableCoverageReport {
        let mut report = ObservableCoverageReport::new();

        // Get unique observable type_ids from the catalog
        let type_ids: HashSet<u32> = self.catalog.type_ids().copied().collect();
        report.total_observables = type_ids.len();

        for type_id in type_ids {
            let type_name = self
                .catalog
                .get_type_name(type_id)
                .unwrap_or("Unknown")
                .to_string();

            let covering_entities: Vec<String> = self
                .map_observable_to_entities(type_id)
                .iter()
                .map(|e| e.name.clone())
                .collect();

            let coverage_percentage = self.calculate_coverage_percentage(type_id);
            let status = self.get_coverage_status(type_id);
            let recommendation = self.generate_recommendation(type_id, status);

            // Update counters
            match status {
                CoverageStatus::FullyCovered => report.covered_by_semantic_layer += 1,
                CoverageStatus::PartiallyCovered => report.partially_covered += 1,
                CoverageStatus::NotCovered => report.not_covered += 1,
                CoverageStatus::Essential => {} // Essential observables are counted separately
            }

            report.details.push(ObservableCoverageDetail {
                type_id,
                type_name,
                status,
                covering_entities,
                coverage_percentage,
                recommendation,
            });
        }

        // Sort details by type_id for consistent output
        report.details.sort_by_key(|d| d.type_id);

        report
    }

    /// Generates a compatibility report.
    pub fn generate_compatibility_report(
        &self,
        schema_version: &str,
    ) -> CompatibilityReport {
        let coverage_report = self.analyze_coverage();

        let mut report = CompatibilityReport {
            schema_version: schema_version.to_string(),
            semantic_model_version: self.model.version.clone(),
            essential_observables: Vec::new(),
            redundant_observables: Vec::new(),
            coverage_gaps: Vec::new(),
        };

        for detail in &coverage_report.details {
            match detail.status {
                CoverageStatus::FullyCovered => {
                    report.redundant_observables.push(detail.type_id);
                }
                CoverageStatus::NotCovered => {
                    report.coverage_gaps.push(CoverageGap {
                        observable_type_id: detail.type_id,
                        type_name: detail.type_name.clone(),
                        description: format!(
                            "Observable '{}' (type_id: {}) has no semantic entity coverage",
                            detail.type_name, detail.type_id
                        ),
                        suggested_entity_name: Some(suggest_entity_name(&detail.type_name)),
                    });
                }
                CoverageStatus::PartiallyCovered => {
                    // Partially covered observables might still be needed
                    report.coverage_gaps.push(CoverageGap {
                        observable_type_id: detail.type_id,
                        type_name: detail.type_name.clone(),
                        description: format!(
                            "Observable '{}' (type_id: {}) is only partially covered by: {}",
                            detail.type_name,
                            detail.type_id,
                            detail.covering_entities.join(", ")
                        ),
                        suggested_entity_name: None,
                    });
                }
                CoverageStatus::Essential => {
                    report.essential_observables.push(detail.type_id);
                }
            }
        }

        report
    }

    /// Checks if an observable is redundant (fully covered by semantic layer).
    pub fn is_observable_redundant(&self, type_id: u32) -> bool {
        self.get_coverage_status(type_id) == CoverageStatus::FullyCovered
    }

    /// Returns all redundant observable type_ids.
    pub fn get_redundant_observables(&self) -> Vec<u32> {
        self.catalog
            .type_ids()
            .filter(|&&type_id| self.is_observable_redundant(type_id))
            .copied()
            .collect()
    }
}

/// Suggests an entity name based on the observable type name.
fn suggest_entity_name(type_name: &str) -> String {
    // Convert to snake_case and add "_entity" suffix
    let name = type_name
        .to_lowercase()
        .replace([' ', '-'], "_");
    format!("{}_entity", name)
}

/// Analyzes observable coverage for a semantic model against an observable catalog.
///
/// This is a convenience function that creates an analyzer and generates a coverage report.
pub fn analyze_observable_coverage(
    catalog: &ObservableCatalog,
    model: &SemanticModel,
) -> ObservableCoverageReport {
    ObservableAnalyzer::new(catalog, model).analyze_coverage()
}

/// Maps observable type_ids to covering semantic entities.
///
/// Returns a map from type_id to list of entity names that cover it.
pub fn map_observables_to_entities(
    catalog: &ObservableCatalog,
    model: &SemanticModel,
) -> HashMap<u32, Vec<String>> {
    let analyzer = ObservableAnalyzer::new(catalog, model);
    
    catalog
        .type_ids()
        .map(|&type_id| {
            let entities: Vec<String> = analyzer
                .map_observable_to_entities(type_id)
                .iter()
                .map(|e| e.name.clone())
                .collect();
            (type_id, entities)
        })
        .collect()
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::SemanticAttribute;
    use ocsf_core::{
        extract_observables, Attribute, Category, OCSFSchema,
        ObservableDefinition, Requirement,
    };
    use std::collections::HashMap as StdHashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");

        // Add base attributes with observables
        schema.add_attribute(Attribute {
            name: "ip_address".to_string(),
            attr_type: "string_t".to_string(),
            caption: "IP Address".to_string(),
            description: "An IP address".to_string(),
            requirement: Requirement::Optional,
            observable: Some(2),
            is_array: false,
            object_type: None,
            enum_values: StdHashMap::new(),
            default: None,
        });

        schema.add_attribute(Attribute {
            name: "email_addr".to_string(),
            attr_type: "string_t".to_string(),
            caption: "Email Address".to_string(),
            description: "An email address".to_string(),
            requirement: Requirement::Optional,
            observable: Some(5),
            is_array: false,
            object_type: None,
            enum_values: StdHashMap::new(),
            default: None,
        });

        schema.add_attribute(Attribute {
            name: "user_name".to_string(),
            attr_type: "string_t".to_string(),
            caption: "User Name".to_string(),
            description: "A user name".to_string(),
            requirement: Requirement::Optional,
            observable: Some(10),
            is_array: false,
            object_type: None,
            enum_values: StdHashMap::new(),
            default: None,
        });

        // Add schema-level observables
        schema.add_observable(
            ObservableDefinition::by_type(22, "Hostname")
                .with_description("A hostname observable"),
        );

        schema.add_observable(
            ObservableDefinition::by_type(30, "File Hash")
                .with_description("A file hash observable"),
        );

        // Add a category
        schema.add_category(Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        });

        schema
    }

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test-model")
            .with_version("1.0")
            .with_ocsf_version("1.4.0")
            .add_entity(
                SemanticEntity::new("user_entity")
                    .with_caption("User Entity")
                    .with_description("Represents a user")
                    .with_covers_observables(vec![5, 10]) // Covers email and user_name
                    .add_attribute(
                        SemanticAttribute::new("email")
                            .with_field_mapping("email_addr"),
                    )
                    .add_attribute(
                        SemanticAttribute::new("name")
                            .with_field_mapping("user_name"),
                    ),
            )
            .add_entity(
                SemanticEntity::new("network_entity")
                    .with_caption("Network Entity")
                    .with_description("Represents network data")
                    .with_covers_observables(vec![2, 22]) // Covers IP and hostname
                    .add_attribute(
                        SemanticAttribute::new("ip")
                            .with_field_mapping("ip_address"),
                    )
                    .add_attribute(
                        SemanticAttribute::new("hostname")
                            .with_field_mapping("hostname"),
                    ),
            )
    }

    #[test]
    fn test_observable_to_entity_mapping() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        // Email (type_id 5) should be covered by user_entity
        let email_entities = analyzer.map_observable_to_entities(5);
        assert_eq!(email_entities.len(), 1);
        assert_eq!(email_entities[0].name, "user_entity");

        // IP (type_id 2) should be covered by network_entity
        let ip_entities = analyzer.map_observable_to_entities(2);
        assert_eq!(ip_entities.len(), 1);
        assert_eq!(ip_entities[0].name, "network_entity");

        // File hash (type_id 30) should not be covered
        let hash_entities = analyzer.map_observable_to_entities(30);
        assert!(hash_entities.is_empty());
    }

    #[test]
    fn test_coverage_percentage() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        // Covered observables should have 100% coverage
        assert_eq!(analyzer.calculate_coverage_percentage(5), 1.0);
        assert_eq!(analyzer.calculate_coverage_percentage(2), 1.0);

        // Uncovered observables should have 0% coverage
        assert_eq!(analyzer.calculate_coverage_percentage(30), 0.0);
    }

    #[test]
    fn test_coverage_status() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        // Covered observables
        assert_eq!(
            analyzer.get_coverage_status(5),
            CoverageStatus::FullyCovered
        );
        assert_eq!(
            analyzer.get_coverage_status(2),
            CoverageStatus::FullyCovered
        );

        // Uncovered observables
        assert_eq!(
            analyzer.get_coverage_status(30),
            CoverageStatus::NotCovered
        );
    }

    #[test]
    fn test_analyze_coverage_report() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.analyze_coverage();

        // Should have 5 unique observable types (2, 5, 10, 22, 30)
        assert_eq!(report.total_observables, 5);

        // 4 should be covered (2, 5, 10, 22)
        assert_eq!(report.covered_by_semantic_layer, 4);

        // 1 should not be covered (30)
        assert_eq!(report.not_covered, 1);

        // Check details
        assert_eq!(report.details.len(), 5);
    }

    #[test]
    fn test_redundant_observables() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);

        // Should identify covered observables as redundant
        assert!(analyzer.is_observable_redundant(5));
        assert!(analyzer.is_observable_redundant(2));
        assert!(!analyzer.is_observable_redundant(30));

        let redundant = analyzer.get_redundant_observables();
        assert!(redundant.contains(&5));
        assert!(redundant.contains(&2));
        assert!(!redundant.contains(&30));
    }

    #[test]
    fn test_compatibility_report() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.generate_compatibility_report("1.4.0");

        assert_eq!(report.schema_version, "1.4.0");
        assert_eq!(report.semantic_model_version, "1.0");

        // Should have redundant observables
        assert!(!report.redundant_observables.is_empty());
        assert!(report.redundant_observables.contains(&5));

        // Should have coverage gaps for uncovered observables
        assert!(!report.coverage_gaps.is_empty());
        let gap = report
            .coverage_gaps
            .iter()
            .find(|g| g.observable_type_id == 30);
        assert!(gap.is_some());
    }

    #[test]
    fn test_empty_model() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = SemanticModel::new("empty");

        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.analyze_coverage();

        // All observables should be uncovered
        assert_eq!(report.covered_by_semantic_layer, 0);
        assert_eq!(report.not_covered, report.total_observables);
    }

    #[test]
    fn test_empty_catalog() {
        let schema = OCSFSchema::new("1.0.0");
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.analyze_coverage();

        assert_eq!(report.total_observables, 0);
        assert_eq!(report.covered_by_semantic_layer, 0);
    }

    #[test]
    fn test_suggest_entity_name() {
        assert_eq!(suggest_entity_name("IP Address"), "ip_address_entity");
        assert_eq!(suggest_entity_name("User Name"), "user_name_entity");
        assert_eq!(suggest_entity_name("File-Hash"), "file_hash_entity");
    }

    #[test]
    fn test_coverage_report_iterators() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.analyze_coverage();

        // Test redundant_observables iterator
        let redundant: Vec<_> = report.redundant_observables().collect();
        assert!(!redundant.is_empty());

        // Test uncovered_observables iterator
        let uncovered: Vec<_> = report.uncovered_observables().collect();
        assert!(!uncovered.is_empty());
        assert!(uncovered.iter().any(|d| d.type_id == 30));
    }

    #[test]
    fn test_overall_coverage_percentage() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let analyzer = ObservableAnalyzer::new(&catalog, &model);
        let report = analyzer.analyze_coverage();

        // 4 out of 5 observables are covered = 80%
        let coverage = report.overall_coverage_percentage();
        assert!((coverage - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_map_observables_to_entities_convenience() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let mapping = map_observables_to_entities(&catalog, &model);

        // Check that covered observables have entity mappings
        assert!(mapping.get(&5).map(|v| !v.is_empty()).unwrap_or(false));
        assert!(mapping.get(&2).map(|v| !v.is_empty()).unwrap_or(false));

        // Check that uncovered observables have empty mappings
        assert!(mapping.get(&30).map(|v| v.is_empty()).unwrap_or(true));
    }

    #[test]
    fn test_analyze_observable_coverage_convenience() {
        let schema = create_test_schema();
        let catalog = extract_observables(&schema);
        let model = create_test_model();

        let report = analyze_observable_coverage(&catalog, &model);

        assert_eq!(report.total_observables, 5);
        assert!(report.covered_by_semantic_layer > 0);
    }
}
