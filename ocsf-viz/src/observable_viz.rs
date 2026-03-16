//! Observable-to-entity mapping visualization.
//!
//! This module provides functionality to visualize the coverage relationships
//! between observables and semantic entities, highlighting redundant observables.

use std::collections::HashSet;

use ocsf_core::OCSFSchema;
use ocsf_semantic::{CoverageStatus, ObservableAnalyzer, ObservableCoverageReport, SemanticModel};

use crate::graph::{
    EdgeType, GraphData, GraphEdge, GraphNode, LineStyle, NodeStyle, NodeType,
};

/// Style configuration for observable coverage visualization.
#[derive(Debug, Clone)]
pub struct ObservableCoverageStyle {
    /// Color for fully covered (redundant) observables.
    pub covered_color: String,
    /// Color for partially covered observables.
    pub partial_color: String,
    /// Color for uncovered observables.
    pub uncovered_color: String,
    /// Color for essential observables.
    pub essential_color: String,
    /// Border width for redundant observables.
    pub redundant_border_width: f32,
    /// Line style for coverage edges.
    pub coverage_edge_style: LineStyle,
}

impl Default for ObservableCoverageStyle {
    fn default() -> Self {
        Self {
            covered_color: "#27ae60".to_string(),    // Green - fully covered
            partial_color: "#f39c12".to_string(),    // Orange - partially covered
            uncovered_color: "#e74c3c".to_string(),  // Red - not covered
            essential_color: "#3498db".to_string(),  // Blue - essential
            redundant_border_width: 3.0,
            coverage_edge_style: LineStyle::Solid,
        }
    }
}

/// Generates a graph showing observable-to-entity coverage relationships.
pub fn generate_observable_coverage_graph(
    model: &SemanticModel,
    schema: &OCSFSchema,
    style: &ObservableCoverageStyle,
) -> GraphData {
    let mut graph = GraphData::new();

    // Extract observables and analyze coverage
    let catalog = ocsf_core::extract_observables(schema);
    let analyzer = ObservableAnalyzer::new(&catalog, model);
    let coverage_report = analyzer.analyze_coverage();

    // Add entity nodes
    for entity in &model.entities {
        let node = GraphNode::new(
            format!("entity_{}", entity.name),
            NodeType::SemanticEntity,
            &entity.caption,
        )
        .with_property("name", entity.name.clone())
        .with_property(
            "covers_observables",
            serde_json::json!(entity.covers_observables),
        );
        graph.add_node(node);
    }

    // Add observable nodes with coverage status styling
    for detail in &coverage_report.details {
        let mut node = GraphNode::new(
            format!("observable_{}", detail.type_id),
            NodeType::Observable,
            &detail.type_name,
        )
        .with_property("type_id", detail.type_id)
        .with_property("coverage_percentage", detail.coverage_percentage)
        .with_property("status", format!("{:?}", detail.status))
        .with_property("recommendation", detail.recommendation.clone());

        // Apply coverage-based styling
        node.style = style_for_coverage_status(detail.status, style);

        graph.add_node(node);
    }

    // Add coverage edges
    for entity in &model.entities {
        let entity_id = format!("entity_{}", entity.name);

        for &type_id in &entity.covers_observables {
            let observable_id = format!("observable_{}", type_id);

            if graph.get_node(&observable_id).is_some() {
                let mut edge = GraphEdge::new(&entity_id, &observable_id, EdgeType::CoversObservable);
                edge.style.line_style = style.coverage_edge_style;
                graph.add_edge(edge);
            }
        }
    }

    graph
}

/// Returns a node style based on coverage status.
fn style_for_coverage_status(status: CoverageStatus, config: &ObservableCoverageStyle) -> NodeStyle {
    let base_style = NodeStyle::for_node_type(NodeType::Observable);

    match status {
        CoverageStatus::FullyCovered => NodeStyle {
            color: config.covered_color.clone(),
            border_color: "#1e8449".to_string(),
            border_width: config.redundant_border_width,
            ..base_style
        },
        CoverageStatus::PartiallyCovered => NodeStyle {
            color: config.partial_color.clone(),
            border_color: "#d68910".to_string(),
            ..base_style
        },
        CoverageStatus::NotCovered => NodeStyle {
            color: config.uncovered_color.clone(),
            border_color: "#c0392b".to_string(),
            ..base_style
        },
        CoverageStatus::Essential => NodeStyle {
            color: config.essential_color.clone(),
            border_color: "#2980b9".to_string(),
            ..base_style
        },
    }
}

