use codex_slash_commands::FrontmatterMetadata;
use codex_slash_commands::parsing::ParsedTemplate;
use codex_slash_commands::parsing::parse_template;
use pretty_assertions::assert_eq;

#[test]
fn parses_frontmatter_metadata_and_body() {
    let raw = r#"---
        description: Deploy the project
        argument_hint: "<env>"
        model: gpt-4o
        ---
        Deploy $ARGUMENTS to production
    "#;

    let ParsedTemplate { metadata, body } =
        parse_template(raw).expect("expected template to parse");

    assert_eq!(metadata.description.as_deref(), Some("Deploy the project"));
    assert_eq!(metadata.argument_hint.as_deref(), Some("<env>"));
    assert_eq!(metadata.model.as_deref(), Some("gpt-4o"));
    assert_eq!(body.trim(), "Deploy $ARGUMENTS to production");
}

#[test]
fn invalid_frontmatter_is_ignored() {
    let raw = "---\n: -not-valid yaml\n---\nbody";
    let ParsedTemplate { metadata, body } =
        parse_template(raw).expect("expected template to parse");
    assert_eq!(metadata, FrontmatterMetadata::default());
    assert_eq!(body, "body");
}

#[test]
fn parses_allowed_tools_list() {
    let raw = r#"---
allowed_tools:
  - git
  -   shell  
---
body
"#;

    let ParsedTemplate { metadata, body } =
        parse_template(raw).expect("expected template to parse");

    assert_eq!(
        metadata.allowed_tools,
        Some(vec!["git".to_string(), "shell".to_string()])
    );
    assert_eq!(body, "body");
}
