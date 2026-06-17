use std::fs::File;
use std::io::{BufRead, BufReader};

/// Read alias definitions from `file_path`.
/// Returns an empty Vec if the file does not exist (no side effects).
/// Used for one-time migration and export of existing alias files.
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

pub fn get_aliases_from_multiple_files(file_paths: &[String]) -> Vec<(String, String)> {
    let mut all_aliases = Vec::new();
    for file_path in file_paths {
        all_aliases.extend(get_aliases(file_path));
    }
    all_aliases
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
    fn get_aliases_returns_empty_for_missing_file() {
        let result = get_aliases("/nonexistent/path/to/file");
        assert!(result.is_empty());
        assert!(
            !std::path::Path::new("/nonexistent/path/to/file").exists(),
            "get_aliases must not create missing files"
        );
    }

    #[test]
    fn get_aliases_parses_single_and_double_quotes() {
        let f = tmp_file("# header\nalias ga='git add -A'\nalias gs=\"git status\"\n");
        let path = f.path().to_string_lossy().to_string();
        let aliases = get_aliases(&path);
        assert!(aliases.iter().any(|(a, c)| a == "ga" && c == "git add -A"));
        assert!(aliases.iter().any(|(a, c)| a == "gs" && c == "git status"));
    }

    #[test]
    fn unescape_round_trips_embedded_single_quote() {
        // 'echo '\''hi'\''' decodes back to echo 'hi'
        let decoded = unescape_shell_value(r"'echo '\''hi'\'''");
        assert_eq!(decoded, "echo 'hi'");
    }
}
