use crate::errors::SlashCommandError;
use crate::models::metadata::FrontmatterMetadata;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ParsedTemplate {
    pub metadata: FrontmatterMetadata,
    pub body: String,
}

#[derive(Debug, Deserialize, Default)]
struct RawFrontmatter {
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "argument_hint", alias = "argument-hint")]
    argument_hint: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default, rename = "allowed_tools", alias = "allowed-tools")]
    allowed_tools: Option<Vec<String>>,
}

pub fn parse_template(raw: &str) -> Result<ParsedTemplate, SlashCommandError> {
    let trimmed = raw.trim_start_matches('\u{FEFF}').trim_start();
    if trimmed.is_empty() {
        return Ok(ParsedTemplate {
            metadata: FrontmatterMetadata::default(),
            body: String::new(),
        });
    }

    if !trimmed.starts_with("---") {
        return Ok(ParsedTemplate {
            metadata: FrontmatterMetadata::default(),
            body: trimmed.to_string(),
        });
    }

    // Split frontmatter and body by locating the terminating delimiter.
    let mut lines = trimmed.lines();
    let _ = lines.next(); // skip the initial --- line
    let mut frontmatter_lines: Vec<&str> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_frontmatter = true;
    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            frontmatter_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    if in_frontmatter {
        return Err(SlashCommandError::InvalidTemplate(
            "missing closing frontmatter delimiter".to_string(),
        ));
    }

    let frontmatter_src = frontmatter_lines.join("\n");
    let raw_meta = if frontmatter_src.trim().is_empty() {
        Ok(RawFrontmatter::default())
    } else {
        serde_yaml::from_str::<RawFrontmatter>(&frontmatter_src)
    };

    let body = if body_lines.is_empty() {
        String::new()
    } else {
        body_lines.join("\n")
    };

    let metadata = match raw_meta {
        Ok(raw_meta) => FrontmatterMetadata {
            description: raw_meta.description.map(|s| s.trim().to_string()),
            argument_hint: raw_meta.argument_hint.map(|s| s.trim().to_string()),
            model: raw_meta.model.map(|s| s.trim().to_string()),
            allowed_tools: raw_meta.allowed_tools.map(|tools| {
                tools
                    .into_iter()
                    .map(|tool| tool.trim().to_string())
                    .filter(|tool| !tool.is_empty())
                    .collect()
            }),
        },
        Err(_) => FrontmatterMetadata::default(),
    };

    Ok(ParsedTemplate { metadata, body })
}
