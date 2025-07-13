#!/bin/sh
# Uninstaller for alman

set -e

# Defaults (should match your install script)
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
MAN_DIR="${MAN_DIR:-$HOME/.local/share/man}"

# Allow overrides via command line
while [ "$#" -gt 0 ]; do
    case "$1" in
        --bin-dir) BIN_DIR="$2"; shift 2 ;;
        --bin-dir=*) BIN_DIR="${1#*=}"; shift 1 ;;
        --man-dir) MAN_DIR="$2"; shift 2 ;;
        --man-dir=*) MAN_DIR="${1#*=}"; shift 1 ;;
        -h|--help)
            echo "Usage: uninstall.sh [--bin-dir DIR] [--man-dir DIR]"
            echo ""
            echo "Uninstalls alman from your system."
            echo ""
            echo "Options:"
            echo "  --bin-dir DIR    Override binary directory [default: $HOME/.local/bin]"
            echo "  --man-dir DIR    Override manpage directory [default: $HOME/.local/share/man]"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

BIN_PATH="$BIN_DIR/alman"
BIN_PATH_WIN="$BIN_DIR/alman.exe"
MAN_PATH="$MAN_DIR/man1/alman.1"

echo "Uninstalling alman..."

# Remove binary (Linux/macOS)
if [ -f "$BIN_PATH" ]; then
    rm -f "$BIN_PATH"
    echo "Removed $BIN_PATH"
else
    echo "Binary not found at $BIN_PATH"
fi

# Remove binary (Windows/MSYS)
if [ -f "$BIN_PATH_WIN" ]; then
    rm -f "$BIN_PATH_WIN"
    echo "Removed $BIN_PATH_WIN"
else
    echo "Windows binary not found at $BIN_PATH_WIN"
fi

# Remove man page
if [ -f "$MAN_PATH" ]; then
    rm -f "$MAN_PATH"
    echo "Removed $MAN_PATH"
else
    echo "Man page not found at $MAN_PATH"
fi

# Optionally, remove empty man1 and man directories
if [ -d "$MAN_DIR/man1" ] && [ ! "$(ls -A "$MAN_DIR/man1")" ]; then
    rmdir "$MAN_DIR/man1"
    echo "Removed empty $MAN_DIR/man1"
fi
if [ -d "$MAN_DIR" ] && [ ! "$(ls -A "$MAN_DIR")" ]; then
    rmdir "$MAN_DIR"
    echo "Removed empty $MAN_DIR"
fi

# Remove old .alman directory (legacy location)
OLD_ALMAN_DIR="$HOME/.alman"
if [ -d "$OLD_ALMAN_DIR" ]; then
    rm -rf "$OLD_ALMAN_DIR"
    echo "Removed legacy directory $OLD_ALMAN_DIR"
else
    echo "Legacy directory not found at $OLD_ALMAN_DIR"
fi

echo ""
echo "alman has been uninstalled."
echo ""
echo "Note: If you added alman initialization to your shell configuration files,"
echo "you may want to remove those lines manually:"
echo ""
echo "  From ~/.bashrc: Remove 'eval \"\$(alman init bash)\"'"
echo "  From ~/.zshrc:  Remove 'eval \"\$(alman init zsh)\"'"
echo "  From ~/.config/fish/config.fish: Remove 'eval (alman init fish)'"

exit 0 