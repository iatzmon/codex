use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SlashCommandError {
    #[error("I/O error while reading slash commands: {0}")]
    Io(#[from] io::Error),

    #[error("duplicate command '{name}' detected")]
    DuplicateCommand { name: String },

    #[error("invalid namespace component '{component}'")]
    InvalidNamespace { component: String },

    #[error("invalid template: {0}")]
    InvalidTemplate(String),

    #[error("interpolation error: {0}")]
    Interpolation(String),
}
