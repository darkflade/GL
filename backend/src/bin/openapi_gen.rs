use anyhow::Context;
use std::fs;

#[path = "../web/api_docs.rs"]
mod api_docs;

fn main() -> anyhow::Result<()> {
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../docs/openapi/tags.generated.yaml".to_string());

    let yaml = api_docs::generate_openapi_yaml().context("failed to generate openapi yaml")?;

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory for {output_path}"))?;
    }

    fs::write(&output_path, yaml)
        .with_context(|| format!("failed to write openapi yaml to {output_path}"))?;

    println!("OpenAPI yaml generated: {output_path}");
    Ok(())
}
