//! Artifact generation command.
//!
//! This command generates warehouse artifacts from semantic models.

use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::error::CliError;

/// Artifact types that can be generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    Dbt,
    CubeJs,
    Views,
    Tables,
    Etl,
    Observables,
    All,
}

impl ArtifactType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dbt" => Some(Self::Dbt),
            "cubejs" | "cube" => Some(Self::CubeJs),
            "views" => Some(Self::Views),
            "tables" => Some(Self::Tables),
            "etl" => Some(Self::Etl),
            "observables" => Some(Self::Observables),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

/// Runs the generate command.
pub async fn run(
    model_path: PathBuf,
    dialect: ocsf_semantic::WarehouseDialect,
    output_dir: PathBuf,
    artifacts_str: &str,
    schema_path: Option<PathBuf>,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    use ocsf_core::SchemaIngester;
    use ocsf_semantic::SemanticModel;
    use ocsf_warehouse::{
        CubeGenerator, DBTGenerator, ETLGenerator, ObservablesTableGenerator,
        TableGenerator, ViewGenerator,
    };

    // Parse artifact types
    let artifact_types: Vec<ArtifactType> = artifacts_str
        .split(',')
        .filter_map(|s| ArtifactType::from_str(s.trim()))
        .collect();

    if artifact_types.is_empty() {
        return Err(CliError::artifact_generation(
            "No valid artifact types specified. Use: dbt, cubejs, views, tables, etl, observables, or all"
        ).into());
    }

    let generate_all = artifact_types.contains(&ArtifactType::All);

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

    // Create output directory
    std::fs::create_dir_all(&output_dir)?;

    println!("Generating artifacts for dialect: {:?}", dialect);
    println!("Output directory: {}", output_dir.display());

    // Generate dbt artifacts
    if generate_all || artifact_types.contains(&ArtifactType::Dbt) {
        if verbose {
            eprintln!("Generating dbt artifacts...");
        }

        let dbt_dir = output_dir.join("dbt");
        std::fs::create_dir_all(&dbt_dir)?;

        let dbt_generator = DBTGenerator::new(dialect);
        let dbt_artifacts = dbt_generator.generate(&model);

        // Write semantic manifest
        std::fs::write(
            dbt_dir.join("semantic_manifest.yml"),
            &dbt_artifacts.semantic_manifest,
        )?;

        // Write sources
        std::fs::write(dbt_dir.join("sources.yml"), &dbt_artifacts.sources)?;

        // Write models
        let models_dir = dbt_dir.join("models");
        std::fs::create_dir_all(&models_dir)?;
        for (name, sql) in &dbt_artifacts.models {
            std::fs::write(models_dir.join(format!("{}.sql", name)), sql)?;
        }

        println!("  ✓ dbt artifacts generated");
    }

    // Generate Cube.js artifacts
    if generate_all || artifact_types.contains(&ArtifactType::CubeJs) {
        if verbose {
            eprintln!("Generating Cube.js artifacts...");
        }

        let cubejs_dir = output_dir.join("cubejs");
        std::fs::create_dir_all(&cubejs_dir)?;

        let cubejs_generator = CubeGenerator::new();
        let cubejs_artifacts = cubejs_generator.generate(&model);

        for (name, content) in &cubejs_artifacts.cubes {
            std::fs::write(cubejs_dir.join(format!("{}.js", name)), content)?;
        }

        println!("  ✓ Cube.js artifacts generated");
    }

    // Generate SQL views
    if generate_all || artifact_types.contains(&ArtifactType::Views) {
        if verbose {
            eprintln!("Generating SQL views...");
        }

        let views_dir = output_dir.join("views");
        std::fs::create_dir_all(&views_dir)?;

        let view_generator = ViewGenerator::new(dialect);
        let views = view_generator.generate(&model);

        for (name, sql) in &views.views {
            std::fs::write(views_dir.join(format!("{}.sql", name)), sql)?;
        }

        // Also write a combined file
        std::fs::write(views_dir.join("all_views.sql"), views.to_sql())?;

        println!("  ✓ SQL views generated");
    }

    // Generate table schemas
    if generate_all || artifact_types.contains(&ArtifactType::Tables) {
        if verbose {
            eprintln!("Generating table schemas...");
        }

        let tables_dir = output_dir.join("tables");
        std::fs::create_dir_all(&tables_dir)?;

        let table_generator = TableGenerator::new(dialect);
        let tables = table_generator.generate_all_tables(schema);

        let mut all_tables_sql = String::new();
        for table in &tables {
            let sql = table.to_create_statement(dialect);
            std::fs::write(tables_dir.join(format!("{}.sql", table.name)), &sql)?;
            all_tables_sql.push_str(&sql);
            all_tables_sql.push_str("\n\n");
        }

        std::fs::write(tables_dir.join("all_tables.sql"), all_tables_sql)?;

        println!("  ✓ Table schemas generated ({} tables)", tables.len());
    }

    // Generate observables table
    if generate_all || artifact_types.contains(&ArtifactType::Observables) {
        if verbose {
            eprintln!("Generating observables table...");
        }

        let observables_dir = output_dir.join("observables");
        std::fs::create_dir_all(&observables_dir)?;

        let obs_generator = ObservablesTableGenerator::new(dialect);
        let obs_schema = obs_generator.generate();

        std::fs::write(
            observables_dir.join("observables_table.sql"),
            obs_schema.to_create_statements(),
        )?;

        println!("  ✓ Observables table generated");
    }

    // Generate ETL pipeline
    if generate_all || artifact_types.contains(&ArtifactType::Etl) {
        if verbose {
            eprintln!("Generating ETL pipeline...");
        }

        let etl_dir = output_dir.join("etl");
        std::fs::create_dir_all(&etl_dir)?;

        let etl_generator = ETLGenerator::new(dialect);
        let etl = etl_generator.generate();

        std::fs::write(etl_dir.join("extract_observables.sql"), etl.to_sql())?;

        println!("  ✓ ETL pipeline generated");
    }

    println!("\nAll artifacts generated successfully!");

    Ok(())
}
