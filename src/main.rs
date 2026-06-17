mod cli;
mod database;
mod ops;
mod tui;
mod shell;

use cli::arg_handler::parse_args;
use cli::cli_data::Operation;
use database::db::{get_db_path, open, open_for_write};
use database::history_loader::compact;
use database::persistence::{
    ensure_config_directory, ensure_data_directory, get_default_alias_file_path,
    load_config, save_config, AppConfig,
};
use ops::apply::{apply_add, apply_change, apply_remove, ApplyOutcome};
use ops::alias_suggestions::is_system_command;
use ops::delete_suggestion::delete_suggestion;
use ops::get_suggestions;
use ops::insert_command::insert_command;
use shell::{render_shell_init, ShellOpts};
use std::fs;
use std::path::{Path, PathBuf};
use tui::run_tui;
use colored::*;
use clap::CommandFactory;

fn to_absolute_path(path: &str) -> String {
    let pb = PathBuf::from(path);
    match pb.canonicalize() {
        Ok(abs) => abs.to_string_lossy().to_string(),
        Err(_) => {
            if let Some(parent) = pb.parent() {
                if let Ok(abs_parent) = parent.canonicalize() {
                    return abs_parent
                        .join(pb.file_name().unwrap_or_default())
                        .to_string_lossy()
                        .to_string();
                }
            }
            pb.to_string_lossy().to_string()
        }
    }
}

