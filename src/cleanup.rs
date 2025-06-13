/*!
 * Agent file cleanup module
 *
 * Handles cleanup of output files for disabled agents
 */

use crate::types::config::AIContextConfig;
use crate::types::config::AgentConfigTrait;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Agent file cleaner
pub struct AgentCleaner;

impl AgentCleaner {
    /// Clean up files for all disabled agents
    pub fn cleanup_disabled_agents(
        config: &AIContextConfig,
        base_dir: Option<&Path>,
    ) -> Result<Vec<String>> {
        let mut cleaned_files = Vec::new();

        // Check each agent and clean up if disabled
        if !config.agents.cursor.is_enabled() {
            cleaned_files.extend(Self::cleanup_cursor_files(base_dir)?);
        }
        if !config.agents.cline.is_enabled() {
            cleaned_files.extend(Self::cleanup_cline_files(base_dir)?);
        }
        if !config.agents.github.is_enabled() {
            cleaned_files.extend(Self::cleanup_github_files(base_dir)?);
        }
        if !config.agents.claude.is_enabled() {
            cleaned_files.extend(Self::cleanup_claude_files(base_dir)?);
        }
        if !config.agents.codex.is_enabled() {
            cleaned_files.extend(Self::cleanup_codex_files(base_dir)?);
        }

        Ok(cleaned_files)
    }

