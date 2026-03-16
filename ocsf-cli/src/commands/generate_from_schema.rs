//! Generate semantic model from compiled OCSF schema.

use std::path::PathBuf;

use anyhow::{Context, Result};
use ocsf_core::CompiledSchema;
use ocsf_semantic::{GenerationConfig, MetricSuggester, SchemaGenerator};

use crate::config::Config;

/// Run the generate-from-schema command.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    schema_path: PathBuf,
    output_path: PathBuf,
    category_filter: Option<Vec<String>>,
    class_filter: Option<Vec<String>>,
    object_filter: Option<Vec<String>>,
    include_metrics: bool,
    max_depth: usize,
    _config: &Config,
    verbose: bool,
) -> Result<()> {
    if verbose {
        eprintln!("Loading compiled schema from: {}", schema_path.display());
    }

    // Load and parse the compiled schema
    let schema = CompiledSchema::parse_file(&schema_path)
        .with_context(|| format!("Failed to parse schema from {}", schema_path.display()))?;

    if verbose {
        let stats = schema.stats();
        eprintln!("Schema loaded: {}", stats);
    }

    // Build generation config
    let config = GenerationConfig {
        category_filter: category_filter.unwrap_or_default(),
        class_filter: class_filter.unwrap_or_default(),
        object_filter: object_filter.unwrap_or_default(),
        exclude_patterns: vec![],
        max_nesting_depth: max_depth,
        include_deprecated: false,
        required_only: false,
    };

    // Generate the semantic model
    let generator = SchemaGenerator::new(&schema, config);
    let (mut model, metadata) = generator.generate_model();

    if verbose {
        eprintln!(
            "Generated {} entities ({} from objects, {} from classes)",
            metadata.stats.entities_generated,
            metadata.stats.objects_processed,
            metadata.stats.classes_processed
        );
        eprintln!("Inferred {} dimensions", metadata.stats.dimensions_inferred);
    }

    // Optionally add suggested metrics
    if include_metrics {
        let suggester = MetricSuggester::new(&schema);
        let metrics = suggester.suggest_all_metrics();
        
        if verbose {
            eprintln!("Suggested {} metrics", metrics.len());
        }
        
        model = model.with_metrics(metrics);
    }

    // Save the model
    model.save(&output_path)
        .with_context(|| format!("Failed to save model to {}", output_path.display()))?;

    println!("Generated semantic model: {}", output_path.display());
    println!("  Schema version: {}", metadata.schema_version);
    println!("  Entities: {}", metadata.stats.entities_generated);
    println!("  Dimensions: {}", metadata.stats.dimensions_inferred);
    if include_metrics {
        println!("  Metrics: {}", model.metrics.len());
    }

    Ok(())
}
