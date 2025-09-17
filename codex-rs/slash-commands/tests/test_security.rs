use codex_slash_commands::InterpolationContext;
use codex_slash_commands::interpolate_template;
use pretty_assertions::assert_eq;

#[test]
fn interpolation_preserves_literal_security_sensitive_tokens() {
    let template = "!bash echo hello\n@file README.md\n$1";
    let ctx = InterpolationContext::new(vec!["ok".to_string()]);
    let interpolated =
        interpolate_template(template, &ctx).expect("expected interpolation to succeed");
    assert!(interpolated.contains("!bash echo hello"));
    assert!(interpolated.contains("@file README.md"));
    assert!(interpolated.ends_with("ok"));
}

#[test]
fn interpolation_does_not_execute_commands() {
    let template = "!bash rm -rf /\n$ARGUMENTS";
    let ctx = InterpolationContext::new(vec!["--dry-run".to_string()]);
    let interpolated =
        interpolate_template(template, &ctx).expect("expected interpolation to succeed");
    assert_eq!(interpolated.lines().next().unwrap(), "!bash rm -rf /");
}
