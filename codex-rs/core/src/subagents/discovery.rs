use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::subagents::definition::{SubagentDefinition, SubagentScope};
use crate::subagents::inventory::DiscoveryEvent;
use crate::subagents::parser::{SubagentParserError, parse_definition};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscoverySource {
    Project(PathBuf),
    User(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SubagentSourceTree {
    pub project: Vec<PathBuf>,
    pub user: Vec<PathBuf>,
}

impl SubagentSourceTree {
    pub fn add_project<P: Into<PathBuf>>(&mut self, path: P) {
        self.project.push(path.into());
    }

    pub fn add_user<P: Into<PathBuf>>(&mut self, path: P) {
        self.user.push(path.into());
    }

    pub fn is_empty(&self) -> bool {
        self.project.is_empty() && self.user.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiscoveryOutcome {
    pub definitions: Vec<SubagentDefinition>,
    pub events: Vec<DiscoveryEvent>,
}

pub fn discover_from_source(source: DiscoverySource) -> DiscoveryOutcome {
    let (root, scope) = match source {
        DiscoverySource::Project(path) => (path, SubagentScope::Project),
        DiscoverySource::User(path) => (path, SubagentScope::User),
    };

    let mut outcome = DiscoveryOutcome::default();

    if !root.exists() {
        return outcome;
    }

    for entry in WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.into_path();
        if !is_markdown(&path) {
            continue;
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match parse_definition(&path, &contents, scope) {
                Ok(definition) => outcome.definitions.push(definition),
                Err(SubagentParserError::ParseError(message)) => {
                    outcome.events.push(DiscoveryEvent {
                        message: format!(
                            "Failed to parse subagent definition at {}: {}",
                            path.display(),
                            message
                        ),
                    });
                }
            },
            Err(error) => outcome.events.push(DiscoveryEvent {
                message: format!(
                    "Failed to read subagent definition at {}: {}",
                    path.display(),
                    error
                ),
            }),
        }
    }

    outcome
}

pub fn detect_scope(path: &PathBuf) -> SubagentScope {
    if let Some(home) = dirs::home_dir() {
        let user_root = home.join(".codex/agents");
        if path.starts_with(&user_root) {
            return SubagentScope::User;
        }
    }

    SubagentScope::Project
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_ascii_lowercase();
            matches!(ext_lower.as_str(), "md" | "markdown")
        })
        .unwrap_or(false)
}
