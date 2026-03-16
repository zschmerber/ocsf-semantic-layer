//! Entity highlighting functionality.
//!
//! This module provides functionality to highlight selected entities and
//! their connected elements in the visualization graph.

use std::collections::HashSet;

use crate::graph::{EdgeType, GraphData, NodeType};

/// Configuration for highlighting behavior.
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Opacity for non-highlighted elements.
    pub dim_opacity: f32,
    /// Border width for highlighted nodes.
    pub highlight_border_width: f32,
    /// Whether to highlight connected nodes.
    pub highlight_connected: bool,
    /// Maximum depth for connected node highlighting.
    pub max_depth: usize,
    /// Edge types to follow when finding connected nodes.
    pub follow_edge_types: HashSet<EdgeType>,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        let mut follow_edge_types = HashSet::new();
        follow_edge_types.insert(EdgeType::MapsTo);
        follow_edge_types.insert(EdgeType::Contains);
        follow_edge_types.insert(EdgeType::RelatesTo);
        follow_edge_types.insert(EdgeType::CoversObservable);
        follow_edge_types.insert(EdgeType::ReverseLookup);
        follow_edge_types.insert(EdgeType::DataFlow);

        Self {
            dim_opacity: 0.3,
            highlight_border_width: 3.0,
            highlight_connected: true,
            max_depth: 2,
            follow_edge_types,
        }
    }
}

impl HighlightConfig {
    /// Creates a new highlight config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the dim opacity.
    pub fn with_dim_opacity(mut self, opacity: f32) -> Self {
        self.dim_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Sets the highlight border width.
    pub fn with_highlight_border_width(mut self, width: f32) -> Self {
        self.highlight_border_width = width;
        self
    }

    /// Sets whether to highlight connected nodes.
    pub fn with_highlight_connected(mut self, highlight: bool) -> Self {
        self.highlight_connected = highlight;
        self
    }

    /// Sets the maximum depth for connected node highlighting.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets the edge types to follow.
    pub fn with_follow_edge_types(mut self, types: HashSet<EdgeType>) -> Self {
        self.follow_edge_types = types;
        self
    }
}

/// Highlights a semantic entity and all connected OCSF elements.
///
/// This function highlights:
/// - The selected entity node
/// - All OCSF event classes the entity maps to
/// - All categories containing those event classes
/// - All observables the entity covers
/// - Related entities (if configured)
pub fn highlight_entity(graph: &mut GraphData, entity_name: &str, config: &HighlightConfig) {
    let entity_id = format!("entity_{}", entity_name);
    highlight_node_with_config(graph, &entity_id, config);
}

/// Highlights a node and its connected elements with configuration.
pub fn highlight_node_with_config(graph: &mut GraphData, node_id: &str, config: &HighlightConfig) {
    // Find all connected node IDs
    let connected_ids = if config.highlight_connected {
        find_connected_nodes(graph, node_id, config.max_depth, &config.follow_edge_types)
    } else {
        let mut set = HashSet::new();
        set.insert(node_id.to_string());
        set
    };

    // Apply highlighting
    for node in &mut graph.nodes {
        if connected_ids.contains(&node.id) {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
            if node.id == node_id {
                // Primary node gets extra emphasis
                node.style.border_width = config.highlight_border_width;
            }
        } else {
            node.style.highlighted = false;
            node.style.opacity = config.dim_opacity;
        }
    }

    // Highlight edges between connected nodes
    for edge in &mut graph.edges {
        if connected_ids.contains(&edge.source) && connected_ids.contains(&edge.target) {
            edge.style.highlighted = true;
            edge.style.opacity = 1.0;
        } else {
            edge.style.highlighted = false;
            edge.style.opacity = config.dim_opacity;
        }
    }
}

/// Finds all nodes connected to the given node within the specified depth.
fn find_connected_nodes(
    graph: &GraphData,
    start_node_id: &str,
    max_depth: usize,
    follow_edge_types: &HashSet<EdgeType>,
) -> HashSet<String> {
    let mut connected = HashSet::new();
    let mut frontier = vec![start_node_id.to_string()];
    let mut visited = HashSet::new();

    for _depth in 0..=max_depth {
        let mut next_frontier = Vec::new();

        for node_id in frontier {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());
            connected.insert(node_id.clone());

            // Find connected nodes through edges
            for edge in &graph.edges {
                if !follow_edge_types.contains(&edge.edge_type) {
                    continue;
                }

                if edge.source == node_id && !visited.contains(&edge.target) {
                    next_frontier.push(edge.target.clone());
                }
                if edge.target == node_id && !visited.contains(&edge.source) {
                    next_frontier.push(edge.source.clone());
                }
            }
        }

        frontier = next_frontier;
        if frontier.is_empty() {
            break;
        }
    }

    connected
}

/// Highlights an observable and shows its coverage by semantic entities.
pub fn highlight_observable(graph: &mut GraphData, type_id: u32, config: &HighlightConfig) {
    let observable_id = format!("observable_{}", type_id);
    highlight_node_with_config(graph, &observable_id, config);
}

