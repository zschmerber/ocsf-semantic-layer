//! Export formats for visualization graphs.
//!
//! This module provides functionality to export graphs to various formats:
//! - JSON: Full graph data in JSON format
//! - SVG: Scalable Vector Graphics for web display
//! - PNG: Raster image format (requires resvg feature)

use anyhow::{Context, Result};

use crate::graph::{GraphData, GraphNode, GraphEdge, NodeShape, LineStyle, ArrowHead};

/// Export configuration options.
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Width of the exported image/SVG.
    pub width: f64,
    /// Height of the exported image/SVG.
    pub height: f64,
    /// Padding around the graph.
    pub padding: f64,
    /// Node width.
    pub node_width: f64,
    /// Node height.
    pub node_height: f64,
    /// Font family for labels.
    pub font_family: String,
    /// Font size for labels.
    pub font_size: f64,
    /// Whether to include a legend.
    pub include_legend: bool,
    /// Background color.
    pub background_color: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            padding: 50.0,
            node_width: 120.0,
            node_height: 40.0,
            font_family: "Arial, sans-serif".to_string(),
            font_size: 12.0,
            include_legend: true,
            background_color: "#ffffff".to_string(),
        }
    }
}

impl ExportConfig {
    /// Creates a new export config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the dimensions.
    pub fn with_dimensions(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the padding.
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the node dimensions.
    pub fn with_node_dimensions(mut self, width: f64, height: f64) -> Self {
        self.node_width = width;
        self.node_height = height;
        self
    }

    /// Sets the font family.
    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    /// Sets the font size.
    pub fn with_font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// Sets whether to include a legend.
    pub fn with_legend(mut self, include: bool) -> Self {
        self.include_legend = include;
        self
    }

    /// Sets the background color.
    pub fn with_background_color(mut self, color: impl Into<String>) -> Self {
        self.background_color = color.into();
        self
    }
}

/// Exports the graph to JSON format.
pub fn export_to_json(graph: &GraphData) -> Result<String> {
    graph.to_json().context("Failed to export graph to JSON")
}

/// Exports the graph to JSON format with pretty printing.
pub fn export_to_json_pretty(graph: &GraphData) -> Result<String> {
    serde_json::to_string_pretty(graph).context("Failed to export graph to JSON")
}

/// Exports the graph to SVG format.
pub fn export_to_svg(graph: &GraphData, config: &ExportConfig) -> Result<String> {
    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        config.width, config.height, config.width, config.height
    ));

    // Defs for markers (arrow heads)
    svg.push_str(&generate_svg_defs());

    // Background
    svg.push_str(&format!(
        r#"  <rect width="100%" height="100%" fill="{}"/>
"#,
        config.background_color
    ));

    // Calculate node positions if not set
    let positioned_nodes = calculate_node_positions(graph, config);

    // Draw edges first (so they appear behind nodes)
    for edge in &graph.edges {
        if let (Some(source_pos), Some(target_pos)) = (
            positioned_nodes.get(&edge.source),
            positioned_nodes.get(&edge.target),
        ) {
            svg.push_str(&render_edge_svg(edge, source_pos, target_pos, config));
        }
    }

    // Draw nodes
    for node in &graph.nodes {
        if let Some(pos) = positioned_nodes.get(&node.id) {
            svg.push_str(&render_node_svg(node, pos, config));
        }
    }

    // Legend
    if config.include_legend {
        svg.push_str(&generate_legend_svg(config));
    }

    // SVG footer
    svg.push_str("</svg>\n");

    Ok(svg)
}

/// Generates SVG defs for markers.
fn generate_svg_defs() -> String {
    r##"  <defs>
    <marker id="arrow-normal" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L0,6 L9,3 z" fill="#666666"/>
    </marker>
    <marker id="arrow-diamond" markerWidth="10" markerHeight="10" refX="5" refY="5" orient="auto" markerUnits="strokeWidth">
      <path d="M0,5 L5,0 L10,5 L5,10 z" fill="#666666"/>
    </marker>
    <marker id="arrow-open" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
      <path d="M0,0 L9,3 L0,6" fill="none" stroke="#666666" stroke-width="1"/>
    </marker>
  </defs>
"##.to_string()
}

