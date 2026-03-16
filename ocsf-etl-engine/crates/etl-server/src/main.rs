use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use etl_server::server::{AppState, create_router, load_schema_from_env};

fn demo_schema() -> codegen_adapter::CompiledSchema {
    use std::collections::BTreeMap;
    use codegen_adapter::{CompiledSchema, CompiledClass, CompiledAttribute};

    let mut attrs = BTreeMap::new();
    for (name, caption, required) in [
        ("query.hostname", "Query Hostname", true),
        ("src_endpoint.ip", "Source IP", false),
        ("dst_endpoint.ip", "Destination IP", false),
        ("time", "Event Time", true),
        ("activity_id", "Activity ID", true),
        ("activity_name", "Activity Name", false),
    ] {
        attrs.insert(name.to_string(), CompiledAttribute {
            name: name.to_string(),
            caption: caption.to_string(),
            type_name: "String".to_string(),
            is_required: required,
        });
    }

    let mut classes = BTreeMap::new();
    classes.insert(4003, CompiledClass {
        uid: 4003,
        name: "DNS Activity".to_string(),
        caption: "DNS Activity".to_string(),
        category_uid: 4,
        attributes: attrs,
    });

    CompiledSchema {
        version: "1.3.0".to_string(),
        classes,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = match load_schema_from_env() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("OCSF_SCHEMA_PATH not set, using built-in demo schema (DNS Activity uid=4003)");
            demo_schema()
        }
    };

    let state = Arc::new(AppState {
        jobs: Arc::new(RwLock::new(HashMap::new())),
        bridge: Arc::new(semantic_bridge::ManualBridge),
        schema,
        codegen: codegen_adapter::CodegenAdapter::new(),
        warehouse_dialect: warehouse_gen::WarehouseDialect::Snowflake,
        tangent_bin: std::env::var("TANGENT_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("tangent")),
        output_dir: std::env::var("ETL_OUTPUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./etl-output")),
    });

    let router = create_router(state);

    let port = std::env::var("ETL_PORT").unwrap_or_else(|_| "3030".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("ETL Engine server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
