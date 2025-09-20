use std::collections::HashSet;
use std::path::Path;

use tracing::warn;
use wildmatch::WildMatch;

const SHELL_PREFIX: &str = "shell(";
const DEFAULT_SHELL_ENTRIES: &[&str] = &[
    "shell(bash -lc cat *)",
    "shell(bash -lc find *)",
    "shell(bash -lc grep *)",
    "shell(bash -lc ls *)",
    "shell(bash -lc tree *)",
    "shell(bash -lc head *)",
    "shell(bash -lc tail *)",
    "shell(bash -lc stat *)",
    "shell(bash -lc pwd *)",
    "shell(bash -lc pwd)",
    "shell(bash -lc git status)",
    "shell(bash -lc git diff --stat)",
];

#[derive(Debug, Clone, Default)]
pub struct PlanModeAllowList {
    raw_entries: Vec<String>,
    tool_rules: Vec<ToolRule>,
    shell_rules: Vec<WildMatch>,
}

impl PlanModeAllowList {
    pub fn new(entries: &[String]) -> Self {
        let mut raw_entries = Vec::new();
        let mut seen_entries = HashSet::new();
        let mut tool_rules = Vec::new();
        let mut shell_rules = Vec::new();

        for entry in DEFAULT_SHELL_ENTRIES {
            push_entry(
                entry,
                &mut raw_entries,
                &mut seen_entries,
                &mut tool_rules,
                &mut shell_rules,
            );
        }

        for entry in entries {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }

            push_entry(
                trimmed,
                &mut raw_entries,
                &mut seen_entries,
                &mut tool_rules,
                &mut shell_rules,
            );
        }

        Self {
            raw_entries,
            tool_rules,
            shell_rules,
        }
    }

    pub fn raw_entries(&self) -> &[String] {
        &self.raw_entries
    }

    pub fn literal_tool_ids(&self) -> impl Iterator<Item = &str> {
        self.tool_rules.iter().filter_map(|rule| match rule {
            ToolRule::Exact(id) => Some(id.as_str()),
            ToolRule::Glob(_) => None,
        })
    }

    pub fn has_tool_rules(&self) -> bool {
        !self.tool_rules.is_empty()
    }

    pub fn matches_tool(&self, candidate: &str) -> bool {
        self.tool_rules.iter().any(|rule| rule.matches(candidate))
    }

    pub fn matches_shell_command(&self, command: &[String]) -> bool {
        if command.is_empty() {
            return false;
        }

        let candidates = shell_command_candidates(command);
        self.shell_rules
            .iter()
            .any(|rule| candidates.iter().any(|candidate| rule.matches(candidate)))
    }
}

#[derive(Debug, Clone)]
enum ToolRule {
    Exact(String),
    Glob(WildMatch),
}

impl ToolRule {
    fn matches(&self, candidate: &str) -> bool {
        match self {
            ToolRule::Exact(id) => id == candidate,
            ToolRule::Glob(matcher) => matcher.matches(candidate),
        }
    }
}

fn parse_shell_pattern(entry: &str) -> Option<&str> {
    if !entry.starts_with(SHELL_PREFIX) {
        return None;
    }

    if let Some(pattern) = entry
        .strip_prefix(SHELL_PREFIX)
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            warn!("Ignoring empty shell() allow rule in Plan Mode");
            None
        } else {
            Some(pattern)
        }
    } else {
        warn!("Ignoring malformed shell() allow rule in Plan Mode: {entry}");
        None
    }
}

fn is_wildcard(entry: &str) -> bool {
    entry.contains('*') || entry.contains('?') || entry.contains('[')
}

fn push_entry(
    entry: &str,
    raw_entries: &mut Vec<String>,
    seen_entries: &mut HashSet<String>,
    tool_rules: &mut Vec<ToolRule>,
    shell_rules: &mut Vec<WildMatch>,
) {
    if !seen_entries.insert(entry.to_string()) {
        return;
    }

    raw_entries.push(entry.to_string());

    if let Some(shell_pattern) = parse_shell_pattern(entry) {
        shell_rules.push(WildMatch::new(shell_pattern));
        return;
    }

    if is_wildcard(entry) {
        tool_rules.push(ToolRule::Glob(WildMatch::new(entry)));
    } else {
        tool_rules.push(ToolRule::Exact(entry.to_string()));
    }
}

fn shell_command_candidates(command: &[String]) -> Vec<String> {
    let mut candidates = Vec::new();
    let joined = command.join(" ");
    if !joined.is_empty() {
        candidates.push(joined);
    }

    if let Some(stripped) = strip_shell_wrapper(command) {
        let stripped_joined = stripped.join(" ");
        if !stripped_joined.is_empty()
            && !candidates
                .iter()
                .any(|existing| existing == &stripped_joined)
        {
            candidates.push(stripped_joined);
        }
    }

    candidates
}

fn strip_shell_wrapper<'a>(command: &'a [String]) -> Option<&'a [String]> {
    if command.is_empty() {
        return None;
    }

    let mut slice = command;
    loop {
        let Some(first) = slice.first() else {
            return None;
        };
        let Some(exe_name) = Path::new(first).file_name().and_then(|os| os.to_str()) else {
            return None;
        };
        let exe_name = exe_name.to_ascii_lowercase();

        if exe_name == "env" || exe_name == "env.exe" {
            if slice.len() <= 1 {
                return None;
            }
            slice = &slice[1..];
            continue;
        }

        if !is_shell_executable(&exe_name) {
            return None;
        }

        let flags = shell_command_flags(&exe_name);
        if flags.is_empty() {
            return if slice.len() > 1 {
                Some(&slice[1..])
            } else {
                None
            };
        }

        for (idx, arg) in slice.iter().enumerate().skip(1) {
            if flags.iter().any(|flag| flag.eq_ignore_ascii_case(arg)) {
                let rest = &slice[idx + 1..];
                if rest.is_empty() {
                    return None;
                }
                return Some(rest);
            }
        }

        return None;
    }
}

fn is_shell_executable(name: &str) -> bool {
    matches!(
        name,
        "sh" | "bash"
            | "zsh"
            | "dash"
            | "ksh"
            | "ash"
            | "busybox"
            | "fish"
            | "elvish"
            | "pwsh"
            | "powershell"
            | "powershell.exe"
            | "cmd"
            | "cmd.exe"
    )
}

fn shell_command_flags(shell: &str) -> &'static [&'static str] {
    match shell {
        "sh" | "bash" | "zsh" | "dash" | "ksh" | "ash" | "busybox" => &["-c", "-lc"],
        "fish" | "elvish" => &["-c"],
        "pwsh" | "powershell" | "powershell.exe" => &["-c", "-command", "-Command"],
        "cmd" | "cmd.exe" => &["/c", "/C"],
        _ => &[],
    }
}