/// Node position for layout.
#[derive(Debug, Clone, Copy)]
struct NodePosition {
    x: f64,
    y: f64,
}

/// Calculates node positions using a simple grid layout.
fn calculate_node_positions(
    graph: &GraphData,
    config: &ExportConfig,
) -> std::collections::HashMap<String, NodePosition> {
    let mut positions = std::collections::HashMap::new();

    if graph.nodes.is_empty() {
        return positions;
    }

    // Use provided positions if available, otherwise calculate
    let nodes_with_positions: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.x.is_some() && n.y.is_some())
        .collect();

    if nodes_with_positions.len() == graph.nodes.len() {
        // All nodes have positions
        for node in &graph.nodes {
            positions.insert(
                node.id.clone(),
                NodePosition {
                    x: node.x.unwrap(),
                    y: node.y.unwrap(),
                },
            );
        }
    } else {
        // Calculate positions using simple grid layout
        let available_width = config.width - 2.0 * config.padding;
        let available_height = config.height - 2.0 * config.padding;

        let cols = ((graph.nodes.len() as f64).sqrt().ceil() as usize).max(1);
        let rows = graph.nodes.len().div_ceil(cols);

        let col_spacing = available_width / (cols as f64 + 1.0);
        let row_spacing = available_height / (rows as f64 + 1.0);

        for (i, node) in graph.nodes.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;

            positions.insert(
                node.id.clone(),
                NodePosition {
                    x: config.padding + col_spacing * (col as f64 + 1.0),
                    y: config.padding + row_spacing * (row as f64 + 1.0),
                },
            );
        }
    }

    positions
}

/// Renders a node as SVG.
fn render_node_svg(node: &GraphNode, pos: &NodePosition, config: &ExportConfig) -> String {
    let half_width = config.node_width / 2.0;
    let half_height = config.node_height / 2.0;

    let opacity = if node.style.opacity < 1.0 {
        format!(r#" opacity="{}""#, node.style.opacity)
    } else {
        String::new()
    };

    let stroke_width = if node.style.highlighted {
        node.style.border_width * 1.5
    } else {
        node.style.border_width
    };

    let shape_svg = match node.style.shape {
        NodeShape::Rectangle => {
            format!(
                r#"    <rect x="{}" y="{}" width="{}" height="{}" rx="5" ry="5" fill="{}" stroke="{}" stroke-width="{}"{}/>"#,
                pos.x - half_width,
                pos.y - half_height,
                config.node_width,
                config.node_height,
                node.style.color,
                node.style.border_color,
                stroke_width,
                opacity
            )
        }
        NodeShape::Ellipse => {
            format!(
                r#"    <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"{}/>"#,
                pos.x,
                pos.y,
                half_width,
                half_height,
                node.style.color,
                node.style.border_color,
                stroke_width,
                opacity
            )
        }
        NodeShape::Diamond => {
            let points = format!(
                "{},{} {},{} {},{} {},{}",
                pos.x, pos.y - half_height,
                pos.x + half_width, pos.y,
                pos.x, pos.y + half_height,
                pos.x - half_width, pos.y
            );
            format!(
                r#"    <polygon points="{}" fill="{}" stroke="{}" stroke-width="{}"{}/>"#,
                points,
                node.style.color,
                node.style.border_color,
                stroke_width,
                opacity
            )
        }
        NodeShape::Hexagon => {
            let w = half_width;
            let h = half_height;
            let points = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{}",
                pos.x - w * 0.5, pos.y - h,
                pos.x + w * 0.5, pos.y - h,
                pos.x + w, pos.y,
                pos.x + w * 0.5, pos.y + h,
                pos.x - w * 0.5, pos.y + h,
                pos.x - w, pos.y
            );
            format!(
                r#"    <polygon points="{}" fill="{}" stroke="{}" stroke-width="{}"{}/>"#,
                points,
                node.style.color,
                node.style.border_color,
                stroke_width,
                opacity
            )
        }
        NodeShape::Cylinder => {
            // Simplified cylinder as rectangle with rounded top/bottom
            format!(
                r#"    <rect x="{}" y="{}" width="{}" height="{}" rx="10" ry="10" fill="{}" stroke="{}" stroke-width="{}"{}/>"#,
                pos.x - half_width,
                pos.y - half_height,
                config.node_width,
                config.node_height,
                node.style.color,
                node.style.border_color,
                stroke_width,
                opacity
            )
        }
    };

    // Truncate label if too long
    let label = if node.label.len() > 15 {
        format!("{}...", &node.label[..12])
    } else {
        node.label.clone()
    };

    format!(
        r#"{}
    <text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-family="{}" font-size="{}" fill="{}"{}>{}</text>
"#,
        shape_svg,
        pos.x,
        pos.y,
        config.font_family,
        config.font_size,
        node.style.font_color,
        opacity,
        escape_xml(&label)
    )
}

