use crate::errors::SlashCommandError;
use std::path::Component;
use std::path::Path;

pub fn build_namespace_components(dir: &Path) -> Result<Vec<String>, SlashCommandError> {
    let mut components = Vec::new();
    for component in dir.components() {
        match component {
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {
                continue;
            }
            Component::ParentDir => {
                return Err(SlashCommandError::InvalidNamespace {
                    component: "..".to_string(),
                });
            }
            Component::Normal(os_str) => {
                let segment = os_str
                    .to_str()
                    .ok_or_else(|| SlashCommandError::InvalidNamespace {
                        component: os_str.to_string_lossy().into_owned(),
                    })?
                    .trim();
                if segment.is_empty() {
                    continue;
                }
                if segment.contains(':') {
                    return Err(SlashCommandError::InvalidNamespace {
                        component: segment.to_string(),
                    });
                }
                components.push(segment.to_string());
            }
        }
    }
    Ok(components)
}
