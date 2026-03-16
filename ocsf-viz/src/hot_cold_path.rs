//! Hot/cold path visualization.
//!
//! This module provides functionality to visualize the data flow from
//! observables table (hot path) to full events (cold path), including
//! threat intel matching flow.

use ocsf_core::OCSFSchema;
use ocsf_semantic::SemanticModel;

use crate::graph::{
    EdgeStyle, EdgeType, GraphData, GraphEdge, GraphNode, LineStyle, NodeStyle, NodeType,
    ViewMode,
};

/// Style configuration for hot/cold path visualization.
#[derive(Debug, Clone)]
pub struct HotColdPathStyle {
    /// Color for hot path elements.
    pub hot_path_color: String,
    /// Color for cold path elements.
    pub cold_path_color: String,
    /// Color for data flow edges.
    pub data_flow_color: String,
    /// Color for reverse lookup edges.
    pub reverse_lookup_color: String,
    /// Width for data flow edges.
    pub data_flow_width: f32,
    /// Width for reverse lookup edges.
    pub reverse_lookup_width: f32,
    /// Color for threat intel matching.
    pub threat_intel_color: String,
}

impl Default for HotColdPathStyle {
    fn default() -> Self {
        Self {
            hot_path_color: "#ff6b6b".to_string(),      // Red/orange for hot
            cold_path_color: "#4ecdc4".to_string(),     // Teal for cold
            data_flow_color: "#3498db".to_string(),     // Blue for data flow
            reverse_lookup_color: "#e74c3c".to_string(), // Red for reverse lookup
            data_flow_width: 2.5,
            reverse_lookup_width: 2.0,
            threat_intel_color: "#9b59b6".to_string(),  // Purple for threat intel
        }
    }
}

/// Generates a hot/cold path visualization graph.
///
/// This graph shows:
/// - Observables table (hot path) for fast threat intel matching
/// - Full events table (cold path) for detailed investigation
/// - Data flow from observables to hot path
/// - Reverse lookup from hot path to cold path
/// - Threat intel matching flow
pub fn generate_hot_cold_path_visualization(
    model: &SemanticModel,
    schema: &OCSFSchema,
    style: &HotColdPathStyle,
) -> GraphData {
    let mut graph = GraphData::with_view_mode(ViewMode::HotColdPath);

    // Add hot path node (observables table)
    let hot_path_node = create_hot_path_node(model, style);
    graph.add_node(hot_path_node);

    // Add cold path node (full events table)
    let cold_path_node = create_cold_path_node(style);
    graph.add_node(cold_path_node);

    // Add threat intel node
    let threat_intel_node = create_threat_intel_node(style);
    graph.add_node(threat_intel_node);

    // Add observable nodes
    add_observable_nodes(&mut graph, schema, style);

    // Add data flow edges from observables to hot path
    add_data_flow_edges(&mut graph, schema, style);

    // Add reverse lookup edge
    add_reverse_lookup_edge(&mut graph, style);

    // Add threat intel matching edge
    add_threat_intel_edge(&mut graph, style);

    graph
}

/// Creates the hot path node.
fn create_hot_path_node(model: &SemanticModel, style: &HotColdPathStyle) -> GraphNode {
    GraphNode::new("hot_path", NodeType::HotPath, "Observables Table")
        .with_property(
            "description",
            "Denormalized observables for fast threat intel matching",
        )
        .with_property("table_name", model.observable_config.table_name.clone())
        .with_property(
            "include_types",
            serde_json::json!(model.observable_config.include_types),
        )
        .with_style(
            NodeStyle::for_node_type(NodeType::HotPath)
                .with_color(&style.hot_path_color)
                .with_border_width(2.5),
        )
}

