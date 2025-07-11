use crate::tui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    match app.mode {
        AppMode::AddAliasStep1 => render_add_alias_step1(f, app, area),
        AppMode::AddAliasStep2 => render_add_alias_step2(f, app, area),
        AppMode::AddAliasConfirmation => render_add_alias_confirmation(f, app, area),
        AppMode::RemoveAliasStep1 => render_remove_alias_step1(f, app, area),
        AppMode::RemoveAliasConfirmation => render_remove_alias_confirmation(f, app, area),
        AppMode::ChangeAliasStep1 => render_change_alias_step1(f, app, area),
        AppMode::ChangeAliasStep2 => render_change_alias_step2(f, app, area),
        AppMode::ListAliases => render_list_aliases(f, app, area),

        _ => render_default_input(f, app, area),
    }
}

fn render_add_alias_step1(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Command input
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Command list
        ])
        .split(area);

    // Command input field
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command to create alias for (type to filter, ↑↓ to navigate)"),
        );
    f.render_widget(input, chunks[0]);

    // Set cursor position for command input
    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    // Command list
    if !app.filtered_commands.is_empty() {
        let commands: Vec<ListItem> = app
            .filtered_commands
            .iter()
            .map(|cmd| ListItem::new(
                ratatui::text::Span::styled(cmd.command_text.as_str(), ratatui::style::Style::default().fg(ratatui::style::Color::Blue))
            ))
            .collect();

        let commands_list = List::new(commands)
            .block(Block::default().borders(Borders::ALL).title("Top Commands"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_stateful_widget(commands_list, chunks[2], &mut app.list_state.clone());
    }
}

fn render_add_alias_step2(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Selected command display
            Constraint::Length(3), // Alias input
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Alias suggestions
        ])
        .split(area);

    // Display selected command
    let selected_cmd = app.selected_command.as_deref().unwrap_or("No command selected");
    let command_display = Paragraph::new(format!("Selected command: {}", selected_cmd))
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Selected Command"));
    f.render_widget(command_display, chunks[0]);

    // Alias input field
    let alias_input = Paragraph::new(app.alias_input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter alias name (use ↑↓ to navigate suggestions, Tab for first suggestion)"),
        );
    f.render_widget(alias_input, chunks[1]);

    // Set cursor position for alias input
    f.set_cursor_position((
        chunks[1].x + app.alias_cursor_position as u16 + 1,
        chunks[1].y + 1,
    ));

    // Alias suggestions list
    if !app.alias_suggestions.is_empty() {
        let suggestions: Vec<ListItem> = app
            .alias_suggestions
            .iter()
            .map(|suggestion| {
                use ratatui::text::{Span, Line};
                ListItem::new(Line::from(vec![
                    Span::styled(&suggestion.alias, ratatui::style::Style::default().fg(ratatui::style::Color::Magenta)),
                    Span::raw(" = "),
                    Span::styled(&suggestion.command, ratatui::style::Style::default().fg(ratatui::style::Color::Blue)),
                    Span::raw(" ("),
                    Span::styled(&suggestion.reason, ratatui::style::Style::default().fg(ratatui::style::Color::Green)),
                    Span::raw(")"),
                ]))
            })
            .collect();

        let suggestions_list = List::new(suggestions)
            .block(Block::default().borders(Borders::ALL).title("Alias Suggestions"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_stateful_widget(suggestions_list, chunks[3], &mut app.alias_suggestions_state.clone());
    }
}

fn render_add_alias_confirmation(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Buttons
        ])
        .split(area);

    // Confirmation message
    let alias = app.confirmation_alias.as_deref().unwrap_or("unknown");
    let command = app.confirmation_command.as_deref().unwrap_or("unknown");
    let message = format!("Alias added: {} = {}", alias, command);
    let confirmation_message = Paragraph::new(message)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Alias Added"))
        .alignment(Alignment::Center);
    f.render_widget(confirmation_message, chunks[0]);

    // Buttons
    let ok_text = if app.confirmation_selection {
        Span::styled(" [OK] ", Style::default().fg(Color::White).bg(Color::Blue))
    } else {
        Span::styled(" [OK] ", Style::default().fg(Color::Blue))
    };
    
    let undo_text = if !app.confirmation_selection {
        Span::styled(" [Undo] ", Style::default().fg(Color::White).bg(Color::Red))
    } else {
        Span::styled(" [Undo] ", Style::default().fg(Color::Red))
    };

    let buttons = Paragraph::new(Line::from(vec![ok_text, undo_text]))
        .block(Block::default().borders(Borders::ALL).title("Confirm"))
        .alignment(Alignment::Center);
    f.render_widget(buttons, chunks[2]);
}

fn render_remove_alias_step1(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Alias input
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Alias list
        ])
        .split(area);

    // Alias input field
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter aliases to remove (type to filter, ↑↓ to navigate)"),
        );
    f.render_widget(input, chunks[0]);

    // Set cursor position for alias input
    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    // Alias list
    if !app.filtered_aliases.is_empty() {
        let aliases: Vec<ListItem> = app
            .filtered_aliases
            .iter()
            .map(|(alias, command)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} = ", alias),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        command,
                        Style::default().fg(Color::Blue),
                    ),
                ]))
            })
            .collect();

        let aliases_list = List::new(aliases)
            .block(Block::default().borders(Borders::ALL).title("Aliases"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_stateful_widget(aliases_list, chunks[2], &mut app.list_state.clone());
    }
}

