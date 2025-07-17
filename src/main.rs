/*!
 * AI Context Management Tool - Main CLI (Simplified)
 *
 * Simplified CLI entry point
 */

use aicm::agents::claude::ClaudeAgent;
use aicm::agents::cline::ClineAgent;
use aicm::agents::codex::CodexAgent;
use aicm::agents::cursor::CursorAgent;
use aicm::agents::gemini::GeminiAgent;
use aicm::agents::github::GitHubAgent;
use aicm::agents::kiro::KiroAgent;
use aicm::config::{error::ConfigError, loader::ConfigLoader};
use aicm::types::{AIContextConfig, GeneratedFile};
use aicm::DEFAULT_CONFIG_FILE;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use tokio::fs;

#[derive(Parser)]
#[command(name = "aicm")]
#[command(
    about = "AI Context Management Tool - Unified context file management for multiple AI coding agents"
)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize project (create configuration file template)
    Init,
    /// Generate context files for AI agents
    Generate {
        /// Generate files for specific agent only
        #[arg(long)]
        agent: Option<String>,
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Validate configuration file
    Validate {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => handle_init().await,
        Commands::Generate { agent, config } => handle_generate(agent, config).await,
        Commands::Validate { config } => handle_validate(config).await,
    };

    // Display error message and exit with appropriate code if error occurs
    if let Err(e) = result {
        // Display ConfigError appropriately
        if let Some(config_error) = e.downcast_ref::<aicm::config::error::ConfigError>() {
            eprintln!("❌ Configuration validation error: {}", config_error);
        } else {
            eprintln!("❌ Error occurred: {}", e);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Handle init command
async fn handle_init() -> Result<()> {
    println!("Initializing project...");

    // Check if configuration file already exists
    if Path::new(DEFAULT_CONFIG_FILE).exists() {
        println!("⚠️  {} already exists", DEFAULT_CONFIG_FILE);
    } else {
        // Create default configuration file
        ConfigLoader::create_default(DEFAULT_CONFIG_FILE).await?;
        println!("✅ Created {}", DEFAULT_CONFIG_FILE);
    }

    Ok(())
}

/// Handle generate command
async fn handle_generate(agent_filter: Option<String>, config_path: Option<String>) -> Result<()> {
    let config_file = config_path.as_deref().unwrap_or(DEFAULT_CONFIG_FILE);
    println!("Generating context files: {}", config_file);

    // Load configuration file
    let config = load_config_from_path(config_file).await?;

    // Check if documentation directory exists
    if !Path::new(&config.base_docs_dir).exists() {
        return Err(anyhow::anyhow!(
            "❌ Documentation directory does not exist: {}\n💡 Please create the directory or change base_docs_dir in the configuration file to the correct path",
            config.base_docs_dir
        ));
    }

    // Get enabled agents
    let enabled_agents = get_enabled_agents(&config, agent_filter);

    if enabled_agents.is_empty() {
        println!("❌ No enabled agents found");
        println!(
            "💡 Please enable agents in the agents section of {}",
            DEFAULT_CONFIG_FILE
        );
        return Ok(());
    }

    // Generate files for each agent
    for agent_name in enabled_agents {
        match generate_agent_files(&config, &agent_name).await {
            Ok(files) => {
                for file in files {
                    write_generated_file(&file).await?;
                    println!("📄 {}", file.path);
                }
            }
            Err(e) => {
                println!("❌ Error generating files for {}: {}", agent_name, e);
            }
        }
    }

    println!("✅ Context file generation completed");
    Ok(())
}

/// Handle validate command
async fn handle_validate(config_path: Option<String>) -> Result<()> {
    let config_file = config_path.as_deref().unwrap_or(DEFAULT_CONFIG_FILE);
    println!("Validating configuration file: {}", config_file);

    let config = load_config_from_path(config_file)
        .await
        .map_err(anyhow::Error::from)?;

    // Check if documentation directory exists
    if !Path::new(&config.base_docs_dir).exists() {
        return Err(anyhow::anyhow!(
            "❌ Documentation directory does not exist: {}\n💡 Please create the directory or change base_docs_dir in the configuration file to the correct path",
            config.base_docs_dir
        ));
    }

    println!("✅ Configuration file is valid");

    // Display basic information
    println!("  Version: {}", config.version);
    println!("  Output mode: {:?}", config.output_mode);
    println!(
        "  Documentation directory: {} (exists)",
        config.base_docs_dir
    );

    // Display enabled agents
    let enabled = config.enabled_agents();
    if enabled.is_empty() {
        println!("  Enabled agents: none");
    } else {
        println!("  Enabled agents: {}", enabled.join(", "));
    }

    Ok(())
}

/// Load configuration file from specified path
async fn load_config_from_path(config_path: &str) -> Result<AIContextConfig, ConfigError> {
    if !Path::new(config_path).exists() {
        return Err(ConfigError::FileNotFound {
            path: config_path.to_string(),
        });
    }

    ConfigLoader::load(config_path).await
}

/// Get list of enabled agents
fn get_enabled_agents(config: &AIContextConfig, filter: Option<String>) -> Vec<String> {
    let all_enabled = config.enabled_agents();

    match filter {
        Some(agent_name) => {
            if all_enabled.contains(&agent_name) {
                vec![agent_name]
            } else {
                println!("❌ Agent '{}' is not enabled", agent_name);
                println!("💡 Available agents: {}", all_enabled.join(", "));
                vec![]
            }
        }
        None => all_enabled,
    }
}

/// Generate files for specified agent
async fn generate_agent_files(
    config: &AIContextConfig,
    agent_name: &str,
) -> Result<Vec<GeneratedFile>> {
    match agent_name {
        "cursor" => {
            let agent = CursorAgent::new(config.clone());
            agent.generate().await
        }
        "claude" => {
            let agent = ClaudeAgent::new(config.clone());
            agent.generate().await
        }
        "github" => {
            let agent = GitHubAgent::new(config.clone());
            agent.generate().await
        }
        "cline" => {
            let agent = ClineAgent::new(config.clone());
            agent.generate().await
        }
        "codex" => {
            let agent = CodexAgent::new(config.clone());
            agent.generate().await
        }
        "gemini" => {
            let agent = GeminiAgent::new(config.clone());
            agent.generate().await
        }
        "kiro" => {
            let agent = KiroAgent::new(config.clone());
            agent.generate().await
        }
        _ => Err(anyhow::anyhow!("Unsupported agent: {}", agent_name)),
    }
}

/// Write generated file
async fn write_generated_file(file: &GeneratedFile) -> Result<()> {
    // Create directory
    if let Some(parent) = Path::new(&file.path).parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write file
    fs::write(&file.path, &file.content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aicm::types::AgentConfigTrait;
    use tempfile::tempdir;
    use tokio::fs;

    #[test]
    fn test_get_enabled_agents_with_filter() {
        let mut config = AIContextConfig::default();
        config.agents.cursor = aicm::types::CursorConfig::Simple(true);
        config.agents.claude = aicm::types::ClaudeConfig::Simple(true);

        // No filter
        let all_agents = get_enabled_agents(&config, None);
        assert_eq!(all_agents.len(), 2);
        assert!(all_agents.contains(&"cursor".to_string()));
        assert!(all_agents.contains(&"claude".to_string()));

        // Filter with valid agent
        let filtered = get_enabled_agents(&config, Some("cursor".to_string()));
        assert_eq!(filtered, vec!["cursor"]);

        // Filter with invalid agent
        let invalid = get_enabled_agents(&config, Some("invalid".to_string()));
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_get_enabled_agents_no_agents() {
        let config = AIContextConfig::default();
        let agents = get_enabled_agents(&config, None);
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_load_config_from_path_valid() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("custom-config.yaml");

        let test_config_content = r#"
version: "1.0"
output_mode: split
base_docs_dir: "./custom-docs"
agents:
  cursor: true
  claude: true
"#;

        fs::write(&config_path, test_config_content).await.unwrap();

        let config = load_config_from_path(&config_path.to_string_lossy())
            .await
            .unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.base_docs_dir, "./custom-docs");
        assert!(config.agents.cursor.is_enabled());
        assert!(config.agents.claude.is_enabled());
    }

    #[tokio::test]
    async fn test_load_config_from_path_not_found() {
        let result = load_config_from_path("/nonexistent/config.yaml").await;
        assert!(result.is_err());

        if let Err(ConfigError::FileNotFound { path }) = result {
            assert_eq!(path, "/nonexistent/config.yaml");
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    #[tokio::test]
    async fn test_load_config_from_path_invalid_yaml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");

        let invalid_yaml = r#"
version: 1.0
invalid_yaml: [
"#;

        fs::write(&config_path, invalid_yaml).await.unwrap();

        let result = load_config_from_path(&config_path.to_string_lossy()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::YamlError { .. }));
    }

    #[tokio::test]
    async fn test_load_config_from_path_with_default_file() {
        // Default file path test
        let result = load_config_from_path(DEFAULT_CONFIG_FILE).await;

        // If default file exists, it should be loaded successfully, if not, FileNotFound error should be returned
        match result {
            Ok(config) => {
                // If file exists, confirm it's loaded properly
                assert!(!config.version.is_empty());
                assert!(!config.base_docs_dir.is_empty());
            }
            Err(ConfigError::FileNotFound { path }) => {
                // If file doesn't exist, FileNotFound error should be returned
                assert_eq!(path, DEFAULT_CONFIG_FILE);
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_validate_with_custom_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("validate-test-config.yaml");
        let docs_dir = temp_dir.path().join("validate-docs");

        // Create docs directory
        fs::create_dir_all(&docs_dir).await.unwrap();

        let test_config_content = format!(
            r#"
version: "1.0"
output_mode: split
base_docs_dir: "{}"
agents:
  cursor: true
  claude: true
"#,
            docs_dir.to_string_lossy()
        );

        fs::write(&config_path, test_config_content).await.unwrap();

        // Confirm handle_validate function works properly
        // Can't verify actual output in tests, but confirm no error occurs
        let result = handle_validate(Some(config_path.to_string_lossy().to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_validate_with_nonexistent_config() {
        // Test behavior when validate is executed with non-existent file
        let result = handle_validate(Some("/nonexistent/config.yaml".to_string())).await;
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Configuration file not found"));
    }

    #[tokio::test]
    async fn test_handle_validate_default_config() {
        // Execute test in temporary directory
        let temp_dir = tempdir().unwrap();

        // Save current working directory
        let original_dir = std::env::current_dir().unwrap();

        // Move to temporary directory during test execution
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = handle_validate(None).await;

        // Restore working directory
        std::env::set_current_dir(original_dir).unwrap();

        // Error should be returned when default file doesn't exist
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Configuration file not found"));
    }

    #[tokio::test]
    async fn test_handle_generate_with_nonexistent_docs_dir() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.yaml");
        let nonexistent_docs = temp_dir.path().join("nonexistent-docs");

        // Create configuration file with non-existent docs directory
        let config_content = format!(
            r#"
version: "1.0"
output_mode: merged
base_docs_dir: "{}"
agents:
  claude: true
"#,
            nonexistent_docs.to_string_lossy()
        );

        fs::write(&config_path, config_content).await.unwrap();

        let result = handle_generate(None, Some(config_path.to_string_lossy().to_string())).await;
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Documentation directory does not exist"));
        assert!(error_message.contains("nonexistent-docs"));
    }

    #[tokio::test]
    async fn test_handle_validate_with_nonexistent_docs_dir() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.yaml");
        let nonexistent_docs = temp_dir.path().join("nonexistent-docs");

        // Create configuration file with non-existent docs directory
        let config_content = format!(
            r#"
version: "1.0"
output_mode: split
base_docs_dir: "{}"
agents:
  claude: true
"#,
            nonexistent_docs.to_string_lossy()
        );

        fs::write(&config_path, config_content).await.unwrap();

        let result = handle_validate(Some(config_path.to_string_lossy().to_string())).await;
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Documentation directory does not exist"));
        assert!(error_message.contains("nonexistent-docs"));
    }

    #[tokio::test]
    async fn test_handle_generate_with_valid_docs_dir() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.yaml");
        let docs_dir = temp_dir.path().join("docs");

        // Create docs directory
        fs::create_dir_all(&docs_dir).await.unwrap();
        fs::write(docs_dir.join("test.md"), "# Test content")
            .await
            .unwrap();

        // Create configuration file with existing docs directory
        let config_content = format!(
            r#"
version: "1.0"
output_mode: merged
base_docs_dir: "{}"
agents:
  claude: true
"#,
            docs_dir.to_string_lossy()
        );

        fs::write(&config_path, config_content).await.unwrap();

        // Save current working directory
        let original_dir = std::env::current_dir().unwrap();

        // Move to temporary directory during test execution
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = handle_generate(None, Some(config_path.to_string_lossy().to_string())).await;

        // Restore working directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_validate_with_valid_docs_dir() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.yaml");
        let docs_dir = temp_dir.path().join("docs");

        // Create docs directory
        fs::create_dir_all(&docs_dir).await.unwrap();

        // Create configuration file with existing docs directory
        let config_content = format!(
            r#"
version: "1.0"
output_mode: split
base_docs_dir: "{}"
agents:
  claude: true
"#,
            docs_dir.to_string_lossy()
        );

        fs::write(&config_path, config_content).await.unwrap();

        let result = handle_validate(Some(config_path.to_string_lossy().to_string())).await;
        assert!(result.is_ok());
    }
}
