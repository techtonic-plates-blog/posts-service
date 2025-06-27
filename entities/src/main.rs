use std::env;
use std::path::{Path, PathBuf};
use pathdiff::diff_paths;
use std::process::{Command, Stdio};
use std::io::{self, Write};


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

#[tokio::main]
async fn main()  {
    let mut path = get_relative_cargo_toml().expect("Cargo.toml not found");
    let database_url = env!("DATABASE_URL").to_string();
    path.pop();
    path.push("src");
    println!("{:?}", path);
    generate_seaorm_entities(&database_url, path.to_str().unwrap()).await.expect("Could not generate entities");
}