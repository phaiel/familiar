use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:warning=Starting familiar-contracts build...");

    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir_str = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir_str);
    let workspace_root = manifest_dir.parent().unwrap();

    // Clone the schemas repo from GitHub
    let schemas_checkout = Path::new(&out_dir).join("schemas-checkout");

    if !schemas_checkout.exists() {
        println!("cargo:warning=Cloning familiar-schemas from GitHub...");
        let clone_result = Command::new("git")
            .args([
                "clone",
                "--depth", "1",
                "--branch", "main",
                "https://github.com/phaiel/familiar-schemas.git",
                &schemas_checkout.to_string_lossy(),
            ])
            .output();

        match clone_result {
            Ok(result) if result.status.success() => {
                println!("cargo:warning=Cloned familiar-schemas successfully");
            }
            Ok(result) => {
                eprintln!("cargo:warning=Git clone failed: {}", String::from_utf8_lossy(&result.stderr));
            }
            Err(e) => {
                eprintln!("cargo:warning=Git clone error: {}", e);
            }
        }
    }

    // Find the latest schema version
    let schema_dir = schemas_checkout
        .join("versions")
        .join("latest")
        .join("json-schema");

    if !schema_dir.exists() {
        eprintln!("cargo:warning=Schema directory not found: {}", schema_dir.display());
        generate_placeholder();
        return;
    }

    println!("cargo:warning=Using schemas from: {}", schema_dir.display());

    // Generate embedded_schemas.rs for include_dir! macro
    let embedded_file = Path::new(&out_dir).join("embedded_schemas.rs");
    let content = format!(
        r#"use include_dir::{{include_dir, Dir}};

pub static SCHEMAS: Dir<'static> = include_dir!("{}");
"#,
        schema_dir.display()
    );
    std::fs::write(&embedded_file, content).unwrap();
    println!("cargo:warning=Generated embedded_schemas.rs");

    // Check if generated.rs already exists (don't overwrite - use explicit regeneration)
    let generated_rs = manifest_dir.join("src").join("generated.rs");

    if generated_rs.exists() {
        // Check if file has more than a placeholder
        let content = std::fs::read_to_string(&generated_rs).unwrap_or_default();
        if content.len() > 100 {
            println!("cargo:warning=Using existing generated.rs ({} bytes)", content.len());
            println!("cargo:warning=To regenerate, run: python3 /tmp/codegen.py <schema-dir> familiar-contracts/src/generated.rs");
            println!("cargo:rerun-if-changed=build.rs");
            return;
        }
    }

    // Only run codegen if generated.rs is missing or is a placeholder
    let xtask_path = workspace_root.join("target").join("debug").join("xtask");

    if xtask_path.exists() {
        println!("cargo:warning=Running xtask codegen generate...");
        let output = Command::new(&xtask_path)
            .args([
                "codegen", "generate",
                "--schema-dir", &schema_dir.to_string_lossy(),
                "-o", &manifest_dir.join("src").to_string_lossy(),
            ])
            .current_dir(workspace_root)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                println!("cargo:warning=Codegen completed successfully");
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                let stdout = String::from_utf8_lossy(&result.stdout);
                if !stderr.is_empty() {
                    println!("cargo:warning=Codegen stderr: {}", stderr);
                }
                if !stdout.is_empty() {
                    println!("cargo:warning=Codegen stdout: {}", stdout);
                }
                // Don't fail - generated.rs might already exist
            }
            Err(e) => {
                println!("cargo:warning=Codegen error: {}", e);
            }
        }
    } else {
        println!("cargo:warning=xtask not built yet - run 'cargo build -p xtask' first");
        println!("cargo:warning=Then run 'cargo xtask codegen generate' to populate generated.rs");
    }

    // Ensure generated.rs exists (placeholder if codegen didn't run)
    if !generated_rs.exists() {
        println!("cargo:warning=Creating placeholder generated.rs");
        std::fs::write(&generated_rs, "// Placeholder - run 'cargo xtask codegen generate'\n").ok();
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_placeholder() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let embedded_file = Path::new(&out_dir).join("embedded_schemas.rs");

    // Create empty dir for include_dir! to point to
    let empty_dir = Path::new(&out_dir).join("empty_schemas");
    std::fs::create_dir_all(&empty_dir).ok();

    let content = format!(
        r#"use include_dir::{{include_dir, Dir}};

pub static SCHEMAS: Dir<'static> = include_dir!("{}");
"#,
        empty_dir.display()
    );
    std::fs::write(&embedded_file, content).unwrap();
}
