pub mod input_view;
pub mod main_view;
pub mod popup;

use crate::tui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("Alman TUI")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main content based on mode or popup
    if app.show_command_details_popup {
        crate::tui::ui::popup::render_command_details_popup(f, app);
    } else {
        match app.mode {
            AppMode::Main => main_view::render(f, app, chunks[1]),
            AppMode::Templates | AppMode::TemplatesNameInput => render_templates(f, app, chunks[1]),
            AppMode::AddAliasStep1 | AppMode::AddAliasStep2 | AppMode::AddAliasConfirmation => input_view::render(f, app, chunks[1]),
            AppMode::RemoveAliasStep1 | AppMode::RemoveAliasConfirmation => input_view::render(f, app, chunks[1]),
            _ => input_view::render(f, app, chunks[1]),
        }
    }

    // Status bar
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[2]);

    // Popup
    if app.show_popup {
        popup::render(f, app);
    }
}

fn render_templates(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use crate::render::{PosixRenderer, ShellRenderer};
    use crate::registry::{Definition, DefinitionKind};
    use crate::template::TemplatePart;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let renderer = PosixRenderer;
    let items: Vec<ListItem> = app
        .mined_templates
        .iter()
        .map(|mt| {
            let kind = if mt.template.is_zero_slot() || mt.template.only_trailing_single_slot() {
                DefinitionKind::Alias
            } else {
                DefinitionKind::Function
            };
            let name = mt
                .template
                .parts
                .iter()
                .find_map(|p| {
                    if let TemplatePart::Literal(s) = p {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("?")
                .chars()
                .take(10)
                .collect::<String>();
            let def = Definition {
                name,
                kind,
                template: mt.template.clone(),
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{:>3}] ", mt.stats.support),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(renderer.render_definition(&def), Style::default().fg(Color::Blue)),
            ]))
        })
        .collect();

    // Name-input box (only meaningful in TemplatesNameInput mode).
    let input_title = if matches!(app.mode, AppMode::TemplatesNameInput) {
        "New name (Enter to save, Esc to cancel)"
    } else {
        "Enter to name & save a template"
    };
    let input = Paragraph::new(app.template_name_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title(input_title));
    f.render_widget(input, chunks[0]);

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Mined Templates"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[1], &mut app.templates_state.clone());
}
