use codex_slash_commands::parse_command_line;
use pretty_assertions::assert_eq;

#[test]
fn parses_basic_command_line() {
    let (command, args) = parse_command_line("/deploy foo bar").expect("expected parse");
    assert_eq!(command, "deploy");
    assert_eq!(args, vec!["foo".to_string(), "bar".to_string()]);
}

#[test]
fn parses_quoted_arguments() {
    let (command, args) =
        parse_command_line("/deploy \"feature branch\" --force").expect("expected parse");
    assert_eq!(command, "deploy");
    assert_eq!(
        args,
        vec!["feature branch".to_string(), "--force".to_string()]
    );
}

#[test]
fn non_commands_return_none() {
    assert!(parse_command_line("hello world").is_none());
    assert!(parse_command_line(" / ").is_none());
}

#[test]
fn trims_leading_whitespace() {
    let (command, args) = parse_command_line("  /status   ").expect("expected parse");
    assert_eq!(command, "status");
    assert!(args.is_empty());
}
