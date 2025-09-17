use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::WidgetRef;

use super::popup_consts::MAX_POPUP_ROWS;
use super::scroll_state::ScrollState;
use super::selection_popup_common::GenericDisplayRow;
use super::selection_popup_common::render_rows;
#[cfg(feature = "slash_commands")]
use crate::slash_command::CustomSlashCommand;
use crate::slash_command::SlashCommand;
use crate::slash_command::built_in_slash_commands;
use codex_common::fuzzy_match::fuzzy_match;
use codex_protocol::custom_prompts::CustomPrompt;
use std::collections::HashSet;

/// A selectable item in the popup: either a built-in command or a user prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommandItem {
    Builtin(SlashCommand),
    // Index into `prompts`
    UserPrompt(usize),
    #[cfg(feature = "slash_commands")]
    CustomCommand(usize),
}

pub(crate) struct CommandPopup {
    command_filter: String,
    builtins: Vec<(&'static str, SlashCommand)>,
    prompts: Vec<CustomPrompt>,
    #[cfg(feature = "slash_commands")]
    commands: Vec<CustomSlashCommand>,
    state: ScrollState,
}

impl CommandPopup {
    pub(crate) fn new(
        mut prompts: Vec<CustomPrompt>,
        #[cfg(feature = "slash_commands")] mut commands: Vec<CustomSlashCommand>,
    ) -> Self {
        let builtins = built_in_slash_commands();
        #[allow(unused_mut)]
        let mut exclude: HashSet<String> = builtins.iter().map(|(n, _)| (*n).to_string()).collect();
        #[cfg(feature = "slash_commands")]
        {
            commands.sort_by(|a, b| a.full_name.cmp(&b.full_name));
            exclude.extend(commands.iter().map(|cmd| cmd.full_name.clone()));
        }
        prompts.retain(|p| !exclude.contains(&p.name));
        prompts.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            command_filter: String::new(),
            builtins,
            prompts,
            #[cfg(feature = "slash_commands")]
            commands,
            state: ScrollState::new(),
        }
    }

    pub(crate) fn set_prompts(&mut self, mut prompts: Vec<CustomPrompt>) {
        #[allow(unused_mut)]
        let mut exclude: HashSet<String> = self
            .builtins
            .iter()
            .map(|(n, _)| (*n).to_string())
            .collect();
        #[cfg(feature = "slash_commands")]
        for command in &self.commands {
            exclude.insert(command.full_name.clone());
        }
        prompts.retain(|p| !exclude.contains(&p.name));
        prompts.sort_by(|a, b| a.name.cmp(&b.name));
        self.prompts = prompts;
    }

    #[cfg(feature = "slash_commands")]
    pub(crate) fn set_custom_commands(&mut self, mut commands: Vec<CustomSlashCommand>) {
        let mut exclude: HashSet<String> = self
            .builtins
            .iter()
            .map(|(n, _)| (*n).to_string())
            .collect();
        commands.sort_by(|a, b| a.full_name.cmp(&b.full_name));
        exclude.extend(commands.iter().map(|cmd| cmd.full_name.clone()));
        self.commands = commands;
        self.prompts.retain(|p| !exclude.contains(&p.name));
        let matches_len = self.filtered_items().len();
        self.state.clamp_selection(matches_len);
        self.state
            .ensure_visible(matches_len, MAX_POPUP_ROWS.min(matches_len));
    }

    pub(crate) fn prompt_name(&self, idx: usize) -> Option<&str> {
        self.prompts.get(idx).map(|p| p.name.as_str())
    }

    pub(crate) fn prompt_content(&self, idx: usize) -> Option<&str> {
        self.prompts.get(idx).map(|p| p.content.as_str())
    }

