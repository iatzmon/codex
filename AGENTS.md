# Repository Guidelines

## Project Structure & Module Organization
The workspace splits into `codex-cli` for the Node.js command-line wrapper and `codex-rs` for the Rust core. Inside `codex-rs`, crates follow the `codex-*` prefix: `core` hosts execution, plan mode, and configuration logic; `tui` renders the interface; `protocol`, `mcp-client`, and friends provide shared libraries. Integration and regression tests live under `codex-rs/core/tests`, while UI snapshot cases reside in `codex-rs/tui/tests`. Documentation and assets sit in `docs/` and `.github/`.

## Build, Test, and Development Commands
Use `pnpm install` to set up Node tooling, then `pnpm build` for the CLI bundle. Rust code is formatted with `just fmt`, linted via `just fix -p <crate>`, and built with `cargo build`. Run targeted tests such as `cargo test -p codex-core` or `cargo test -p codex-tui`. After touching shared crates (`core`, `common`, or `protocol`), finish with `cargo test --all-features`. Snapshot workflows employ `cargo insta pending-snapshots -p codex-tui` and `cargo insta accept -p codex-tui` when updates are intentional.

## Coding Style & Naming Conventions
Rust files rely on `rustfmt` defaults; prefer compact imports and inlining variables directly inside `format!` braces. Keep module names snake_case and crates prefixed with `codex-`. In the TUI, use Ratatui’s `Stylize` helpers (`"text".dim()`, `url.cyan().underlined()`) instead of manual `Style` construction. Avoid introducing hard-coded white foregrounds; let the theme choose defaults.

## Testing Guidelines
Unit tests should use `pretty_assertions::assert_eq` for readable diffs. Follow the crate’s existing naming pattern (`mod tests` inside the module or dedicated files in `tests/`). Snapshot updates require reviewing the generated `.snap.new` files before accepting. When network-dependent checks are gated by `CODEX_SANDBOX` or `CODEX_SANDBOX_NETWORK_DISABLED`, leave the guard in place so sandboxes skip external calls.

## Commit & Pull Request Guidelines
Write commits in the imperative mood (e.g., `Add default plan-mode shell rules`) and keep them focused on a single concern. Pull requests should summarize the change, reference related issues, and call out user-facing impacts. Include test evidence (`cargo test -p …`, `cargo insta pending-snapshots`) in the PR description, and attach screenshots for TUI updates when visuals change.
