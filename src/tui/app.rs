use crate::database::database_structs::Command;
use crate::database::db::now_secs;
use crate::ops::alias_suggestions::AliasSuggestion;
use crate::ops::get_suggestions::query_top_commands;
use ratatui::widgets::ListState;
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AppMode {
    Main,
    AddAliasStep1,
    AddAliasStep2,
    AddAliasConfirmation,
    RemoveAliasStep1,
    RemoveAliasConfirmation,
    ChangeAliasStep1,
    ChangeAliasStep2,
    ListAliases,
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub input: String,
    pub cursor_position: usize,
    pub list_state: ListState,
    pub commands: Vec<Command>,
    pub filtered_commands: Vec<Command>,
    pub alias_file_path: PathBuf,
    pub alias_file_paths: Vec<String>,
    pub should_quit: bool,
    pub status_message: String,
    pub show_popup: bool,
    pub popup_message: String,
    pub selected_command: Option<String>,
    pub alias_input: String,
    pub alias_cursor_position: usize,
    pub alias_suggestions: Vec<AliasSuggestion>,
    pub alias_suggestions_state: ListState,
    pub confirmation_alias: Option<String>,
    pub confirmation_command: Option<String>,
    pub confirmation_selection: bool,
    pub remove_confirmation_alias: Option<String>,
    pub remove_confirmation_command: Option<String>,
    pub remove_confirmation_selection: bool,
    pub change_old_alias: Option<String>,
    pub change_old_command: Option<String>,
    pub change_new_alias: String,
    pub change_new_alias_cursor_position: usize,
    pub change_alias_suggestions: Vec<AliasSuggestion>,
    pub change_alias_suggestions_state: ListState,
    pub aliases: Vec<(String, String)>,
    pub filtered_aliases: Vec<(String, String)>,
    pub list_aliases_state: ListState,
    pub selected_command_details: Option<Command>,
    pub command_details_selection: usize,
    pub show_command_details_popup: bool,
    pub config_changed: bool,
}

impl App {
    pub fn new(alias_file_path: PathBuf, alias_file_paths: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut alias_suggestions_state = ListState::default();
        alias_suggestions_state.select(Some(0));
        let mut change_alias_suggestions_state = ListState::default();
        change_alias_suggestions_state.select(Some(0));
        let mut list_aliases_state = ListState::default();
        list_aliases_state.select(Some(0));
        App {
            mode: AppMode::Main,
            input: String::new(),
            cursor_position: 0,
            list_state,
            commands: Vec::new(),
            filtered_commands: Vec::new(),
            alias_file_path,
            alias_file_paths,
            should_quit: false,
            status_message: "Welcome to Alman TUI!".to_string(),
            show_popup: false,
            popup_message: String::new(),
            selected_command: None,
            alias_input: String::new(),
            alias_cursor_position: 0,
            alias_suggestions: Vec::new(),
            alias_suggestions_state,
            confirmation_alias: None,
            confirmation_command: None,
            confirmation_selection: true,
            remove_confirmation_alias: None,
            remove_confirmation_command: None,
            remove_confirmation_selection: true,
            change_old_alias: None,
            change_old_command: None,
            change_new_alias: String::new(),
            change_new_alias_cursor_position: 0,
            change_alias_suggestions: Vec::new(),
            change_alias_suggestions_state,
            aliases: Vec::new(),
            filtered_aliases: Vec::new(),
            list_aliases_state,
            selected_command_details: None,
            command_details_selection: 0,
            show_command_details_popup: false,
            config_changed: false,
        }
    }

    pub fn load_commands(&mut self, conn: &Connection) {
        let now = now_secs();
        self.commands = query_top_commands(conn, now, 20);
        self.filtered_commands = self.commands.clone();
        self.list_state.select(None);
    }

    pub fn filter_commands(&mut self) {
        if self.input.is_empty() {
            self.filtered_commands = self.commands.clone();
        } else {
            let filter = self.input.to_lowercase();
            self.filtered_commands = self
                .commands
                .iter()
                .filter(|cmd| cmd.command_text.to_lowercase().contains(&filter))
                .cloned()
                .collect();
        }
        self.list_state.select(None);
    }

    pub fn show_popup(&mut self, message: String) {
        self.popup_message = message;
        self.show_popup = true;
    }

