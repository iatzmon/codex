use crate::config::SlashCommandConfig;
use crate::discovery::discover_commands;
use crate::errors::SlashCommandError;
use crate::models::command::Command;
use crate::models::scope::CommandScope;
use crate::performance;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Default)]
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    qualified_index: HashMap<String, Vec<String>>,
    last_loaded: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub enum CommandLookup {
    NotFound { suggestions: Vec<String> },
    Ambiguous { matches: Vec<String> },
    Command(Command),
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            qualified_index: HashMap::new(),
            last_loaded: None,
        }
    }

    pub async fn load(config: &SlashCommandConfig) -> Result<Self, SlashCommandError> {
        let mut registry = CommandRegistry::new();
        registry.reload(config).await?;
        Ok(registry)
    }

    pub async fn reload(
        &mut self,
        config: &SlashCommandConfig,
    ) -> Result<usize, SlashCommandError> {
        self.commands.clear();
        self.qualified_index.clear();

        let mut loaded = 0usize;
        if let Some(dir) = config.project_dir.as_ref() {
            loaded += discover_commands(self, CommandScope::Project, dir).await?;
        }
        if let Some(dir) = config.user_dir.as_ref() {
            loaded += discover_commands(self, CommandScope::User, dir).await?;
        }
        self.last_loaded = Some(SystemTime::now());
        performance::record_load_metrics();
        Ok(loaded)
    }

    pub fn lookup(&self, name: &str) -> CommandLookup {
        let normalized = normalize_query(name);
        if normalized.is_empty() {
            return CommandLookup::NotFound {
                suggestions: Vec::new(),
            };
        }

        if let Some(command) = self.commands.get(&normalized) {
            return CommandLookup::Command(command.clone());
        }

        if let Some(matches) = self.qualified_index.get(&normalized) {
            if matches.len() == 1 {
                if let Some(cmd) = matches.first().and_then(|full| self.commands.get(full)) {
                    return CommandLookup::Command(cmd.clone());
                }
            } else if !matches.is_empty() {
                let mut names = matches.clone();
                names.sort();
                names.dedup();
                return CommandLookup::Ambiguous { matches: names };
            }
        }

        CommandLookup::NotFound {
            suggestions: self.suggestions_for(&normalized),
        }
    }

    pub fn insert(&mut self, command: Command) -> Result<(), SlashCommandError> {
        let full_name = command.full_name();
        if self.commands.contains_key(&full_name) {
            return Err(SlashCommandError::DuplicateCommand { name: full_name });
        }
        let qualified = command.qualified_name();
        self.commands.insert(full_name.clone(), command);
        self.qualified_index
            .entry(qualified)
            .or_default()
            .push(full_name);
        Ok(())
    }

    pub fn all(&self) -> Vec<&Command> {
        let mut commands: Vec<&Command> = self.commands.values().collect();
        commands.sort_by_key(|a| a.full_name());
        commands
    }

    pub fn last_loaded(&self) -> Option<SystemTime> {
        self.last_loaded
    }

    fn suggestions_for(&self, normalized: &str) -> Vec<String> {
        if normalized.is_empty() {
            return Vec::new();
        }
        let mut suggestions: Vec<String> = Vec::new();
        for key in self.qualified_index.keys() {
            if key.starts_with(normalized) {
                suggestions.extend(self.qualified_index[key].clone());
            }
        }
        if suggestions.is_empty() {
            for name in self.commands.keys() {
                if name.starts_with(normalized) {
                    suggestions.push(name.clone());
                }
            }
        }
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(8);
        suggestions
    }
}

fn normalize_query(input: &str) -> String {
    let trimmed = input.trim();
    let without_slash = trimmed.trim_start_matches('/');
    let token = without_slash.split_whitespace().next().unwrap_or("");
    token.to_string()
}