fn print_source_message() {
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
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Show help before touching the DB.
    if args.iter().any(|a| a == "--help" || a == "-h") {
        let default_path = load_config()
            .and_then(|c| c.alias_file_paths.first().map(|p| to_absolute_path(p)))
            .unwrap_or_else(get_default_alias_file_path);
        println!("Current default alias file path: {}\n", default_path.green());
        <crate::cli::cli_data::Cli as CommandFactory>::command()
            .print_help()
            .unwrap();
        println!();
        std::process::exit(0);
    }

    // Fast path: list-alias-files only needs config, not the DB.
    if args.len() > 1 && args[1] == "list-alias-files" {
        let paths = load_config()
            .map(|c| c.alias_file_paths)
            .unwrap_or_else(|| vec![get_default_alias_file_path()]);
        for path in paths {
            println!("{}", path);
        }
        return;
    }

    if let Err(e) = ensure_data_directory() {
        eprintln!("Failed to create data directory: {}", e);
        return;
    }

    let config = load_config();
    let mut alias_file_paths = config
        .as_ref()
        .map(|c| c.alias_file_paths.clone())
        .unwrap_or_else(|| vec![get_default_alias_file_path()]);

    // Fast path: `custom` uses the lightweight write-only open (no UDF, no migration).
    if args.len() > 1 && args[1] == "custom" {
        if args.len() < 3 {
            eprintln!("Usage: {} custom <command> [--cwd <dir>] [--session <id>]", args[0]);
            return;
        }

        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut cmd_parts: Vec<&str> = Vec::new();
        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--session" if i + 1 < args.len() => { session_id = Some(args[i + 1].clone()); i += 2; }
                "--cwd" if i + 1 < args.len() => { cwd = Some(args[i + 1].clone()); i += 2; }
                _ => { cmd_parts.push(&args[i]); i += 1; }
            }
        }
        let command = cmd_parts.join(" ");

        let db_path = get_db_path();
        let conn = match open_for_write(&db_path) {
            Ok(c) => c,
            Err(e) => { eprintln!("alman: DB error: {e}"); return; }
        };
        insert_command(command, &conn, session_id.as_deref(), cwd.as_deref());
        return;
    }

    // Parse CLI for all other subcommands.
    let cli = parse_args();

    // Bare --alias-file-path with no subcommand: update config and exit.
    if cli.operation.is_none() {
        if let Some(ref cli_path) = cli.alias_file_path {
            let cli_path_str = to_absolute_path(&cli_path.to_string_lossy());
            if !alias_file_paths.contains(&cli_path_str) {
                alias_file_paths.push(cli_path_str.clone());
            }
            if let Some(pos) = alias_file_paths.iter().position(|p| p == &cli_path_str) {
                let new_default = alias_file_paths.remove(pos);
                alias_file_paths.insert(0, new_default);
            }
            let _ = save_config(&AppConfig { alias_file_paths });
            println!("Default alias file path set to {}", cli_path_str.green());
        } else {
            // No subcommand and no path flag → launch TUI.
            let file_path = alias_file_paths
                .first()
                .cloned()
                .unwrap_or_else(get_default_alias_file_path);
            let db_path = get_db_path();
            let conn = match open(&db_path) {
                Ok(c) => c,
                Err(e) => { eprintln!("alman: DB error: {e}"); return; }
            };
            if let Err(e) = run_tui(PathBuf::from(file_path), alias_file_paths, conn) {
                eprintln!("{}", format!("TUI error: {}", e).red());
            }
        }
        return;
    }

    // Add CLI-provided alias path to config if new.
    if let Some(ref cli_path) = cli.alias_file_path {
        let cli_path_str = to_absolute_path(&cli_path.to_string_lossy());
        if !alias_file_paths.contains(&cli_path_str) {
            alias_file_paths.push(cli_path_str);
            let _ = save_config(&AppConfig { alias_file_paths: alias_file_paths.clone() });
        }
    }

    // Open DB for subcommands that need it (all except Init, InitData, ListAliasFiles).
    let open_conn = || -> Option<rusqlite::Connection> {
        match open(&get_db_path()) {
            Ok(c) => Some(c),
            Err(e) => { eprintln!("alman: DB error: {e}"); None }
        }
    };

    match cli.operation.as_ref().unwrap() {
        Operation::Add { alias, command } => {
            let Some(conn) = open_conn() else { return; };
            match apply_add(&conn, &alias_file_paths, alias, command) {
                Ok(_) => print_source_message(),
                Err(e) => eprintln!("{}", format!("Error adding alias: {}", e).red()),
            }
        }
        Operation::Remove { alias } => {
            let Some(conn) = open_conn() else { return; };
            match apply_remove(&conn, &alias_file_paths, alias) {
                Ok(ApplyOutcome::NotFound { alias }) => {
                    eprintln!("{}", format!("Alias '{}' not found.", alias).red());
                }
                Ok(_) => print_source_message(),
                Err(e) => eprintln!("{}", format!("Error removing alias: {}", e).red()),
            }
        }
        Operation::List => {
            use ops::alias_ops::get_aliases_from_multiple_files;
            let aliases = get_aliases_from_multiple_files(&alias_file_paths);
            if aliases.is_empty() {
                println!("{}", "No aliases found.".yellow());
                return;
            }
            let max_alias_length = aliases.iter().map(|(a, _)| a.len()).max().unwrap_or(5).max(5);
            let max_command_length = aliases.iter().map(|(_, c)| c.len()).max().unwrap_or(7).max(7);
            println!("{}", format!("┌{:─<alias$}┬{:─<cmd$}┐", "", "", alias = max_alias_length + 2, cmd = max_command_length + 2).cyan());
            println!("{}", format!("│ {:<alias$} │ {:<cmd$} │", "ALIAS", "COMMAND", alias = max_alias_length, cmd = max_command_length).cyan());
            println!("{}", format!("├{:─<alias$}┼{:─<cmd$}┤", "", "", alias = max_alias_length + 2, cmd = max_command_length + 2).cyan());
            for (alias, command) in &aliases {
                println!("│ {} │ {} │",
                    format!("{:<width$}", alias, width = max_alias_length).cyan(),
                    format!("{:<width$}", command, width = max_command_length),
                );
            }
            println!("{}", format!("└{:─<alias$}┴{:─<cmd$}┘", "", "", alias = max_alias_length + 2, cmd = max_command_length + 2).cyan());
            println!("{}", format!("Total: {} alias(es) across {} file(s)", aliases.len(), alias_file_paths.len()).green());
        }
        Operation::Change { old_alias, new_alias } => {
            let Some(conn) = open_conn() else { return; };
            match apply_change(&conn, &alias_file_paths, old_alias, new_alias) {
                Ok(ApplyOutcome::NotFound { alias }) => {
                    eprintln!("{}", format!("Alias '{}' not found.", alias).red());
                }
                Ok(_) => print_source_message(),
                Err(e) => eprintln!("{}", format!("Error changing alias: {}", e).red()),
            }
        }
        Operation::GetSuggestions { num } => {
            let Some(conn) = open_conn() else { return; };
            if let Some(n) = num {
                if *n == 0 {
                    eprintln!("{}", "Number of suggestions must be greater than 0.".red());
                    return;
                }
            }
            let default_path = get_default_alias_file_path();
            let alias_path = alias_file_paths.first().unwrap_or(&default_path);
            let list = get_suggestions::get_suggestions_with_aliases(*num, &conn, alias_path);

            if list.is_empty() {
                println!("{}", "No suggestions found.".yellow());
                return;
            }

            let filtered: Vec<_> = list.iter().map(|cmd| {
                let top_alias = cmd.alias_suggestions.iter().find(|a| !is_system_command(&a.alias));
                (cmd, top_alias)
            }).collect();

            let max_command_length = filtered.iter().map(|(c, _)| c.command.command_text.len()).max().unwrap_or(7).max(7);
            let max_alias_length   = filtered.iter().map(|(_, a)| a.map(|x| x.alias.len()).unwrap_or(0)).max().unwrap_or(9).max(9);
            let max_score_length   = filtered.iter().map(|(c, _)| format!("{:.0}", c.command.score).len()).max().unwrap_or(5).max(5);

            println!("{}", format!("┌{:─<cmd$}┬{:─<alias$}┬{:─<score$}┐", "", "", "", cmd = max_command_length + 2, alias = max_alias_length + 2, score = max_score_length + 2).cyan());
            println!("{}", format!("│ {:<cmd$} │ {:>alias$} │ {:>score$} │", "COMMAND", "TOP ALIAS", "SCORE", cmd = max_command_length, alias = max_alias_length, score = max_score_length).cyan());
            println!("{}", format!("├{:─<cmd$}┼{:─<alias$}┼{:─<score$}┤", "", "", "", cmd = max_command_length + 2, alias = max_alias_length + 2, score = max_score_length + 2).cyan());

            for (cmd_with_alias, top_alias_opt) in &filtered {
                let command_text = format!("{:<width$}", cmd_with_alias.command.command_text, width = max_command_length);
                let alias_text = top_alias_opt
                    .map(|a| format!("{:>width$}", a.alias, width = max_alias_length))
                    .unwrap_or_else(|| format!("{:>width$}", "", width = max_alias_length));
                let score_text = format!("{:>width$}", format!("{:.0}", cmd_with_alias.command.score), width = max_score_length);
                println!("│ {} │ {} │ {} │",
                    command_text.bold(),
                    alias_text.cyan(),
                    score_text.yellow(),
                );
            }

            println!("{}", format!("└{:─<cmd$}┴{:─<alias$}┴{:─<score$}┘", "", "", "", cmd = max_command_length + 2, alias = max_alias_length + 2, score = max_score_length + 2).cyan());
            println!("{}", format!("Total: {} suggestion(s)", filtered.len()).green());
        }
        Operation::DeleteSuggestion { alias } => {
            let Some(conn) = open_conn() else { return; };
            delete_suggestion(alias, &conn);
            println!("{}", format!("Deleted suggestions for: {}", alias).yellow());
        }
        Operation::Tui => {
            let tui_path = cli.alias_file_path.clone().unwrap_or_else(|| {
                alias_file_paths.first().unwrap_or(&get_default_alias_file_path()).into()
            });
            let Some(conn) = open_conn() else { return; };
            if let Err(e) = run_tui(tui_path, alias_file_paths, conn) {
                eprintln!("{}", format!("TUI error: {}", e).red());
            }
        }
        Operation::Init { shell } => {
            let opts = ShellOpts::new();
            println!("{}", render_shell_init(shell.clone(), &opts));
        }
        Operation::InitData => {
            if let Err(e) = ensure_data_directory() {
                eprintln!("Failed to create data directory: {}", e);
                return;
            }
            if let Err(e) = ensure_config_directory() {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
            if load_config().is_none() {
                let _ = save_config(&AppConfig {
                    alias_file_paths: vec![get_default_alias_file_path()],
                });
            }
            // Opening the DB creates the schema, runs migration, and seeds from history if new.
            let conn = match open(&get_db_path()) {
                Ok(c) => c,
                Err(e) => { eprintln!("Failed to initialize database: {}", e); return; }
            };
            compact(&conn);
            let default_alias_path = get_default_alias_file_path();
            if !Path::new(&default_alias_path).exists() {
                if let Some(parent) = Path::new(&default_alias_path).parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("Failed to create alias file directory: {}", e);
                        return;
                    }
                }
                let _ = fs::write(&default_alias_path, "# Alman aliases file\n");
            }
        }
        Operation::ListAliasFiles => {
            // Handled in the fast-path above; unreachable here.
        }
    }
}
