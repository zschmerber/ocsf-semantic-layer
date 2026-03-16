//! Property-based tests for view mode filtering.
//!
//! **Feature: ocsf-semantic-layer, Property 6: View Mode Filtering**
//! **Validates: Requirements 4.4**
//!
//! For any visualization graph and view mode (semantic_only, physical_only, interchange),
//! the filtered graph should contain only nodes appropriate for that mode.

use proptest::collection::vec;
use proptest::prelude::*;

use crate::filter::{filter_graph_by_mode, is_node_type_visible_in_mode};
use crate::graph::{EdgeType, GraphData, GraphEdge, GraphNode, NodeType, ViewMode};

// ============================================================================
// Generators
// ============================================================================

/// Generate a valid identifier string.
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,19}".prop_map(|s| s.to_string())
}

/// Generate a NodeType.
fn node_type_strategy() -> impl Strategy<Value = NodeType> {
    prop_oneof![
        Just(NodeType::SemanticEntity),
        Just(NodeType::OcsfCategory),
        Just(NodeType::OcsfClass),
        Just(NodeType::OcsfAttribute),
        Just(NodeType::Observable),
        Just(NodeType::HotPath),
        Just(NodeType::ColdPath),
    ]
}

/// Generate a ViewMode.
fn view_mode_strategy() -> impl Strategy<Value = ViewMode> {
    prop_oneof![
        Just(ViewMode::SemanticOnly),
        Just(ViewMode::PhysicalOnly),
        Just(ViewMode::Interchange),
        Just(ViewMode::HotColdPath),
    ]
}

/// Generate an EdgeType.
fn edge_type_strategy() -> impl Strategy<Value = EdgeType> {
    prop_oneof![
        Just(EdgeType::MapsTo),
        Just(EdgeType::Contains),
        Just(EdgeType::RelatesTo),
        Just(EdgeType::CoversObservable),
        Just(EdgeType::ReverseLookup),
        Just(EdgeType::DataFlow),
    ]
}

/// Generate a GraphNode with a specific type.
fn graph_node_strategy() -> impl Strategy<Value = GraphNode> {
    (identifier_strategy(), node_type_strategy(), identifier_strategy()).prop_map(
        |(id, node_type, label)| GraphNode::new(format!("{}_{}", id, rand_suffix()), node_type, label),
    )
}

/// Generate a random suffix to ensure unique IDs.
fn rand_suffix() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

/// Generate a graph with random nodes and edges.
fn graph_strategy() -> impl Strategy<Value = GraphData> {
    vec(graph_node_strategy(), 3..15).prop_flat_map(|nodes| {
        let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
        let node_count = node_ids.len();

        // Generate edges between existing nodes
        let edge_count = (node_count / 2).max(1);
        vec(
            (
                proptest::sample::select(node_ids.clone()),
                proptest::sample::select(node_ids.clone()),
                edge_type_strategy(),
            ),
            0..edge_count,
        )
        .prop_map(move |edge_specs| {
            let mut graph = GraphData::new();

            for node in &nodes {
                graph.add_node(node.clone());
            }

            for (source, target, edge_type) in edge_specs {
                if source != target {
                    graph.add_edge(GraphEdge::new(&source, &target, edge_type));
                }
            }

            graph
        })
    })
}