    /// Update the filter string based on the current composer text. The text
    /// passed in is expected to start with a leading '/'. Everything after the
    /// *first* '/" on the *first* line becomes the active filter that is used
    /// to narrow down the list of available commands.
    pub(crate) fn on_composer_text_change(&mut self, text: String) {
        let first_line = text.lines().next().unwrap_or("");

        if let Some(stripped) = first_line.strip_prefix('/') {
            // Extract the *first* token (sequence of non-whitespace
            // characters) after the slash so that `/clear something` still
            // shows the help for `/clear`.
            let token = stripped.trim_start();
            let cmd_token = token.split_whitespace().next().unwrap_or("");

            // Update the filter keeping the original case (commands are all
            // lower-case for now but this may change in the future).
            self.command_filter = cmd_token.to_string();
        } else {
            // The composer no longer starts with '/'. Reset the filter so the
            // popup shows the *full* command list if it is still displayed
            // for some reason.
            self.command_filter.clear();
        }

        // Reset or clamp selected index based on new filtered list.
        let matches_len = self.filtered_items().len();
        self.state.clamp_selection(matches_len);
        self.state
            .ensure_visible(matches_len, MAX_POPUP_ROWS.min(matches_len));
    }

    /// Determine the preferred height of the popup for a given width.
    /// Accounts for wrapped descriptions so that long tooltips don't overflow.
    pub(crate) fn calculate_required_height(&self, width: u16) -> u16 {
        use super::selection_popup_common::GenericDisplayRow;
        use super::selection_popup_common::measure_rows_height;
        let matches = self.filtered();
        let rows_all: Vec<GenericDisplayRow> = if matches.is_empty() {
            Vec::new()
        } else {
            matches
                .into_iter()
                .map(|(item, indices, _)| match item {
                    CommandItem::Builtin(cmd) => GenericDisplayRow {
                        name: format!("/{}", cmd.command()),
                        match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                        is_current: false,
                        description: Some(cmd.description().to_string()),
                    },
                    CommandItem::UserPrompt(i) => GenericDisplayRow {
                        name: format!("/{}", self.prompts[i].name),
                        match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                        is_current: false,
                        description: Some("send saved prompt".to_string()),
                    },
                    #[cfg(feature = "slash_commands")]
                    CommandItem::CustomCommand(i) => {
                        let command = &self.commands[i];
                        let mut parts = vec![format!("[{}]", command.scope_label())];
                        if let Some(description) = &command.description
                            && !description.is_empty()
                        {
                            parts.push(description.clone());
                        }
                        if let Some(hint) = &command.argument_hint
                            && !hint.is_empty()
                        {
                            parts.push(format!("arguments: {hint}"));
                        }
                        GenericDisplayRow {
                            name: format!("/{}", command.full_name),
                            match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                            is_current: false,
                            description: Some(parts.join(" — ")),
                        }
                    }
                })
                .collect()
        };

        measure_rows_height(&rows_all, &self.state, MAX_POPUP_ROWS, width)
    }

    /// Compute fuzzy-filtered matches over built-in commands and user prompts,
    /// paired with optional highlight indices and score. Sorted by ascending
    /// score, then by name for stability.
    fn filtered(&self) -> Vec<(CommandItem, Option<Vec<usize>>, i32)> {
        let filter = self.command_filter.trim();
        let mut out: Vec<(CommandItem, Option<Vec<usize>>, i32)> = Vec::new();
        if filter.is_empty() {
            // Built-ins first, in presentation order.
            for (_, cmd) in self.builtins.iter() {
                out.push((CommandItem::Builtin(*cmd), None, 0));
            }
            #[cfg(feature = "slash_commands")]
            for idx in 0..self.commands.len() {
                out.push((CommandItem::CustomCommand(idx), None, 0));
            }
            // Then prompts, already sorted by name.
            for idx in 0..self.prompts.len() {
                out.push((CommandItem::UserPrompt(idx), None, 0));
            }
            return out;
        }

        for &(_, cmd) in self.builtins.iter() {
            if let Some((indices, score)) = fuzzy_match(cmd.command(), filter) {
                out.push((CommandItem::Builtin(cmd), Some(indices), score));
            }
        }
        #[cfg(feature = "slash_commands")]
        for (idx, command) in self.commands.iter().enumerate() {
            if let Some((indices, score)) = fuzzy_match(&command.full_name, filter) {
                out.push((CommandItem::CustomCommand(idx), Some(indices), score));
                continue;
            }
            if let Some((indices, score)) = fuzzy_match(&command.qualified_name, filter) {
                let prefix_len = command.full_name.len() - command.qualified_name.len();
                let adjusted: Vec<usize> = indices.into_iter().map(|i| i + prefix_len).collect();
                out.push((CommandItem::CustomCommand(idx), Some(adjusted), score));
            }
        }
        for (idx, p) in self.prompts.iter().enumerate() {
            if let Some((indices, score)) = fuzzy_match(&p.name, filter) {
                out.push((CommandItem::UserPrompt(idx), Some(indices), score));
            }
        }
        // When filtering, sort by ascending score and then by name for stability.
        out.sort_by(|a, b| {
            a.2.cmp(&b.2).then_with(|| {
                let an = match a.0 {
                    CommandItem::Builtin(c) => c.command(),
                    CommandItem::UserPrompt(i) => &self.prompts[i].name,
                    #[cfg(feature = "slash_commands")]
                    CommandItem::CustomCommand(i) => &self.commands[i].full_name,
                };
                let bn = match b.0 {
                    CommandItem::Builtin(c) => c.command(),
                    CommandItem::UserPrompt(i) => &self.prompts[i].name,
                    #[cfg(feature = "slash_commands")]
                    CommandItem::CustomCommand(i) => &self.commands[i].full_name,
                };
                an.cmp(bn)
            })
        });
        out
    }

