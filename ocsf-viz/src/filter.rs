//! View mode filtering and visual differentiation.
//!
//! This module provides functionality to filter graphs by view mode and
//! apply visual differentiation to distinguish semantic from physical elements.

use crate::graph::{
    EdgeStyle, EdgeType, GraphData, GraphEdge, GraphNode, LineStyle, NodeStyle, NodeType, ViewMode,
};

/// Configuration for visual differentiation between layers.
#[derive(Debug, Clone)]
pub struct VisualDifferentiation {
    /// Style overrides for semantic layer nodes.
    pub semantic_node_style: Option<NodeStyleOverride>,
    /// Style overrides for physical layer nodes.
    pub physical_node_style: Option<NodeStyleOverride>,
    /// Style overrides for interchange edges.
    pub interchange_edge_style: Option<EdgeStyleOverride>,
    /// Opacity for non-highlighted elements.
    pub dim_opacity: f32,
}

impl Default for VisualDifferentiation {
    fn default() -> Self {
        Self {
            semantic_node_style: None,
            physical_node_style: None,
            interchange_edge_style: None,
            dim_opacity: 0.3,
        }
    }
}

/// Style overrides for nodes.
#[derive(Debug, Clone, Default)]
pub struct NodeStyleOverride {
    /// Override border width.
    pub border_width: Option<f32>,
    /// Override border color.
    pub border_color: Option<String>,
    /// Override opacity.
    pub opacity: Option<f32>,
}

/// Style overrides for edges.
#[derive(Debug, Clone, Default)]
pub struct EdgeStyleOverride {
    /// Override line style.
    pub line_style: Option<LineStyle>,
    /// Override width.
    pub width: Option<f32>,
    /// Override opacity.
    pub opacity: Option<f32>,
}

/// Filters a graph by view mode with visual differentiation.
pub fn filter_graph_by_mode(graph: &GraphData, mode: ViewMode) -> GraphData {
    let mut filtered = graph.filter_by_view_mode(mode);
    apply_visual_differentiation(&mut filtered, mode);
    filtered
}

/// Applies visual differentiation based on view mode.
pub fn apply_visual_differentiation(graph: &mut GraphData, mode: ViewMode) {
    match mode {
        ViewMode::SemanticOnly => {
            // Emphasize semantic elements
            for node in &mut graph.nodes {
                if node.node_type.is_semantic() {
                    node.style.border_width = 2.0;
                }
            }
        }
        ViewMode::PhysicalOnly => {
            // Emphasize physical elements
            for node in &mut graph.nodes {
                if node.node_type.is_physical() {
                    node.style.border_width = 2.0;
                }
            }
        }
        ViewMode::Interchange => {
            // Apply distinct styling to differentiate layers
            for node in &mut graph.nodes {
                if node.node_type.is_semantic() {
                    // Semantic nodes get thicker borders
                    node.style.border_width = 2.5;
                } else if node.node_type.is_physical() {
                    // Physical nodes get dashed-style borders (simulated with color)
                    node.style.border_width = 1.5;
                }
            }

            // Style interchange edges distinctly
            for edge in &mut graph.edges {
                if edge.is_interchange_edge() {
                    edge.style.width = 2.0;
                }
            }
        }
        ViewMode::HotColdPath => {
            // Emphasize data flow
            for edge in &mut graph.edges {
                if edge.edge_type == EdgeType::DataFlow {
                    edge.style.width = 3.0;
                } else if edge.edge_type == EdgeType::ReverseLookup {
                    edge.style.width = 2.5;
                    edge.style.line_style = LineStyle::Dashed;
                }
            }
        }
    }
}

/// Applies custom visual differentiation settings.
pub fn apply_custom_differentiation(
    graph: &mut GraphData,
    config: &VisualDifferentiation,
) {
    // Apply semantic node overrides
    if let Some(ref override_style) = config.semantic_node_style {
        for node in &mut graph.nodes {
            if node.node_type.is_semantic() {
                apply_node_override(&mut node.style, override_style);
            }
        }
    }

    // Apply physical node overrides
    if let Some(ref override_style) = config.physical_node_style {
        for node in &mut graph.nodes {
            if node.node_type.is_physical() {
                apply_node_override(&mut node.style, override_style);
            }
        }
    }

    // Apply interchange edge overrides
    if let Some(ref override_style) = config.interchange_edge_style {
        for edge in &mut graph.edges {
            if edge.is_interchange_edge() {
                apply_edge_override(&mut edge.style, override_style);
            }
        }
    }
}

/// Applies node style overrides.
fn apply_node_override(style: &mut NodeStyle, override_style: &NodeStyleOverride) {
    if let Some(width) = override_style.border_width {
        style.border_width = width;
    }
    if let Some(ref color) = override_style.border_color {
        style.border_color = color.clone();
    }
    if let Some(opacity) = override_style.opacity {
        style.opacity = opacity;
    }
}

/// Applies edge style overrides.
fn apply_edge_override(style: &mut EdgeStyle, override_style: &EdgeStyleOverride) {
    if let Some(line_style) = override_style.line_style {
        style.line_style = line_style;
    }
    if let Some(width) = override_style.width {
        style.width = width;
    }
    if let Some(opacity) = override_style.opacity {
        style.opacity = opacity;
    }
}