/// Generate a graph with specific node types for testing.
fn mixed_graph_strategy() -> impl Strategy<Value = GraphData> {
    (
        vec(identifier_strategy(), 1..4), // semantic entity ids
        vec(identifier_strategy(), 1..4), // category ids
        vec(identifier_strategy(), 1..4), // class ids
        vec(identifier_strategy(), 0..3), // observable ids
        vec(identifier_strategy(), 0..2), // path ids
    )
        .prop_map(|(semantic_ids, category_ids, class_ids, observable_ids, path_ids)| {
            let mut graph = GraphData::new();

            // Add semantic nodes
            for id in &semantic_ids {
                graph.add_node(GraphNode::new(
                    format!("entity_{}", id),
                    NodeType::SemanticEntity,
                    format!("Entity {}", id),
                ));
            }

            // Add category nodes
            for id in &category_ids {
                graph.add_node(GraphNode::new(
                    format!("category_{}", id),
                    NodeType::OcsfCategory,
                    format!("Category {}", id),
                ));
            }

            // Add class nodes
            for id in &class_ids {
                graph.add_node(GraphNode::new(
                    format!("class_{}", id),
                    NodeType::OcsfClass,
                    format!("Class {}", id),
                ));
            }

            // Add observable nodes
            for id in &observable_ids {
                graph.add_node(GraphNode::new(
                    format!("observable_{}", id),
                    NodeType::Observable,
                    format!("Observable {}", id),
                ));
            }

            // Add path nodes
            for (i, id) in path_ids.iter().enumerate() {
                let node_type = if i % 2 == 0 {
                    NodeType::HotPath
                } else {
                    NodeType::ColdPath
                };
                graph.add_node(GraphNode::new(
                    format!("path_{}", id),
                    node_type,
                    format!("Path {}", id),
                ));
            }

            // Add some edges
            if !semantic_ids.is_empty() && !class_ids.is_empty() {
                graph.add_edge(GraphEdge::new(
                    format!("entity_{}", &semantic_ids[0]),
                    format!("class_{}", &class_ids[0]),
                    EdgeType::MapsTo,
                ));
            }

            if !category_ids.is_empty() && !class_ids.is_empty() {
                graph.add_edge(GraphEdge::new(
                    format!("category_{}", &category_ids[0]),
                    format!("class_{}", &class_ids[0]),
                    EdgeType::Contains,
                ));
            }

            if semantic_ids.len() >= 2 {
                graph.add_edge(GraphEdge::new(
                    format!("entity_{}", &semantic_ids[0]),
                    format!("entity_{}", &semantic_ids[1]),
                    EdgeType::RelatesTo,
                ));
            }

            graph
        })
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: ocsf-semantic-layer, Property 6: View Mode Filtering**
    ///
    /// For any graph and SemanticOnly view mode, the filtered graph should
    /// contain only semantic entity nodes.
    #[test]
    fn semantic_only_contains_only_semantic_nodes(graph in mixed_graph_strategy()) {
        let filtered = filter_graph_by_mode(&graph, ViewMode::SemanticOnly);

        // All nodes in filtered graph should be semantic
        for node in &filtered.nodes {
            prop_assert!(
                node.node_type.is_semantic(),
                "SemanticOnly mode should only contain semantic nodes, found {:?}",
                node.node_type
            );
        }
    }

    /// **Feature: ocsf-semantic-layer, Property 6: View Mode Filtering**
    ///
    /// For any graph and PhysicalOnly view mode, the filtered graph should
    /// contain only physical layer nodes (categories, classes, attributes).
    #[test]
    fn physical_only_contains_only_physical_nodes(graph in mixed_graph_strategy()) {
        let filtered = filter_graph_by_mode(&graph, ViewMode::PhysicalOnly);

        // All nodes in filtered graph should be physical
        for node in &filtered.nodes {
            prop_assert!(
                node.node_type.is_physical(),
                "PhysicalOnly mode should only contain physical nodes, found {:?}",
                node.node_type
            );
        }
    }

    /// **Feature: ocsf-semantic-layer, Property 6: View Mode Filtering**
    ///
    /// For any graph and Interchange view mode, the filtered graph should
    /// contain all nodes from the original graph.
    #[test]
    fn interchange_contains_all_nodes(graph in mixed_graph_strategy()) {
        let filtered = filter_graph_by_mode(&graph, ViewMode::Interchange);

        // Interchange mode should preserve all nodes
        prop_assert_eq!(
            filtered.node_count(),
            graph.node_count(),
            "Interchange mode should contain all nodes"
        );
    }

    /// **Feature: ocsf-semantic-layer, Property 6: View Mode Filtering**
    ///
    /// For any graph and HotColdPath view mode, the filtered graph should
    /// contain only path and observable nodes.
    #[test]
    fn hot_cold_path_contains_only_path_and_observable_nodes(graph in mixed_graph_strategy()) {
        let filtered = filter_graph_by_mode(&graph, ViewMode::HotColdPath);

        // All nodes should be path or observable types
        for node in &filtered.nodes {
            prop_assert!(
                node.node_type.is_path() || node.node_type.is_observable(),
                "HotColdPath mode should only contain path or observable nodes, found {:?}",
                node.node_type
            );
        }
    }

    /// For any graph and view mode, filtered edges should only connect
    /// nodes that exist in the filtered graph.
    #[test]
    fn filtered_edges_connect_existing_nodes(
        graph in mixed_graph_strategy(),
        mode in view_mode_strategy()
    ) {
        let filtered = filter_graph_by_mode(&graph, mode);

        let node_ids: std::collections::HashSet<&str> =
            filtered.nodes.iter().map(|n| n.id.as_str()).collect();

        for edge in &filtered.edges {
            prop_assert!(
                node_ids.contains(edge.source.as_str()),
                "Edge source '{}' should exist in filtered graph",
                edge.source
            );
            prop_assert!(
                node_ids.contains(edge.target.as_str()),
                "Edge target '{}' should exist in filtered graph",
                edge.target
            );
        }
    }

    /// For any node type and view mode, is_node_type_visible_in_mode should
    /// correctly predict whether the node will be included.
    #[test]
    fn visibility_predicate_matches_filtering(
        node_type in node_type_strategy(),
        mode in view_mode_strategy()
    ) {
        let predicted_visible = is_node_type_visible_in_mode(node_type, mode);

        // Create a graph with just this node type
        let mut graph = GraphData::new();
        graph.add_node(GraphNode::new("test_node", node_type, "Test"));

        let filtered = filter_graph_by_mode(&graph, mode);
        let actually_visible = !filtered.nodes.is_empty();

        prop_assert_eq!(
            predicted_visible,
            actually_visible,
            "is_node_type_visible_in_mode({:?}, {:?}) = {} but filtering gave {}",
            node_type,
            mode,
            predicted_visible,
            actually_visible
        );
    }

    /// Filtering should be idempotent - filtering twice should give same result.
    #[test]
    fn filtering_is_idempotent(
        graph in mixed_graph_strategy(),
        mode in view_mode_strategy()
    ) {
        let filtered_once = filter_graph_by_mode(&graph, mode);
        let filtered_twice = filter_graph_by_mode(&filtered_once, mode);

        prop_assert_eq!(
            filtered_once.node_count(),
            filtered_twice.node_count(),
            "Filtering twice should give same node count"
        );
        prop_assert_eq!(
            filtered_once.edge_count(),
            filtered_twice.edge_count(),
            "Filtering twice should give same edge count"
        );
    }

    /// SemanticOnly filtered graph should preserve all semantic nodes from original.
    #[test]
    fn semantic_only_preserves_all_semantic_nodes(graph in mixed_graph_strategy()) {
        let filtered = filter_graph_by_mode(&graph, ViewMode::SemanticOnly);

        let original_semantic_count = graph.nodes_of_type(NodeType::SemanticEntity).count();
        let filtered_semantic_count = filtered.nodes_of_type(NodeType::SemanticEntity).count();

        prop_assert_eq!(
            original_semantic_count,
            filtered_semantic_count,
            "SemanticOnly should preserve all semantic nodes"
        );
    }

    /// PhysicalOnly filtered graph should preserve all physical nodes from original.
    #[test]
    fn physical_only_preserves_all_physical_nodes(graph in mixed_graph_strategy()) {
        let filtered = filter_graph_by_mode(&graph, ViewMode::PhysicalOnly);

        let original_physical_count = graph
            .nodes
            .iter()
            .filter(|n| n.node_type.is_physical())
            .count();
        let filtered_physical_count = filtered
            .nodes
            .iter()
            .filter(|n| n.node_type.is_physical())
            .count();

        prop_assert_eq!(
            original_physical_count,
            filtered_physical_count,
            "PhysicalOnly should preserve all physical nodes"
        );
    }

    /// Filtered graph metadata should reflect the view mode used.
    #[test]
    fn filtered_metadata_reflects_view_mode(
        graph in mixed_graph_strategy(),
        mode in view_mode_strategy()
    ) {
        let filtered = filter_graph_by_mode(&graph, mode);

        prop_assert_eq!(
            filtered.metadata.view_mode,
            mode,
            "Filtered graph metadata should reflect the view mode"
        );
    }
}
