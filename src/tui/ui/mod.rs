pub mod input_view;
pub mod main_view;
pub mod popup;

use crate::tui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
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
