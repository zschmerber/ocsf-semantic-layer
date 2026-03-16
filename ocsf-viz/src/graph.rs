//! Graph data structures for visualization.
//!
//! This module defines the core graph data structures used to represent
//! the semantic-to-physical layer interchange visualization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of node in the visualization graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// A semantic entity node.
    SemanticEntity,
    /// An OCSF category node.
    OcsfCategory,
    /// An OCSF event class node.
    OcsfClass,
    /// An OCSF attribute node.
    OcsfAttribute,
    /// An observable node.
    Observable,
    /// A hot path node (observables table).
    HotPath,
    /// A cold path node (full events table).
    ColdPath,
}

impl NodeType {
    /// Returns true if this is a semantic layer node type.
    pub fn is_semantic(&self) -> bool {
        matches!(self, NodeType::SemanticEntity)
    }

    /// Returns true if this is a physical layer node type.
    pub fn is_physical(&self) -> bool {
        matches!(
            self,
            NodeType::OcsfCategory | NodeType::OcsfClass | NodeType::OcsfAttribute
        )
    }

    /// Returns true if this is an observable-related node type.
    pub fn is_observable(&self) -> bool {
        matches!(self, NodeType::Observable)
    }

    /// Returns true if this is a path-related node type.
    pub fn is_path(&self) -> bool {
        matches!(self, NodeType::HotPath | NodeType::ColdPath)
    }
}

/// Type of edge in the visualization graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// Semantic entity maps to OCSF element.
    MapsTo,
    /// Container relationship (category contains class, class contains attribute).
    Contains,
    /// Relationship between semantic entities.
    RelatesTo,
    /// Semantic entity covers an observable.
    CoversObservable,
    /// Reverse lookup from observable to full event.
    ReverseLookup,
    /// Data flow edge.
    DataFlow,
}

/// Shape of a node in the visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum NodeShape {
    /// Rectangle shape.
    #[default]
    Rectangle,
    /// Ellipse/oval shape.
    Ellipse,
    /// Diamond shape.
    Diamond,
    /// Hexagon shape.
    Hexagon,
    /// Cylinder shape (for data stores).
    Cylinder,
}


/// Line style for edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LineStyle {
    /// Solid line.
    #[default]
    Solid,
    /// Dashed line.
    Dashed,
    /// Dotted line.
    Dotted,
}


/// Arrow head style for edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ArrowHead {
    /// Normal arrow head.
    #[default]
    Normal,
    /// No arrow head.
    None,
    /// Diamond arrow head.
    Diamond,
    /// Open arrow head.
    Open,
}


/// Visual style for a node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeStyle {
    /// Shape of the node.
    #[serde(default)]
    pub shape: NodeShape,
    /// Fill color (CSS color string).
    #[serde(default = "default_fill_color")]
    pub color: String,
    /// Border color (CSS color string).
    #[serde(default = "default_border_color")]
    pub border_color: String,
    /// Border width in pixels.
    #[serde(default = "default_border_width")]
    pub border_width: f32,
    /// Optional icon identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Font size for label.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Font color.
    #[serde(default = "default_font_color")]
    pub font_color: String,
    /// Whether the node is highlighted.
    #[serde(default)]
    pub highlighted: bool,
    /// Opacity (0.0 to 1.0).
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_fill_color() -> String {
    "#ffffff".to_string()
}

fn default_border_color() -> String {
    "#333333".to_string()
}

fn default_border_width() -> f32 {
    1.0
}

fn default_font_size() -> f32 {
    12.0
}

fn default_font_color() -> String {
    "#000000".to_string()
}

fn default_opacity() -> f32 {
    1.0
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            shape: NodeShape::default(),
            color: default_fill_color(),
            border_color: default_border_color(),
            border_width: default_border_width(),
            icon: None,
            font_size: default_font_size(),
            font_color: default_font_color(),
            highlighted: false,
            opacity: default_opacity(),
        }
    }
}

