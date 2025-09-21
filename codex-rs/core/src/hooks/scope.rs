//! Hook scope precedence definitions.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Origin of a hook definition with precedence semantics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HookScope {
    ManagedPolicy { name: String },
    Project { project_root: PathBuf },
    LocalUser { codex_home: PathBuf },
}

impl Default for HookScope {
    fn default() -> Self {
        HookScope::LocalUser {
            codex_home: PathBuf::new(),
        }
    }
}
