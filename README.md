# alman - Intelligent Alias Manager

[![Last Commit](https://img.shields.io/github/last-commit/vaibhav-mattoo/cxt)](https://github.com/vaibhav-mattoo/cxt/commits)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/alman)](https://crates.io/crates/alman)
[![AUR version](https://img.shields.io/aur/version/alman?logo=arch-linux)](https://aur.archlinux.org/packages/alman)

A command-line tool and TUI for managing shell aliases with intelligent suggestions based on your command history. Alman helps you organize, create, and manage aliases across multiple files and shells, making your workflow faster and smarter.

## 🎨 Showcase

Watch alman in action! See how it can transform your command-line workflow with intelligent alias suggestions and intuitive management.

## 🚀 Installation

> [!IMPORTANT]
> **Shell Configuration Required**: After installation, you **must** add the shell configuration line to your shell config file (check with `which $SHELL`) or the app will not work. See the [Shell Configuration](#%EF%B8%8F-shell-configuration) section below for detailed instructions.

### Universal Install Script

The easiest way to install `alman` on any system:

```bash
curl -sSfL https://raw.githubusercontent.com/vaibhav-mattoo/alman/main/install.sh | sh
```

This script will automatically detect your system and install the appropriate binary.

> [!NOTE]
> Remember to add `~/.local/bin` to your `$PATH` if prompted by the install script, by adding `export PATH="$HOME/.local/bin:$PATH"` in the end of your shell config (~/.bashrc, ~/.zshrc etc).

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

## ⚙️ Shell Configuration

After installation, you need to configure your shell to use alman. The installer will detect your shell and provide specific instructions, but here are the general steps:

> [!NOTE]
> The installer will automatically detect your shell and show you the exact configuration line to add to your shell config file.

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
alman init fish | source
```

Then reload your configuration:
```fish
source ~/.config/fish/config.fish
```

> [!TIP]
> Alman automatically initializes with your shell history when first run, so you'll have intelligent suggestions right from the start!

## 📋 Table of Contents

<!-- disabledMarkdownTOC autolink="false" markdown_preview="github" -->

- [Showcase](#-showcase)
- [Installation](#-installation)
    - [Universal Install Script](#universal-install-script)
    - [From Cargo](#from-cargo)
    - [From AUR (Arch Linux)](#from-aur-arch-linux)
    - [From Source](#from-source)
- [Shell Configuration](#%EF%B8%8F-shell-configuration)
    - [Bash](#bash)
    - [Zsh](#zsh)
    - [Fish](#fish)
- [Quick Start](#-quick-start)
    - [Interactive Mode](#interactive-mode)
    - [Command Line Mode](#command-line-mode)
- [Interactive Mode](#-interactive-mode)
    - [Navigation](#navigation)
    - [TUI Features](#tui-features)
- [Command Line Usage](#-command-line-usage)
    - [Basic Commands](#basic-commands)
- [Usage Examples](#-usage-examples)
    - [Basic Usage](#basic-usage)
    - [Advanced Usage](#advanced-usage)
    - [Change Command Examples](#change-command-examples)
- [Advanced Usage](#-advanced-usage)
    - [Multi-file Management](#multi-file-management)
    - [Suggestion Management](#suggestion-management)
    - [Alias Management](#alias-management)
- [TUI Navigation](#-tui-navigation)
    - [Key Bindings](#key-bindings)
    - [Features](#features)
- [Command Line Options](#-command-line-options)
    - [Output Options](#output-options)
    - [Examples](#examples)
- [Output Format](#-output-format)
- [Ranking Algorithm](#-ranking-algorithm)
    - [Scoring Formula](#scoring-formula)
    - [Time-Based Multipliers](#time-based-multipliers)
    - [Factors Explained](#factors-explained)
- [Alias Suggestion Schemes](#-alias-suggestion-schemes)
    - [Vowel Removal](#vowel-removal)
    - [Abbreviation](#abbreviation)
    - [First Letter Combination](#first-letter-combination)
    - [Smart Truncation](#smart-truncation)
    - [Prefix Matching](#prefix-matching)
- [Use Cases](#-use-cases)
- [Uninstallation](#-uninstallation)
- [License](#-license)

<!-- /MarkdownTOC -->

## 🚀 Quick Start

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

## 🖥️ Interactive Mode

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

## 💻 Command Line Usage

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

## 📝 Usage Examples

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

> [!IMPORTANT]
> After running `alman change old new` and sourcing your aliases, only the new alias will work. The old alias will be completely removed from all managed alias files.

## 🔧 Advanced Usage

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

## 🎮 TUI Navigation

The Terminal User Interface provides an intuitive way to manage aliases:

> [!TIP]
> The TUI mode is perfect for browsing your command history and discovering new alias opportunities!

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

## ⚙️ Command Line Options

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

## 📊 Output Format

Alman displays aliases in a clear, tabular format:

```
┌─────────┬───────────────┐
│ ALIAS   │ COMMAND       │
├─────────┼───────────────┤
│ gs      │ git status    │
│ ll      │ ls -la        │
└─────────┴───────────────┘
```

## 🧠 Ranking Algorithm

Alman uses a sophisticated scoring algorithm to rank commands based on three key factors:

### Scoring Formula
```
Score = Time Multiplier × Length^(3/5) × Frequency
```

### Time-Based Multipliers
- **≤ 1 hour**: 4.0× (recent commands get highest priority)
- **≤ 1 day**: 2.0× (recent commands)
- **≤ 1 week**: 0.5× (older commands)
- **> 1 week**: 0.25× (very old commands)

### Factors Explained
- **Recency**: Recently used commands score higher, encouraging current workflow patterns
- **Frequency**: More frequently used commands get higher scores
- **Length**: Longer commands get slightly higher scores (using length^(3/5) to avoid excessive bias)
- **Automatic Reset**: When total score exceeds 70,000, all frequencies are reduced by 50% to prevent score inflation

> [!TIP]
> The algorithm automatically adapts to your usage patterns, prioritizing commands you use most frequently and recently!

## 🎯 Alias Suggestion Schemes

Alman employs multiple intelligent schemes to generate meaningful alias suggestions:

### Vowel Removal
Removes vowels to create shorter, memorable aliases:
- `git status` → `gst` (removes 'i', 'a', 'u')
- `docker ps` → `dckr ps` (removes 'o', 'e')

### Abbreviation
Creates abbreviations from command words:
- `git pull` → `gp`
- `ls -la` → `ll`
- `npm install` → `ni`

### First Letter Combination
Combines first letters of each word:
- `git status` → `gs`
- `docker compose` → `dc`
- `systemctl status` → `ss`

### Smart Truncation
Truncates long commands intelligently:
- `git checkout` → `gco`
- `docker build` → `db`
- `npm run dev` → `nrd`

### Prefix Matching
Suggests aliases based on common command prefixes:
- `git` commands → `g` + first letter of subcommand
- `docker` commands → `d` + first letter of subcommand

> [!NOTE]
> Alman evaluates all these schemes and ranks suggestions by their effectiveness and memorability, ensuring you get the most useful aliases first.

## 🎯 Use Cases

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

> [!TIP]
> Try the interactive TUI mode (`alman tui`) for the most intuitive alias management experience!

## 🗑️ Uninstallation

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

> [!NOTE]
> After uninstalling, remember to remove the shell configuration lines from your shell config files:
> - From `~/.bashrc`: Remove `eval "$(alman init bash)"`
> - From `~/.zshrc`: Remove `eval "$(alman init zsh)"`
> - From `~/.config/fish/config.fish`: Remove `eval (alman init fish)`

## 📄 License

MIT License - see LICENSE file for details.