/// Highlights redundant observables in the graph.
pub fn highlight_redundant_observables(graph: &mut GraphData, dim_opacity: f32) {
    // Find redundant observable IDs
    let redundant_ids: HashSet<String> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.node_type == NodeType::Observable
                && n.get_property("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s.contains("FullyCovered"))
                    .unwrap_or(false)
        })
        .map(|n| n.id.clone())
        .collect();

    // Find entities that cover redundant observables
    let covering_entity_ids: HashSet<String> = graph
        .edges
        .iter()
        .filter(|e| {
            e.edge_type == EdgeType::CoversObservable && redundant_ids.contains(&e.target)
        })
        .map(|e| e.source.clone())
        .collect();

    // Apply highlighting
    for node in &mut graph.nodes {
        if redundant_ids.contains(&node.id) || covering_entity_ids.contains(&node.id) {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
        } else {
            node.style.highlighted = false;
            node.style.opacity = dim_opacity;
        }
    }

    for edge in &mut graph.edges {
        if redundant_ids.contains(&edge.target) && covering_entity_ids.contains(&edge.source) {
            edge.style.highlighted = true;
            edge.style.opacity = 1.0;
        } else {
            edge.style.highlighted = false;
            edge.style.opacity = dim_opacity;
        }
    }
}

/// Highlights uncovered observables in the graph.
pub fn highlight_uncovered_observables(graph: &mut GraphData, dim_opacity: f32) {
    // Find uncovered observable IDs
    let uncovered_ids: HashSet<String> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.node_type == NodeType::Observable
                && n.get_property("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s.contains("NotCovered"))
                    .unwrap_or(false)
        })
        .map(|n| n.id.clone())
        .collect();

    // Apply highlighting
    for node in &mut graph.nodes {
        if uncovered_ids.contains(&node.id) {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
        } else {
            node.style.highlighted = false;
            node.style.opacity = dim_opacity;
        }
    }

    for edge in &mut graph.edges {
        edge.style.highlighted = false;
        edge.style.opacity = dim_opacity;
    }
}

/// Generates a coverage summary for the graph.
#[derive(Debug, Clone)]
pub struct CoverageSummary {
    /// Total number of observables.
    pub total_observables: usize,
    /// Number of fully covered observables.
    pub fully_covered: usize,
    /// Number of partially covered observables.
    pub partially_covered: usize,
    /// Number of uncovered observables.
    pub uncovered: usize,
    /// Overall coverage percentage.
    pub coverage_percentage: f64,
    /// List of redundant observable type_ids.
    pub redundant_type_ids: Vec<u32>,
    /// List of uncovered observable type_ids.
    pub uncovered_type_ids: Vec<u32>,
}

/// Extracts coverage summary from a graph.
pub fn extract_coverage_summary(graph: &GraphData) -> CoverageSummary {
    let mut summary = CoverageSummary {
        total_observables: 0,
        fully_covered: 0,
        partially_covered: 0,
        uncovered: 0,
        coverage_percentage: 0.0,
        redundant_type_ids: Vec::new(),
        uncovered_type_ids: Vec::new(),
    };

    for node in &graph.nodes {
        if node.node_type != NodeType::Observable {
            continue;
        }

        summary.total_observables += 1;

        let type_id = node
            .get_property("type_id")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(0);

        let status = node
            .get_property("status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if status.contains("FullyCovered") {
            summary.fully_covered += 1;
            summary.redundant_type_ids.push(type_id);
        } else if status.contains("PartiallyCovered") {
            summary.partially_covered += 1;
        } else if status.contains("NotCovered") {
            summary.uncovered += 1;
            summary.uncovered_type_ids.push(type_id);
        }
    }

    if summary.total_observables > 0 {
        summary.coverage_percentage = (summary.fully_covered as f64
            + summary.partially_covered as f64 * 0.5)
            / summary.total_observables as f64;
    }

    summary
}