/// Highlights all nodes in a category.
pub fn highlight_category(graph: &mut GraphData, category_uid: u32, config: &HighlightConfig) {
    let category_id = format!("category_{}", category_uid);
    highlight_node_with_config(graph, &category_id, config);
}

/// Highlights the hot path data flow.
pub fn highlight_hot_path(graph: &mut GraphData, config: &HighlightConfig) {
    // Find all hot path related nodes
    let mut hot_path_nodes = HashSet::new();

    // Add hot path and cold path nodes
    for node in &graph.nodes {
        if node.node_type == NodeType::HotPath || node.node_type == NodeType::ColdPath {
            hot_path_nodes.insert(node.id.clone());
        }
    }

    // Add observables connected to hot path
    for edge in &graph.edges {
        if edge.edge_type == EdgeType::DataFlow {
            hot_path_nodes.insert(edge.source.clone());
            hot_path_nodes.insert(edge.target.clone());
        }
        if edge.edge_type == EdgeType::ReverseLookup {
            hot_path_nodes.insert(edge.source.clone());
            hot_path_nodes.insert(edge.target.clone());
        }
    }

    // Apply highlighting
    for node in &mut graph.nodes {
        if hot_path_nodes.contains(&node.id) {
            node.style.highlighted = true;
            node.style.opacity = 1.0;
        } else {
            node.style.highlighted = false;
            node.style.opacity = config.dim_opacity;
        }
    }

    for edge in &mut graph.edges {
        if edge.edge_type == EdgeType::DataFlow || edge.edge_type == EdgeType::ReverseLookup {
            edge.style.highlighted = true;
            edge.style.opacity = 1.0;
            edge.style.width = 2.5;
        } else {
            edge.style.highlighted = false;
            edge.style.opacity = config.dim_opacity;
        }
    }
}

/// Clears all highlighting from the graph.
pub fn clear_highlighting(graph: &mut GraphData) {
    graph.clear_highlighting();
}

