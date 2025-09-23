use codex_tui::subagent_quickstart_steps;
use pretty_assertions::assert_eq;

#[test]
fn subagent_quickstart_walkthrough_covers_enable_list_run_show() {
    let steps = subagent_quickstart_steps();
    assert!(
        steps.len() >= 4,
        "quickstart should outline enable, list, run, and show steps",
    );
    assert_eq!(steps[0], "Enable subagents via ~/.codex/config.toml");
    assert!(steps.iter().any(|s| s.contains("codex agents list")));
    assert!(steps.iter().any(|s| s.contains("codex agents run")));
    assert!(steps.iter().any(|s| s.contains("codex agents show")));
}
