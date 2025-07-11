mod cli;
mod database;
mod ops;
mod tui;
mod shell;

use cli::arg_handler::parse_args;
use cli::cli_data::Operation;
use database::database_structs::{Database, DeletedCommands};
use database::persistence::{
    ensure_data_directory, get_database_path, get_deleted_commands_path, load_database,
    load_deleted_commands, save_database, save_deleted_commands, load_config, save_config, AppConfig
};
use ops::add_alias::add_alias;
use ops::delete_suggestion::delete_suggestion;
use ops::get_suggestions;
use ops::insert_command::insert_command;
use ops::remove_alias::remove_alias;
use shell::{ShellOpts, render_shell_init};
use std::env;
use tui::run_tui;
use colored::*;
use clap::CommandFactory;
use std::path::PathBuf;
use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

fn to_absolute_path(path: &str) -> String {
    let pb = PathBuf::from(path);
    match pb.canonicalize() {
        Ok(abs) => abs.to_string_lossy().to_string(),
        Err(_) => {
            // If the file doesn't exist yet, canonicalize the parent
            if let Some(parent) = pb.parent() {
                if let Ok(abs_parent) = parent.canonicalize() {
                    return abs_parent.join(pb.file_name().unwrap_or_default()).to_string_lossy().to_string();
                }
            }
            pb.to_string_lossy().to_string()
        }
    }
}

fn is_system_command(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }
    if let Ok(paths) = env::var("PATH") {
        for path in paths.split(":") {
            let full_path = Path::new(path).join(cmd);
            if full_path.exists() && fs::metadata(&full_path).map(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0)).unwrap_or(false) {
                return true;
            }
        }
    }
    false
}