/// Renders an edge as SVG.
fn render_edge_svg(
    edge: &GraphEdge,
    source: &NodePosition,
    target: &NodePosition,
    config: &ExportConfig,
) -> String {
    let opacity = if edge.style.opacity < 1.0 {
        format!(r#" opacity="{}""#, edge.style.opacity)
    } else {
        String::new()
    };

    let stroke_dasharray = match edge.style.line_style {
        LineStyle::Solid => String::new(),
        LineStyle::Dashed => r#" stroke-dasharray="5,5""#.to_string(),
        LineStyle::Dotted => r#" stroke-dasharray="2,2""#.to_string(),
    };

    let marker_end = match edge.style.arrow_head {
        ArrowHead::Normal => r##" marker-end="url(#arrow-normal)""##,
        ArrowHead::Diamond => r##" marker-end="url(#arrow-diamond)""##,
        ArrowHead::Open => r##" marker-end="url(#arrow-open)""##,
        ArrowHead::None => "",
    };

    // Calculate edge endpoints (offset from node centers)
    let dx = target.x - source.x;
    let dy = target.y - source.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 0.001 {
        return String::new();
    }

    let offset = config.node_width / 2.0 + 5.0;
    let start_x = source.x + dx / dist * offset;
    let start_y = source.y + dy / dist * offset;
    let end_x = target.x - dx / dist * offset;
    let end_y = target.y - dy / dist * offset;

    let mut svg = format!(
        r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"{}{}{}/>"#,
        start_x,
        start_y,
        end_x,
        end_y,
        edge.style.color,
        edge.style.width,
        stroke_dasharray,
        marker_end,
        opacity
    );

    // Add label if present
    if let Some(ref label) = edge.label {
        let mid_x = (start_x + end_x) / 2.0;
        let mid_y = (start_y + end_y) / 2.0;
        svg.push_str(&format!(
            r##"
    <text x="{}" y="{}" text-anchor="middle" font-family="{}" font-size="{}" fill="#666666"{}>{}</text>"##,
            mid_x,
            mid_y - 5.0,
            config.font_family,
            config.font_size * 0.8,
            opacity,
            escape_xml(label)
        ));
    }

    svg.push('\n');
    svg
}

/// Generates a legend for the SVG.
fn generate_legend_svg(config: &ExportConfig) -> String {
    let legend_x = config.width - 180.0;
    let legend_y = config.padding;

    format!(
        r##"  <g id="legend">
    <rect x="{}" y="{}" width="160" height="140" fill="#f8f8f8" stroke="#cccccc" rx="5"/>
    <text x="{}" y="{}" font-family="{}" font-size="12" font-weight="bold">Legend</text>
    <rect x="{}" y="{}" width="20" height="15" fill="#4a90d9" stroke="#2c5aa0"/>
    <text x="{}" y="{}" font-family="{}" font-size="10">Semantic Entity</text>
    <polygon points="{},{} {},{} {},{} {},{} {},{} {},{}" fill="#f5a623" stroke="#c78c1c"/>
    <text x="{}" y="{}" font-family="{}" font-size="10">OCSF Category</text>
    <rect x="{}" y="{}" width="20" height="15" fill="#7ed321" stroke="#5ca018"/>
    <text x="{}" y="{}" font-family="{}" font-size="10">OCSF Class</text>
    <polygon points="{},{} {},{} {},{} {},{}" fill="#bd10e0" stroke="#8a0ba5"/>
    <text x="{}" y="{}" font-family="{}" font-size="10">Observable</text>
  </g>
"##,
        legend_x, legend_y,
        legend_x + 10.0, legend_y + 20.0, config.font_family,
        legend_x + 10.0, legend_y + 35.0,
        legend_x + 40.0, legend_y + 47.0, config.font_family,
        // Hexagon for category
        legend_x + 15.0, legend_y + 55.0,
        legend_x + 25.0, legend_y + 55.0,
        legend_x + 30.0, legend_y + 62.0,
        legend_x + 25.0, legend_y + 70.0,
        legend_x + 15.0, legend_y + 70.0,
        legend_x + 10.0, legend_y + 62.0,
        legend_x + 40.0, legend_y + 67.0, config.font_family,
        legend_x + 10.0, legend_y + 80.0,
        legend_x + 40.0, legend_y + 92.0, config.font_family,
        // Diamond for observable
        legend_x + 20.0, legend_y + 100.0,
        legend_x + 30.0, legend_y + 107.0,
        legend_x + 20.0, legend_y + 115.0,
        legend_x + 10.0, legend_y + 107.0,
        legend_x + 40.0, legend_y + 112.0, config.font_family
    )
}

/// Escapes special XML characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Exports the graph to PNG format.
///
/// Note: PNG export is not yet implemented. Returns an error.
pub fn export_to_png(graph: &GraphData, config: &ExportConfig) -> Result<Vec<u8>> {
    let _ = (graph, config);
    anyhow::bail!("PNG export requires the 'resvg' feature to be enabled")
}

/// Saves the graph to a JSON file.
pub fn save_to_json(graph: &GraphData, path: impl AsRef<std::path::Path>) -> Result<()> {
    let json = export_to_json_pretty(graph)?;
    std::fs::write(path.as_ref(), json)
        .with_context(|| format!("Failed to write JSON to {:?}", path.as_ref()))?;
    Ok(())
}

/// Saves the graph to an SVG file.
pub fn save_to_svg(
    graph: &GraphData,
    path: impl AsRef<std::path::Path>,
    config: &ExportConfig,
) -> Result<()> {
    let svg = export_to_svg(graph, config)?;
    std::fs::write(path.as_ref(), svg)
        .with_context(|| format!("Failed to write SVG to {:?}", path.as_ref()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphEdge, GraphNode, NodeType, EdgeType};

    fn create_test_graph() -> GraphData {
        let mut graph = GraphData::new();

        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "User Entity"));
        graph.add_node(GraphNode::new("c1", NodeType::OcsfCategory, "IAM"));
        graph.add_node(GraphNode::new("cl1", NodeType::OcsfClass, "Authentication"));
        graph.add_node(GraphNode::new("o1", NodeType::Observable, "Email"));

        graph.add_edge(GraphEdge::new("e1", "cl1", EdgeType::MapsTo));
        graph.add_edge(GraphEdge::new("c1", "cl1", EdgeType::Contains));
        graph.add_edge(GraphEdge::new("e1", "o1", EdgeType::CoversObservable));

        graph
    }

    #[test]
    fn test_export_to_json() {
        let graph = create_test_graph();
        let json = export_to_json(&graph).unwrap();

        assert!(json.contains("nodes"));
        assert!(json.contains("edges"));
        assert!(json.contains("User Entity"));
    }

    #[test]
    fn test_export_to_json_pretty() {
        let graph = create_test_graph();
        let json = export_to_json_pretty(&graph).unwrap();

        // Pretty JSON should have newlines
        assert!(json.contains('\n'));
        assert!(json.contains("nodes"));
    }

    #[test]
    fn test_export_to_svg() {
        let graph = create_test_graph();
        let config = ExportConfig::default();
        let svg = export_to_svg(&graph, &config).unwrap();

        // Check SVG structure
        assert!(svg.starts_with("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));

        // Check that nodes are rendered
        assert!(svg.contains("User Entity"));
        assert!(svg.contains("IAM"));

        // Check that defs are included
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("marker"));
    }

    #[test]
    fn test_export_config_builder() {
        let config = ExportConfig::new()
            .with_dimensions(1600.0, 1200.0)
            .with_padding(100.0)
            .with_node_dimensions(150.0, 50.0)
            .with_font_family("Helvetica")
            .with_font_size(14.0)
            .with_legend(false)
            .with_background_color("#f0f0f0");

        assert_eq!(config.width, 1600.0);
        assert_eq!(config.height, 1200.0);
        assert_eq!(config.padding, 100.0);
        assert_eq!(config.node_width, 150.0);
        assert_eq!(config.node_height, 50.0);
        assert_eq!(config.font_family, "Helvetica");
        assert_eq!(config.font_size, 14.0);
        assert!(!config.include_legend);
        assert_eq!(config.background_color, "#f0f0f0");
    }

    #[test]
    fn test_svg_with_positioned_nodes() {
        let mut graph = GraphData::new();

        graph.add_node(
            GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1")
                .with_position(100.0, 100.0),
        );
        graph.add_node(
            GraphNode::new("e2", NodeType::SemanticEntity, "Entity 2")
                .with_position(300.0, 100.0),
        );
        graph.add_edge(GraphEdge::new("e1", "e2", EdgeType::RelatesTo));

        let config = ExportConfig::default();
        let svg = export_to_svg(&graph, &config).unwrap();

        assert!(svg.contains("Entity 1"));
        assert!(svg.contains("Entity 2"));
    }

    #[test]
    fn test_svg_with_different_shapes() {
        let mut graph = GraphData::new();

        graph.add_node(GraphNode::new("rect", NodeType::SemanticEntity, "Rectangle"));
        graph.add_node(GraphNode::new("hex", NodeType::OcsfCategory, "Hexagon"));
        graph.add_node(GraphNode::new("diamond", NodeType::Observable, "Diamond"));
        graph.add_node(GraphNode::new("ellipse", NodeType::OcsfAttribute, "Ellipse"));

        let config = ExportConfig::default();
        let svg = export_to_svg(&graph, &config).unwrap();

        // Check that different shapes are rendered
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("<ellipse"));
    }

    #[test]
    fn test_svg_with_edge_labels() {
        let mut graph = GraphData::new();

        graph.add_node(GraphNode::new("e1", NodeType::SemanticEntity, "Entity 1"));
        graph.add_node(GraphNode::new("e2", NodeType::SemanticEntity, "Entity 2"));
        graph.add_edge(
            GraphEdge::new("e1", "e2", EdgeType::RelatesTo).with_label("relates to"),
        );

        let config = ExportConfig::default();
        let svg = export_to_svg(&graph, &config).unwrap();

        assert!(svg.contains("relates to"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
        assert_eq!(escape_xml("a > b"), "a &gt; b");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml(r#"a "b" c"#), "a &quot;b&quot; c");
    }

    #[test]
    fn test_svg_without_legend() {
        let graph = create_test_graph();
        let config = ExportConfig::default().with_legend(false);
        let svg = export_to_svg(&graph, &config).unwrap();

        assert!(!svg.contains(r#"id="legend""#));
    }

    #[test]
    fn test_empty_graph_export() {
        let graph = GraphData::new();
        let config = ExportConfig::default();

        let json = export_to_json(&graph).unwrap();
        assert!(json.contains("nodes"));

        let svg = export_to_svg(&graph, &config).unwrap();
        assert!(svg.contains("<svg"));
    }
}
