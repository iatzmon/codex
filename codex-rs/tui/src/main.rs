use anyhow::Error;
use clap::Parser;
use codex_arg0::arg0_dispatch_or_else;
use codex_common::CliConfigOverrides;
use codex_tui::Cli;
use codex_tui::run_main;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

fn maybe_run_hooks_namespace() -> anyhow::Result<Option<ExitStatus>> {
    let mut args = std::env::args().collect::<Vec<_>>();
    if let Some(index) = args.iter().position(|arg| arg == "hooks") {
        let hook_args = args.split_off(index + 1);
        // Ensure the initial segment (up to "hooks") does not interfere with downstream parsing.
        args.truncate(index);
        let script_path = hooks_cli_entrypoint();
        if !script_path.exists() {
            eprintln!(
                "Hooks CLI entrypoint not found at {}. Run `pnpm --dir codex-cli build` to generate it.",
                script_path.display()
            );
            std::process::exit(1);
        }

        let status = Command::new("node")
            .arg(script_path)
            .arg("hooks")
            .args(hook_args)
            .status()
            .map_err(Error::from)?;
        return Ok(Some(status));
    }
    Ok(None)
}

fn hooks_cli_entrypoint() -> PathBuf {
    // Workspace root is two levels up from codex-rs/tui (../../)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir)
        .join("codex-cli/bin/codex.js")
}

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    if let Some(status) = maybe_run_hooks_namespace()? {
        if let Some(code) = status.code() {
            std::process::exit(code);
        }
        return Ok(());
    }

    arg0_dispatch_or_else(|codex_linux_sandbox_exe| async move {
        let top_cli = TopCli::parse();
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let usage = run_main(inner, codex_linux_sandbox_exe).await?;
        if !usage.is_zero() {
            println!("{}", codex_core::protocol::FinalOutput::from(usage));
        }
        Ok(())
    })
}