impl NodeStyle {
    /// Creates a new node style with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the shape.
    pub fn with_shape(mut self, shape: NodeShape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the fill color.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Sets the border color.
    pub fn with_border_color(mut self, color: impl Into<String>) -> Self {
        self.border_color = color.into();
        self
    }

    /// Sets the border width.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the highlighted state.
    pub fn with_highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    /// Sets the opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Returns the default style for a node type.
    pub fn for_node_type(node_type: NodeType) -> Self {
        match node_type {
            NodeType::SemanticEntity => Self::new()
                .with_shape(NodeShape::Rectangle)
                .with_color("#4a90d9")
                .with_border_color("#2c5aa0"),
            NodeType::OcsfCategory => Self::new()
                .with_shape(NodeShape::Hexagon)
                .with_color("#f5a623")
                .with_border_color("#c78c1c"),
            NodeType::OcsfClass => Self::new()
                .with_shape(NodeShape::Rectangle)
                .with_color("#7ed321")
                .with_border_color("#5ca018"),
            NodeType::OcsfAttribute => Self::new()
                .with_shape(NodeShape::Ellipse)
                .with_color("#9b9b9b")
                .with_border_color("#6b6b6b"),
            NodeType::Observable => Self::new()
                .with_shape(NodeShape::Diamond)
                .with_color("#bd10e0")
                .with_border_color("#8a0ba5"),
            NodeType::HotPath => Self::new()
                .with_shape(NodeShape::Cylinder)
                .with_color("#ff6b6b")
                .with_border_color("#cc5555"),
            NodeType::ColdPath => Self::new()
                .with_shape(NodeShape::Cylinder)
                .with_color("#4ecdc4")
                .with_border_color("#3ba99e"),
        }
    }
}

/// Visual style for an edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeStyle {
    /// Line style.
    #[serde(default)]
    pub line_style: LineStyle,
    /// Line color (CSS color string).
    #[serde(default = "default_edge_color")]
    pub color: String,
    /// Line width in pixels.
    #[serde(default = "default_edge_width")]
    pub width: f32,
    /// Arrow head style.
    #[serde(default)]
    pub arrow_head: ArrowHead,
    /// Whether the edge is highlighted.
    #[serde(default)]
    pub highlighted: bool,
    /// Opacity (0.0 to 1.0).
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_edge_color() -> String {
    "#666666".to_string()
}

fn default_edge_width() -> f32 {
    1.0
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            line_style: LineStyle::default(),
            color: default_edge_color(),
            width: default_edge_width(),
            arrow_head: ArrowHead::default(),
            highlighted: false,
            opacity: default_opacity(),
        }
    }
}

impl EdgeStyle {
    /// Creates a new edge style with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the line style.
    pub fn with_line_style(mut self, style: LineStyle) -> Self {
        self.line_style = style;
        self
    }

    /// Sets the color.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Sets the width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the arrow head style.
    pub fn with_arrow_head(mut self, arrow_head: ArrowHead) -> Self {
        self.arrow_head = arrow_head;
        self
    }

    /// Sets the highlighted state.
    pub fn with_highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    /// Sets the opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Returns the default style for an edge type.
    pub fn for_edge_type(edge_type: EdgeType) -> Self {
        match edge_type {
            EdgeType::MapsTo => Self::new()
                .with_color("#4a90d9")
                .with_arrow_head(ArrowHead::Normal),
            EdgeType::Contains => Self::new()
                .with_color("#7ed321")
                .with_line_style(LineStyle::Dashed)
                .with_arrow_head(ArrowHead::Diamond),
            EdgeType::RelatesTo => Self::new()
                .with_color("#9b59b6")
                .with_line_style(LineStyle::Dotted)
                .with_arrow_head(ArrowHead::Open),
            EdgeType::CoversObservable => Self::new()
                .with_color("#bd10e0")
                .with_arrow_head(ArrowHead::Normal),
            EdgeType::ReverseLookup => Self::new()
                .with_color("#e74c3c")
                .with_line_style(LineStyle::Dashed)
                .with_arrow_head(ArrowHead::Normal),
            EdgeType::DataFlow => Self::new()
                .with_color("#3498db")
                .with_width(2.0)
                .with_arrow_head(ArrowHead::Normal),
        }
    }
}


/// A node in the visualization graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node.
    pub id: String,
    /// Type of the node.
    #[serde(rename = "type")]
    pub node_type: NodeType,
    /// Display label for the node.
    pub label: String,
    /// Additional properties for the node.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, serde_json::Value>,
    /// Visual style for the node.
    #[serde(default)]
    pub style: NodeStyle,
    /// X position (optional, for layout).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position (optional, for layout).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

impl GraphNode {
    /// Creates a new graph node.
    pub fn new(id: impl Into<String>, node_type: NodeType, label: impl Into<String>) -> Self {
        let node_type_val = node_type;
        Self {
            id: id.into(),
            node_type: node_type_val,
            label: label.into(),
            properties: HashMap::new(),
            style: NodeStyle::for_node_type(node_type_val),
            x: None,
            y: None,
        }
    }

