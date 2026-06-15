use crate::cli::cli_data::Operation;
use crate::database::persistence::ensure_data_directory;
use crate::ops::apply::{apply_add, apply_change, apply_remove, ApplyOutcome};
use crate::ops::delete_suggestion;
use crate::tui::app::{App, AppMode};
use crate::tui::ui::render_ui;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use rusqlite::Connection;
use std::io;
use std::path::PathBuf;

pub fn run_tui(
    alias_file_path: PathBuf,
    alias_file_paths: Vec<String>,
    conn: Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = ensure_data_directory() {
        eprintln!("Failed to create data directory: {}", e);
        return Err(e);
    }
    if let Err(e) = crate::database::persistence::ensure_config_directory() {
        eprintln!("Failed to create config directory: {}", e);
        return Err(e);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    struct TerminalGuard {
        terminal: Terminal<CrosstermBackend<io::Stdout>>,
    }
    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let _ = execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            );
            let _ = self.terminal.show_cursor();
        }
    }

    let mut terminal_guard = TerminalGuard { terminal };
    let mut app = App::new(alias_file_path, alias_file_paths);
    app.load_commands(&conn);

    let res = run_app(&mut terminal_guard.terminal, &mut app, &conn);

    drop(terminal_guard);

    let shell_path = std::env::var("SHELL").unwrap_or_default();
    let shell_file = if shell_path.contains("zsh") {
        "~/.zshrc"
    } else if shell_path.contains("bash") {
        "~/.bashrc"
    } else if shell_path.contains("fish") {
        "~/.config/fish/config.fish"
    } else {
        "your shell's config file"
    };
    println!("\nTo use your new aliases immediately, run: \x1b[32msource {}\x1b[0m", shell_file);

    Ok(res?)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    conn: &Connection,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(operation) = app.handle_key_event(key.code) {
                    handle_operation(operation, app, conn);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_operation(operation: Operation, app: &mut App, conn: &Connection) {
    match operation {
        Operation::Add { alias, command } => {
            apply_add(conn, &app.alias_file_paths, &alias, &command);
            app.status_message = format!("Added alias: {} = {}", alias, command);
            app.load_commands(conn);
            app.config_changed = true;
        }
        Operation::Remove { alias } => {
            match apply_remove(conn, &app.alias_file_paths, &alias) {
                ApplyOutcome::NotFound { .. } => {
                    app.status_message = format!("Alias '{}' not found.", alias);
                }
                _ => {
                    app.status_message = format!("Removed alias: {}", alias);
                    app.load_commands(conn);
                    app.config_changed = true;
                }
            }
        }
        Operation::Change { old_alias, new_alias } => {
            match apply_change(conn, &app.alias_file_paths, &old_alias, &new_alias) {
                ApplyOutcome::NotFound { .. } => {
                    app.status_message = format!("Alias '{}' not found.", old_alias);
                }
                _ => {
                    app.status_message = format!("Changed alias: {} -> {}", old_alias, new_alias);
                    app.load_commands(conn);
                    app.config_changed = true;
                }
            }
        }
        Operation::List => {
            app.status_message = "List operation handled in TUI mode".to_string();
        }
        Operation::DeleteSuggestion { alias } => {
            delete_suggestion::delete_suggestion(&alias, conn);
            app.status_message = format!("Deleted suggestions for: {}", alias);
            app.load_commands(conn);
            app.set_mode(AppMode::Main);
        }
        Operation::GetSuggestions { .. } => {
            app.status_message = "Get suggestions not available in TUI mode".to_string();
        }
        Operation::Tui => {}
        Operation::Init { .. } => {
            app.status_message = "Init command not available in TUI mode".to_string();
        }
        Operation::InitData | Operation::ListAliasFiles => {
            app.status_message = "Command not available in TUI mode".to_string();
        }
    }
}
