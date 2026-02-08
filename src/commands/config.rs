use std::path::PathBuf;
use crate::config::Config;
use crate::error::Result;

pub async fn execute(local: bool) -> Result<()> {
    let config_path = if local {
        PathBuf::from("chaba.yaml")
    } else {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::error::ChabaError::ConfigError(
                "Cannot find config directory".to_string()
            ))?;
        let chaba_dir = config_dir.join("chaba");
        tokio::fs::create_dir_all(&chaba_dir).await?;
        chaba_dir.join("chaba.yaml")
    };

    if config_path.exists() {
        println!("Configuration file already exists at: {}", config_path.display());
        println!("Edit it manually or delete it to regenerate.");
        return Ok(());
    }

    let example_config = Config::example();
    tokio::fs::write(&config_path, example_config).await?;

    println!("✓ Created configuration file at: {}", config_path.display());
    println!("\nEdit this file to customize Chaba's behavior.");

    Ok(())
}
