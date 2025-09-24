use std::path::PathBuf;

use crate::config::Config;

use super::builder::SubagentBuilder;
use super::discovery::SubagentSourceTree;
use super::inventory::SubagentInventory;

/// Build the subagent source tree from the provided configuration. This
/// captures the project and user directories that should be scanned when
/// discovering Markdown definitions.
pub fn source_tree_from_config(config: &Config) -> SubagentSourceTree {
    let mut tree = SubagentSourceTree::default();

    let mut project_paths = vec![config.cwd().join(".codex/agents")];
    let mut user_paths = vec![config.codex_home().join("agents")];
    dedup_paths(&mut project_paths);
    dedup_paths(&mut user_paths);

    tree.project = project_paths;
    tree.user = user_paths;
    tree
}

/// Discover subagent definitions for the provided configuration and construct
/// the resulting inventory, including precedence conflicts and validation
/// diagnostics.
pub fn build_inventory_for_config(config: &Config) -> SubagentInventory {
    let sources = source_tree_from_config(config);
    SubagentBuilder::new(config.subagents.clone())
        .discover_tree(&sources)
        .build()
}

fn dedup_paths(paths: &mut Vec<PathBuf>) {
    paths.sort();
    paths.dedup();
}
