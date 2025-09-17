use crate::models::metadata::FrontmatterMetadata;
use crate::models::scope::CommandScope;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub scope: CommandScope,
    pub namespace: Vec<String>,
    pub name: String,
    pub metadata: FrontmatterMetadata,
    pub body: String,
    pub path: PathBuf,
}

impl Command {
    pub fn qualified_name(&self) -> String {
        if self.namespace.is_empty() {
            return self.name.clone();
        }
        let mut parts = self.namespace.clone();
        parts.push(self.name.clone());
        parts.join(":")
    }

    pub fn full_name(&self) -> String {
        format!("{}:{}", self.scope.as_str(), self.qualified_name())
    }
}
