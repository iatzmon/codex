use crate::errors::SlashCommandError;
use crate::models::command::Command;
use crate::models::scope::CommandScope;
use crate::namespace::build_namespace_components;
use crate::parsing::parse_template;
use crate::registry::CommandRegistry;
use std::path::Path;
use tokio::fs;
use tokio::io;

pub async fn discover_commands(
    registry: &mut CommandRegistry,
    scope: CommandScope,
    dir: &Path,
) -> Result<usize, SlashCommandError> {
    let metadata = match fs::metadata(dir).await {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(err.into()),
    };

    if !metadata.is_dir() {
        return Ok(0);
    }

    scan_directory(registry, scope, dir).await
}

async fn scan_directory(
    registry: &mut CommandRegistry,
    scope: CommandScope,
    root: &Path,
) -> Result<usize, SlashCommandError> {
    let mut stack = vec![root.to_path_buf()];
    let mut inserted = 0usize;
    while let Some(current) = stack.pop() {
        let mut entries = fs::read_dir(&current).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() || !is_markdown(&path) {
                continue;
            }
            let contents = fs::read_to_string(&path).await?;
            let parsed = parse_template(&contents)?;
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let namespace_path = relative.parent().unwrap_or(Path::new(""));
            let namespace = build_namespace_components(namespace_path)?;
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    SlashCommandError::InvalidTemplate(format!(
                        "command file name is not valid UTF-8: {}",
                        path.display()
                    ))
                })?
                .to_string();
            let command = Command {
                scope,
                namespace,
                name,
                metadata: parsed.metadata,
                body: parsed.body,
                path: path.clone(),
            };
            registry.insert(command)?;
            inserted += 1;
        }
    }
    Ok(inserted)
}

fn is_markdown(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => {
            let lower = ext.to_ascii_lowercase();
            lower == "md" || lower == "markdown"
        }
        None => false,
    }
}
