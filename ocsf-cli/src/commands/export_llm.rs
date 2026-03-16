//! Export LLM-friendly context from semantic models.

use std::path::PathBuf;

use anyhow::Result;
use ocsf_semantic::{
    dns_templates, computed_fields::dns_computed_fields, threat_intel::prebuilt,
    ExportConfig, ExportSection, LLMExport, SemanticModel,
};

use crate::config::Config;

/// Run the export-llm-context command.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    model_path: PathBuf,
    output_path: PathBuf,
    sections: Option<String>,
    format: ExportFormat,
    minimal: bool,
    include_dns_templates: bool,
    _config: &Config,
    verbose: bool,
) -> Result<()> {
    if verbose {
        eprintln!("Loading semantic model from: {}", model_path.display());
    }

    // Load the semantic model
    let model = SemanticModel::load(&model_path)?;

    // Build export config
    let mut export_config = if minimal {
        ExportConfig::minimal()
    } else {
        ExportConfig::default()
    };

    // Parse sections filter if provided
    if let Some(sections_str) = sections {
        let parsed_sections: Vec<ExportSection> = sections_str
            .split(',')
            .filter_map(|s| parse_section(s.trim()))
            .collect();
        
        if !parsed_sections.is_empty() {
            export_config = export_config.with_sections(parsed_sections);
        }
    }

    if verbose {
        eprintln!("Export config: {:?}", export_config.sections);
    }

    // Create the export
    let mut export = LLMExport::from_model(&model, &export_config);

    // Add DNS templates if requested
    if include_dns_templates {
        let templates = dns_templates::all_templates();
        let computed = dns_computed_fields::all();
        let threat_intel = prebuilt::all();

        export = export
            .with_query_templates(&templates, &export_config)
            .with_computed_fields(&computed, &export_config)
            .with_threat_intel(&threat_intel, &export_config);

        if verbose {
            eprintln!("Added {} DNS query templates", templates.len());
            eprintln!("Added {} computed fields", computed.len());
            eprintln!("Added {} threat intel joins", threat_intel.len());
        }
    }

    // Serialize to the requested format
    let output = match format {
        ExportFormat::Json => export.to_json()?,
        ExportFormat::JsonCompact => export.to_json_compact()?,
        ExportFormat::Yaml => export.to_yaml()?,
    };

    // Write to file
    std::fs::write(&output_path, &output)?;

    println!("Exported LLM context to: {}", output_path.display());
    println!("  Format: {:?}", format);
    println!("  Entities: {}", export.entities.len());
    println!("  Synonyms: {}", export.synonyms.len());
    println!("  Query templates: {}", export.query_templates.len());
    println!("  Computed fields: {}", export.computed_fields.len());

    // Print size info
    let size_bytes = output.len();
    let size_kb = size_bytes as f64 / 1024.0;
    println!("  Output size: {:.1} KB ({} bytes)", size_kb, size_bytes);

    // Estimate token count (rough approximation: ~4 chars per token)
    let estimated_tokens = size_bytes / 4;
    println!("  Estimated tokens: ~{}", estimated_tokens);

    Ok(())
}

/// Output format for LLM export.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    /// Pretty-printed JSON.
    Json,
    /// Compact JSON (no whitespace).
    JsonCompact,
    /// YAML format.
    Yaml,
}

fn parse_section(s: &str) -> Option<ExportSection> {
    match s.to_lowercase().as_str() {
        "entities" => Some(ExportSection::Entities),
        "synonyms" => Some(ExportSection::Synonyms),
        "query_templates" | "templates" => Some(ExportSection::QueryTemplates),
        "computed_fields" | "computed" => Some(ExportSection::ComputedFields),
        "threat_intel" | "threatintel" => Some(ExportSection::ThreatIntel),
        "sample_values" | "samples" => Some(ExportSection::SampleValues),
        "security_context" | "security" => Some(ExportSection::SecurityContext),
        _ => None,
    }
}
