use std::env;
use std::path::{Path, PathBuf};
use pathdiff::diff_paths;
use std::process::{Command, Stdio};
use std::io::{self, Write, Read};
use std::fs;


async fn generate_seaorm_entities(
    database_uri: &str,
    output_dir: &str,
) -> io::Result<()> {
    // Build base command
    let mut cmd = Command::new("sea-orm-cli");
    
    // Configure command arguments
    cmd.arg("generate")
        .arg("entity")
        .arg("--database-url")
        .arg(database_uri)
        .arg("-o")
        .arg(output_dir)
        .arg("--with-serde")
        .arg("both")
        .arg("--seaography")
        .arg("--lib")
        .arg("--model-extra-derives")
        .arg("poem_openapi::Object");
    ;
 
    
    // Capture and display output in real-time
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    // Execute the command
    let status = cmd.status()?;
    
    if status.success() {
        println!("\n✅ Entity generation successful!");
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command failed with exit code: {:?}", status.code()),
        ))
    }
}


fn get_relative_cargo_toml() -> Option<PathBuf> {
    // Get absolute path to Cargo.toml directory from env var
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let abs_cargo_toml = Path::new(&manifest_dir).join("Cargo.toml");
    
    // Get current working directory
    let current_dir = env::current_dir().ok()?;
    
    // Compute relative path from current dir to Cargo.toml
    diff_paths(&abs_cargo_toml, current_dir)
}

fn patch_column_type_ignore_attr(dir: &Path, types: &[&str]) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
            let mut content = String::new();
            fs::File::open(&path)?.read_to_string(&mut content)?;
            let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            let mut changed = false;
            let mut i = 0;
            while i < lines.len() {
                for t in types {
                    let search = format!(
                        "#[sea_orm(column_type = \"custom(\\\"{}\\\")\")]",
                        t
                    );
                    if lines[i].trim() == search {
                        // Only insert #[sea_orm(ignore)] if not already present on next line
                        if i + 1 >= lines.len() || lines[i + 1].trim() != "#[sea_orm(ignore)]" {
                            lines.insert(i + 1, "    #[sea_orm(ignore)]".to_string());
                            lines.insert(i + 1, "    #[serde(skip)]".to_string());
                            lines.insert(i + 1, "    #[oai(skip)]".to_string());
                            changed = true;
                            i += 1; // Skip the inserted line
                        }
                    }
                }
                i += 1;
            }
            if changed {
                let new_content = lines.join("\n");
                let mut file = fs::File::create(&path)?;
                file.write_all(new_content.as_bytes())?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main()  {
    let mut path = get_relative_cargo_toml().expect("Cargo.toml not found");
    let database_url = env!("DATABASE_URL").to_string();
    path.pop();
    path.push("src");
    println!("{:?}", path);
    generate_seaorm_entities(&database_url, path.to_str().unwrap()).await.expect("Could not generate entities");
    patch_column_type_ignore_attr(&path, &["tsvector"]).expect("Failed to patch column_type attributes");
}