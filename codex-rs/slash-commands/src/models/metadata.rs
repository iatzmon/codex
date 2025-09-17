#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontmatterMetadata {
    pub description: Option<String>,
    pub argument_hint: Option<String>,
    pub model: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
}
