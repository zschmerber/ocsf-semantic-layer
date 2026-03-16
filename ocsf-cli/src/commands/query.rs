//! Query translation command.
//!
//! This command translates semantic queries to SQL.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::error::CliError;
use crate::QueryOutputFormat;

/// Runs the query command.
pub async fn run(
    model_path: PathBuf,
    query_str: &str,
    dialect: ocsf_semantic::WarehouseDialect,
    output_format: QueryOutputFormat,
    path_preference: ocsf_semantic::PathPreference,
    verbose: bool,
) -> Result<()> {
    use ocsf_semantic::{QueryValidator, SemanticModel, SqlGenerator};

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

    // Parse the query
    let query = parse_query(query_str, path_preference)
        .map_err(|e| CliError::query_translation(format!("Failed to parse query: {}", e)))?;

    if verbose {
        eprintln!("Query parsed:");
        eprintln!("  Entity: {}", query.entity);
        eprintln!("  Select: {:?}", query.select);
        eprintln!("  Metrics: {:?}", query.metrics);
        eprintln!("  Path preference: {:?}", query.path_preference);
    }

    // Validate the query
    let validator = QueryValidator::new(&model);
    let validated = validator.validate(&query).map_err(|e| {
        CliError::query_translation(format!("Query validation failed: {}", e))
    })?;

    if verbose {
        eprintln!("Query validated successfully");
        eprintln!("  Uses hot path: {}", validated.uses_hot_path);
    }

    // Generate SQL
    let generator = SqlGenerator::new(dialect);
    let translated = generator.generate(&validated);

    // Output results
    match output_format {
        QueryOutputFormat::Sql => {
            println!("{}", translated.sql);
        }

        QueryOutputFormat::Json => {
            let output = serde_json::json!({
                "sql": translated.sql,
                "dialect": dialect.name(),
                "uses_hot_path": translated.uses_hot_path,
                "query": {
                    "entity": query.entity,
                    "select": query.select,
                    "metrics": query.metrics,
                    "filters": query.filters,
                    "group_by": query.group_by,
                    "path_preference": format!("{:?}", query.path_preference),
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

/// Parses a query string into a SemanticQuery.
///
/// Supports two formats:
/// 1. JSON format: `{"entity": "...", "select": [...], ...}`
/// 2. Simple format: `entity.attr1,attr2 [metric1,metric2]`
fn parse_query(
    query_str: &str,
    path_preference: ocsf_semantic::PathPreference,
) -> Result<ocsf_semantic::SemanticQuery> {
    use ocsf_semantic::SemanticQuery;

    // Try JSON format first
    if query_str.trim().starts_with('{') {
        let mut query: SemanticQuery = serde_json::from_str(query_str)
            .context("Failed to parse query as JSON")?;
        query.path_preference = path_preference;
        return Ok(query);
    }

    // Parse simple format: entity.attr1,attr2 [metric1,metric2]
    let parts: Vec<&str> = query_str.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty query");
    }

    // Parse entity and attributes
    let entity_part = parts[0];
    let (entity, attrs) = if entity_part.contains('.') {
        let mut split = entity_part.splitn(2, '.');
        let entity = split.next().unwrap();
        let attrs_str = split.next().unwrap_or("");
        let attrs: Vec<String> = attrs_str
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        (entity, attrs)
    } else {
        (entity_part, Vec::new())
    };

    // Parse metrics (in square brackets)
    let metrics: Vec<String> = parts
        .iter()
        .skip(1)
        .filter(|p| p.starts_with('[') && p.ends_with(']'))
        .flat_map(|p| {
            p.trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .collect();

    let mut query = SemanticQuery::new(entity);
    
    if !attrs.is_empty() {
        query = query.select(attrs);
    }
    
    if !metrics.is_empty() {
        query = query.with_metrics(metrics);
    }
    
    query = query.with_path_preference(path_preference);

    // Ensure query has at least some selection
    if query.select.is_empty() && query.metrics.is_empty() {
        anyhow::bail!(
            "Query must select at least one attribute or metric. \
             Use format: entity.attr1,attr2 or entity [metric1,metric2]"
        );
    }

    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_semantic::PathPreference;

    #[test]
    fn test_parse_simple_query() {
        let query = parse_query("auth_event.user_email,source_ip", PathPreference::Auto).unwrap();
        assert_eq!(query.entity, "auth_event");
        assert_eq!(query.select, vec!["user_email", "source_ip"]);
    }

    #[test]
    fn test_parse_query_with_metrics() {
        let query = parse_query("auth_event.user_email [auth_attempts]", PathPreference::Auto).unwrap();
        assert_eq!(query.entity, "auth_event");
        assert_eq!(query.select, vec!["user_email"]);
        assert_eq!(query.metrics, vec!["auth_attempts"]);
    }

    #[test]
    fn test_parse_json_query() {
        let json = r#"{"entity": "auth_event", "select": ["user_email"], "metrics": ["auth_attempts"]}"#;
        let query = parse_query(json, PathPreference::Hot).unwrap();
        assert_eq!(query.entity, "auth_event");
        assert_eq!(query.select, vec!["user_email"]);
        assert_eq!(query.metrics, vec!["auth_attempts"]);
        assert_eq!(query.path_preference, PathPreference::Hot);
    }

    #[test]
    fn test_parse_empty_query_fails() {
        let result = parse_query("", PathPreference::Auto);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_entity_only_fails() {
        let result = parse_query("auth_event", PathPreference::Auto);
        assert!(result.is_err());
    }
}
