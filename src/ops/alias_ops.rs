use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// POSIX-safe single-quote: wrap `s` in `'...'`, replacing every `'` with `'\''`.
pub fn shell_quote_single(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Read alias definitions from `file_path`.
/// Returns an empty Vec if the file does not exist (no side effects).
pub fn get_aliases(file_path: &str) -> Vec<(String, String)> {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut aliases = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("alias ") {
            let rest = rest.trim_start();
            if let Some(eq) = rest.find('=') {
                let alias_name = rest[..eq].trim().to_string();
                let val = rest[eq + 1..].trim();
                let command = unescape_shell_value(val);
                aliases.push((alias_name, command));
            }
        }
    }
    aliases
}

/// Add an alias to `file_path`, preserving all other lines verbatim.
/// Does nothing if the alias already exists. Creates the file if absent.
pub fn add_alias_to_file(file_path: &str, alias: &str, command: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    let lines: Vec<String> = if path.exists() {
        let file = File::open(path)?;
        BufReader::new(file).lines().collect::<io::Result<_>>()?
    } else {
        Vec::new()
    };

    if lines.iter().any(|l| is_alias_line(l, alias)) {
        return Ok(());
    }

    let new_line = format!("alias {}={}", alias, shell_quote_single(command));
    let mut new_lines = lines;
    new_lines.push(new_line);
    write_lines_atomic(file_path, &new_lines)
}

/// Remove an alias from `file_path`, preserving all other lines verbatim.
pub fn remove_alias_from_file(file_path: &str, alias: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let lines: Vec<String> = BufReader::new(file).lines().collect::<io::Result<_>>()?;
    let new_lines: Vec<String> = lines.into_iter().filter(|l| !is_alias_line(l, alias)).collect();
    write_lines_atomic(file_path, &new_lines)
}

pub fn get_aliases_from_multiple_files(file_paths: &[String]) -> Vec<(String, String)> {
    let mut all_aliases = Vec::new();
    for file_path in file_paths {
        all_aliases.extend(get_aliases(file_path));
    }
    all_aliases
}

/// Add alias to the first (primary) file, unless it already exists in any file.
pub fn add_alias_to_multiple_files(file_paths: &[String], alias: &str, command: &str) -> io::Result<()> {
    let all = get_aliases_from_multiple_files(file_paths);
    if all.iter().any(|(a, _)| a == alias) {
        return Ok(());
    }
    if let Some(primary) = file_paths.first() {
        add_alias_to_file(primary, alias, command)?;
    }
    Ok(())
}

/// Force-upsert alias in the primary file (replaces if it exists; adds if not).
/// Also removes the alias from all other files first.
pub fn add_alias_to_multiple_files_force(file_paths: &[String], alias: &str, command: &str) -> io::Result<()> {
    for path in file_paths {
        remove_alias_from_file(path, alias)?;
    }
    if let Some(primary) = file_paths.first() {
        add_alias_to_file(primary, alias, command)?;
    }
    Ok(())
}

/// Remove alias from every file in which it appears.
pub fn remove_alias_from_multiple_files(file_paths: &[String], alias: &str) -> io::Result<()> {
    for file_path in file_paths {
        remove_alias_from_file(file_path, alias)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// True if `line` is an `alias <name>=...` definition for exactly `alias_name`.
fn is_alias_line(line: &str, alias_name: &str) -> bool {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("alias ") {
        let rest = rest.trim_start();
        if let Some(rest2) = rest.strip_prefix(alias_name) {
            let rest2 = rest2.trim_start();
            return rest2.starts_with('=');
        }
    }
    false
}

/// Decode a shell value that may use single-quote, double-quote, or the `'\''` idiom.
fn unescape_shell_value(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    let mut in_single = false;

    while i < chars.len() {
        if in_single {
            if chars[i] == '\'' {
                in_single = false;
            } else {
                result.push(chars[i]);
            }
        } else {
            match chars[i] {
                '\'' => in_single = true,
                '\\' if i + 1 < chars.len() => {
                    result.push(chars[i + 1]);
                    i += 1;
                }
                '"' => {
                    // Consume double-quoted section
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            result.push(chars[i + 1]);
                            i += 2;
                        } else {
                            result.push(chars[i]);
                            i += 1;
                        }
                    }
                }
                c => result.push(c),
            }
        }
        i += 1;
    }
    result
}

/// Write `lines` to `file_path` atomically via a temp file + rename.
/// On failure, the temp file is cleaned up.
fn write_lines_atomic(file_path: &str, lines: &[String]) -> io::Result<()> {
    let path = Path::new(file_path);
    let dir = path.parent().unwrap_or(Path::new("."));

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let tmp_name = format!(".alman_tmp_{}_{}", std::process::id(), ts);
    let tmp_path = dir.join(&tmp_name);

    let result: io::Result<()> = (|| {
        let mut tmp_file = File::create(&tmp_path)?;
        for line in lines {
            writeln!(tmp_file, "{}", line)?;
        }
        tmp_file.flush()?;
        fs::rename(&tmp_path, path)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_file(content: &str) -> tempfile::NamedTempFile {
        let f = tempfile::NamedTempFile::new().unwrap();
        fs::write(f.path(), content).unwrap();
        f
    }

    #[test]
    fn shell_quote_round_trips_simple() {
        let cmd = "git add -A";
        let quoted = shell_quote_single(cmd);
        assert_eq!(unescape_shell_value(&quoted), cmd);
    }

    #[test]
    fn shell_quote_round_trips_with_single_quote() {
        let cmd = "echo 'hello world'";
        let quoted = shell_quote_single(cmd);
        assert_eq!(unescape_shell_value(&quoted), cmd);
    }

    #[test]
    fn add_preserves_comments() {
        let f = tmp_file("# Alman aliases\nalias ga='git add -A'\n");
        let path = f.path().to_string_lossy().to_string();

        add_alias_to_file(&path, "gs", "git status").unwrap();

        let content = fs::read_to_string(f.path()).unwrap();
        assert!(content.contains("# Alman aliases"), "comment lost");
        assert!(content.contains("alias ga="), "existing alias lost");
        assert!(content.contains("alias gs="), "new alias missing");
    }

    #[test]
    fn remove_preserves_other_lines() {
        let f = tmp_file("# header\nalias ga='git add -A'\nalias gs='git status'\n");
        let path = f.path().to_string_lossy().to_string();

        remove_alias_from_file(&path, "ga").unwrap();

        let content = fs::read_to_string(f.path()).unwrap();
        assert!(content.contains("# header"), "comment lost");
        assert!(content.contains("alias gs="), "other alias lost");
        assert!(!content.contains("alias ga="), "removed alias still present");
    }

    #[test]
    fn get_aliases_returns_empty_for_missing_file() {
        let result = get_aliases("/nonexistent/path/to/file");
        assert!(result.is_empty());
        assert!(!std::path::Path::new("/nonexistent/path/to/file").exists(),
            "get_aliases must not create missing files");
    }

    #[test]
    fn add_does_not_duplicate() {
        let f = tmp_file("alias ga='git add -A'\n");
        let path = f.path().to_string_lossy().to_string();

        add_alias_to_file(&path, "ga", "git add .").unwrap();

        let aliases = get_aliases(&path);
        let ga_count = aliases.iter().filter(|(a, _)| a == "ga").count();
        assert_eq!(ga_count, 1, "alias must not be duplicated");
    }
}
