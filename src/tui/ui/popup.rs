use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let popup = Paragraph::new(
        ratatui::text::Span::styled(app.popup_message.as_str(), ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
    )
        .block(Block::default().borders(Borders::ALL).title("Info"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(popup, area);
}

pub fn render_command_details_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 30, f.area());
    f.render_widget(Clear, area);

    if let Some(cmd) = &app.selected_command_details {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Details
                Constraint::Length(3), // Buttons
            ])
            .split(area);

        let details_lines = vec![
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Cyan)),
                Span::styled(&cmd.command_text, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Score: ", Style::default().fg(Color::Cyan)),
                Span::styled(cmd.score.to_string(), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Frequency: ", Style::default().fg(Color::Cyan)),
                Span::styled(cmd.frequency.to_string(), Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("Last Used: ", Style::default().fg(Color::Cyan)),
                Span::styled(app.format_last_access_time(cmd.last_access_time), Style::default().fg(Color::Magenta)),
            ]),
        ];
        let details = Paragraph::new(details_lines)
            .block(Block::default().borders(Borders::ALL).title("Command Details"))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White));
        f.render_widget(details, chunks[0]);

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(chunks[1]);

        let add_alias_style = if app.command_details_selection == 0 {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Green)
        };
        let delete_style = if app.command_details_selection == 1 {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Red)
        };
        let back_style = if app.command_details_selection == 2 {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let add_alias_btn = Paragraph::new("Add Alias")
            .style(add_alias_style)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(add_alias_btn, button_chunks[0]);

        let delete_btn = Paragraph::new("Delete Suggestion")
            .style(delete_style)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(delete_btn, button_chunks[1]);

        let back_btn = Paragraph::new("Back")
            .style(back_style)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(back_btn, button_chunks[2]);
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