    /// Clean up Cursor agent files
    fn cleanup_cursor_files(base_dir: Option<&Path>) -> Result<Vec<String>> {
        let mut cleaned = Vec::new();
        let base = base_dir.unwrap_or_else(|| Path::new("."));
        let rules_dir = base.join(".cursor/rules");

        if rules_dir.exists() {
            // Remove all .mdc files in the rules directory
            let entries = fs::read_dir(&rules_dir)
                .with_context(|| format!("Failed to read directory: {}", rules_dir.display()))?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().is_some_and(|ext| ext == "mdc") {
                    match fs::remove_file(&path) {
                        Ok(_) => {
                            cleaned.push(path.display().to_string());
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to remove {}: {}", path.display(), e);
                        }
                    }
                }
            }

            // Remove directory if empty
            if rules_dir.read_dir()?.next().is_none() {
                match fs::remove_dir(&rules_dir) {
                    Ok(_) => {
                        cleaned.push(format!("{} (directory)", rules_dir.display()));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to remove directory {}: {}",
                            rules_dir.display(),
                            e
                        );
                    }
                }

                // Also try to remove .cursor directory if it's empty
                let cursor_dir = base.join(".cursor");
                if cursor_dir.exists() && cursor_dir.read_dir()?.next().is_none() {
                    match fs::remove_dir(&cursor_dir) {
                        Ok(_) => {
                            cleaned.push(format!("{} (directory)", cursor_dir.display()));
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to remove directory {}: {}",
                                cursor_dir.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(cleaned)
    }

    /// Clean up Cline agent files
    fn cleanup_cline_files(base_dir: Option<&Path>) -> Result<Vec<String>> {
        let mut cleaned = Vec::new();
        let base = base_dir.unwrap_or_else(|| Path::new("."));

        // Check for .clinerules file (merged mode)
        let clinerules_file = base.join(".clinerules");
        if clinerules_file.is_file() {
            match fs::remove_file(&clinerules_file) {
                Ok(_) => {
                    cleaned.push(clinerules_file.display().to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to remove {}: {}",
                        clinerules_file.display(),
                        e
                    );
                }
            }
        }

        // Check for .clinerules directory (split mode)
        let clinerules_dir = base.join(".clinerules");
        if clinerules_dir.is_dir() {
            // Remove all .md files in the directory
            let entries = fs::read_dir(&clinerules_dir).with_context(|| {
                format!("Failed to read directory: {}", clinerules_dir.display())
            })?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                    match fs::remove_file(&path) {
                        Ok(_) => {
                            cleaned.push(path.display().to_string());
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to remove {}: {}", path.display(), e);
                        }
                    }
                }
            }

            // Remove directory if empty
            if clinerules_dir.read_dir()?.next().is_none() {
                match fs::remove_dir(&clinerules_dir) {
                    Ok(_) => {
                        cleaned.push(format!("{} (directory)", clinerules_dir.display()));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to remove directory {}: {}",
                            clinerules_dir.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(cleaned)
    }

    /// Clean up GitHub agent files
    fn cleanup_github_files(base_dir: Option<&Path>) -> Result<Vec<String>> {
        let mut cleaned = Vec::new();
        let base = base_dir.unwrap_or_else(|| Path::new("."));

        // Check for merged mode file
        let copilot_instructions = base.join(".github/copilot-instructions.md");
        if copilot_instructions.is_file() {
            match fs::remove_file(&copilot_instructions) {
                Ok(_) => {
                    cleaned.push(copilot_instructions.display().to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to remove {}: {}",
                        copilot_instructions.display(),
                        e
                    );
                }
            }
        }

        // Check for split mode directory
        let instructions_dir = base.join(".github/instructions");
        if instructions_dir.is_dir() {
            // Remove all .instructions.md files in the directory
            let entries = fs::read_dir(&instructions_dir).with_context(|| {
                format!("Failed to read directory: {}", instructions_dir.display())
            })?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    if filename.ends_with(".instructions.md") {
                        match fs::remove_file(&path) {
                            Ok(_) => {
                                cleaned.push(path.display().to_string());
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to remove {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }

            // Remove directory if empty
            if instructions_dir.read_dir()?.next().is_none() {
                match fs::remove_dir(&instructions_dir) {
                    Ok(_) => {
                        cleaned.push(format!("{} (directory)", instructions_dir.display()));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to remove directory {}: {}",
                            instructions_dir.display(),
                            e
                        );
                    }
                }

                // Also try to remove .github directory if it's empty
                let github_dir = base.join(".github");
                if github_dir.exists() && github_dir.read_dir()?.next().is_none() {
                    match fs::remove_dir(&github_dir) {
                        Ok(_) => {
                            cleaned.push(format!("{} (directory)", github_dir.display()));
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to remove directory {}: {}",
                                github_dir.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(cleaned)
    }

    /// Clean up Claude agent files
    fn cleanup_claude_files(base_dir: Option<&Path>) -> Result<Vec<String>> {
        let mut cleaned = Vec::new();
        let base = base_dir.unwrap_or_else(|| Path::new("."));

        let claude_file = base.join("CLAUDE.md");
        if claude_file.is_file() {
            match fs::remove_file(&claude_file) {
                Ok(_) => {
                    cleaned.push(claude_file.display().to_string());
                }
                Err(e) => {
                    eprintln!("Warning: Failed to remove {}: {}", claude_file.display(), e);
                }
            }
        }

        Ok(cleaned)
    }

    /// Clean up Codex agent files
    fn cleanup_codex_files(base_dir: Option<&Path>) -> Result<Vec<String>> {
        let mut cleaned = Vec::new();
        let base = base_dir.unwrap_or_else(|| Path::new("."));

        let codex_file = base.join("AGENTS.md");
        if codex_file.is_file() {
            match fs::remove_file(&codex_file) {
                Ok(_) => {
                    cleaned.push(codex_file.display().to_string());
                }
                Err(e) => {
                    eprintln!("Warning: Failed to remove {}: {}", codex_file.display(), e);
                }
            }
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::AIContextConfig;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(agents_config: &str) -> AIContextConfig {
        let config_content = format!(
            r#"
version: "1.0"
output_mode: split
base_docs_dir: ./ai-context
agents:
{}
"#,
            agents_config
        );

        serde_yaml::from_str(&config_content).expect("Failed to parse test config")
    }

    #[test]
    fn test_cleanup_cursor_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create cursor files
        let cursor_dir = base_path.join(".cursor/rules");
        fs::create_dir_all(&cursor_dir)?;
        fs::write(cursor_dir.join("test1.mdc"), "test content")?;
        fs::write(cursor_dir.join("test2.mdc"), "test content")?;
        fs::write(cursor_dir.join("other.txt"), "other content")?; // Should not be deleted

        let cleaned = AgentCleaner::cleanup_cursor_files(Some(base_path))?;

        assert_eq!(cleaned.len(), 2); // Only .mdc files should be cleaned
        assert!(!cursor_dir.join("test1.mdc").exists());
        assert!(!cursor_dir.join("test2.mdc").exists());
        assert!(cursor_dir.join("other.txt").exists()); // Should still exist

        Ok(())
    }

    #[test]
    fn test_cleanup_cline_files_merged() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create cline file (merged mode)
        fs::write(base_path.join(".clinerules"), "test content")?;

        let cleaned = AgentCleaner::cleanup_cline_files(Some(base_path))?;

        assert_eq!(cleaned.len(), 1);
        assert!(!base_path.join(".clinerules").exists());

        Ok(())
    }

    #[test]
    fn test_cleanup_cline_files_split() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create cline directory (split mode)
        let cline_dir = base_path.join(".clinerules");
        fs::create_dir_all(&cline_dir)?;
        fs::write(cline_dir.join("test1.md"), "test content")?;
        fs::write(cline_dir.join("test2.md"), "test content")?;

        let cleaned = AgentCleaner::cleanup_cline_files(Some(base_path))?;

        assert_eq!(cleaned.len(), 3); // 2 files + 1 directory
        assert!(!cline_dir.exists());

        Ok(())
    }

    #[test]
    fn test_cleanup_github_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create github files
        let github_dir = base_path.join(".github");
        fs::create_dir_all(&github_dir)?;
        fs::write(github_dir.join("copilot-instructions.md"), "merged content")?;

        let instructions_dir = github_dir.join("instructions");
        fs::create_dir_all(&instructions_dir)?;
        fs::write(
            instructions_dir.join("test1.instructions.md"),
            "split content",
        )?;
        fs::write(
            instructions_dir.join("test2.instructions.md"),
            "split content",
        )?;

        let cleaned = AgentCleaner::cleanup_github_files(Some(base_path))?;

        assert!(cleaned.len() >= 3); // At least merged file + 2 split files
        assert!(!github_dir.join("copilot-instructions.md").exists());
        assert!(!instructions_dir.join("test1.instructions.md").exists());
        assert!(!instructions_dir.join("test2.instructions.md").exists());

        Ok(())
    }

    #[test]
    fn test_cleanup_claude_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create claude file
        fs::write(base_path.join("CLAUDE.md"), "claude content")?;

        let cleaned = AgentCleaner::cleanup_claude_files(Some(base_path))?;

        assert_eq!(cleaned.len(), 1);
        assert!(!base_path.join("CLAUDE.md").exists());

        Ok(())
    }

    #[test]
    fn test_cleanup_codex_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create codex file
        fs::write(base_path.join("AGENTS.md"), "codex content")?;

        let cleaned = AgentCleaner::cleanup_codex_files(Some(base_path))?;

        assert_eq!(cleaned.len(), 1);
        assert!(!base_path.join("AGENTS.md").exists());

        Ok(())
    }

    #[test]
    fn test_cleanup_disabled_agents() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create files for all agents
        let cursor_dir = base_path.join(".cursor/rules");
        fs::create_dir_all(&cursor_dir)?;
        fs::write(cursor_dir.join("test.mdc"), "cursor content")?;
        fs::write(base_path.join("CLAUDE.md"), "claude content")?;
        fs::write(base_path.join("AGENTS.md"), "codex content")?;

        // Config with cursor and claude disabled, codex enabled
        let config = create_test_config(
            r#"
  cursor: false
  cline: true
  github: true
  claude: false
  codex: true
"#,
        );

        let cleaned = AgentCleaner::cleanup_disabled_agents(&config, Some(base_path))?;

        // Should clean cursor and claude files, but not codex
        assert!(cleaned.len() >= 2);
        assert!(!cursor_dir.join("test.mdc").exists());
        assert!(!base_path.join("CLAUDE.md").exists());
        assert!(base_path.join("AGENTS.md").exists()); // Should still exist

        Ok(())
    }
}