/// Creates the cold path node.
fn create_cold_path_node(style: &HotColdPathStyle) -> GraphNode {
    GraphNode::new("cold_path", NodeType::ColdPath, "Full Events Table")
        .with_property(
            "description",
            "Complete OCSF event data for detailed investigation",
        )
        .with_style(
            NodeStyle::for_node_type(NodeType::ColdPath)
                .with_color(&style.cold_path_color)
                .with_border_width(2.5),
        )
}

/// Creates the threat intel node.
fn create_threat_intel_node(style: &HotColdPathStyle) -> GraphNode {
    GraphNode::new("threat_intel", NodeType::Observable, "Threat Intel Feed")
        .with_property("description", "External threat intelligence data")
        .with_style(
            NodeStyle::for_node_type(NodeType::Observable)
                .with_color(&style.threat_intel_color)
                .with_border_width(2.0),
        )
}

/// Adds observable nodes to the graph.
fn add_observable_nodes(graph: &mut GraphData, schema: &OCSFSchema, _style: &HotColdPathStyle) {
    let mut added_type_ids = std::collections::HashSet::new();

    for observable in schema.all_observables() {
        if !added_type_ids.contains(&observable.type_id) {
            let node = GraphNode::new(
                format!("observable_{}", observable.type_id),
                NodeType::Observable,
                &observable.type_name,
            )
            .with_property("type_id", observable.type_id)
            .with_property("type_name", observable.type_name.clone())
            .with_property("description", observable.description.clone());

            graph.add_node(node);
            added_type_ids.insert(observable.type_id);
        }
    }
}

/// Adds data flow edges from observables to hot path.
fn add_data_flow_edges(graph: &mut GraphData, schema: &OCSFSchema, style: &HotColdPathStyle) {
    let mut added_type_ids = std::collections::HashSet::new();

    for observable in schema.all_observables() {
        if !added_type_ids.contains(&observable.type_id) {
            let observable_id = format!("observable_{}", observable.type_id);

            if graph.get_node(&observable_id).is_some() {
                let edge = GraphEdge::new(&observable_id, "hot_path", EdgeType::DataFlow)
                    .with_label("extract")
                    .with_style(
                        EdgeStyle::for_edge_type(EdgeType::DataFlow)
                            .with_color(&style.data_flow_color)
                            .with_width(style.data_flow_width),
                    );
                graph.add_edge(edge);
            }

            added_type_ids.insert(observable.type_id);
        }
    }
}

/// Adds the reverse lookup edge from hot path to cold path.
fn add_reverse_lookup_edge(graph: &mut GraphData, style: &HotColdPathStyle) {
    let edge = GraphEdge::new("hot_path", "cold_path", EdgeType::ReverseLookup)
        .with_label("reverse lookup")
        .with_style(
            EdgeStyle::for_edge_type(EdgeType::ReverseLookup)
                .with_color(&style.reverse_lookup_color)
                .with_width(style.reverse_lookup_width)
                .with_line_style(LineStyle::Dashed),
        );
    graph.add_edge(edge);
}

/// Adds the threat intel matching edge.
fn add_threat_intel_edge(graph: &mut GraphData, style: &HotColdPathStyle) {
    let edge = GraphEdge::new("threat_intel", "hot_path", EdgeType::DataFlow)
        .with_label("match")
        .with_style(
            EdgeStyle::for_edge_type(EdgeType::DataFlow)
                .with_color(&style.threat_intel_color)
                .with_width(style.data_flow_width)
                .with_line_style(LineStyle::Dotted),
        );
    graph.add_edge(edge);
}

/// Highlights the threat intel matching flow.
pub fn highlight_threat_intel_flow(graph: &mut GraphData, dim_opacity: f32) {
    // Nodes involved in threat intel flow
    let threat_intel_nodes = ["threat_intel".to_string(),
        "hot_path".to_string(),
        "cold_path".to_string()];

    // Apply highlighting
    for node in &mut graph.nodes {
        if threat_intel_nodes.contains(&node.id) {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
        } else {
            node.style.highlighted = false;
            node.style.opacity = dim_opacity;
        }
    }

    for edge in &mut graph.edges {
        let is_threat_flow = (edge.source == "threat_intel" && edge.target == "hot_path")
            || (edge.source == "hot_path" && edge.target == "cold_path");

        if is_threat_flow {
            edge.style.highlighted = true;
            edge.style.opacity = 1.0;
            edge.style.width = 3.0;
        } else {
            edge.style.highlighted = false;
            edge.style.opacity = dim_opacity;
        }
    }
}

