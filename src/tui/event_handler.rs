use crate::cli::cli_data::Operation;
use crate::tui::app::{App, AppMode};
use ratatui::crossterm::event::KeyCode;

impl App {
    pub fn handle_key_event(&mut self, key: KeyCode) -> Option<Operation> {

        if self.show_popup {
            match key {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                    self.hide_popup();
                    return None;
                }
                _ => return None,
            }
        }

        if self.show_command_details_popup {
            match key {
                KeyCode::Left => {
                    if self.command_details_selection > 0 {
                        self.command_details_selection -= 1;
                    }
                    return None;
                }
                KeyCode::Right => {
                    if self.command_details_selection < 2 {
                        self.command_details_selection += 1;
                    }
                    return None;
                }
                KeyCode::Enter => {
                    match self.command_details_selection {
                        0 => { // Add Alias
                            if let Some(cmd) = &self.selected_command_details {
                                self.selected_command = Some(cmd.command_text.clone());
                                self.input = cmd.command_text.clone();
                                self.cursor_position = self.input.len();
                                self.set_mode(AppMode::AddAliasStep2);
                                self.generate_alias_suggestions();
                                self.status_message = "Enter alias name for the selected command:".to_string();
                            }
                            self.show_command_details_popup = false;
                        }
                        1 => { // Delete Suggestion
                            if let Some(cmd) = &self.selected_command_details {
                                return Some(Operation::DeleteSuggestion { alias: cmd.command_text.clone() });
                            }
                            self.show_command_details_popup = false;
                        }
                        2 => { // Back
                            self.show_command_details_popup = false;
                            self.selected_command_details = None;
                            self.command_details_selection = 0;
                            self.status_message = "Returned to main menu.".to_string();
                        }
                        _ => {}
                    }
                    return None;
                }
                KeyCode::Esc => {
                    self.show_command_details_popup = false;
                    self.selected_command_details = None;
                    self.command_details_selection = 0;
                    self.status_message = "Returned to main menu.".to_string();
                    return None;
                }
                _ => return None,
            }
        }

