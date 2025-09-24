use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubagentScope {
    Project,
    User,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentDefinition {
    pub raw_name: String,
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub instructions: String,
    pub scope: SubagentScope,
    pub source_path: PathBuf,
    pub validation_errors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentValidationError {
    pub message: String,
}

impl SubagentValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl SubagentDefinition {
    pub fn new(
        raw_name: impl Into<String>,
        description: impl Into<String>,
        scope: SubagentScope,
        source_path: PathBuf,
    ) -> Self {
        let raw_name = raw_name.into();
        let description = description.into();
        let name = Self::normalize_name(&raw_name);

        let mut validation_errors = Vec::new();
        if raw_name.trim().is_empty() {
            validation_errors.push("name is required".to_string());
        }
        if description.trim().is_empty() {
            validation_errors.push("description is required".to_string());
        }
        if name.is_empty() {
            validation_errors.push("normalized name is empty".to_string());
        }

        Self {
            raw_name,
            name,
            description,
            tools: Vec::new(),
            model: None,
            instructions: String::new(),
            scope,
            source_path,
            validation_errors,
        }
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_model(mut self, model: Option<String>) -> Self {
        self.model = model;
        self
    }

    pub fn add_validation_error(&mut self, message: impl Into<String>) {
        self.validation_errors.push(message.into());
    }

    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    pub fn normalize_name(raw: &str) -> String {
        let mut normalized = String::new();
        let mut seen_separator = false;

        for ch in raw.chars() {
            let lower = ch.to_ascii_lowercase();
            if lower.is_ascii_alphanumeric() {
                normalized.push(lower);
                seen_separator = false;
            } else if matches!(lower, '-' | '_' | ' ' | '\t' | '\n' | '\r') {
                if !normalized.is_empty() && !seen_separator {
                    normalized.push('-');
                    seen_separator = true;
                }
            } else {
                if !normalized.is_empty() && !seen_separator {
                    normalized.push('-');
                    seen_separator = true;
                }
            }
        }

        while normalized.ends_with('-') {
            normalized.pop();
        }

        normalized
    }

    pub fn is_valid(&self) -> bool {
        self.validation_errors.is_empty()
    }
}
