//! Command-line interface for OCSF semantic layer.
//!
//! This CLI provides commands for:
//! - Ingesting OCSF schemas from local files or GitHub
//! - Validating semantic models against OCSF schemas
//! - Generating warehouse artifacts (dbt, Cube.js, SQL views)
//! - Visualizing semantic layer interchange
//! - Translating semantic queries to SQL

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

mod commands;
mod config;
mod error;

use commands::{editor, export_llm, generate, generate_from_schema, ingest, query, validate, visualize};
use config::Config;

/// OCSF Semantic Layer CLI
///
/// A tool for building and managing semantic layers on top of OCSF schemas.
#[derive(Parser)]
#[command(name = "ocsf")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest OCSF schema from a source
    Ingest {
        /// Source type: local, github
        #[arg(short, long, default_value = "github")]
        source: SourceType,

        /// Path to local schema directory (for local source)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// OCSF schema version tag (for github source, e.g., "v1.4.0")
        #[arg(long = "ocsf-version")]
        ocsf_version: Option<String>,

        /// Output path for cached schema
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate a semantic model against an OCSF schema
    Validate {
        /// Path to semantic model YAML file
        #[arg(short, long)]
        model: PathBuf,

        /// Path to OCSF schema directory (uses cached if not specified)
        #[arg(short, long)]
        schema: Option<PathBuf>,

        /// Output validation report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate warehouse artifacts from a semantic model
    Generate {
        /// Path to semantic model YAML file
        #[arg(short, long)]
        model: PathBuf,

        /// Target warehouse dialect
        #[arg(short, long, default_value = "snowflake")]
        dialect: Dialect,

        /// Output directory for generated artifacts
        #[arg(short, long)]
        output: PathBuf,

        /// Artifact types to generate (comma-separated: dbt,cubejs,views,tables,etl)
        #[arg(short, long, default_value = "all")]
        artifacts: String,

        /// Path to OCSF schema directory (uses cached if not specified)
        #[arg(long)]
        schema: Option<PathBuf>,
    },

    /// Visualize the semantic layer interchange
    Visualize {
        /// Path to semantic model YAML file
        #[arg(short, long)]
        model: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "svg")]
        format: OutputFormat,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// View mode for visualization
        #[arg(long, default_value = "interchange")]
        view: ViewMode,

        /// Path to OCSF schema directory (uses cached if not specified)
        #[arg(long)]
        schema: Option<PathBuf>,

        /// Include observables in visualization
        #[arg(long, default_value = "true")]
        show_observables: bool,

        /// Include entity relationships
        #[arg(long, default_value = "true")]
        show_relationships: bool,
    },

    /// Translate a semantic query to SQL
    Query {
        /// Path to semantic model YAML file
        #[arg(short, long)]
        model: PathBuf,

        /// Semantic query (JSON format or simple entity.attribute syntax)
        query: String,

        /// Target warehouse dialect
        #[arg(short, long, default_value = "snowflake")]
        dialect: Dialect,

        /// Output format (sql, json)
        #[arg(short, long, default_value = "sql")]
        format: QueryOutputFormat,

        /// Path preference (auto, hot, cold)
        #[arg(long, default_value = "auto")]
        path: PathPreference,
    },

    /// Initialize a new semantic model
    Init {
        /// Output path for the new model
        #[arg(short, long, default_value = "semantic-model.yaml")]
        output: PathBuf,

        /// Model name
        #[arg(short, long)]
        name: Option<String>,

        /// OCSF version to target
        #[arg(long, default_value = "1.4.0")]
        ocsf_version: String,
    },

    /// Generate semantic model from compiled OCSF schema
    GenerateFromSchema {
        /// Path to compiled OCSF schema JSON file
        #[arg(short, long)]
        schema: PathBuf,

        /// Output path for generated semantic model
        #[arg(short, long, default_value = "generated-model.yaml")]
        output: PathBuf,

        /// Filter by category names (comma-separated)
        #[arg(long)]
        categories: Option<String>,

        /// Filter by class names (comma-separated)
        #[arg(long)]
        classes: Option<String>,

        /// Filter by object names (comma-separated)
        #[arg(long)]
        objects: Option<String>,

        /// Include suggested metrics
        #[arg(long, default_value = "true")]
        include_metrics: bool,

        /// Maximum nesting depth for path generation
        #[arg(long, default_value = "3")]
        max_depth: usize,
    },

    /// Export LLM-friendly context from a semantic model
    ExportLlmContext {
        /// Path to semantic model YAML file
        #[arg(short, long)]
        model: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Sections to include (comma-separated: entities,synonyms,templates,computed,threatintel,samples,security)
        #[arg(long)]
        sections: Option<String>,

        /// Output format (json, json-compact, yaml)
        #[arg(short, long, default_value = "json")]
        format: LlmExportFormat,

        /// Use minimal export (smaller context window)
        #[arg(long)]
        minimal: bool,

        /// Include pre-built DNS security query templates
        #[arg(long, default_value = "true")]
        include_dns_templates: bool,
    },

    /// Start the Semantic Model Editor server
    Editor {
        /// Port to run the server on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Path to OCSF schema file or directory to load
        #[arg(short, long)]
        schema: Option<PathBuf>,

        /// Path to frontend static files directory
        #[arg(long)]
        static_dir: Option<PathBuf>,

        /// Don't automatically open browser on startup
        #[arg(long)]
        no_open: bool,
    },
}

