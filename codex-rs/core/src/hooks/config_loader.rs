//! Loading layered hook configuration files.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{HookDefinition, HookEvent, HookMatchers, HookScope};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use thiserror::Error;
use toml::Value as TomlValue;

use super::layer_summary::HookLayerSummary;
use super::skipped::{HookSkipReason, SkippedHook};

/// Placeholder loader until full implementation lands in T020.
#[derive(Debug, Default)]
pub struct HookConfigLoader;

#[derive(Debug, Error, PartialEq)]
pub enum HookConfigError {
    #[error("hook configuration validation not implemented")]
    NotImplemented,
    #[error("invalid hook configuration: {0}")]
    InvalidConfiguration(String),
    #[error("failed to read configuration: {0}")]
    Io(String),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HookFileToml {
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    #[serde(rename = "defaultTimeoutMs", default)]
    default_timeout_ms: Option<u64>,
    #[serde(default)]
    env: HashMap<String, String>,
    hooks: Vec<HookDefinitionToml>,
}

#[derive(Debug, Deserialize)]
struct HookDefinitionToml {
    id: String,
    event: HookEvent,
    #[serde(default)]
    notes: Option<String>,
    command: Vec<String>,
    #[serde(rename = "workingDir", default)]
    working_dir: Option<PathBuf>,
    #[serde(rename = "timeoutMs", default)]
    timeout_ms: Option<u64>,
    #[serde(rename = "allowParallel", default)]
    allow_parallel: bool,
    #[serde(rename = "schemaVersions")]
    schema_versions: Vec<String>,
    #[serde(default)]
    matchers: HookMatchers,
    #[serde(default)]
    env: HashMap<String, String>,
}

impl HookConfigLoader {
    /// Validate a hook configuration document represented as TOML text.
    pub fn validate_document(document: &str) -> Result<(), HookConfigError> {
        let parsed: HookFileToml = toml::from_str(document)
            .map_err(|err| HookConfigError::InvalidConfiguration(err.to_string()))?;

        if parsed.schema_version != "1.0" {
            return Err(HookConfigError::InvalidConfiguration(
                "unsupported schemaVersion".to_string(),
            ));
        }

        if parsed.hooks.is_empty() {
            return Ok(());
        }

        for hook in parsed.hooks.iter() {
            if hook.id.trim().is_empty() {
                return Err(HookConfigError::InvalidConfiguration(
                    "hook id must not be empty".to_string(),
                ));
            }

            if hook.command.is_empty() {
                return Err(HookConfigError::InvalidConfiguration(format!(
                    "hook `{}` missing command",
                    hook.id
                )));
            }

            if hook.schema_versions.is_empty() {
                return Err(HookConfigError::InvalidConfiguration(format!(
                    "hook `{}` missing schemaVersions",
                    hook.id
                )));
            }
        }

        Ok(())
    }

    /// Validate configuration by reading from the provided path.
    pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<(), HookConfigError> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|err| HookConfigError::Io(err.to_string()))?;
        Self::validate_document(&contents)
    }

    /// Translate legacy notify configuration into a synthetic Notification hook.
    pub fn synthesize_legacy_notify(config_toml: &str) -> Result<(), HookConfigError> {
        let value: TomlValue = toml::from_str(config_toml)
            .map_err(|err| HookConfigError::InvalidConfiguration(err.to_string()))?;
        let table = value.as_table().ok_or_else(|| {
            HookConfigError::InvalidConfiguration("config must be a TOML table".into())
        })?;

        if let Some(notifications) = table.get("notifications") {
            if let Some(notify) = notifications.get("notify") {
                if notify
                    .as_array()
                    .map(|arr| !arr.is_empty())
                    .unwrap_or(false)
                {
                    return Ok(());
                }
            }
        }

        Err(HookConfigError::InvalidConfiguration(
            "legacy notify configuration not found".to_string(),
        ))
    }

    /// Load hooks from a list of configuration files, returning registry entries and layer summaries.
    pub fn load_layers(
        sources: Vec<(HookScope, PathBuf)>,
    ) -> Result<(Vec<HookDefinition>, Vec<HookLayerSummary>), HookConfigError> {
        let mut definitions = Vec::new();
        let mut summaries = Vec::new();

        for (scope, path) in sources {
            let contents =
                fs::read_to_string(&path).map_err(|err| HookConfigError::Io(err.to_string()))?;
            let parsed: HookFileToml = toml::from_str(&contents)
                .map_err(|err| HookConfigError::InvalidConfiguration(err.to_string()))?;
            let mut summary = HookLayerSummary::new(scope.clone(), path.clone());
            summary.checksum = checksum(&contents);
            let mut duplicates = HashMap::new();

            for hook in parsed.hooks {
                if hook.id.trim().is_empty() {
                    summary.skipped_hooks.push(
                        SkippedHook::new(HookSkipReason::InvalidSchema)
                            .with_details("hook id must not be empty"),
                    );
                    continue;
                }

                if duplicates.insert(hook.id.clone(), ()).is_some() {
                    summary.skipped_hooks.push(
                        SkippedHook::new(HookSkipReason::DuplicateId).with_hook_id(hook.id.clone()),
                    );
                    continue;
                }

                if hook.schema_versions.is_empty() {
                    summary.skipped_hooks.push(
                        SkippedHook::new(HookSkipReason::InvalidSchema)
                            .with_hook_id(hook.id.clone())
                            .with_details("schemaVersions must not be empty"),
                    );
                    continue;
                }

                let mut definition = HookDefinition::default();
                definition.id = hook.id;
                definition.event = hook.event;
                definition.notes = hook.notes;
                definition.command = hook.command;
                definition.working_dir = hook.working_dir;
                definition.timeout_ms = hook.timeout_ms;
                definition.allow_parallel = hook.allow_parallel;
                definition.schema_versions = hook.schema_versions;
                definition.env = hook.env;
                definition.matchers = hook.matchers;
                definition.scope = scope.clone();
                definition.source_path = Some(path.clone());

                definitions.push(definition);
                summary.loaded_hooks += 1;
            }

            summaries.push(summary);
        }

        Ok((definitions, summaries))
    }
}

fn checksum(contents: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(contents.as_bytes());
    hex::encode(hasher.finalize())
}
