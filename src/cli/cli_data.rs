use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Alman is a command-line tool and TUI for managing shell aliases with intelligent suggestions based on your command history.",
    long_about = "A powerful command-line tool and TUI for managing shell aliases with intelligent suggestions, analytics, and multi-shell support.",
    disable_help_subcommand = true,
    after_help = "EXAMPLES:
  alman add --command \"git status\" gs
  alman remove gs
  alman change old-alias new-alias
  alman list
  alman get-suggestions -n 10
  alman tui"
)]
pub struct Cli {
    /// Path to the alias file to use (default: ~/.alman/aliases)
    #[arg(short, long, value_name = "ALIAS_FILE_PATH", help = "Path to the alias file to use")]
    pub alias_file_path: Option<PathBuf>,

    #[command(subcommand)]
    pub operation: Option<Operation>,
}

#[derive(Subcommand, Debug)]
pub enum Operation {
    /// Add a new alias
    #[command(after_help = "EXAMPLE:
  alman add --command \"git status\" gs")]
    Add {
        /// Command to associate with the alias
        #[arg(short = 'c', long, help = "Command to associate with the alias")]
        command: String,
        /// Alias name to add
        alias: String,
    },
    /// Remove an existing alias
    #[command(after_help = "EXAMPLE:
  alman remove gs")]
    Remove {
        /// Alias name to remove
        alias: String,
    },
    /// List all aliases
    #[command(after_help = "EXAMPLE:
  alman list")]
    List,
    /// Change an existing alias to a new alias
    #[command(after_help = "EXAMPLE:
  alman change old-alias new-alias")]
    Change {
        /// Old alias name
        old_alias: String,
        /// New alias name
        new_alias: String,
    },
    /// Get intelligent alias suggestions based on command history
    #[command(after_help = "EXAMPLE:
  alman get-suggestions -n 10")]
    GetSuggestions {
        /// Number of suggestions to display
        #[arg(short = 'n', long, help = "Number of suggestions to display")]
        num: Option<usize>,
    },
    /// Delete alias suggestions for a specific alias
    #[command(after_help = "EXAMPLE:
  alman delete-suggestion gs")]
    DeleteSuggestion {
        /// Alias name to delete suggestions for
        alias: String,
    },
    /// Launch the interactive terminal user interface (TUI)
    #[command(after_help = "EXAMPLE:
  alman tui")]
    Tui,
    #[command(hide = true)]
    Init {
        #[arg(value_enum, help = "Shell type to initialize (bash, zsh, fish, posix)")]
        shell: InitShell,
    },
    #[command(hide = true)]
    InitData,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum InitShell {
    Bash,
    Zsh,
    Fish,
    #[clap(alias = "ksh")]
    Posix,
}
