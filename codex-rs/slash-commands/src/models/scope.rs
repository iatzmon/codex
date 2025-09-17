use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandScope {
    User,
    Project,
}

impl CommandScope {
    pub fn as_str(self) -> &'static str {
        match self {
            CommandScope::User => "user",
            CommandScope::Project => "project",
        }
    }
}

impl fmt::Display for CommandScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
