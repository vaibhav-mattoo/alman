use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let commands: Vec<ListItem> = app
        .filtered_commands
        .iter()
        .map(|cmd| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("Score: {} | ", cmd.score),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    &cmd.command_text,
                    Style::default().fg(Color::Blue),
                ),
            ]))
        })
        .collect();

    let commands_list = List::new(commands)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    f.render_stateful_widget(commands_list, chunks[0], &mut app.list_state.clone());

    let controls_lines = vec![
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - Add alias"),
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - Remove alias"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - Change alias"),
        ]),
        Line::from(vec![
            Span::styled("l", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - List aliases"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - Quit"),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - Navigate"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(" - Cancel/Close"),
        ]),
    ];
    let controls = Paragraph::new(controls_lines)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true });

    f.render_widget(controls, chunks[1]);
}