/// Highlights the data extraction flow (observables to hot path).
pub fn highlight_extraction_flow(graph: &mut GraphData, dim_opacity: f32) {
    // Find all observable nodes
    let observable_ids: Vec<String> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Observable && n.id != "threat_intel")
        .map(|n| n.id.clone())
        .collect();

    // Apply highlighting
    for node in &mut graph.nodes {
        if observable_ids.contains(&node.id) || node.id == "hot_path" {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
        } else {
            node.style.highlighted = false;
            node.style.opacity = dim_opacity;
        }
    }

    for edge in &mut graph.edges {
        if edge.edge_type == EdgeType::DataFlow && edge.target == "hot_path" {
            edge.style.highlighted = true;
            edge.style.opacity = 1.0;
        } else {
            edge.style.highlighted = false;
            edge.style.opacity = dim_opacity;
        }
    }
}

/// Highlights the reverse lookup flow (hot path to cold path).
pub fn highlight_reverse_lookup_flow(graph: &mut GraphData, dim_opacity: f32) {
    // Apply highlighting
    for node in &mut graph.nodes {
        if node.id == "hot_path" || node.id == "cold_path" {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
        } else {
            node.style.highlighted = false;
            node.style.opacity = dim_opacity;
        }
    }

    for edge in &mut graph.edges {
        if edge.edge_type == EdgeType::ReverseLookup {
            edge.style.highlighted = true;
            edge.style.opacity = 1.0;
            edge.style.width = 3.0;
        } else {
            edge.style.highlighted = false;
            edge.style.opacity = dim_opacity;
        }
    }
}

/// Data flow statistics for the hot/cold path.
#[derive(Debug, Clone)]
pub struct DataFlowStats {
    /// Number of observable types flowing to hot path.
    pub observable_count: usize,
    /// Whether threat intel matching is configured.
    pub has_threat_intel: bool,
    /// Whether reverse lookup is available.
    pub has_reverse_lookup: bool,
    /// Observable type IDs in the hot path.
    pub hot_path_type_ids: Vec<u32>,
}

