use shlex::Shlex;

/// Parse a slash command invocation into the command token and whitespace-delimited
/// arguments. Returns `None` when the input is not a slash command or does not
/// contain a command token.
pub fn parse_command_line(input: &str) -> Option<(String, Vec<String>)> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let without_slash = trimmed.trim_start_matches('/').trim_start();
    if without_slash.is_empty() {
        return None;
    }

    let lexer = Shlex::new(without_slash);
    let mut tokens: Vec<String> = lexer.collect();
    if tokens.is_empty() {
        return None;
    }

    let command = tokens.remove(0);
    Some((command, tokens))
}