    /// Sets a property on the node.
    pub fn with_property(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Sets multiple properties on the node.
    pub fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties.extend(properties);
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: NodeStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the position.
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Gets a property value.
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Returns true if this node should be visible in the given view mode.
    pub fn is_visible_in_mode(&self, mode: ViewMode) -> bool {
        match mode {
            ViewMode::SemanticOnly => self.node_type.is_semantic(),
            ViewMode::PhysicalOnly => self.node_type.is_physical(),
            ViewMode::Interchange => true,
            ViewMode::HotColdPath => self.node_type.is_path() || self.node_type.is_observable(),
        }
    }
}

/// An edge in the visualization graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Type of the edge.
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    /// Optional label for the edge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Visual style for the edge.
    #[serde(default)]
    pub style: EdgeStyle,
}

impl GraphEdge {
    /// Creates a new graph edge.
    pub fn new(source: impl Into<String>, target: impl Into<String>, edge_type: EdgeType) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            edge_type,
            label: None,
            style: EdgeStyle::for_edge_type(edge_type),
        }
    }

    /// Sets the label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the style.
    pub fn with_style(mut self, style: EdgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Returns true if this edge connects semantic and physical nodes.
    pub fn is_interchange_edge(&self) -> bool {
        matches!(self.edge_type, EdgeType::MapsTo | EdgeType::CoversObservable)
    }
}

/// View mode for the visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ViewMode {
    /// Show only semantic layer elements.
    SemanticOnly,
    /// Show only physical layer elements.
    PhysicalOnly,
    /// Show both layers with interchange connections.
    #[default]
    Interchange,
    /// Show hot/cold path data flow.
    HotColdPath,
}


/// Layout algorithm for the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LayoutType {
    /// Hierarchical layout (top-to-bottom or left-to-right).
    #[default]
    Hierarchical,
    /// Force-directed layout.
    Force,
    /// Radial layout.
    Radial,
}


/// Metadata about the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// View mode used to generate the graph.
    pub view_mode: ViewMode,
    /// Timestamp when the graph was generated.
    pub generated_at: String,
    /// OCSF schema version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    /// Semantic model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// Layout type used.
    #[serde(default)]
    pub layout: LayoutType,
    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::default(),
            generated_at: current_timestamp(),
            schema_version: None,
            model_version: None,
            layout: LayoutType::default(),
            extra: HashMap::new(),
        }
    }
}

impl GraphMetadata {
    /// Creates new metadata with the given view mode.
    pub fn new(view_mode: ViewMode) -> Self {
        Self {
            view_mode,
            generated_at: current_timestamp(),
            ..Default::default()
        }
    }

    /// Sets the schema version.
    pub fn with_schema_version(mut self, version: impl Into<String>) -> Self {
        self.schema_version = Some(version.into());
        self
    }

    /// Sets the model version.
    pub fn with_model_version(mut self, version: impl Into<String>) -> Self {
        self.model_version = Some(version.into());
        self
    }

    /// Sets the layout type.
    pub fn with_layout(mut self, layout: LayoutType) -> Self {
        self.layout = layout;
        self
    }
}

/// The complete graph data structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphData {
    /// Nodes in the graph.
    pub nodes: Vec<GraphNode>,
    /// Edges in the graph.
    pub edges: Vec<GraphEdge>,
    /// Metadata about the graph.
    pub metadata: GraphMetadata,
}

