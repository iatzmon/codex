use std::path::PathBuf;

fn contract_path() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(codex_rs_dir) = crate_dir.parent() else {
        panic!("codex CLI crate should have parent directory");
    };
    let Some(workspace_root) = codex_rs_dir.parent() else {
        panic!("codex workspace root should exist");
    };
    workspace_root.join("specs/002-plan-mode/contracts/plan-mode.yaml")
}

#[test]
fn plan_mode_contract_declares_required_endpoints() {
    let contract = std::fs::read_to_string(contract_path())
        .expect("plan-mode contract spec should be present");
    for required in [
        "/commands/plan",
        "/commands/exit-plan",
        "/commands/apply-plan",
        "/events/plan-update",
    ] {
        assert!(
            contract.contains(required),
            "contract file should contain {required} definition",
        );
    }
}