    fn filtered_items(&self) -> Vec<CommandItem> {
        self.filtered().into_iter().map(|(c, _, _)| c).collect()
    }

    /// Return currently selected command, if any.
    pub(crate) fn selected_item(&self) -> Option<CommandItem> {
        let matches = self.filtered_items();
        self.state
            .selected_idx
            .and_then(|idx| matches.get(idx).copied())
    }

    /// Move the selection cursor one step up.
    pub(crate) fn move_up(&mut self) {
        let len = self.filtered_items().len();
        self.state.move_up_wrap(len);
        self.state.ensure_visible(len, MAX_POPUP_ROWS.min(len));
    }

    /// Move the selection cursor one step down.
    pub(crate) fn move_down(&mut self) {
        let matches_len = self.filtered_items().len();
        self.state.move_down_wrap(matches_len);
        self.state
            .ensure_visible(matches_len, MAX_POPUP_ROWS.min(matches_len));
    }

    #[cfg(feature = "slash_commands")]
    pub(crate) fn custom_command(&self, idx: usize) -> Option<&CustomSlashCommand> {
        self.commands.get(idx)
    }
}

impl WidgetRef for CommandPopup {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let matches = self.filtered();
        let rows_all: Vec<GenericDisplayRow> = if matches.is_empty() {
            Vec::new()
        } else {
            matches
                .into_iter()
                .map(|(item, indices, _)| match item {
                    CommandItem::Builtin(cmd) => GenericDisplayRow {
                        name: format!("/{}", cmd.command()),
                        match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                        is_current: false,
                        description: Some(cmd.description().to_string()),
                    },
                    CommandItem::UserPrompt(i) => GenericDisplayRow {
                        name: format!("/{}", self.prompts[i].name),
                        match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                        is_current: false,
                        description: Some("send saved prompt".to_string()),
                    },
                    #[cfg(feature = "slash_commands")]
                    CommandItem::CustomCommand(i) => {
                        let command = &self.commands[i];
                        let mut parts = vec![format!("[{}]", command.scope_label())];
                        if let Some(description) = &command.description
                            && !description.is_empty()
                        {
                            parts.push(description.clone());
                        }
                        if let Some(hint) = &command.argument_hint
                            && !hint.is_empty()
                        {
                            parts.push(format!("arguments: {hint}"));
                        }
                        GenericDisplayRow {
                            name: format!("/{}", command.full_name),
                            match_indices: indices.map(|v| v.into_iter().map(|i| i + 1).collect()),
                            is_current: false,
                            description: Some(parts.join(" — ")),
                        }
                    }
                })
                .collect()
        };
        render_rows(
            area,
            buf,
            &rows_all,
            &self.state,
            MAX_POPUP_ROWS,
            false,
            "no matches",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "slash_commands")]
    fn new_test_popup(prompts: Vec<CustomPrompt>) -> CommandPopup {
        CommandPopup::new(prompts, Vec::new())
    }

    #[cfg(not(feature = "slash_commands"))]
    fn new_test_popup(prompts: Vec<CustomPrompt>) -> CommandPopup {
        CommandPopup::new(prompts)
    }

    #[test]
    fn filter_includes_init_when_typing_prefix() {
        let mut popup = new_test_popup(Vec::new());
        // Simulate the composer line starting with '/in' so the popup filters
        // matching commands by prefix.
        popup.on_composer_text_change("/in".to_string());

        // Access the filtered list via the selected command and ensure that
        // one of the matches is the new "init" command.
        let matches = popup.filtered_items();
        let has_init = matches.iter().any(|item| match item {
            CommandItem::Builtin(cmd) => cmd.command() == "init",
            CommandItem::UserPrompt(_) => false,
            #[cfg(feature = "slash_commands")]
            CommandItem::CustomCommand(_) => false,
        });
        assert!(
            has_init,
            "expected '/init' to appear among filtered commands"
        );
    }

    #[test]
    fn selecting_init_by_exact_match() {
        let mut popup = new_test_popup(Vec::new());
        popup.on_composer_text_change("/init".to_string());

        // When an exact match exists, the selected command should be that
        // command by default.
        let selected = popup.selected_item();
        match selected {
            Some(CommandItem::Builtin(cmd)) => assert_eq!(cmd.command(), "init"),
            Some(CommandItem::UserPrompt(_)) => panic!("unexpected prompt selected for '/init'"),
            #[cfg(feature = "slash_commands")]
            Some(CommandItem::CustomCommand(_)) => {
                panic!("unexpected custom command selected for '/init'")
            }
            None => panic!("expected a selected command for exact match"),
        }
    }

    #[test]
    fn model_is_first_suggestion_for_mo() {
        let mut popup = new_test_popup(Vec::new());
        popup.on_composer_text_change("/mo".to_string());
        let matches = popup.filtered_items();
        match matches.first() {
            Some(CommandItem::Builtin(cmd)) => assert_eq!(cmd.command(), "model"),
            Some(CommandItem::UserPrompt(_)) => {
                panic!("unexpected prompt ranked before '/model' for '/mo'")
            }
            #[cfg(feature = "slash_commands")]
            Some(CommandItem::CustomCommand(_)) => {
                panic!("unexpected custom command ranked before '/model' for '/mo'")
            }
            None => panic!("expected at least one match for '/mo'"),
        }
    }

    #[test]
    fn prompt_discovery_lists_custom_prompts() {
        let prompts = vec![
            CustomPrompt {
                name: "foo".to_string(),
                path: "/tmp/foo.md".to_string().into(),
                content: "hello from foo".to_string(),
            },
            CustomPrompt {
                name: "bar".to_string(),
                path: "/tmp/bar.md".to_string().into(),
                content: "hello from bar".to_string(),
            },
        ];
        let popup = new_test_popup(prompts);
        let items = popup.filtered_items();
        let mut prompt_names: Vec<String> = items
            .into_iter()
            .filter_map(|it| match it {
                CommandItem::UserPrompt(i) => popup.prompt_name(i).map(|s| s.to_string()),
                _ => None,
            })
            .collect();
        prompt_names.sort();
        assert_eq!(prompt_names, vec!["bar".to_string(), "foo".to_string()]);
    }

    #[test]
    fn prompt_name_collision_with_builtin_is_ignored() {
        // Create a prompt named like a builtin (e.g. "init").
        let popup = new_test_popup(vec![CustomPrompt {
            name: "init".to_string(),
            path: "/tmp/init.md".to_string().into(),
            content: "should be ignored".to_string(),
        }]);
        let items = popup.filtered_items();
        let has_collision_prompt = items.into_iter().any(|it| match it {
            CommandItem::UserPrompt(i) => popup.prompt_name(i) == Some("init"),
            _ => false,
        });
        assert!(
            !has_collision_prompt,
            "prompt with builtin name should be ignored"
        );
    }
}
