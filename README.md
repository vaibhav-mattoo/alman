# alman - Intelligent Alias Manager

A command-line tool and TUI for managing shell aliases with intelligent suggestions based on your command history. Alman helps you organize, create, and manage aliases across multiple files and shells, making your workflow faster and smarter.

## Use Case

When you want to efficiently manage and discover useful shell aliases, Alman makes it effortless. Instead of manually editing alias files, you can add, remove, list, and get intelligent suggestions for aliases, all from the command line or an interactive TUI.

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

### From Source

```bash
git clone https://github.com/vaibhav-mattoo/alman.git
cd alman
cargo install --path .
```

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
# Change an alias and its command
alman change old-alias new-alias "new command"

# Delete suggestions for an alias
alman delete-suggestion gs

# Use a specific alias file
alman --alias-file-path ~/.my-aliases add -c "htop" h
```

## Interactive TUI Mode

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

## License

MIT License - see LICENSE file for details.

## Uninstall

To uninstall `alman`, you can run the command:

```bash
curl -sSfL https://raw.githubusercontent.com/vaibhav-mattoo/alman/main/uninstall.sh | sh
```

If you installed the software using a package manager, remove it using the package manager's uninstall command.

---
