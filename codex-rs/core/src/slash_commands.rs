use codex_slash_commands::Command;
use codex_slash_commands::CommandLookup;
use codex_slash_commands::CommandRegistry;
use codex_slash_commands::InterpolationContext;
use codex_slash_commands::SlashCommandConfig;
use codex_slash_commands::SlashCommandError;
use codex_slash_commands::interpolate_template;
use codex_slash_commands::parse_command_line;
use tokio::sync::RwLock;

use crate::config::Config;

#[derive(Debug, Clone)]
pub(crate) struct CommandInvocation {
    pub command: Command,
    pub rendered_body: String,
}

#[derive(Debug, Clone)]
pub(crate) enum InvocationError {
    NotCommand,
    NotFound {
        name: String,
        suggestions: Vec<String>,
    },
    Ambiguous {
        #[allow(dead_code)]
        name: String,
        matches: Vec<String>,
    },
    Interpolation(String),
}

pub(crate) struct SlashCommandService {
    registry: RwLock<CommandRegistry>,
    #[allow(dead_code)]
    config: SlashCommandConfig,
}

impl SlashCommandService {
    pub(crate) async fn new(config: &Config) -> Result<Self, SlashCommandError> {
        let slash_config = SlashCommandConfig::from_environment(
            Some(config.cwd.clone()),
            Some(config.codex_home.clone()),
        );
        let registry = CommandRegistry::load(&slash_config).await?;
        Ok(Self {
            registry: RwLock::new(registry),
            config: slash_config,
        })
    }

    #[allow(dead_code)]
    pub(crate) async fn reload(&self) -> Result<usize, SlashCommandError> {
        let mut guard = self.registry.write().await;
        guard.reload(&self.config).await
    }

    #[allow(dead_code)]
    pub(crate) async fn list_commands(&self) -> Vec<Command> {
        let guard = self.registry.read().await;
        guard.all().into_iter().cloned().collect()
    }

    pub(crate) async fn resolve(&self, input: &str) -> Result<CommandInvocation, InvocationError> {
        let (name, args) = match parse_command_line(input) {
            Some((name, args)) if !name.is_empty() => (name, args),
            _ => return Err(InvocationError::NotCommand),
        };

        let guard = self.registry.read().await;
        match guard.lookup(&name) {
            CommandLookup::NotFound { suggestions } => {
                Err(InvocationError::NotFound { name, suggestions })
            }
            CommandLookup::Ambiguous { matches } => {
                Err(InvocationError::Ambiguous { name, matches })
            }
            CommandLookup::Command(command) => {
                let ctx = InterpolationContext::new(args);
                interpolate_template(&command.body, &ctx)
                    .map(|rendered_body| CommandInvocation {
                        command,
                        rendered_body,
                    })
                    .map_err(|err| InvocationError::Interpolation(err.to_string()))
            }
        }
    }
}
