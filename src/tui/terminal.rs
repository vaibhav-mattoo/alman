use crate::cli::cli_data::Operation;
use crate::database::database_structs::{Database, DeletedCommands};
use crate::database::persistence::{
    ensure_data_directory, get_database_path, get_deleted_commands_path, load_database,
    load_deleted_commands, save_database, save_deleted_commands,
};
use crate::ops::{add_alias , remove_alias, delete_suggestion};
use crate::tui::app::App;
use crate::tui::ui::render_ui;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::io;
use std::path::PathBuf;


pub fn run_tui(alias_file_path: PathBuf, alias_file_paths: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure data directory exists
    if let Err(e) = ensure_data_directory() {
        eprintln!("Failed to create data directory: {}", e);
        return Err(e);
    }

    // Load database and deleted commands from persistent storage
    let db_path = get_database_path();
    let deleted_commands_path = get_deleted_commands_path();

    let mut database = match load_database(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to load database: {}", e);
            return Err(e);
        }
    };

    let mut deleted_commands = match load_deleted_commands(&deleted_commands_path) {
        Ok(dc) => dc,
        Err(e) => {
            eprintln!("Failed to load deleted commands: {}", e);
            return Err(e);
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Use a struct to ensure cleanup happens
    struct TerminalGuard {
        terminal: Terminal<CrosstermBackend<io::Stdout>>,
    }

    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            // Always try to restore terminal state, even if it fails
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

    // Create app and load initial data
    let mut app = App::new(alias_file_path.clone(), alias_file_paths);
    app.load_commands(&mut database);

    // Run the app
    let res = run_app(
        &mut terminal_guard.terminal,
        &mut app,
        &mut database,
        &mut deleted_commands,
        &db_path,
        &deleted_commands_path,
    );

    // Drop the terminal guard to restore the main screen
    drop(terminal_guard);

    // Now print the message
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
    // ANSI green: \x1b[32m ... \x1b[0m
    println!("\nTo use your new aliases immediately, run: \x1b[32msource {}\x1b[0m", shell_file);

    Ok(res?)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    database: &mut Database,
    deleted_commands: &mut DeletedCommands,
    db_path: &str,
    deleted_commands_path: &str,
) -> io::Result<()> {
    loop {
        // Remove the full screen clear to prevent flicker
        // terminal.clear()?;
        terminal.draw(|f| render_ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(operation) = app.handle_key_event(key.code) {
                    handle_operation(operation, app, database, deleted_commands, db_path, deleted_commands_path);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_operation(
    operation: Operation,
    app: &mut App,
    database: &mut Database,
    deleted_commands: &mut DeletedCommands,
    db_path: &str,
    deleted_commands_path: &str,
) {
    match operation {
        Operation::Add { alias, command } => {
            use crate::ops::alias_ops::add_alias_to_multiple_files;
            add_alias_to_multiple_files(&app.alias_file_paths, &alias, &command);
            if let Some(first_path) = app.alias_file_paths.first() {
                add_alias::add_alias(database, deleted_commands, first_path, &alias, &command);
            }
            app.status_message = format!("Added alias: {} = {}", alias, command);
            // Save after adding alias
            if let Err(e) = save_database(database, db_path) {
                eprintln!("Failed to save database: {}", e);
            }
            if let Err(e) = save_deleted_commands(deleted_commands, deleted_commands_path) {
                eprintln!("Failed to save deleted commands: {}", e);
            }
            app.load_commands(database);
            app.config_changed = true;
        }
        Operation::Remove { alias } => {
            use crate::ops::alias_ops::remove_alias_from_multiple_files;
            remove_alias_from_multiple_files(&app.alias_file_paths, &alias);
            if let Some(first_path) = app.alias_file_paths.first() {
                remove_alias::remove_alias(deleted_commands, first_path, &alias);
            }
            app.status_message = format!("Removed alias: {}", alias);
            // Save after removing alias
            if let Err(e) = save_deleted_commands(deleted_commands, deleted_commands_path) {
                eprintln!("Failed to save deleted commands: {}", e);
            }
            app.config_changed = true;
        }
        Operation::Change { old_alias, new_alias, command } => {
            use crate::ops::alias_ops::remove_alias_from_multiple_files;
            use crate::ops::alias_ops::add_alias_to_multiple_files;
            // First remove the old alias
            remove_alias_from_multiple_files(&app.alias_file_paths, old_alias.as_str());
            if let Some(first_path) = app.alias_file_paths.first() {
                remove_alias::remove_alias(deleted_commands, first_path, old_alias.as_str());
            }
            // Then add the new alias with the provided command
            add_alias_to_multiple_files(&app.alias_file_paths, new_alias.as_str(), command.as_str());
            if let Some(first_path) = app.alias_file_paths.first() {
                add_alias::add_alias(database, deleted_commands, first_path, new_alias.as_str(), command.as_str());
            }
            app.status_message = format!("Changed alias: {} -> {}", old_alias, new_alias);
            // Save after changing alias
            if let Err(e) = save_database(database, db_path) {
                eprintln!("Failed to save database: {}", e);
            }
            if let Err(e) = save_deleted_commands(deleted_commands, deleted_commands_path) {
                eprintln!("Failed to save deleted commands: {}", e);
            }
            app.config_changed = true;
        }
        Operation::List => {
            // List operation is now handled in the TUI as a separate mode
            app.status_message = "List operation handled in TUI mode".to_string();
        }
        Operation::DeleteSuggestion { alias } => {
            delete_suggestion::delete_suggestion(&alias, database, deleted_commands);
            app.status_message = format!("Deleted suggestions for: {}", alias);
            app.load_commands(database);
            // Save after deleting suggestion
            if let Err(e) = save_database(database, db_path) {
                eprintln!("Failed to save database: {}", e);
            }
            if let Err(e) = save_deleted_commands(deleted_commands, deleted_commands_path) {
                eprintln!("Failed to save deleted commands: {}", e);
            }
        }
        Operation::GetSuggestions { .. } => {
            // Get suggestions is not available in TUI mode
            app.status_message = "Get suggestions not available in TUI mode".to_string();
        }
        Operation::Tui => {
            // Already in TUI mode, do nothing
        }
        Operation::Init { .. } => {
            // Init is not available in TUI mode
            app.status_message = "Init command not available in TUI mode".to_string();
        }
    }
}