fn render_remove_alias_confirmation(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Buttons
        ])
        .split(area);

    // Confirmation message
    let alias = app.remove_confirmation_alias.as_deref().unwrap_or("unknown");
    let command = app.remove_confirmation_command.as_deref().unwrap_or("");
    let message = if command.is_empty() {
        format!("Remove alias: {}", alias)
    } else {
        format!("Remove alias: {} = {}", alias, command)
    };
    let confirmation_message = Paragraph::new(message)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL).title("Remove Alias"))
        .alignment(Alignment::Center);
    f.render_widget(confirmation_message, chunks[0]);

    // Buttons
    let ok_text = if app.remove_confirmation_selection {
        Span::styled(" [OK] ", Style::default().fg(Color::White).bg(Color::Blue))
    } else {
        Span::styled(" [OK] ", Style::default().fg(Color::Blue))
    };
    
    let undo_text = if !app.remove_confirmation_selection {
        Span::styled(" [Undo] ", Style::default().fg(Color::White).bg(Color::Red))
    } else {
        Span::styled(" [Undo] ", Style::default().fg(Color::Red))
    };

    let buttons = Paragraph::new(Line::from(vec![ok_text, undo_text]))
        .block(Block::default().borders(Borders::ALL).title("Confirm"))
        .alignment(Alignment::Center);
    f.render_widget(buttons, chunks[2]);
}

fn render_change_alias_step1(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Alias input
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Alias list
        ])
        .split(area);

    // Alias input field
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter aliases to change (type to filter, ↑↓ to navigate)"),
        );
    f.render_widget(input, chunks[0]);

    // Set cursor position for alias input
    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    // Alias list
    if !app.filtered_aliases.is_empty() {
        let aliases: Vec<ListItem> = app
            .filtered_aliases
            .iter()
            .map(|(alias, command)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} = ", alias),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        command,
                        Style::default().fg(Color::Blue),
                    ),
                ]))
            })
            .collect();

        let aliases_list = List::new(aliases)
            .block(Block::default().borders(Borders::ALL).title("Aliases"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_stateful_widget(aliases_list, chunks[2], &mut app.list_state.clone());
    }
}

fn render_change_alias_step2(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Selected alias display
            Constraint::Length(3), // New alias input
            Constraint::Length(1), // Spacer
            Constraint::Min(0),    // Alias suggestions
        ])
        .split(area);

    // Display selected alias
    let old_alias = app.change_old_alias.as_deref().unwrap_or("No alias selected");
    let old_command = app.change_old_command.as_deref().unwrap_or("");
    let alias_display = if old_command.is_empty() {
        format!("Selected alias: {}", old_alias)
    } else {
        format!("Selected alias: {} = {}", old_alias, old_command)
    };
    let alias_display_widget = Paragraph::new(alias_display)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Selected Alias"));
    f.render_widget(alias_display_widget, chunks[0]);

    // New alias input field
    let new_alias_input = Paragraph::new(app.change_new_alias.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter new alias name (use ↑↓ to navigate suggestions, Tab for first suggestion)"),
        );
    f.render_widget(new_alias_input, chunks[1]);

    // Set cursor position for new alias input
    f.set_cursor_position((
        chunks[1].x + app.change_new_alias_cursor_position as u16 + 1,
        chunks[1].y + 1,
    ));

    // Alias suggestions list (show all)
    if !app.change_alias_suggestions.is_empty() {
        let suggestions: Vec<ListItem> = app
            .change_alias_suggestions
            .iter()
            .map(|suggestion| {
                use ratatui::text::{Span, Line};
                ListItem::new(Line::from(vec![
                    Span::styled(&suggestion.alias, ratatui::style::Style::default().fg(ratatui::style::Color::Magenta)),
                    Span::raw(" = "),
                    Span::styled(&suggestion.command, ratatui::style::Style::default().fg(ratatui::style::Color::Blue)),
                    Span::raw(" ("),
                    Span::styled(&suggestion.reason, ratatui::style::Style::default().fg(ratatui::style::Color::Green)),
                    Span::raw(")"),
                ]))
            })
            .collect();

        let mut state = app.change_alias_suggestions_state.clone();
        f.render_stateful_widget(
            List::new(suggestions)
                .block(Block::default().borders(Borders::ALL).title("Alias Suggestions (all)"))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol(">> "),
            chunks[3],
            &mut state,
        );
    }
}

fn render_default_input(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{:?}", app.mode)),
        );
    f.render_widget(input, chunks[0]);

    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    if !app.filtered_commands.is_empty() {
        let commands: Vec<ListItem> = app
            .filtered_commands
            .iter()
            .map(|cmd| ListItem::new(
                ratatui::text::Span::styled(cmd.command_text.as_str(), ratatui::style::Style::default().fg(ratatui::style::Color::Blue))
            ))
            .collect();

        let commands_list = List::new(commands)
            .block(Block::default().borders(Borders::ALL).title("Matching Commands"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_widget(commands_list, chunks[1]);
    }
}

fn render_list_aliases(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Alias list
            Constraint::Length(3), // Controls
        ])
        .split(area);

    // Header
    let header = Paragraph::new("Aliases")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Alias List"));
    f.render_widget(header, chunks[0]);

    // Alias list
    if !app.aliases.is_empty() {
        let aliases: Vec<ListItem> = app
            .aliases
            .iter()
            .map(|(alias, command)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} = ", alias),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        command,
                        Style::default().fg(Color::Blue),
                    ),
                ]))
            })
            .collect();

        let aliases_list = List::new(aliases)
            .block(Block::default().borders(Borders::ALL).title("Aliases"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_stateful_widget(aliases_list, chunks[1], &mut app.list_aliases_state.clone());
    } else {
        let empty_message = Paragraph::new("No aliases found")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Aliases"));
        f.render_widget(empty_message, chunks[1]);
    }

    // Controls
    let controls = Paragraph::new("↑/↓ - Navigate | Enter - Select | Esc - Back to main menu")
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(controls, chunks[2]);
}