impl Default for GraphData {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphData {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::default(),
        }
    }

    /// Creates a new graph with the given view mode.
    pub fn with_view_mode(view_mode: ViewMode) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::new(view_mode),
        }
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Gets a node by ID.
    pub fn get_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Gets a mutable node by ID.
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Returns all nodes of a given type.
    pub fn nodes_of_type(&self, node_type: NodeType) -> impl Iterator<Item = &GraphNode> {
        self.nodes.iter().filter(move |n| n.node_type == node_type)
    }

    /// Returns all edges of a given type.
    pub fn edges_of_type(&self, edge_type: EdgeType) -> impl Iterator<Item = &GraphEdge> {
        self.edges.iter().filter(move |e| e.edge_type == edge_type)
    }

    /// Returns all edges connected to a node.
    pub fn edges_for_node<'a>(&'a self, node_id: &'a str) -> impl Iterator<Item = &'a GraphEdge> {
        self.edges
            .iter()
            .filter(move |e| e.source == node_id || e.target == node_id)
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns true if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Filters the graph to only include nodes visible in the given view mode.
    pub fn filter_by_view_mode(&self, mode: ViewMode) -> Self {
        let visible_nodes: Vec<GraphNode> = self
            .nodes
            .iter()
            .filter(|n| n.is_visible_in_mode(mode))
            .cloned()
            .collect();

        let visible_node_ids: std::collections::HashSet<&str> =
            visible_nodes.iter().map(|n| n.id.as_str()).collect();

        let visible_edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .filter(|e| {
                visible_node_ids.contains(e.source.as_str())
                    && visible_node_ids.contains(e.target.as_str())
            })
            .cloned()
            .collect();

        Self {
            nodes: visible_nodes,
            edges: visible_edges,
            metadata: GraphMetadata::new(mode)
                .with_layout(self.metadata.layout),
        }
    }

    /// Highlights a node and all connected nodes/edges.
    pub fn highlight_node(&mut self, node_id: &str) {
        // First, collect the IDs of connected nodes
        let connected_ids: std::collections::HashSet<String> = self
            .edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .flat_map(|e| vec![e.source.clone(), e.target.clone()])
            .collect();

        // Highlight the main node and connected nodes
        for node in &mut self.nodes {
            node.style.highlighted = node.id == node_id || connected_ids.contains(&node.id);
            if !node.style.highlighted {
                node.style.opacity = 0.3;
            }
        }

        // Highlight connected edges
        for edge in &mut self.edges {
            edge.style.highlighted = edge.source == node_id || edge.target == node_id;
            if !edge.style.highlighted {
                edge.style.opacity = 0.3;
            }
        }
    }

    /// Clears all highlighting.
    pub fn clear_highlighting(&mut self) {
        for node in &mut self.nodes {
            node.style.highlighted = false;
            node.style.opacity = 1.0;
        }
        for edge in &mut self.edges {
            edge.style.highlighted = false;
            edge.style.opacity = 1.0;
        }
    }

    /// Serializes the graph to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a graph from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Returns the current timestamp as an ISO 8601 string.
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_classification() {
        assert!(NodeType::SemanticEntity.is_semantic());
        assert!(!NodeType::SemanticEntity.is_physical());

        assert!(NodeType::OcsfCategory.is_physical());
        assert!(NodeType::OcsfClass.is_physical());
        assert!(NodeType::OcsfAttribute.is_physical());
        assert!(!NodeType::OcsfCategory.is_semantic());

        assert!(NodeType::Observable.is_observable());
        assert!(NodeType::HotPath.is_path());
        assert!(NodeType::ColdPath.is_path());
    }

    #[test]
    fn test_node_style_for_type() {
        let semantic_style = NodeStyle::for_node_type(NodeType::SemanticEntity);
        assert_eq!(semantic_style.shape, NodeShape::Rectangle);
        assert_eq!(semantic_style.color, "#4a90d9");

        let category_style = NodeStyle::for_node_type(NodeType::OcsfCategory);
        assert_eq!(category_style.shape, NodeShape::Hexagon);
    }

    #[test]
    fn test_edge_style_for_type() {
        let maps_to_style = EdgeStyle::for_edge_type(EdgeType::MapsTo);
        assert_eq!(maps_to_style.arrow_head, ArrowHead::Normal);

        let contains_style = EdgeStyle::for_edge_type(EdgeType::Contains);
        assert_eq!(contains_style.line_style, LineStyle::Dashed);
    }

    #[test]
    fn test_graph_node_creation() {
        let node = GraphNode::new("entity_1", NodeType::SemanticEntity, "User Entity")
            .with_property("description", "A user entity")
            .with_position(100.0, 200.0);

        assert_eq!(node.id, "entity_1");
        assert_eq!(node.node_type, NodeType::SemanticEntity);
        assert_eq!(node.label, "User Entity");
        assert!(node.get_property("description").is_some());
        assert_eq!(node.x, Some(100.0));
        assert_eq!(node.y, Some(200.0));
    }

    #[test]
    fn test_graph_edge_creation() {
        let edge = GraphEdge::new("entity_1", "class_3002", EdgeType::MapsTo)
            .with_label("maps to");

        assert_eq!(edge.source, "entity_1");
        assert_eq!(edge.target, "class_3002");
        assert_eq!(edge.edge_type, EdgeType::MapsTo);
        assert_eq!(edge.label, Some("maps to".to_string()));
    }

    #[test]
    fn test_graph_data_operations() {
        let mut graph = GraphData::with_view_mode(ViewMode::Interchange);

        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1"));
        graph.add_node(GraphNode::new("c1", NodeType::OcsfClass, "Class 1"));
        graph.add_edge(GraphEdge::new("e1", "c1", EdgeType::MapsTo));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.get_node("e1").is_some());
        assert!(graph.get_node("nonexistent").is_none());

        let semantic_nodes: Vec<_> = graph.nodes_of_type(NodeType::SemanticEntity).collect();
        assert_eq!(semantic_nodes.len(), 1);

        let edges_for_e1: Vec<_> = graph.edges_for_node("e1").collect();
        assert_eq!(edges_for_e1.len(), 1);
    }

    #[test]
    fn test_view_mode_filtering() {
        let mut graph = GraphData::new();

        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1"));
        graph.add_node(GraphNode::new("c1", NodeType::OcsfCategory, "Category 1"));
        graph.add_node(GraphNode::new("cl1", NodeType::OcsfClass, "Class 1"));
        graph.add_edge(GraphEdge::new("e1", "cl1", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("c1", "cl1", EdgeType::Contains));

        // Semantic only
        let semantic_graph = graph.filter_by_view_mode(ViewMode::SemanticOnly);
        assert_eq!(semantic_graph.node_count(), 1);
        assert_eq!(semantic_graph.edge_count(), 0);

        // Physical only
        let physical_graph = graph.filter_by_view_mode(ViewMode::PhysicalOnly);
        assert_eq!(physical_graph.node_count(), 2);
        assert_eq!(physical_graph.edge_count(), 1);

        // Interchange
        let interchange_graph = graph.filter_by_view_mode(ViewMode::Interchange);
        assert_eq!(interchange_graph.node_count(), 3);
        assert_eq!(interchange_graph.edge_count(), 2);
    }

    #[test]
    fn test_node_highlighting() {
        let mut graph = GraphData::new();

        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1"));
        graph.add_node(GraphNode::new("e2", NodeType::SemanticEntity, "Entity 2"));
        graph.add_node(GraphNode::new("c1", NodeType::OcsfClass, "Class 1"));
        graph.add_edge(GraphEdge::new("e1", "c1", EdgeType::MapsTo));

        graph.highlight_node("e1");

        // e1 and c1 should be highlighted (connected)
        assert!(graph.get_node("e1").unwrap().style.highlighted);
        assert!(graph.get_node("c1").unwrap().style.highlighted);
        // e2 should not be highlighted
        assert!(!graph.get_node("e2").unwrap().style.highlighted);
        assert!(graph.get_node("e2").unwrap().style.opacity < 1.0);

        graph.clear_highlighting();
        assert!(!graph.get_node("e1").unwrap().style.highlighted);
        assert_eq!(graph.get_node("e2").unwrap().style.opacity, 1.0);
    }

    #[test]
    fn test_graph_json_roundtrip() {
        let mut graph = GraphData::with_view_mode(ViewMode::Interchange);
        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1"));
        graph.add_edge(GraphEdge::new("e1", "c1", EdgeType::MapsTo));

        let json = graph.to_json().unwrap();
        let restored = GraphData::from_json(&json).unwrap();

        assert_eq!(graph.node_count(), restored.node_count());
        assert_eq!(graph.edge_count(), restored.edge_count());
        assert_eq!(graph.metadata.view_mode, restored.metadata.view_mode);
    }

    #[test]
    fn test_node_visibility_in_mode() {
        let semantic_node = GraphNode::new("e1", NodeType::SemanticEntity, "Entity");
        let physical_node = GraphNode::new("c1", NodeType::OcsfClass, "Class");
        let observable_node = GraphNode::new("o1", NodeType::Observable, "Observable");
        let hot_path_node = GraphNode::new("h1", NodeType::HotPath, "Hot Path");

        // Semantic only mode
        assert!(semantic_node.is_visible_in_mode(ViewMode::SemanticOnly));
        assert!(!physical_node.is_visible_in_mode(ViewMode::SemanticOnly));
        assert!(!observable_node.is_visible_in_mode(ViewMode::SemanticOnly));

        // Physical only mode
        assert!(!semantic_node.is_visible_in_mode(ViewMode::PhysicalOnly));
        assert!(physical_node.is_visible_in_mode(ViewMode::PhysicalOnly));

        // Interchange mode
        assert!(semantic_node.is_visible_in_mode(ViewMode::Interchange));
        assert!(physical_node.is_visible_in_mode(ViewMode::Interchange));
        assert!(observable_node.is_visible_in_mode(ViewMode::Interchange));

        // Hot/cold path mode
        assert!(!semantic_node.is_visible_in_mode(ViewMode::HotColdPath));
        assert!(observable_node.is_visible_in_mode(ViewMode::HotColdPath));
        assert!(hot_path_node.is_visible_in_mode(ViewMode::HotColdPath));
    }
}