/// Adds coverage annotations to observable nodes.
pub fn annotate_observable_coverage(
    graph: &mut GraphData,
    coverage_report: &ObservableCoverageReport,
) {
    for detail in &coverage_report.details {
        let node_id = format!("observable_{}", detail.type_id);

        if let Some(node) = graph.get_node_mut(&node_id) {
            node.properties
                .insert("coverage_status".to_string(), serde_json::json!(format!("{:?}", detail.status)));
            node.properties.insert(
                "covering_entities".to_string(),
                serde_json::json!(detail.covering_entities),
            );
            node.properties.insert(
                "coverage_percentage".to_string(),
                serde_json::json!(detail.coverage_percentage),
            );
            node.properties.insert(
                "recommendation".to_string(),
                serde_json::json!(detail.recommendation),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_core::{Attribute, Category, ObservableDefinition, Requirement};
    use ocsf_semantic::{ObservableConfig, SemanticAttribute, SemanticEntity};
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
                        SemanticAttribute::new("email").with_field_mapping("email_addr"),
                    ),
            )
            .add_entity(
                SemanticEntity::new("network_entity")
                    .with_caption("Network Entity")
                    .with_description("Represents network data")
                    .with_covers_observables(vec![2, 22]) // Covers IP and hostname
                    .add_attribute(
                        SemanticAttribute::new("ip").with_field_mapping("ip_address"),
                    ),
            )
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5, 10, 22, 30],
            })
    }

    #[test]
    fn test_generate_observable_coverage_graph() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = ObservableCoverageStyle::default();

        let graph = generate_observable_coverage_graph(&model, &schema, &style);

        // Should have entity nodes
        assert_eq!(graph.nodes_of_type(NodeType::SemanticEntity).count(), 2);

        // Should have observable nodes
        assert!(graph.nodes_of_type(NodeType::Observable).count() > 0);

        // Should have coverage edges
        assert!(graph.edges_of_type(EdgeType::CoversObservable).count() > 0);
    }

    #[test]
    fn test_observable_coverage_styling() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = ObservableCoverageStyle::default();

        let graph = generate_observable_coverage_graph(&model, &schema, &style);

        // Covered observables should have green color
        for node in graph.nodes_of_type(NodeType::Observable) {
            let status = node.get_property("status").and_then(|v| v.as_str());
            if let Some(s) = status {
                if s.contains("FullyCovered") {
                    assert_eq!(node.style.color, style.covered_color);
                } else if s.contains("NotCovered") {
                    assert_eq!(node.style.color, style.uncovered_color);
                }
            }
        }
    }

    #[test]
    fn test_highlight_redundant_observables() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = ObservableCoverageStyle::default();

        let mut graph = generate_observable_coverage_graph(&model, &schema, &style);
        highlight_redundant_observables(&mut graph, 0.3);

        // Redundant observables should be highlighted
        for node in &graph.nodes {
            if node.node_type == NodeType::Observable {
                let status = node.get_property("status").and_then(|v| v.as_str());
                if let Some(s) = status {
                    if s.contains("FullyCovered") {
                        assert!(node.style.highlighted);
                    }
                }
            }
        }
    }

    #[test]
    fn test_highlight_uncovered_observables() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = ObservableCoverageStyle::default();

        let mut graph = generate_observable_coverage_graph(&model, &schema, &style);
        highlight_uncovered_observables(&mut graph, 0.3);

        // Uncovered observables should be highlighted
        for node in &graph.nodes {
            if node.node_type == NodeType::Observable {
                let status = node.get_property("status").and_then(|v| v.as_str());
                if let Some(s) = status {
                    if s.contains("NotCovered") {
                        assert!(node.style.highlighted);
                    } else {
                        assert!(!node.style.highlighted);
                    }
                }
            }
        }
    }

    #[test]
    fn test_extract_coverage_summary() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = ObservableCoverageStyle::default();

        let graph = generate_observable_coverage_graph(&model, &schema, &style);
        let summary = extract_coverage_summary(&graph);

        assert_eq!(summary.total_observables, 5);
        assert!(summary.fully_covered > 0);
        assert!(summary.coverage_percentage > 0.0);
    }

    #[test]
    fn test_coverage_summary_with_uncovered() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = ObservableCoverageStyle::default();

        let graph = generate_observable_coverage_graph(&model, &schema, &style);
        let summary = extract_coverage_summary(&graph);

        // File hash (30) is not covered
        assert!(summary.uncovered > 0);
        assert!(summary.uncovered_type_ids.contains(&30));
    }
}
