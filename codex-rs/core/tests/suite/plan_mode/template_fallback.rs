use chrono::Utc;
use codex_core::plan_mode::PlanArtifactMetadata;
use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

#[test]
fn warns_when_plan_template_missing() {
    let mut session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        vec![ToolCapability::new("fs.read", ToolMode::ReadOnly)],
        &PlanModeConfig::default(),
        true,
    );

    let metadata = PlanArtifactMetadata {
        template: None,
        model: Some("gpt-5".to_string()),
        updated_at: Some(Utc::now()),
    };
    session.set_artifact_metadata(metadata.clone());

    assert_eq!(session.plan_artifact().metadata, metadata);
}