/// Source type for schema ingestion.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum SourceType {
    /// Load from local directory
    Local,
    /// Load from GitHub repository
    Github,
}

/// Target warehouse dialect.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Dialect {
    /// Snowflake SQL dialect
    Snowflake,
    /// Databricks SQL dialect
    Databricks,
    /// Google BigQuery SQL dialect
    Bigquery,
    /// PostgreSQL dialect
    Postgres,
}

impl From<Dialect> for ocsf_semantic::WarehouseDialect {
    fn from(d: Dialect) -> Self {
        match d {
            Dialect::Snowflake => ocsf_semantic::WarehouseDialect::Snowflake,
            Dialect::Databricks => ocsf_semantic::WarehouseDialect::Databricks,
            Dialect::Bigquery => ocsf_semantic::WarehouseDialect::BigQuery,
            Dialect::Postgres => ocsf_semantic::WarehouseDialect::Postgres,
        }
    }
}

/// Output format for visualization.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// SVG vector graphics
    Svg,
    /// PNG raster image
    Png,
    /// JSON graph data
    Json,
}

/// View mode for visualization.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ViewMode {
    /// Show only semantic entities
    Semantic,
    /// Show only physical OCSF elements
    Physical,
    /// Show both layers with mappings
    Interchange,
    /// Show hot/cold path data flow
    HotCold,
}

impl From<ViewMode> for ocsf_viz::ViewMode {
    fn from(v: ViewMode) -> Self {
        match v {
            ViewMode::Semantic => ocsf_viz::ViewMode::SemanticOnly,
            ViewMode::Physical => ocsf_viz::ViewMode::PhysicalOnly,
            ViewMode::Interchange => ocsf_viz::ViewMode::Interchange,
            ViewMode::HotCold => ocsf_viz::ViewMode::HotColdPath,
        }
    }
}

/// Output format for query results.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum QueryOutputFormat {
    /// Raw SQL output
    Sql,
    /// JSON with SQL and metadata
    Json,
}

/// Path preference for query execution.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum PathPreference {
    /// Automatically choose based on query
    Auto,
    /// Use hot path (observables table)
    Hot,
    /// Use cold path (full events table)
    Cold,
}

impl From<PathPreference> for ocsf_semantic::PathPreference {
    fn from(p: PathPreference) -> Self {
        match p {
            PathPreference::Auto => ocsf_semantic::PathPreference::Auto,
            PathPreference::Hot => ocsf_semantic::PathPreference::Hot,
            PathPreference::Cold => ocsf_semantic::PathPreference::Cold,
        }
    }
}

/// Output format for LLM export.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum LlmExportFormat {
    /// Pretty-printed JSON
    Json,
    /// Compact JSON (no whitespace)
    JsonCompact,
    /// YAML format
    Yaml,
}