fn main() {
    // Intercept --help/-h to show dynamic default alias file path
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        // Load config and get default alias file path
        let default_path = {
            if let Some(cfg) = crate::database::persistence::load_config() {
                cfg.alias_file_paths.first().map(|p| to_absolute_path(p)).unwrap_or_else(|| crate::database::persistence::get_default_alias_file_path())
            } else {
                crate::database::persistence::get_default_alias_file_path()
            }
        };
        println!("Current default alias file path: {}\n", default_path.green());
        <crate::cli::cli_data::Cli as CommandFactory>::command().print_help().unwrap();
        println!();
        std::process::exit(0);
    }

    // Ensure data directory exists
    if let Err(e) = ensure_data_directory() {
        eprintln!("Failed to create data directory: {}", e);
        return;
    }

    // Load config for alias file paths
    let config = load_config();
    let mut alias_file_paths = if let Some(cfg) = &config {
        cfg.alias_file_paths.clone()
    } else {
        vec![crate::database::persistence::get_default_alias_file_path()]
    };

    // Parse CLI args
    let command_strings: Vec<String> = env::args().collect();

    // If no arguments (just the binary name), launch TUI by default
    if command_strings.len() == 1 {
        // Use the first alias file path for TUI
        let file_path = alias_file_paths.first().unwrap_or(&crate::database::persistence::get_default_alias_file_path()).clone();
        if let Err(e) = run_tui(std::path::PathBuf::from(file_path), alias_file_paths) {
            eprintln!("{}", format!("TUI error: {}", e).red());
        }
        return;
    }

    let cli = if command_strings.len() > 1 && command_strings[1] != "custom" {
        Some(parse_args())
    } else {
        None
    };

    // If only --alias-file-path is provided (no subcommand), update config and exit
    if let Some(cli) = &cli {
        if cli.operation.is_none() && cli.alias_file_path.is_some() {
            let cli_path_str = to_absolute_path(&cli.alias_file_path.as_ref().unwrap().to_string_lossy());
            if !alias_file_paths.contains(&cli_path_str) {
                alias_file_paths.push(cli_path_str.clone());
            }
            // Move the new path to the front (make it default)
            if let Some(pos) = alias_file_paths.iter().position(|p| p == &cli_path_str) {
                let new_default = alias_file_paths.remove(pos);
                alias_file_paths.insert(0, new_default);
            }
            let new_config = AppConfig { alias_file_paths: alias_file_paths.clone() };
            let _ = save_config(&new_config);
            println!("Default alias file path set to {}", cli_path_str.green());
            return;
        }
    }

    // If CLI provided alias_file_path, add it to the list and update config
    if let Some(ref cli) = cli {
        if let Some(ref cli_path) = cli.alias_file_path {
            let cli_path_str = to_absolute_path(&cli_path.to_string_lossy());
            if !alias_file_paths.contains(&cli_path_str) {
                alias_file_paths.push(cli_path_str.clone());
                let new_config = AppConfig { alias_file_paths: alias_file_paths.clone() };
                let _ = save_config(&new_config);
            }
        }
    }

    // Load database and deleted commands from persistent storage
    let db_path = get_database_path();
    let deleted_commands_path = get_deleted_commands_path();

    let mut db = match load_database(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("{}", format!("Failed to load database: {}", e).red());
            Database {
                command_list: std::collections::BTreeSet::new(),
                reverse_command_map: std::collections::HashMap::new(),
                total_num_commands: 0,
                total_score: 0,
            }
        }
    };

    let mut deleted_commands = match load_deleted_commands(&deleted_commands_path) {
        Ok(dc) => dc,
        Err(e) => {
            eprintln!("{}", format!("Failed to load deleted commands: {}", e).red());
            DeletedCommands {
                deleted_commands: std::collections::BTreeSet::new(),
            }
        }
    };

    let db_ref: &mut Database = &mut db;
    let dc_ref: &mut DeletedCommands = &mut deleted_commands;

    // Check if this is a custom command (starts with "custom")
    if command_strings[1] == "custom" {
        // This is a direct command to insert
        if command_strings.len() < 3 {
            eprintln!("Usage: {} custom <command>", command_strings[0]);
            eprintln!("Example: {} custom 'ls -la'", command_strings[0]);
            return;
        }
        
        let command = command_strings[2..].join(" ");
        insert_command(command.to_string(), db_ref, dc_ref);

        // Save database after inserting command
        if let Err(e) = save_database(db_ref, &db_path) {
            eprintln!("Failed to save database: {}", e);
        }
        if let Err(e) = save_deleted_commands(dc_ref, &deleted_commands_path) {
            eprintln!("Failed to save deleted commands: {}", e);
        }
    } else {
        // This is a subcommand, parse and handle it
        let cli = parse_args();

        match &cli.operation {
            Some(Operation::Add { alias, command }) => {
                use ops::alias_ops::add_alias_to_multiple_files;
                add_alias_to_multiple_files(&alias_file_paths, alias, command);
                if let Some(first_path) = alias_file_paths.first() {
                    add_alias(db_ref, dc_ref, first_path, alias, command);
                }
                if let Err(e) = save_database(db_ref, &db_path) {
                    eprintln!("{}", format!("Failed to save database: {}", e).red());
                }
                if let Err(e) = save_deleted_commands(dc_ref, &deleted_commands_path) {
                    eprintln!("{}", format!("Failed to save deleted commands: {}", e).red());
                }
            }
            Some(Operation::Remove { alias }) => {
                use ops::alias_ops::remove_alias_from_multiple_files;
                remove_alias_from_multiple_files(&alias_file_paths, alias);
                if let Some(first_path) = alias_file_paths.first() {
                    remove_alias(dc_ref, first_path, alias);
                }
                if let Err(e) = save_deleted_commands(dc_ref, &deleted_commands_path) {
                    eprintln!("{}", format!("Failed to save deleted commands: {}", e).red());
                }
            }
            Some(Operation::List) => {
                use ops::alias_ops::get_aliases_from_multiple_files;
                let aliases = get_aliases_from_multiple_files(&alias_file_paths);
                if aliases.is_empty() {
                    println!("{}", "No aliases found.".yellow());
                    return;
                }
                let max_alias_length = aliases.iter().map(|(alias, _)| alias.len()).max().unwrap_or(5).max(5);
                let max_command_length = aliases.iter().map(|(_, command)| command.len()).max().unwrap_or(7).max(7);
                let _total_width = 3 + max_alias_length + 3 + max_command_length + 2;
                println!("{}", format!("┌{:─<alias$}┬{:─<cmd$}┐", "", "", alias = max_alias_length + 2, cmd = max_command_length + 2).cyan());
                println!("{}", format!("│ {:<alias$} │ {:<cmd$} │", "ALIAS", "COMMAND", alias = max_alias_length, cmd = max_command_length).cyan());
                println!("{}", format!("├{:─<alias$}┼{:─<cmd$}┤", "", "", alias = max_alias_length + 2, cmd = max_command_length + 2).cyan());
                for (alias, command) in &aliases {
                    println!("│ {} │ {} │",
                        format!("{:<width$}", alias, width = max_alias_length).cyan(),
                        format!("{:<width$}", command, width = max_command_length)
                    );
                }
                println!("{}", format!("└{:─<alias$}┴{:─<cmd$}┘", "", "", alias = max_alias_length + 2, cmd = max_command_length + 2).cyan());
                println!("{}", format!("Total: {} alias(es) across {} file(s)", aliases.len(), alias_file_paths.len()).green());
            }
            Some(Operation::Change { old_alias, new_alias, command }) => {
                use ops::alias_ops::remove_alias_from_multiple_files;
                use ops::alias_ops::add_alias_to_multiple_files;
                // First remove the old alias
                remove_alias_from_multiple_files(&alias_file_paths, old_alias);
                if let Some(first_path) = alias_file_paths.first() {
                    remove_alias(dc_ref, first_path, old_alias);
                }
                // Then add the new alias with the provided command
                add_alias_to_multiple_files(&alias_file_paths, new_alias, command);
                if let Some(first_path) = alias_file_paths.first() {
                    add_alias(db_ref, dc_ref, first_path, new_alias, command);
                }
                if let Err(e) = save_database(db_ref, &db_path) {
                    eprintln!("{}", format!("Failed to save database: {}", e).red());
                }
                if let Err(e) = save_deleted_commands(dc_ref, &deleted_commands_path) {
                    eprintln!("{}", format!("Failed to save deleted commands: {}", e).red());
                }
            }
            Some(Operation::GetSuggestions { num }) => {
                // Get total number of commands in the database
                let total_commands = db_ref.command_list.len();
                if let Some(n) = num {
                    if *n == 0 {
                        eprintln!("{}", "Number of suggestions must be greater than 0.".red());
                        return;
                    }
                    if *n > total_commands {
                        eprintln!("{}", format!("Requested number of suggestions (n = {}) exceeds total available commands ({}).", n, total_commands).red());
                        return;
                    }
                }
                let list = get_suggestions::get_suggestions_with_aliases(*num, db_ref, alias_file_paths.first().unwrap_or(&crate::database::persistence::get_default_alias_file_path()));
                
                if list.is_empty() {
                    println!("{}", "No suggestions found.".yellow());
                    return;
                }

                // Prepare filtered list: only top alias, and not a system command
                let filtered: Vec<_> = list.iter().map(|cmd| {
                    let top_alias = cmd.alias_suggestions.iter().find(|a| !is_system_command(&a.alias));
                    (cmd, top_alias)
                }).collect();

                // Find the longest command, alias, and score for alignment
                let max_command_length = filtered.iter().map(|(cmd, _)| cmd.command.command_text.len()).max().unwrap_or(7).max(7); // at least 'COMMAND'
                let max_alias_length = filtered.iter().map(|(_, alias_opt)| alias_opt.map(|a| a.alias.len()).unwrap_or(0)).max().unwrap_or(9).max(9); // at least 'TOP ALIAS'
                let max_score_length = filtered.iter().map(|(cmd, _)| cmd.command.score.to_string().len()).max().unwrap_or(5).max(5); // at least 'SCORE'

                // Table width: borders + padding + columns
                let _total_width = 3 + max_command_length + 3 + max_alias_length + 3 + max_score_length + 2; // | command | alias | score |

                // Top border
                println!("{}", format!("┌{:─<cmd$}┬{:─<alias$}┬{:─<score$}┐", "", "", "", cmd = max_command_length + 2, alias = max_alias_length + 2, score = max_score_length + 2).cyan());
                // Header
                println!("{}", format!("│ {:<cmd$} │ {:>alias$} │ {:>score$} │", "COMMAND", "TOP ALIAS", "SCORE", cmd = max_command_length, alias = max_alias_length, score = max_score_length).cyan());
                // Separator
                println!("{}", format!("├{:─<cmd$}┼{:─<alias$}┼{:─<score$}┤", "", "", "", cmd = max_command_length + 2, alias = max_alias_length + 2, score = max_score_length + 2).cyan());

                // Rows
                for (cmd_with_alias, top_alias_opt) in &filtered {
                    let command_text = format!("{:<width$}", cmd_with_alias.command.command_text, width = max_command_length);
                    let alias_text = if let Some(top_alias) = top_alias_opt {
                        format!("{:>width$}", top_alias.alias, width = max_alias_length)
                    } else {
                        format!("{:>width$}", "", width = max_alias_length)
                    };
                    let score_text = format!("{:>width$}", cmd_with_alias.command.score, width = max_score_length);
                    println!("│ {} │ {} │ {} │",
                        command_text.bold(),
                        alias_text.cyan(),
                        score_text.yellow()
                    );
                }

                // Bottom border
                println!("{}", format!("└{:─<cmd$}┴{:─<alias$}┴{:─<score$}┘", "", "", "", cmd = max_command_length + 2, alias = max_alias_length + 2, score = max_score_length + 2).cyan());
                println!("{}", format!("Total: {} suggestion(s)", filtered.len()).green());
            }
            Some(Operation::DeleteSuggestion { alias }) => {
                delete_suggestion(alias, db_ref, dc_ref);
                println!("{}", format!("Deleted suggestions for: {}", alias).yellow());
                if let Err(e) = save_database(db_ref, &db_path) {
                    eprintln!("{}", format!("Failed to save database: {}", e).red());
                }
                if let Err(e) = save_deleted_commands(dc_ref, &deleted_commands_path) {
                    eprintln!("{}", format!("Failed to save deleted commands: {}", e).red());
                }
            }
            Some(Operation::Tui) => {
                let tui_path = cli.alias_file_path.clone().unwrap_or_else(|| {
                    alias_file_paths.first().unwrap_or(&crate::database::persistence::get_default_alias_file_path()).into()
                });
                if let Err(e) = run_tui(tui_path, alias_file_paths) {
                    eprintln!("{}", format!("TUI error: {}", e).red());
                }
            }
            Some(Operation::Init { shell }) => {
                let opts = ShellOpts::new();
                let init_script = render_shell_init(shell.clone(), &opts);
                println!("{}", init_script);
            }
            None => {}
        }
    }
}
