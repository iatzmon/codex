use codex_slash_commands::InterpolationContext;
use codex_slash_commands::interpolate_template;
use pretty_assertions::assert_eq;

#[test]
fn replaces_arguments_tokens_and_positionals() {
    let template = "Deploy $1 to $2 using $ARGUMENTS";
    let ctx = InterpolationContext::new(vec!["web".to_string(), "prod".to_string()]);
    let interpolated =
        interpolate_template(template, &ctx).expect("expected interpolation to succeed");
    assert_eq!(interpolated, "Deploy web to prod using web prod");
}

#[test]
fn missing_positionals_become_empty_strings() {
    let template = "Review $1 then $3";
    let ctx = InterpolationContext::new(vec!["first".to_string()]);
    let interpolated =
        interpolate_template(template, &ctx).expect("expected interpolation to succeed");
    assert_eq!(interpolated, "Review first then ");
}