impl From<LlmExportFormat> for export_llm::ExportFormat {
    fn from(f: LlmExportFormat) -> Self {
        match f {
            LlmExportFormat::Json => export_llm::ExportFormat::Json,
            LlmExportFormat::JsonCompact => export_llm::ExportFormat::JsonCompact,
            LlmExportFormat::Yaml => export_llm::ExportFormat::Yaml,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration if specified
    let config = if let Some(config_path) = &cli.config {
        if !config_path.exists() {
            let err = error::CliError::file_not_found(config_path);
            error::print_error(&err);
            std::process::exit(1);
        }
        Config::load(config_path).map_err(|e| {
            error::CliError::configuration(format!("Failed to load configuration: {}", e))
        })?
    } else {
        Config::default()
    };

    // Set up logging based on verbosity
    if cli.verbose {
        eprintln!("Verbose mode enabled");
    }

    // Execute the command
    let result = match cli.command {
        Commands::Ingest {
            source,
            path,
            ocsf_version,
            output,
        } => {
            ingest::run(source, path, ocsf_version, output, &config, cli.verbose).await
        }

        Commands::Validate {
            model,
            schema,
            json,
        } => {
            validate::run(model, schema, json, &config, cli.verbose).await
        }

        Commands::Generate {
            model,
            dialect,
            output,
            artifacts,
            schema,
        } => {
            generate::run(model, dialect.into(), output, &artifacts, schema, &config, cli.verbose)
                .await
        }

        Commands::Visualize {
            model,
            format,
            output,
            view,
            schema,
            show_observables,
            show_relationships,
        } => {
            visualize::run(
                model,
                format,
                output,
                view.into(),
                schema,
                show_observables,
                show_relationships,
                &config,
                cli.verbose,
            )
            .await
        }

        Commands::Query {
            model,
            query,
            dialect,
            format,
            path,
        } => {
            query::run(model, &query, dialect.into(), format, path.into(), cli.verbose).await
        }

        Commands::Init {
            output,
            name,
            ocsf_version,
        } => {
            init_model(output, name, ocsf_version)
        }

        Commands::GenerateFromSchema {
            schema,
            output,
            categories,
            classes,
            objects,
            include_metrics,
            max_depth,
        } => {
            let category_filter = categories.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
            let class_filter = classes.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
            let object_filter = objects.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
            
            generate_from_schema::run(
                schema,
                output,
                category_filter,
                class_filter,
                object_filter,
                include_metrics,
                max_depth,
                &config,
                cli.verbose,
            )
            .await
        }

        Commands::ExportLlmContext {
            model,
            output,
            sections,
            format,
            minimal,
            include_dns_templates,
        } => {
            export_llm::run(
                model,
                output,
                sections,
                format.into(),
                minimal,
                include_dns_templates,
                &config,
                cli.verbose,
            )
            .await
        }

        Commands::Editor {
            port,
            schema,
            static_dir,
            no_open,
        } => {
            editor::run(
                port,
                schema,
                static_dir,
                no_open,
                &config,
                cli.verbose,
            )
            .await
        }
    };

    // Handle errors with suggestions
    if let Err(e) = result {
        if let Some(cli_err) = e.downcast_ref::<error::CliError>() {
            error::print_error(cli_err);
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize a new semantic model file.
fn init_model(output: PathBuf, name: Option<String>, ocsf_version: String) -> Result<()> {
    use ocsf_semantic::{SemanticModel, ObservableConfig};

    let model_name = name.unwrap_or_else(|| {
        output
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("my-semantic-model")
            .to_string()
    });

    let model = SemanticModel::new(&model_name)
        .with_version("1.0")
        .with_ocsf_version(&ocsf_version)
        .with_description("Semantic layer for security analytics on OCSF data")
        .with_observable_config(ObservableConfig {
            extract_to_table: true,
            table_name: "ocsf_observables".to_string(),
            include_types: vec![2, 5, 10, 22, 30], // Common observable types
        });

    model.save(&output)?;
    println!("Created new semantic model: {}", output.display());
    println!("\nNext steps:");
    println!("  1. Edit {} to add entities and metrics", output.display());
    println!("  2. Run 'ocsf validate -m {}' to validate your model", output.display());
    println!("  3. Run 'ocsf generate -m {} -o ./output' to generate artifacts", output.display());

    Ok(())
}
