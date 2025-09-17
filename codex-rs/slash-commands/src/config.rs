use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct SlashCommandConfig {
    pub project_dir: Option<PathBuf>,
    pub user_dir: Option<PathBuf>,
}

impl SlashCommandConfig {
    pub fn from_environment(project_root: Option<PathBuf>, codex_home: Option<PathBuf>) -> Self {
        let project_dir = env::var("CODEX_SLASH_COMMANDS_DIR_PROJECT")
            .ok()
            .and_then(|value| normalize_override(&value))
            .or_else(|| project_root.map(|root| root.join(".codex/commands")));

        let user_dir = env::var("CODEX_SLASH_COMMANDS_DIR_USER")
            .ok()
            .and_then(|value| normalize_override(&value))
            .or_else(|| codex_home.map(|home| home.join("commands")))
            .or_else(|| dirs::home_dir().map(|home| home.join(".codex/commands")));

        Self {
            project_dir,
            user_dir,
        }
    }
}

fn normalize_override(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}
