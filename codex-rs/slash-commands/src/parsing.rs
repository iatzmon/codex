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

fn strip_leading_blank_lines(s: &str) -> &str {
    let mut offset = 0;
    for segment in s.split_inclusive('\n') {
        let without_newline = segment.trim_end_matches(|c| c == '\n' || c == '\r');
        if without_newline.trim().is_empty() {
            offset += segment.len();
            continue;
        }
        break;
    }

    &s[offset..]
}

pub fn parse_template(raw: &str) -> Result<ParsedTemplate, SlashCommandError> {
    let no_bom = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    let frontmatter_check = strip_leading_blank_lines(no_bom);

    if frontmatter_check.is_empty() {
        return Ok(ParsedTemplate {
            metadata: FrontmatterMetadata::default(),
            body: raw.to_string(),
        });
    }

    if !frontmatter_check.starts_with("---") {
        return Ok(ParsedTemplate {
            metadata: FrontmatterMetadata::default(),
            body: raw.to_string(),
        });
    }

    // Split frontmatter and body by locating the terminating delimiter.
    let mut lines = frontmatter_check.lines();
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
        RawFrontmatter::default()
    } else {
        serde_yaml::from_str::<RawFrontmatter>(&frontmatter_src).map_err(|error| {
            SlashCommandError::InvalidTemplate(format!("failed to parse frontmatter: {error}"))
        })?
    };

    let body = if body_lines.is_empty() {
        String::new()
    } else {
        body_lines.join("\n")
    };

    let metadata = FrontmatterMetadata {
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
    };

    Ok(ParsedTemplate { metadata, body })
}