/// Extracts data flow statistics from the graph.
pub fn extract_data_flow_stats(graph: &GraphData) -> DataFlowStats {
    let observable_count = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Observable && n.id != "threat_intel")
        .count();

    let has_threat_intel = graph.get_node("threat_intel").is_some();

    let has_reverse_lookup = graph
        .edges
        .iter()
        .any(|e| e.edge_type == EdgeType::ReverseLookup);

    let hot_path_type_ids: Vec<u32> = graph
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Observable && n.id != "threat_intel")
        .filter_map(|n| {
            n.get_property("type_id")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
        })
        .collect();

    DataFlowStats {
        observable_count,
        has_threat_intel,
        has_reverse_lookup,
        hot_path_type_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_core::{Attribute, Category, ObservableDefinition, Requirement};
    use ocsf_semantic::ObservableConfig;
    use std::collections::HashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");

        schema.add_attribute(Attribute {
            name: "ip_address".to_string(),
            attr_type: "string_t".to_string(),
            caption: "IP Address".to_string(),
            description: "An IP address".to_string(),
            requirement: Requirement::Optional,
            observable: Some(2),
            is_array: false,
            object_type: None,
            enum_values: HashMap::new(),
            default: None,
        });

        schema.add_observable(
            ObservableDefinition::by_type(5, "Email Address")
                .with_description("Email address observable"),
        );

        schema.add_observable(
            ObservableDefinition::by_type(10, "User Name")
                .with_description("User name observable"),
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
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5, 10],
            })
    }

    #[test]
    fn test_generate_hot_cold_path_graph() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let graph = generate_hot_cold_path_visualization(&model, &schema, &style);

        // Should have hot and cold path nodes
        assert!(graph.get_node("hot_path").is_some());
        assert!(graph.get_node("cold_path").is_some());

        // Should have threat intel node
        assert!(graph.get_node("threat_intel").is_some());

        // Should have observable nodes
        assert!(graph.nodes_of_type(NodeType::Observable).count() > 0);

        // Should have data flow edges
        assert!(graph.edges_of_type(EdgeType::DataFlow).count() > 0);

        // Should have reverse lookup edge
        assert_eq!(graph.edges_of_type(EdgeType::ReverseLookup).count(), 1);
    }

    #[test]
    fn test_hot_path_node_properties() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let graph = generate_hot_cold_path_visualization(&model, &schema, &style);

        let hot_path = graph.get_node("hot_path").unwrap();
        assert_eq!(
            hot_path.get_property("table_name").unwrap(),
            &serde_json::json!("ocsf_observables")
        );
    }

    #[test]
    fn test_highlight_threat_intel_flow() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let mut graph = generate_hot_cold_path_visualization(&model, &schema, &style);
        highlight_threat_intel_flow(&mut graph, 0.3);

        // Threat intel, hot path, and cold path should be highlighted
        assert!(graph.get_node("threat_intel").unwrap().style.highlighted);
        assert!(graph.get_node("hot_path").unwrap().style.highlighted);
        assert!(graph.get_node("cold_path").unwrap().style.highlighted);

        // Observable nodes should be dimmed
        for node in graph.nodes_of_type(NodeType::Observable) {
            if node.id != "threat_intel" {
                assert!(!node.style.highlighted);
            }
        }
    }

    #[test]
    fn test_highlight_extraction_flow() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let mut graph = generate_hot_cold_path_visualization(&model, &schema, &style);
        highlight_extraction_flow(&mut graph, 0.3);

        // Hot path should be highlighted
        assert!(graph.get_node("hot_path").unwrap().style.highlighted);

        // Cold path should be dimmed
        assert!(!graph.get_node("cold_path").unwrap().style.highlighted);

        // Data flow edges should be highlighted
        for edge in graph.edges_of_type(EdgeType::DataFlow) {
            if edge.target == "hot_path" && edge.source != "threat_intel" {
                assert!(edge.style.highlighted);
            }
        }
    }

    #[test]
    fn test_highlight_reverse_lookup_flow() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let mut graph = generate_hot_cold_path_visualization(&model, &schema, &style);
        highlight_reverse_lookup_flow(&mut graph, 0.3);

        // Hot and cold path should be highlighted
        assert!(graph.get_node("hot_path").unwrap().style.highlighted);
        assert!(graph.get_node("cold_path").unwrap().style.highlighted);

        // Reverse lookup edge should be highlighted
        for edge in graph.edges_of_type(EdgeType::ReverseLookup) {
            assert!(edge.style.highlighted);
        }
    }

    #[test]
    fn test_extract_data_flow_stats() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let graph = generate_hot_cold_path_visualization(&model, &schema, &style);
        let stats = extract_data_flow_stats(&graph);

        assert!(stats.observable_count > 0);
        assert!(stats.has_threat_intel);
        assert!(stats.has_reverse_lookup);
        assert!(!stats.hot_path_type_ids.is_empty());
    }

    #[test]
    fn test_view_mode_is_hot_cold_path() {
        let schema = create_test_schema();
        let model = create_test_model();
        let style = HotColdPathStyle::default();

        let graph = generate_hot_cold_path_visualization(&model, &schema, &style);

        assert_eq!(graph.metadata.view_mode, ViewMode::HotColdPath);
    }
}
