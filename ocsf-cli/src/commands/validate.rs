//! Model validation command.
//!
//! This command validates semantic models against OCSF schemas.

use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::error::{self, CliError};

/// Runs the validate command.
pub async fn run(
    model_path: PathBuf,
    schema_path: Option<PathBuf>,
    json_output: bool,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    use ocsf_core::SchemaIngester;
    use ocsf_semantic::{SemanticModel, SemanticModelValidator};

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

    if verbose {
        eprintln!("Model loaded: {} (version {})", model.name, model.version);
        eprintln!("  Entities: {}", model.entities.len());
        eprintln!("  Metrics: {}", model.metrics.len());
    }

    // Load the OCSF schema
    let mut ingester = SchemaIngester::new();

    if let Some(schema_dir) = schema_path {
        if !schema_dir.exists() {
            return Err(CliError::file_not_found(&schema_dir).into());
        }
        if verbose {
            eprintln!("Loading schema from: {}", schema_dir.display());
        }
        ingester
            .load_from_local(&schema_dir)
            .map_err(|e| CliError::schema_ingestion_with_source(
                format!("Failed to load schema from: {}", schema_dir.display()),
                e
            ))?;
    } else {
        // Try to load from cache or fetch from GitHub
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
                eprintln!(
                    "Fetching schema version {} from GitHub",
                    model.ocsf_version
                );
            }
            ingester
                .load_from_github(Some(&model.ocsf_version))
                .await
                .map_err(|e| CliError::network(format!(
                    "Failed to fetch schema from GitHub: {}",
                    e
                )))?;
        }
    }

    let schema = ingester.schema();

    if verbose {
        eprintln!("Schema loaded: version {}", schema.version);
    }

    // Validate the model
    let validator = SemanticModelValidator::new(schema);
    let result = validator.validate(&model);

    // Output results
    if json_output {
        let output = serde_json::json!({
            "valid": result.is_valid(),
            "errors": result.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
            "warnings": result.warnings,
            "model": {
                "name": model.name,
                "version": model.version,
                "ocsf_version": model.ocsf_version,
                "entities": model.entities.len(),
                "metrics": model.metrics.len(),
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if result.is_valid() {
        println!("✓ Model validation passed!");
        println!("\nModel Summary:");
        println!("  Name: {}", model.name);
        println!("  Version: {}", model.version);
        println!("  OCSF Version: {}", model.ocsf_version);
        println!("  Entities: {}", model.entities.len());
        println!("  Metrics: {}", model.metrics.len());

        if !result.warnings.is_empty() {
            println!("\nWarnings ({}):", result.warnings.len());
            for warning in &result.warnings {
                println!("  ⚠ {}", warning);
            }
        }
    } else {
        eprintln!("✗ Model validation failed!");
        error::print_validation_summary(&result.errors, &result.warnings);
        std::process::exit(1);
    }

    Ok(())
}
