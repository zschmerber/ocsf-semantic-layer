//! Schema ingestion command.
//!
//! This command loads OCSF schemas from local files or GitHub.

use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::error::CliError;
use crate::SourceType;

/// Runs the ingest command.
pub async fn run(
    source: SourceType,
    path: Option<PathBuf>,
    version: Option<String>,
    output: Option<PathBuf>,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    use ocsf_core::SchemaIngester;

    let mut ingester = SchemaIngester::new();

    match source {
        SourceType::Local => {
            let schema_path = path.ok_or_else(|| {
                CliError::schema_ingestion("Local source requires --path argument")
            })?;

            if !schema_path.exists() {
                return Err(CliError::file_not_found(&schema_path).into());
            }

            if verbose {
                eprintln!("Loading schema from: {}", schema_path.display());
            }

            ingester
                .load_from_local(&schema_path)
                .map_err(|e| CliError::schema_ingestion_with_source(
                    format!("Failed to load schema from: {}", schema_path.display()),
                    e
                ))?;
        }

        SourceType::Github => {
            let version_str = version.as_deref().or(Some(&config.default_ocsf_version));

            if verbose {
                eprintln!(
                    "Fetching schema from GitHub (version: {})",
                    version_str.unwrap_or("latest")
                );
            }

            ingester
                .load_from_github(version_str)
                .await
                .map_err(|e| CliError::network(format!(
                    "Failed to fetch schema from GitHub: {}",
                    e
                )))?;
        }
    }

    let schema = ingester.schema();

    // Print schema summary
    println!("Schema loaded successfully!");
    println!("  Version: {}", schema.version);
    println!("  Categories: {}", schema.categories.len());
    println!("  Event Classes: {}", schema.event_classes.len());
    println!("  Objects: {}", schema.objects.len());
    println!("  Attributes: {}", schema.attributes.len());

    // Count observables
    let observable_count: usize = schema
        .all_event_classes()
        .map(|ec| ec.observables.len())
        .sum();
    println!("  Observables: {}", observable_count);

    // Save to output if specified
    if let Some(output_path) = output {
        if verbose {
            eprintln!("Saving schema info to: {}", output_path.display());
        }

        // Create a summary JSON
        let summary = serde_json::json!({
            "version": schema.version,
            "categories": schema.categories.len(),
            "event_classes": schema.event_classes.len(),
            "objects": schema.objects.len(),
            "attributes": schema.attributes.len(),
            "observables": observable_count,
            "category_names": schema.all_categories().map(|c| &c.name).collect::<Vec<_>>(),
            "event_class_names": schema.all_event_classes().map(|ec| &ec.name).collect::<Vec<_>>(),
        });

        std::fs::create_dir_all(output_path.parent().unwrap_or(&output_path))?;
        std::fs::write(&output_path, serde_json::to_string_pretty(&summary)?)?;
        println!("\nSchema summary saved to: {}", output_path.display());
    }

    Ok(())
}