/// Returns nodes that should be visible in the given view mode.
pub fn visible_nodes_for_mode(graph: &GraphData, mode: ViewMode) -> Vec<&GraphNode> {
    graph
        .nodes
        .iter()
        .filter(|n| n.is_visible_in_mode(mode))
        .collect()
}

/// Returns edges that should be visible in the given view mode.
///
/// An edge is visible if both its source and target nodes are visible.
pub fn visible_edges_for_mode<'a>(
    graph: &'a GraphData,
    _mode: ViewMode,
    visible_node_ids: &std::collections::HashSet<&str>,
) -> Vec<&'a GraphEdge> {
    graph
        .edges
        .iter()
        .filter(|e| {
            visible_node_ids.contains(e.source.as_str())
                && visible_node_ids.contains(e.target.as_str())
        })
        .collect()
}

/// Checks if a node type is appropriate for a view mode.
pub fn is_node_type_visible_in_mode(node_type: NodeType, mode: ViewMode) -> bool {
    match mode {
        ViewMode::SemanticOnly => node_type.is_semantic(),
        ViewMode::PhysicalOnly => node_type.is_physical(),
        ViewMode::Interchange => true,
        ViewMode::HotColdPath => node_type.is_path() || node_type.is_observable(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> GraphData {
        let mut graph = GraphData::new();

        // Add semantic nodes
        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1"));
        graph.add_node(GraphNode::new("e2", NodeType::SemanticEntity, "Entity 2"));

        // Add physical nodes
        graph.add_node(GraphNode::new("cat1", NodeType::OcsfCategory, "Category 1"));
        graph.add_node(GraphNode::new("cls1", NodeType::OcsfClass, "Class 1"));
        graph.add_node(GraphNode::new("cls2", NodeType::OcsfClass, "Class 2"));

        // Add observable nodes
        graph.add_node(GraphNode::new("obs1", NodeType::Observable, "Observable 1"));

        // Add path nodes
        graph.add_node(GraphNode::new("hot", NodeType::HotPath, "Hot Path"));
        graph.add_node(GraphNode::new("cold", NodeType::ColdPath, "Cold Path"));

        // Add edges
        graph.add_edge(GraphEdge::new("e1", "cls1", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("e2", "cls2", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("cat1", "cls1", EdgeType::Contains));
        graph.add_edge(GraphEdge::new("cat1", "cls2", EdgeType::Contains));
        graph.add_edge(GraphEdge::new("e1", "e2", EdgeType::RelatesTo));
        graph.add_edge(GraphEdge::new("e1", "obs1", EdgeType::CoversObservable));
        graph.add_edge(GraphEdge::new("obs1", "hot", EdgeType::DataFlow));
        graph.add_edge(GraphEdge::new("hot", "cold", EdgeType::ReverseLookup));

        graph
    }

    #[test]
    fn test_filter_semantic_only() {
        let graph = create_test_graph();
        let filtered = filter_graph_by_mode(&graph, ViewMode::SemanticOnly);

        // Should only have semantic nodes
        assert_eq!(filtered.nodes_of_type(NodeType::SemanticEntity).count(), 2);
        assert_eq!(filtered.nodes_of_type(NodeType::OcsfCategory).count(), 0);
        assert_eq!(filtered.nodes_of_type(NodeType::OcsfClass).count(), 0);
        assert_eq!(filtered.nodes_of_type(NodeType::Observable).count(), 0);

        // Should only have RelatesTo edges (between semantic entities)
        assert_eq!(filtered.edges_of_type(EdgeType::RelatesTo).count(), 1);
        assert_eq!(filtered.edges_of_type(EdgeType::MapsTo).count(), 0);
    }

    #[test]
    fn test_filter_physical_only() {
        let graph = create_test_graph();
        let filtered = filter_graph_by_mode(&graph, ViewMode::PhysicalOnly);

        // Should only have physical nodes
        assert_eq!(filtered.nodes_of_type(NodeType::SemanticEntity).count(), 0);
        assert_eq!(filtered.nodes_of_type(NodeType::OcsfCategory).count(), 1);
        assert_eq!(filtered.nodes_of_type(NodeType::OcsfClass).count(), 2);

        // Should only have Contains edges
        assert_eq!(filtered.edges_of_type(EdgeType::Contains).count(), 2);
        assert_eq!(filtered.edges_of_type(EdgeType::MapsTo).count(), 0);
    }

    #[test]
    fn test_filter_interchange() {
        let graph = create_test_graph();
        let filtered = filter_graph_by_mode(&graph, ViewMode::Interchange);

        // Should have all nodes
        assert_eq!(filtered.node_count(), graph.node_count());

        // Should have all edges
        assert_eq!(filtered.edge_count(), graph.edge_count());
    }

    #[test]
    fn test_filter_hot_cold_path() {
        let graph = create_test_graph();
        let filtered = filter_graph_by_mode(&graph, ViewMode::HotColdPath);

        // Should only have path and observable nodes
        assert_eq!(filtered.nodes_of_type(NodeType::HotPath).count(), 1);
        assert_eq!(filtered.nodes_of_type(NodeType::ColdPath).count(), 1);
        assert_eq!(filtered.nodes_of_type(NodeType::Observable).count(), 1);
        assert_eq!(filtered.nodes_of_type(NodeType::SemanticEntity).count(), 0);

        // Should have DataFlow and ReverseLookup edges
        assert_eq!(filtered.edges_of_type(EdgeType::DataFlow).count(), 1);
        assert_eq!(filtered.edges_of_type(EdgeType::ReverseLookup).count(), 1);
    }

    #[test]
    fn test_visual_differentiation_interchange() {
        let graph = create_test_graph();
        let filtered = filter_graph_by_mode(&graph, ViewMode::Interchange);

        // Semantic nodes should have thicker borders
        for node in filtered.nodes_of_type(NodeType::SemanticEntity) {
            assert!(node.style.border_width > 2.0);
        }

        // Interchange edges should be wider
        for edge in filtered.edges_of_type(EdgeType::MapsTo) {
            assert!(edge.style.width >= 2.0);
        }
    }

    #[test]
    fn test_visual_differentiation_hot_cold_path() {
        let graph = create_test_graph();
        let filtered = filter_graph_by_mode(&graph, ViewMode::HotColdPath);

        // DataFlow edges should be wider
        for edge in filtered.edges_of_type(EdgeType::DataFlow) {
            assert!(edge.style.width >= 3.0);
        }

        // ReverseLookup edges should be dashed
        for edge in filtered.edges_of_type(EdgeType::ReverseLookup) {
            assert_eq!(edge.style.line_style, LineStyle::Dashed);
        }
    }

    #[test]
    fn test_custom_differentiation() {
        let mut graph = create_test_graph();

        let config = VisualDifferentiation {
            semantic_node_style: Some(NodeStyleOverride {
                border_width: Some(5.0),
                border_color: Some("#ff0000".to_string()),
                opacity: None,
            }),
            physical_node_style: Some(NodeStyleOverride {
                border_width: Some(3.0),
                border_color: None,
                opacity: Some(0.8),
            }),
            interchange_edge_style: Some(EdgeStyleOverride {
                line_style: Some(LineStyle::Dotted),
                width: Some(4.0),
                opacity: None,
            }),
            dim_opacity: 0.3,
        };

        apply_custom_differentiation(&mut graph, &config);

        // Check semantic node styling
        for node in graph.nodes_of_type(NodeType::SemanticEntity) {
            assert_eq!(node.style.border_width, 5.0);
            assert_eq!(node.style.border_color, "#ff0000");
        }

        // Check physical node styling
        for node in graph.nodes_of_type(NodeType::OcsfClass) {
            assert_eq!(node.style.border_width, 3.0);
            assert_eq!(node.style.opacity, 0.8);
        }

        // Check interchange edge styling
        for edge in graph.edges_of_type(EdgeType::MapsTo) {
            assert_eq!(edge.style.line_style, LineStyle::Dotted);
            assert_eq!(edge.style.width, 4.0);
        }
    }

    #[test]
    fn test_is_node_type_visible_in_mode() {
        // Semantic only
        assert!(is_node_type_visible_in_mode(
            NodeType::SemanticEntity,
            ViewMode::SemanticOnly
        ));
        assert!(!is_node_type_visible_in_mode(
            NodeType::OcsfClass,
            ViewMode::SemanticOnly
        ));

        // Physical only
        assert!(!is_node_type_visible_in_mode(
            NodeType::SemanticEntity,
            ViewMode::PhysicalOnly
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::OcsfClass,
            ViewMode::PhysicalOnly
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::OcsfCategory,
            ViewMode::PhysicalOnly
        ));

        // Interchange
        assert!(is_node_type_visible_in_mode(
            NodeType::SemanticEntity,
            ViewMode::Interchange
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::OcsfClass,
            ViewMode::Interchange
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::Observable,
            ViewMode::Interchange
        ));

        // Hot/cold path
        assert!(!is_node_type_visible_in_mode(
            NodeType::SemanticEntity,
            ViewMode::HotColdPath
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::HotPath,
            ViewMode::HotColdPath
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::ColdPath,
            ViewMode::HotColdPath
        ));
        assert!(is_node_type_visible_in_mode(
            NodeType::Observable,
            ViewMode::HotColdPath
        ));
    }

    #[test]
    fn test_visible_nodes_for_mode() {
        let graph = create_test_graph();

        let semantic_nodes = visible_nodes_for_mode(&graph, ViewMode::SemanticOnly);
        assert_eq!(semantic_nodes.len(), 2);
        assert!(semantic_nodes.iter().all(|n| n.node_type.is_semantic()));

        let physical_nodes = visible_nodes_for_mode(&graph, ViewMode::PhysicalOnly);
        assert_eq!(physical_nodes.len(), 3); // 1 category + 2 classes
        assert!(physical_nodes.iter().all(|n| n.node_type.is_physical()));
    }
}
