use std::path::Path;

use serde::Deserialize;

use crate::subagents::definition::{SubagentDefinition, SubagentScope};

#[derive(thiserror::Error, Debug)]
pub enum SubagentParserError {
    #[error("failed to parse subagent file: {0}")]
    ParseError(String),
}

#[derive(Debug, Deserialize, Default)]
struct SubagentFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tools: Option<Vec<String>>,
    #[serde(default)]
    model: Option<String>,
}

pub fn parse_definition(
    path: &Path,
    contents: &str,
    scope: SubagentScope,
) -> Result<SubagentDefinition, SubagentParserError> {
    let sanitized = trim_bom(contents);
    let (frontmatter_src, body) = extract_frontmatter(sanitized)?;
    let raw: SubagentFrontmatter = serde_yaml::from_str(&frontmatter_src)
        .map_err(|error| SubagentParserError::ParseError(error.to_string()))?;

    let fallback_name = fallback_name_from_path(path);
    let provided_name = raw
        .name
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_name.clone());

    let provided_description = raw
        .description
        .as_ref()
        .map(|value| value.trim())
        .unwrap_or_default()
        .to_string();

    let mut definition = SubagentDefinition::new(
        provided_name,
        provided_description,
        scope,
        path.to_path_buf(),
    );

    if let Some(tool_entries) = raw.tools {
        let mut tools = Vec::new();
        for tool in tool_entries {
            let trimmed = tool.trim();
            if trimmed.is_empty() {
                definition.add_validation_error("`tools` entries must be non-empty strings");
                continue;
            }
            if !tools.iter().any(|existing: &String| existing == trimmed) {
                tools.push(trimmed.to_string());
            }
        }
        definition = definition.with_tools(tools);
    }

    if let Some(model) = raw
        .model
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        definition = definition.with_model(Some(model.to_string()));
    }

    let trimmed_body = body.trim();
    if trimmed_body.is_empty() {
        definition
            .add_validation_error("subagent definition must include a Markdown instructions body");
    } else {
        definition.instructions = trimmed_body.to_string();
    }

    if raw
        .name
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        definition.add_validation_error("frontmatter is missing a non-empty `name` field");
    }

    if raw
        .description
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        definition.add_validation_error("frontmatter is missing a non-empty `description` field");
    }

    Ok(definition)
}

fn fallback_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| "subagent".to_string())
}

fn trim_bom(input: &str) -> &str {
    input.strip_prefix('\u{feff}').unwrap_or(input)
}

fn extract_frontmatter(contents: &str) -> Result<(String, &str), SubagentParserError> {
    let trimmed =
        contents.trim_start_matches(|ch: char| ch == '\n' || ch == '\r' || ch == ' ' || ch == '\t');
    if !trimmed.starts_with("---") {
        return Err(SubagentParserError::ParseError(
            "subagent definitions must start with `---` frontmatter".to_string(),
        ));
    }

    let mut remainder = &trimmed[3..];
    remainder = strip_leading_newline(remainder);

    let closing_marker = "\n---";
    let closing_index = remainder
        .find(closing_marker)
        .ok_or_else(|| SubagentParserError::ParseError("unterminated YAML frontmatter".into()))?;
    let frontmatter_slice = &remainder[..closing_index];
    let mut body = &remainder[closing_index + closing_marker.len()..];
    body = strip_leading_newline(body);

    Ok((frontmatter_slice.replace('\r', ""), body))
}

fn strip_leading_newline(input: &str) -> &str {
    if let Some(stripped) = input.strip_prefix("\r\n") {
        stripped
    } else if let Some(stripped) = input.strip_prefix('\n') {
        stripped
    } else if let Some(stripped) = input.strip_prefix('\r') {
        stripped
    } else {
        input
    }
}