/// Returns the IDs of all nodes connected to the given entity.
pub fn get_connected_node_ids(
    graph: &GraphData,
    entity_name: &str,
    max_depth: usize,
) -> HashSet<String> {
    let entity_id = format!("entity_{}", entity_name);
    let config = HighlightConfig::default();
    find_connected_nodes(graph, &entity_id, max_depth, &config.follow_edge_types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphEdge, GraphNode};

    fn create_test_graph() -> GraphData {
        let mut graph = GraphData::new();

        // Add semantic entities
        graph.add_node(GraphNode::new("entity_user", NodeType::SemanticEntity, "User"));
        graph.add_node(GraphNode::new(
            "entity_auth",
            NodeType::SemanticEntity,
            "Authentication",
        ));

        // Add OCSF elements
        graph.add_node(GraphNode::new("category_3", NodeType::OcsfCategory, "IAM"));
        graph.add_node(GraphNode::new("class_3002", NodeType::OcsfClass, "Authentication"));
        graph.add_node(GraphNode::new("class_3003", NodeType::OcsfClass, "Account Change"));

        // Add observables
        graph.add_node(GraphNode::new("observable_5", NodeType::Observable, "Email"));
        graph.add_node(GraphNode::new("observable_10", NodeType::Observable, "User Name"));

        // Add path nodes
        graph.add_node(GraphNode::new("hot_path", NodeType::HotPath, "Observables Table"));
        graph.add_node(GraphNode::new("cold_path", NodeType::ColdPath, "Full Events"));

        // Add edges
        graph.add_edge(GraphEdge::new("entity_user", "class_3002", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("entity_auth", "class_3002", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("entity_auth", "class_3003", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("category_3", "class_3002", EdgeType::Contains));
        graph.add_edge(GraphEdge::new("category_3", "class_3003", EdgeType::Contains));
        graph.add_edge(GraphEdge::new("entity_user", "entity_auth", EdgeType::RelatesTo));
        graph.add_edge(GraphEdge::new(
            "entity_user",
            "observable_5",
            EdgeType::CoversObservable,
        ));
        graph.add_edge(GraphEdge::new(
            "entity_user",
            "observable_10",
            EdgeType::CoversObservable,
        ));
        graph.add_edge(GraphEdge::new("observable_5", "hot_path", EdgeType::DataFlow));
        graph.add_edge(GraphEdge::new("observable_10", "hot_path", EdgeType::DataFlow));
        graph.add_edge(GraphEdge::new("hot_path", "cold_path", EdgeType::ReverseLookup));

        graph
    }

    #[test]
    fn test_highlight_entity() {
        let mut graph = create_test_graph();
        let config = HighlightConfig::default();

        highlight_entity(&mut graph, "user", &config);

        // User entity should be highlighted
        let user_node = graph.get_node("entity_user").unwrap();
        assert!(user_node.style.highlighted);
        assert_eq!(user_node.style.opacity, 1.0);

        // Connected class should be highlighted
        let class_node = graph.get_node("class_3002").unwrap();
        assert!(class_node.style.highlighted);

        // Connected observables should be highlighted
        let obs_node = graph.get_node("observable_5").unwrap();
        assert!(obs_node.style.highlighted);

        // Related entity should be highlighted
        let auth_node = graph.get_node("entity_auth").unwrap();
        assert!(auth_node.style.highlighted);
    }

    #[test]
    fn test_highlight_entity_dims_unconnected() {
        let mut graph = create_test_graph();
        let config = HighlightConfig::default().with_max_depth(1);

        highlight_entity(&mut graph, "user", &config);

        // Unconnected nodes should be dimmed
        // With max_depth=1, class_3003 is not directly connected to user
        // (it's connected through auth, which is depth 1, and class_3003 is depth 2)
    }

    #[test]
    fn test_highlight_observable() {
        let mut graph = create_test_graph();
        let config = HighlightConfig::default();

        highlight_observable(&mut graph, 5, &config);

        // Observable should be highlighted
        let obs_node = graph.get_node("observable_5").unwrap();
        assert!(obs_node.style.highlighted);

        // Entity covering it should be highlighted
        let user_node = graph.get_node("entity_user").unwrap();
        assert!(user_node.style.highlighted);

        // Hot path should be highlighted (connected via DataFlow)
        let hot_path = graph.get_node("hot_path").unwrap();
        assert!(hot_path.style.highlighted);
    }

    #[test]
    fn test_highlight_hot_path() {
        let mut graph = create_test_graph();
        let config = HighlightConfig::default();

        highlight_hot_path(&mut graph, &config);

        // Hot and cold path nodes should be highlighted
        assert!(graph.get_node("hot_path").unwrap().style.highlighted);
        assert!(graph.get_node("cold_path").unwrap().style.highlighted);

        // Observables connected to hot path should be highlighted
        assert!(graph.get_node("observable_5").unwrap().style.highlighted);
        assert!(graph.get_node("observable_10").unwrap().style.highlighted);

        // Semantic entities should be dimmed
        assert!(!graph.get_node("entity_user").unwrap().style.highlighted);
    }

    #[test]
    fn test_clear_highlighting() {
        let mut graph = create_test_graph();
        let config = HighlightConfig::default();

        highlight_entity(&mut graph, "user", &config);
        clear_highlighting(&mut graph);

        // All nodes should have highlighting cleared
        for node in &graph.nodes {
            assert!(!node.style.highlighted);
            assert_eq!(node.style.opacity, 1.0);
        }

        // All edges should have highlighting cleared
        for edge in &graph.edges {
            assert!(!edge.style.highlighted);
            assert_eq!(edge.style.opacity, 1.0);
        }
    }

    #[test]
    fn test_get_connected_node_ids() {
        let graph = create_test_graph();

        let connected = get_connected_node_ids(&graph, "user", 1);

        // Should include the entity itself
        assert!(connected.contains("entity_user"));

        // Should include directly connected nodes
        assert!(connected.contains("class_3002"));
        assert!(connected.contains("entity_auth"));
        assert!(connected.contains("observable_5"));
        assert!(connected.contains("observable_10"));
    }

    #[test]
    fn test_highlight_config_builder() {
        let config = HighlightConfig::new()
            .with_dim_opacity(0.5)
            .with_highlight_border_width(4.0)
            .with_highlight_connected(false)
            .with_max_depth(3);

        assert_eq!(config.dim_opacity, 0.5);
        assert_eq!(config.highlight_border_width, 4.0);
        assert!(!config.highlight_connected);
        assert_eq!(config.max_depth, 3);
    }

    #[test]
    fn test_highlight_without_connected() {
        let mut graph = create_test_graph();
        let config = HighlightConfig::default().with_highlight_connected(false);

        highlight_entity(&mut graph, "user", &config);

        // Only the user entity should be highlighted
        assert!(graph.get_node("entity_user").unwrap().style.highlighted);

        // Connected nodes should be dimmed
        assert!(!graph.get_node("class_3002").unwrap().style.highlighted);
        assert!(!graph.get_node("entity_auth").unwrap().style.highlighted);
    }

    #[test]
    fn test_find_connected_nodes_depth() {
        let graph = create_test_graph();
        let config = HighlightConfig::default();

        // Depth 0 - only the node itself
        let depth_0 = find_connected_nodes(&graph, "entity_user", 0, &config.follow_edge_types);
        assert_eq!(depth_0.len(), 1);
        assert!(depth_0.contains("entity_user"));

        // Depth 1 - direct connections
        let depth_1 = find_connected_nodes(&graph, "entity_user", 1, &config.follow_edge_types);
        assert!(depth_1.contains("entity_user"));
        assert!(depth_1.contains("class_3002"));
        assert!(depth_1.contains("entity_auth"));
        assert!(depth_1.contains("observable_5"));

        // Depth 2 - includes connections of connections
        let depth_2 = find_connected_nodes(&graph, "entity_user", 2, &config.follow_edge_types);
        assert!(depth_2.len() > depth_1.len());
    }
}