    pub fn hide_popup(&mut self) {
        self.show_popup = false;
        self.popup_message.clear();
    }

    pub fn get_selected_command(&self) -> Option<&Command> {
        if let Some(selected) = self.list_state.selected() {
            self.filtered_commands.get(selected)
        } else {
            None
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.selected_command = None;
        self.alias_input.clear();
        self.alias_cursor_position = 0;
        self.alias_suggestions.clear();
        self.alias_suggestions_state.select(None);
        self.confirmation_alias = None;
        self.confirmation_command = None;
        self.confirmation_selection = false;
        self.remove_confirmation_alias = None;
        self.remove_confirmation_command = None;
        self.remove_confirmation_selection = false;
        self.change_old_alias = None;
        self.change_old_command = None;
        self.change_new_alias = String::new();
        self.change_new_alias_cursor_position = 0;
        self.change_alias_suggestions.clear();
        self.change_alias_suggestions_state.select(None);
        self.aliases.clear();
        self.filtered_aliases.clear();
        self.list_aliases_state.select(None);
        self.selected_command_details = None;
        self.command_details_selection = 0;
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        match mode {
            AppMode::AddAliasStep2 => {
                self.alias_input.clear();
                self.alias_cursor_position = 0;
                self.alias_suggestions.clear();
                self.alias_suggestions_state.select(None);
                self.mode = mode;
            }
            AppMode::AddAliasConfirmation => {
                self.mode = mode;
            }
            AppMode::RemoveAliasStep1 => {
                self.load_aliases();
                self.mode = mode;
            }
            AppMode::RemoveAliasConfirmation => {
                self.mode = mode;
            }
            AppMode::ChangeAliasStep1 | AppMode::ChangeAliasStep2 => {
                self.mode = mode;
            }
            AppMode::Main => {
                self.mode = mode;
                self.clear_input();
                self.list_state.select(None);
            }
            _ => {
                self.mode = mode;
                self.clear_input();
            }
        }
    }

    pub fn generate_alias_suggestions(&mut self) {
        if let Some(command) = &self.selected_command {
            use crate::ops::alias_suggestions::AliasSuggester;
            let suggester = AliasSuggester::new(&self.alias_file_path.to_string_lossy());
            self.alias_suggestions = suggester.suggest_aliases(command);
        }
    }

    pub fn load_aliases(&mut self) {
        use crate::ops::alias_ops::get_aliases_from_multiple_files;
        self.aliases = get_aliases_from_multiple_files(&self.alias_file_paths);
        self.filtered_aliases = self.aliases.clone();
        self.list_state.select(None);
    }

    pub fn filter_aliases(&mut self) {
        if self.input.is_empty() {
            self.filtered_aliases = self.aliases.clone();
        } else {
            let filter = self.input.to_lowercase();
            self.filtered_aliases = self
                .aliases
                .iter()
                .filter(|(alias, command)| {
                    alias.to_lowercase().contains(&filter)
                        || command.to_lowercase().contains(&filter)
                })
                .cloned()
                .collect();
        }
    }

    pub fn get_selected_alias(&self) -> Option<&(String, String)> {
        if let Some(selected) = self.list_state.selected() {
            self.filtered_aliases.get(selected)
        } else {
            None
        }
    }

    pub fn generate_change_alias_suggestions(&mut self) {
        if let Some(old_alias) = &self.change_old_alias {
            if let Some((_, command)) = self.aliases.iter().find(|(alias, _)| alias == old_alias) {
                use crate::ops::alias_suggestions::AliasSuggester;
                let suggester = AliasSuggester::new(&self.alias_file_path.to_string_lossy());
                self.change_alias_suggestions = suggester.suggest_aliases(command);
            }
        }
    }

    pub fn load_aliases_for_listing(&mut self) {
        use crate::ops::alias_ops::get_aliases_from_multiple_files;
        self.aliases = get_aliases_from_multiple_files(&self.alias_file_paths);
        self.list_aliases_state.select(None);
    }

    pub fn format_last_access_time(&self, timestamp: i64) -> String {
        use chrono::{DateTime, TimeZone, Utc};
        let dt: DateTime<Utc> =
            Utc.timestamp_opt(timestamp, 0).single().unwrap_or_else(Utc::now);
        dt.format("%m/%d/%Y %H:%M:%S").to_string()
    }
}
