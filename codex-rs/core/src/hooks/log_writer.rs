//! JSONL execution log writer for hook runs.

use std::path::{Path, PathBuf};

use super::HookExecutionRecord;
use serde_json::Value;
use thiserror::Error;
use tokio::fs::{OpenOptions, create_dir_all};
use tokio::io::AsyncWriteExt;

/// Append-only JSONL writer for hook execution records.
#[derive(Debug, Clone)]
pub struct HookLogWriter {
    path: PathBuf,
}

impl HookLogWriter {
    /// Create a new writer that appends to the provided path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Absolute path to the JSONL log file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append the given record as a JSON line, creating parent directories
    /// and the log file if necessary.
    pub async fn append(&self, record: &HookExecutionRecord) -> Result<(), HookLogWriterError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                create_dir_all(parent).await?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        let mut value = serde_json::to_value(record)?;
        scrub_private_fields(&mut value);

        let line = serde_json::to_string(&value)?;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        Ok(())
    }
}

/// Remove fields that should not be persisted to disk (e.g. synthesized data).
fn scrub_private_fields(value: &mut Value) {
    if let Some(obj) = value.as_object_mut() {
        if let Some(extra) = obj.get_mut("extra") {
            if extra.is_null() {
                obj.remove("extra");
            }
        }
    }
}

/// Errors produced while writing hook execution logs.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HookLogWriterError {
    #[error("failed to serialize hook record: {0}")]
    Serialize(String),
    #[error("failed to write hook execution log: {0}")]
    Io(String),
}

impl From<serde_json::Error> for HookLogWriterError {
    fn from(err: serde_json::Error) -> Self {
        HookLogWriterError::Serialize(err.to_string())
    }
}

impl From<std::io::Error> for HookLogWriterError {
    fn from(err: std::io::Error) -> Self {
        HookLogWriterError::Io(err.to_string())
    }
}
