//! Visualization command.
//!
//! This command generates visualizations of the semantic layer interchange.

use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::error::CliError;
use crate::OutputFormat;

/// Runs the visualize command.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    model_path: PathBuf,
    format: OutputFormat,
    output_path: PathBuf,
    view_mode: ocsf_viz::ViewMode,
    schema_path: Option<PathBuf>,
    show_observables: bool,
    show_relationships: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    use ocsf_core::SchemaIngester;
    use ocsf_semantic::SemanticModel;
    use ocsf_viz::{export_to_json, export_to_svg, generate_graph, GeneratorOptions, ExportConfig};

    // Load the semantic model
    if !model_path.exists() {
        return Err(CliError::file_not_found(&model_path).into());
    }

    if verbose {
        eprintln!("Loading semantic model from: {}", model_path.display());
    }

    let model = SemanticModel::load(&model_path)
        .map_err(|e| CliError::invalid_file_format(
            &model_path,
            format!("Invalid semantic model YAML: {}", e)
        ))?;

    // Load the OCSF schema
    let mut ingester = SchemaIngester::new();

    if let Some(schema_dir) = schema_path {
        if !schema_dir.exists() {
            return Err(CliError::file_not_found(&schema_dir).into());
        }
        if verbose {
            eprintln!("Loading schema from: {}", schema_dir.display());
        }
        ingester.load_from_local(&schema_dir)
            .map_err(|e| CliError::schema_ingestion_with_source(
                format!("Failed to load schema from: {}", schema_dir.display()),
                e
            ))?;
    } else {
        let cache_path = config.schema_cache_path(&model.ocsf_version);
        if cache_path.exists() {
            if verbose {
                eprintln!("Loading cached schema from: {}", cache_path.display());
            }
            ingester.load_from_local(&cache_path)
                .map_err(|e| CliError::schema_ingestion_with_source(
                    "Failed to load cached schema",
                    e
                ))?;
        } else {
            if verbose {
                eprintln!("Fetching schema version {} from GitHub", model.ocsf_version);
            }
            ingester.load_from_github(Some(&model.ocsf_version)).await
                .map_err(|e| CliError::network(format!(
                    "Failed to fetch schema from GitHub: {}",
                    e
                )))?;
        }
    }

    let schema = ingester.schema();

    // Generate the graph
    if verbose {
        eprintln!("Generating visualization graph...");
        eprintln!("  View mode: {:?}", view_mode);
        eprintln!("  Show observables: {}", show_observables);
        eprintln!("  Show relationships: {}", show_relationships);
    }

    let options = GeneratorOptions::with_view_mode(view_mode)
        .show_observables(show_observables)
        .show_relationships(show_relationships);

    let graph = generate_graph(&model, schema, options);

    if verbose {
        eprintln!("Graph generated:");
        eprintln!("  Nodes: {}", graph.nodes.len());
        eprintln!("  Edges: {}", graph.edges.len());
    }

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Export in the requested format
    let export_config = ExportConfig::default();
    
    match format {
        OutputFormat::Svg => {
            let svg = export_to_svg(&graph, &export_config)
                .map_err(|e| CliError::visualization(format!("Failed to export SVG: {}", e)))?;
            std::fs::write(&output_path, svg)
                .map_err(|e| CliError::Io { 
                    message: format!("Failed to write SVG file: {}", output_path.display()),
                    source: Some(e)
                })?;
            println!("SVG visualization saved to: {}", output_path.display());
        }

        OutputFormat::Png => {
            // PNG export requires additional processing
            // For now, we'll generate SVG and note that PNG requires external tools
            let svg = export_to_svg(&graph, &export_config)
                .map_err(|e| CliError::visualization(format!("Failed to export SVG: {}", e)))?;
            let svg_path = output_path.with_extension("svg");
            std::fs::write(&svg_path, &svg)
                .map_err(|e| CliError::Io { 
                    message: format!("Failed to write SVG file: {}", svg_path.display()),
                    source: Some(e)
                })?;
            
            println!("Note: PNG export requires external tools.");
            println!("SVG saved to: {}", svg_path.display());
            println!("Convert to PNG using: rsvg-convert {} -o {}", svg_path.display(), output_path.display());
        }

        OutputFormat::Json => {
            let json = export_to_json(&graph)
                .map_err(|e| CliError::visualization(format!("Failed to export JSON: {}", e)))?;
            std::fs::write(&output_path, json)
                .map_err(|e| CliError::Io { 
                    message: format!("Failed to write JSON file: {}", output_path.display()),
                    source: Some(e)
                })?;
            println!("JSON graph data saved to: {}", output_path.display());
        }
    }

    // Print summary
    println!("\nVisualization Summary:");
    println!("  Model: {}", model.name);
    println!("  View Mode: {:?}", view_mode);
    println!("  Nodes: {}", graph.nodes.len());
    println!("  Edges: {}", graph.edges.len());

    // Count node types
    use ocsf_viz::NodeType;
    let entity_count = graph.nodes_of_type(NodeType::SemanticEntity).count();
    let category_count = graph.nodes_of_type(NodeType::OcsfCategory).count();
    let class_count = graph.nodes_of_type(NodeType::OcsfClass).count();
    let observable_count = graph.nodes_of_type(NodeType::Observable).count();

    if entity_count > 0 {
        println!("  Semantic Entities: {}", entity_count);
    }
    if category_count > 0 {
        println!("  OCSF Categories: {}", category_count);
    }
    if class_count > 0 {
        println!("  OCSF Event Classes: {}", class_count);
    }
    if observable_count > 0 {
        println!("  Observables: {}", observable_count);
    }

    Ok(())
}
