#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InterpolationContext {
    arguments: Vec<String>,
    concatenated: String,
}

impl InterpolationContext {
    pub fn new(arguments: Vec<String>) -> Self {
        let concatenated = if arguments.is_empty() {
            String::new()
        } else {
            arguments.join(" ")
        };
        Self {
            arguments,
            concatenated,
        }
    }

    pub fn positional(&self, index: usize) -> Option<&str> {
        if index == 0 {
            return None;
        }
        self.arguments.get(index - 1).map(|s| s.as_str())
    }

    pub fn all_arguments(&self) -> &str {
        &self.concatenated
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}
