# alman - Intelligent Alias Manager

A command-line tool and TUI for managing shell aliases with intelligent suggestions based on your command history. Alman helps you organize, create, and manage aliases across multiple files and shells, making your workflow faster and smarter.

## Installation

### Universal Install Script

The easiest way to install `alman` on any system:

```bash
curl -sSfL https://raw.githubusercontent.com/vaibhav-mattoo/alman/main/install.sh | sh
```

This script will automatically detect your system and install the appropriate binary.

Remember to add `~/.local/bin` to your `$PATH` if prompted by the install script, by adding `export PATH="$HOME/.local/bin:$PATH"` in the end of your shell config (~/.bashrc, ~/.zshrc etc).

### From Cargo

```bash
cargo install alman
```

### From AUR (Arch Linux)

Using `yay`:
```bash
yay -S alman
```

Using `paru`:
```bash
paru -S alman
```

### From Source

```bash
git clone https://github.com/vaibhav-mattoo/alman.git
cd alman
cargo install --path .
```

## Shell Configuration

After installation, you need to configure your shell to use alman. The installer will detect your shell and provide specific instructions, but here are the general steps:

### Bash
Add this line to your `~/.bashrc`:
```bash
eval "$(alman init bash)"
```

Then reload your configuration:
```bash
source ~/.bashrc
```

### Zsh
Add this line to your `~/.zshrc`:
```bash
eval "$(alman init zsh)"
```

Then reload your configuration:
```bash
source ~/.zshrc
```

### Fish
Add this line to your `~/.config/fish/config.fish`:
```fish
eval (alman init fish)
```

Then reload your configuration:
```fish
source ~/.config/fish/config.fish
```

## Table of Contents

- [Quick Start](#quick-start)
- [Interactive Mode](#interactive-mode)
- [Command Line Usage](#command-line-usage)
- [Usage Examples](#usage-examples)
- [Advanced Usage](#advanced-usage)
- [TUI Navigation](#tui-navigation)
- [Command Line Options](#command-line-options)
- [Output Format](#output-format)
- [Use Cases](#use-cases)
- [Uninstallation](#uninstallation)
- [License](#license)

## Quick Start

### Interactive Mode
Launch the interactive alias manager:

```bash
alman
# or
alman tui
```

Navigate with arrow keys or `jk`, select aliases, and manage them interactively.

### Command Line Mode
Add, remove, list, and get suggestions for aliases directly from the command line:

```bash
# Add an alias
alman add -c "git status" gs

# Remove an alias
alman remove gs

# List all aliases
alman list

# Get alias suggestions
alman get-suggestions -n 10
```

## Interactive Mode

The Terminal User Interface (TUI) provides an intuitive way to browse, add, remove, and change aliases:

### Navigation
- **Arrow keys** or **jk**: Move cursor
- **Enter**: Select
- **a**: Add alias
- **r**: Remove alias
- **l**: List aliases
- **q** or **Ctrl+C**: Quit

### TUI Features
- **Visual selection**: Selected items are highlighted
- **Alias suggestions**: Get smart suggestions based on your command history
- **Multi-file support**: Manage aliases across multiple files

## Command Line Usage

### Basic Commands

```bash
# Add a new alias
alman add -c "ls -la" ll

# Remove an alias
alman remove ll

# List all aliases
alman list

# Get intelligent suggestions
alman get-suggestions -n 5
```

## Usage Examples

### Basic Usage

```bash
# Add a new alias
alman add -c "ls -la" ll

# Remove an alias
alman remove ll

# List all aliases
alman list

# Get intelligent suggestions
alman get-suggestions -n 5
```

### Advanced Usage

```bash
# Change an alias name (keeps the same command)
alman change old-alias new-alias

# Delete suggestions for an alias
alman delete-suggestion gs

# Use a specific alias file
alman --alias-file-path ~/.my-aliases add -c "htop" h
```

## Advanced Usage

### Multi-file Management

```bash
# Add alias to a specific file
alman --alias-file-path ~/.bash_aliases add -c "ls -lh" lh

# List aliases from a specific file
alman --alias-file-path ~/.zsh_aliases list
```

### Suggestion Management

```bash
# Get more suggestions
alman get-suggestions -n 10

# Delete a specific suggestion
alman delete-suggestion gs
```

## TUI Navigation

The Terminal User Interface provides an intuitive way to manage aliases:

### Key Bindings
- **Arrow keys** or **jk**: Navigate through aliases
- **Enter**: Select an alias or action
- **a**: Add a new alias
- **r**: Remove selected alias
- **l**: List all aliases
- **q** or **Ctrl+C**: Exit the interface

### Features
- **Visual feedback**: Selected items are highlighted
- **Smart suggestions**: Get intelligent alias suggestions
- **Multi-file support**: Manage aliases across different files

## Command Line Options

### Output Options
- `-c, --command <COMMAND>`: Command to associate with the alias (for `add` and `change`)
- `-n, --num <N>`: Number of suggestions to display (for `get-suggestions`)
- `--alias-file-path <PATH>`: Path to the alias file to use

### Examples

```bash
# Add an alias to a specific file
alman --alias-file-path ~/.bash_aliases add -c "ls -lh" lh

# Get 10 suggestions
alman get-suggestions -n 10
```

## Output Format

Alman displays aliases in a clear, tabular format:

```
┌─────────┬───────────────┐
│ ALIAS   │ COMMAND       │
├─────────┼───────────────┤
│ gs      │ git status    │
│ ll      │ ls -la        │
└─────────┴───────────────┘
```

## Use Cases

Perfect for managing your shell aliases, discovering new shortcuts, and keeping your workflow efficient:

```bash
# Quick alias management
alman tui

# Add and remove aliases on the fly
alman add -c "git pull" gp
alman remove gp

# Get suggestions for new aliases
alman get-suggestions -n 5
```

## Uninstallation

To uninstall `alman`, you can run the command:

```bash
curl -sSfL https://raw.githubusercontent.com/vaibhav-mattoo/alman/main/uninstall.sh | sh
```

Or download and run the uninstall script manually:

```bash
curl -sSfL https://raw.githubusercontent.com/vaibhav-mattoo/alman/main/uninstall.sh -o uninstall.sh
chmod +x uninstall.sh
./uninstall.sh
```

**Note**: After uninstalling, remember to remove the shell configuration lines from your shell config files:
- From `~/.bashrc`: Remove `eval "$(alman init bash)"`
- From `~/.zshrc`: Remove `eval "$(alman init zsh)"`
- From `~/.config/fish/config.fish`: Remove `eval (alman init fish)`

## License

MIT License - see LICENSE file for details.