        match self.mode {
            AppMode::Main => self.handle_main_mode(key),
            AppMode::AddAliasStep1 => self.handle_add_alias_step1(key),
            AppMode::AddAliasStep2 => self.handle_add_alias_step2(key),
            AppMode::AddAliasConfirmation => self.handle_add_alias_confirmation(key),
            AppMode::RemoveAliasStep1 => self.handle_remove_alias_step1(key),
            AppMode::RemoveAliasConfirmation => self.handle_remove_alias_confirmation(key),
            AppMode::ChangeAliasStep1 => self.handle_change_alias_step1(key),
            AppMode::ChangeAliasStep2 => self.handle_change_alias_step2(key),
            AppMode::ListAliases => self.handle_list_aliases(key),

        }
    }

    fn handle_main_mode(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                None
            }
            KeyCode::Char('a') => {
                self.set_mode(AppMode::AddAliasStep1);
                self.status_message = "Select a command to create an alias for (use ↑↓ to navigate, Enter to select):".to_string();
                None
            }
            KeyCode::Char('r') => {
                self.set_mode(AppMode::RemoveAliasStep1);
                self.status_message = "Select an alias to remove (type to filter, ↑↓ to navigate):".to_string();
                None
            }
            KeyCode::Char('c') => {
                self.set_mode(AppMode::ChangeAliasStep1);
                self.status_message = "Select an alias to change (type to filter, ↑↓ to navigate):".to_string();
                self.load_aliases();
                self.filter_aliases();
                None
            }
            KeyCode::Char('l') => {
                self.set_mode(AppMode::ListAliases);
                self.status_message = "Listing aliases (use ↑↓ to navigate, Esc to return):".to_string();
                self.load_aliases_for_listing();
                None
            }
            KeyCode::Up => {
                if !self.filtered_commands.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                }
                None
            }
            KeyCode::Down => {
                if !self.filtered_commands.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i < self.filtered_commands.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                }
                None
            }
            KeyCode::Enter => {
                if let Some(cmd) = self.get_selected_command() {
                    self.selected_command_details = Some(cmd.clone());
                    self.command_details_selection = 0;
                    self.show_command_details_popup = true;
                }
                None
            }
            KeyCode::Char('/') => {
                self.clear_input();
                self.status_message = "Type to filter commands... (Esc to clear)".to_string();
                None
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.cursor_position = self.input.len();
                self.filter_commands();
                self.status_message = format!("Filtering: '{}' (Esc to clear)", self.input);
                None
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                    self.cursor_position = self.input.len();
                    self.filter_commands();
                    if self.input.is_empty() {
                        self.status_message = "Filter cleared".to_string();
                    } else {
                        self.status_message = format!("Filtering: '{}'", self.input);
                    }
                }
                None
            }
            KeyCode::Esc => {
                if !self.input.is_empty() {
                    self.clear_input();
                    self.filter_commands();
                    self.status_message = "Filter cleared".to_string();
                }
                None
            }
            _ => None,
        }
    }

    fn handle_add_alias_step1(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Enter => {
                let command_text = if let Some(cmd) = self.get_selected_command() {
                    cmd.command_text.clone()
                } else if !self.input.trim().is_empty() {
                    self.input.trim().to_string()
                } else {
                    self.status_message = "Please select a command or type one".to_string();
                    return None;
                };
                
                self.selected_command = Some(command_text.clone());
                self.input = command_text;
                self.cursor_position = self.input.len();
                self.set_mode(AppMode::AddAliasStep2);
                self.generate_alias_suggestions();
                self.status_message = "Enter alias name for the selected command:".to_string();
                None
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.filter_commands();
                None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.filter_commands();
                }
                None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
                None
            }
            KeyCode::Up => {
                if !self.filtered_commands.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                    // Autofill the input with the selected command
                    if let Some(cmd) = self.get_selected_command() {
                        self.input = cmd.command_text.clone();
                        self.cursor_position = self.input.len();
                    }
                }
                None
            }
            KeyCode::Down => {
                if !self.filtered_commands.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i < self.filtered_commands.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                    // Autofill the input with the selected command
                    if let Some(cmd) = self.get_selected_command() {
                        self.input = cmd.command_text.clone();
                        self.cursor_position = self.input.len();
                    }
                }
                None
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Add alias cancelled.".to_string();
                None
            }
            _ => None,
        }
    }

    fn handle_add_alias_step2(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Enter => {
                let alias = self.alias_input.trim().to_string();
                if alias.is_empty() {
                    self.status_message = "Alias name cannot be empty".to_string();
                    return None;
                }
                
                if let Some(command) = &self.selected_command {
                    // Store the alias and command for confirmation
                    self.confirmation_alias = Some(alias.clone());
                    self.confirmation_command = Some(command.clone());
                    self.confirmation_selection = true; // Default to OK selected
                    self.status_message = format!("Confirm adding alias: {} = {} (OK/Undo)", alias, command);
                    self.set_mode(AppMode::AddAliasConfirmation);
                    None
                } else {
                    self.status_message = "No command selected".to_string();
                    None
                }
            }
            KeyCode::Char(c) => {
                self.alias_input.insert(self.alias_cursor_position, c);
                self.alias_cursor_position += 1;
                None
            }
            KeyCode::Backspace => {
                if self.alias_cursor_position > 0 {
                    self.alias_input.remove(self.alias_cursor_position - 1);
                    self.alias_cursor_position -= 1;
                }
                None
            }
            KeyCode::Left => {
                if self.alias_cursor_position > 0 {
                    self.alias_cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.alias_cursor_position < self.alias_input.len() {
                    self.alias_cursor_position += 1;
                }
                None
            }
            KeyCode::Up => {
                if !self.alias_suggestions.is_empty() {
                    let selected = match self.alias_suggestions_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.alias_suggestions_state.select(Some(selected));
                    // Autofill the input with the selected alias
                    if let Some(suggestion) = self.alias_suggestions.get(selected) {
                        self.alias_input = suggestion.alias.clone();
                        self.alias_cursor_position = self.alias_input.len();
                    }
                }
                None
            }
            KeyCode::Down => {
                if !self.alias_suggestions.is_empty() {
                    let selected = match self.alias_suggestions_state.selected() {
                        Some(i) => {
                            if i < self.alias_suggestions.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.alias_suggestions_state.select(Some(selected));
                    // Autofill the input with the selected alias
                    if let Some(suggestion) = self.alias_suggestions.get(selected) {
                        self.alias_input = suggestion.alias.clone();
                        self.alias_cursor_position = self.alias_input.len();
                    }
                }
                None
            }
            KeyCode::Tab => {
                if let Some(suggestion) = self.alias_suggestions.get(0) {
                    self.alias_input = suggestion.alias.clone();
                    self.alias_cursor_position = self.alias_input.len();
                }
                None
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Add alias cancelled.".to_string();
                None
            }
            _ => None,
        }
    }

    fn handle_add_alias_confirmation(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Left => {
                self.confirmation_selection = true; // Select OK
                None
            }
            KeyCode::Right => {
                self.confirmation_selection = false; // Select Undo
                None
            }
            KeyCode::Enter => {
                if self.confirmation_selection {
                    // OK pressed - check if this is a change operation
                    if let (Some(alias), Some(command)) = (&self.confirmation_alias, &self.confirmation_command) {
                        if let Some(old_alias) = &self.change_old_alias {
                            // This is a change operation
                            let old_alias_str = old_alias.clone();
                            let new_alias_str = alias.to_string();
                            let command_str = command.to_string();
                            self.status_message = format!("Changed alias: {} = {} → {} = {}", 
                                old_alias_str, command_str, new_alias_str, command_str);
                            self.set_mode(AppMode::Main);
                            Some(Operation::Change { 
                                old_alias: old_alias_str, 
                                new_alias: new_alias_str,
                                command: command_str
                            })
                        } else {
                            // This is an add operation
                            let alias_str = alias.to_string();
                            let command_str = command.to_string();
                            self.status_message = format!("Added alias: {} = {}", alias_str, command_str);
                            self.set_mode(AppMode::Main);
                            Some(Operation::Add { 
                                alias: alias_str, 
                                command: command_str
                            })
                        }
                    } else {
                        self.status_message = "No alias or command selected".to_string();
                        self.set_mode(AppMode::Main);
                        None
                    }
                } else {
                    // Undo pressed - just don't add/change the alias
                    if let (Some(alias), Some(command)) = (&self.confirmation_alias, &self.confirmation_command) {
                        if self.change_old_alias.is_some() {
                            // This was a change operation
                            self.status_message = format!("Cancelled changing alias: {} = {}", alias, command);
                        } else {
                            // This was an add operation
                            self.status_message = format!("Cancelled adding alias: {} = {}", alias, command);
                        }
                        self.set_mode(AppMode::Main);
                        None
                    } else {
                        self.status_message = "No alias to cancel".to_string();
                        self.set_mode(AppMode::Main);
                        None
                    }
                }
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                if self.change_old_alias.is_some() {
                    self.status_message = "Change alias cancelled.".to_string();
                } else {
                    self.status_message = "Add alias cancelled.".to_string();
                }
                None
            }
            _ => None,
        }
    }

    fn handle_remove_alias_step1(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Enter => {
                let (alias_text, command_text) = if let Some((alias, command)) = self.get_selected_alias() {
                    (alias.clone(), command.clone())
                } else if !self.input.trim().is_empty() {
                    (self.input.trim().to_string(), String::new())
                } else {
                    self.status_message = "Please select an alias or type one".to_string();
                    return None;
                };
                self.remove_confirmation_alias = Some(alias_text);
                self.remove_confirmation_command = Some(command_text);
                self.remove_confirmation_selection = true; // OK selected by default
                self.status_message = format!("Confirm removing alias: {} (OK/Undo)", self.remove_confirmation_alias.as_deref().unwrap_or(""));
                self.set_mode(AppMode::RemoveAliasConfirmation);
                None
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.filter_aliases();
                None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.filter_aliases();
                }
                None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
                None
            }
            KeyCode::Up => {
                if !self.filtered_aliases.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                    // Autofill the input with the selected alias
                    if let Some((alias, _)) = self.get_selected_alias() {
                        self.input = alias.clone();
                        self.cursor_position = self.input.len();
                    }
                }
                None
            }
            KeyCode::Down => {
                if !self.filtered_aliases.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i < self.filtered_aliases.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                    // Autofill the input with the selected alias
                    if let Some((alias, _)) = self.get_selected_alias() {
                        self.input = alias.clone();
                        self.cursor_position = self.input.len();
                    }
                }
                None
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Remove alias cancelled.".to_string();
                None
            }
            _ => None,
        }
    }

    fn handle_remove_alias_confirmation(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Left => {
                self.remove_confirmation_selection = true; // OK
                None
            }
            KeyCode::Right => {
                self.remove_confirmation_selection = false; // Undo
                None
            }
            KeyCode::Enter => {
                if self.remove_confirmation_selection {
                    // OK pressed - remove the alias
                    if let Some(alias) = &self.remove_confirmation_alias {
                        let alias_str = alias.clone();
                        self.status_message = format!("Removed alias: {}", alias_str);
                        self.set_mode(AppMode::Main);
                        Some(Operation::Remove { alias: alias_str })
                    } else {
                        self.status_message = "No alias selected".to_string();
                        self.set_mode(AppMode::Main);
                        None
                    }
                } else {
                    // Undo pressed - do nothing
                    if let Some(alias) = &self.remove_confirmation_alias {
                        self.status_message = format!("Cancelled removing alias: {}", alias);
                    } else {
                        self.status_message = "No alias to cancel".to_string();
                    }
                    self.set_mode(AppMode::Main);
                    None
                }
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Remove alias cancelled.".to_string();
                None
            }
            _ => None,
        }
    }

    fn handle_change_alias_step1(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Enter => {
                if let Some((alias, command)) = self.get_selected_alias() {
                    let alias_clone = alias.clone();
                    let command_clone = command.clone();
                    self.change_old_alias = Some(alias_clone);
                    self.change_old_command = Some(command_clone);
                    self.change_new_alias.clear();
                    self.change_new_alias_cursor_position = 0;
                    self.set_mode(AppMode::ChangeAliasStep2);
                    self.generate_change_alias_suggestions();
                    self.status_message = "Enter new alias name:".to_string();
                } else if !self.input.trim().is_empty() {
                    // Try to find alias by name
                    let search_alias = self.input.trim();
                    if let Some((alias, command)) = self.aliases.iter().find(|(a, _)| a == search_alias) {
                        self.change_old_alias = Some(alias.clone());
                        self.change_old_command = Some(command.clone());
                        self.change_new_alias.clear();
                        self.change_new_alias_cursor_position = 0;
                        self.set_mode(AppMode::ChangeAliasStep2);
                        self.generate_change_alias_suggestions();
                        self.status_message = "Enter new alias name:".to_string();
                    } else {
                        self.status_message = "Alias not found".to_string();
                    }
                } else {
                    self.status_message = "Please select an alias or type one".to_string();
                }
                None
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.filter_aliases();
                None
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.filter_aliases();
                }
                None
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
                None
            }
            KeyCode::Up => {
                if !self.filtered_aliases.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                }
                None
            }
            KeyCode::Down => {
                if !self.filtered_aliases.is_empty() {
                    let selected = match self.list_state.selected() {
                        Some(i) => {
                            if i < self.filtered_aliases.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(selected));
                }
                None
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Change alias cancelled.".to_string();
                None
            }
            _ => None,
        }
    }

    fn handle_change_alias_step2(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Enter => {
                let new_alias = self.change_new_alias.trim().to_string();
                if new_alias.is_empty() {
                    self.status_message = "New alias name cannot be empty".to_string();
                    return None;
                }
                
                if let (Some(old_alias), Some(old_command)) = (&self.change_old_alias, &self.change_old_command) {
                    // Store the old and new alias for confirmation
                    self.confirmation_alias = Some(new_alias.clone());
                    self.confirmation_command = Some(old_command.clone());
                    self.confirmation_selection = true; // Default to OK selected
                    self.status_message = format!("Confirm changing alias: {} = {} → {} = {} (OK/Undo)", 
                        old_alias, old_command, new_alias, old_command);
                    self.set_mode(AppMode::AddAliasConfirmation);
                    None
                } else {
                    self.status_message = "No alias selected for change".to_string();
                    None
                }
            }
            KeyCode::Char(c) => {
                self.change_new_alias.insert(self.change_new_alias_cursor_position, c);
                self.change_new_alias_cursor_position += 1;
                None
            }
            KeyCode::Backspace => {
                if self.change_new_alias_cursor_position > 0 {
                    self.change_new_alias.remove(self.change_new_alias_cursor_position - 1);
                    self.change_new_alias_cursor_position -= 1;
                }
                None
            }
            KeyCode::Left => {
                if self.change_new_alias_cursor_position > 0 {
                    self.change_new_alias_cursor_position -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.change_new_alias_cursor_position < self.change_new_alias.len() {
                    self.change_new_alias_cursor_position += 1;
                }
                None
            }
            KeyCode::Up => {
                if !self.change_alias_suggestions.is_empty() {
                    let selected = match self.change_alias_suggestions_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.change_alias_suggestions_state.select(Some(selected));
                    // Autofill the input with the selected alias
                    if let Some(suggestion) = self.change_alias_suggestions.get(selected) {
                        self.change_new_alias = suggestion.alias.clone();
                        self.change_new_alias_cursor_position = self.change_new_alias.len();
                    }
                }
                None
            }
            KeyCode::Down => {
                if !self.change_alias_suggestions.is_empty() {
                    let selected = match self.change_alias_suggestions_state.selected() {
                        Some(i) => {
                            if i < self.change_alias_suggestions.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.change_alias_suggestions_state.select(Some(selected));
                    // Autofill the input with the selected alias
                    if let Some(suggestion) = self.change_alias_suggestions.get(selected) {
                        self.change_new_alias = suggestion.alias.clone();
                        self.change_new_alias_cursor_position = self.change_new_alias.len();
                    }
                }
                None
            }
            KeyCode::Tab => {
                if let Some(suggestion) = self.change_alias_suggestions.get(0) {
                    self.change_new_alias = suggestion.alias.clone();
                    self.change_new_alias_cursor_position = self.change_new_alias.len();
                }
                None
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Change alias cancelled.".to_string();
                None
            }
            _ => None,
        }
    }

    fn handle_list_aliases(&mut self, key: KeyCode) -> Option<Operation> {
        match key {
            KeyCode::Up => {
                if !self.aliases.is_empty() {
                    let selected = match self.list_aliases_state.selected() {
                        Some(i) => {
                            if i > 0 {
                                i - 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_aliases_state.select(Some(selected));
                }
                None
            }
            KeyCode::Down => {
                if !self.aliases.is_empty() {
                    let selected = match self.list_aliases_state.selected() {
                        Some(i) => {
                            if i < self.aliases.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };
                    self.list_aliases_state.select(Some(selected));
                }
                None
            }
            KeyCode::Enter => {
                // Show details of selected alias
                if let Some(selected) = self.list_aliases_state.selected() {
                    if let Some((alias, command)) = self.aliases.get(selected) {
                        self.show_popup(format!("Alias: {} = {}", alias, command));
                    }
                }
                None
            }
            KeyCode::Esc => {
                self.set_mode(AppMode::Main);
                self.status_message = "Returned to main menu.".to_string();
                None
            }
            _ => None,
        }
    }
}
