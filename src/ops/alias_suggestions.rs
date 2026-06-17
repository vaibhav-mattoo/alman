use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AliasSuggestion {
    pub alias: String,
    pub command: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct AliasSuggester {
    existing_aliases: HashSet<String>,
    system_commands: HashSet<String>,
}

/// Check whether `cmd` is an executable file on `PATH`.
pub fn is_system_command(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }
    if let Ok(paths) = env::var("PATH") {
        for path_dir in paths.split(':') {
            let full = Path::new(path_dir).join(cmd);
            if full.exists()
                && fs::metadata(&full)
                    .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

/// Build the set of all executable names found on `PATH`.
fn scan_path_commands() -> HashSet<String> {
    let mut commands = HashSet::new();
    if let Ok(path) = env::var("PATH") {
        for path_dir in path.split(':') {
            if let Ok(entries) = fs::read_dir(path_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                            if let Some(name) = entry.file_name().to_str() {
                                commands.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    commands
}

impl AliasSuggester {
    /// Build a suggester from one or more alias files.
    /// System-command detection uses only PATH (no interactive shell spawn).
    pub fn new(alias_file_paths: &[String]) -> Self {
        let mut existing_aliases = HashSet::new();
        for path in alias_file_paths {
            use crate::ops::alias_ops::get_aliases;
            for (alias, _) in get_aliases(path) {
                existing_aliases.insert(alias);
            }
        }
        let system_commands = scan_path_commands();
        Self { existing_aliases, system_commands }
    }

    pub fn suggest_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();

        suggestions.extend(self.generate_semantic_aliases(command));
        suggestions.extend(self.generate_abbreviation_aliases(command));
        suggestions.extend(self.generate_vowel_removal_aliases(command));
        suggestions.extend(self.generate_combined_aliases(command));
        suggestions.extend(self.generate_single_word_aliases(command));
        suggestions.extend(self.generate_truncated_aliases(command));
        suggestions.extend(self.generate_syllable_aliases(command));
        suggestions.extend(self.generate_phonetic_aliases(command));
        suggestions.extend(self.generate_keyboard_pattern_aliases(command));
        suggestions.extend(self.generate_smart_prefix_aliases(command));
        suggestions.extend(self.generate_common_pattern_aliases(command));

        suggestions.retain(|s| !self.has_conflicts(&s.alias));

        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.alias.clone()));

        suggestions.sort_by_key(|b| std::cmp::Reverse(self.get_priority(b)));
        suggestions
    }

    fn generate_semantic_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return suggestions;
        }

        let tool = parts[0];
        let args = &parts[1..];

        if tool.starts_with("./") || tool.starts_with("../") {
            suggestions.extend(self.generate_relative_path_aliases(command, tool, args));
            return suggestions;
        }

        if let Some(semantic_alias) = self.generate_tool_specific_alias(tool, args) {
            suggestions.push(semantic_alias);
        }

        if !args.is_empty() {
            let subcommand = args[0];
            let combined = format!("{}{}", tool.chars().next().unwrap_or('x'), subcommand);
            suggestions.push(AliasSuggestion {
                alias: combined,
                command: command.to_string(),
                reason: format!("{}-{} combination", tool, subcommand),
            });
        }

        suggestions
    }

    fn generate_relative_path_aliases(&self, command: &str, tool: &str, args: &[&str]) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let executable_name = tool.split('/').next_back().unwrap_or(tool);

        if let Some(name) = executable_name.strip_suffix(".exe") {
            suggestions.push(AliasSuggestion {
                alias: name.to_string(),
                command: command.to_string(),
                reason: "Executable name".to_string(),
            });
        } else {
            suggestions.push(AliasSuggestion {
                alias: executable_name.to_string(),
                command: command.to_string(),
                reason: "Executable name".to_string(),
            });
        }

        if executable_name.len() > 2 {
            let abbrev = executable_name.chars().take(3).collect::<String>();
            suggestions.push(AliasSuggestion {
                alias: abbrev,
                command: command.to_string(),
                reason: "Executable abbreviation".to_string(),
            });
        }

        if !args.is_empty() {
            let first_arg = args[0];
            let combined = format!("{}{}", executable_name.chars().next().unwrap_or('x'), first_arg);
            suggestions.push(AliasSuggestion {
                alias: combined,
                command: command.to_string(),
                reason: format!("{}-{} combination", executable_name, first_arg),
            });
        }

        suggestions
    }

    fn generate_tool_specific_alias(&self, tool: &str, args: &[&str]) -> Option<AliasSuggestion> {
        if args.is_empty() {
            return None;
        }
        let subcommand = args[0];
        let remaining_args = &args[1..];
        match tool {
            "git" => self.generate_git_alias(subcommand, remaining_args),
            "docker" => self.generate_docker_alias(subcommand),
            "npm" => self.generate_npm_alias(subcommand),
            "ssh" => self.generate_ssh_alias(subcommand),
            _ => None,
        }
    }

    fn generate_git_alias(&self, subcommand: &str, remaining_args: &[&str]) -> Option<AliasSuggestion> {
        match subcommand {
            "status" => Some(AliasSuggestion { alias: "gs".to_string(), command: "git status".to_string(), reason: "Git status".to_string() }),
            "add" => {
                if !remaining_args.is_empty() && remaining_args[0] == "." {
                    Some(AliasSuggestion { alias: "gaa".to_string(), command: "git add .".to_string(), reason: "Git add all".to_string() })
                } else {
                    Some(AliasSuggestion { alias: "ga".to_string(), command: "git add".to_string(), reason: "Git add".to_string() })
                }
            }
            "commit" => {
                if remaining_args.len() >= 2 && remaining_args[0] == "-m" {
                    Some(AliasSuggestion { alias: "gcm".to_string(), command: format!("git commit -m \"{}\"", remaining_args[1]), reason: "Git commit with message".to_string() })
                } else {
                    Some(AliasSuggestion { alias: "gc".to_string(), command: "git commit".to_string(), reason: "Git commit".to_string() })
                }
            }
            "checkout" => {
                if !remaining_args.is_empty() && remaining_args[0] == "-b" {
                    Some(AliasSuggestion { alias: "gcb".to_string(), command: format!("git checkout -b {}", remaining_args.get(1).unwrap_or(&"")), reason: "Git checkout new branch".to_string() })
                } else {
                    Some(AliasSuggestion { alias: "gco".to_string(), command: "git checkout".to_string(), reason: "Git checkout".to_string() })
                }
            }
            "push" => Some(AliasSuggestion { alias: "gp".to_string(), command: "git push".to_string(), reason: "Git push".to_string() }),
            "pull" => Some(AliasSuggestion { alias: "gl".to_string(), command: "git pull".to_string(), reason: "Git pull".to_string() }),
            "log" => Some(AliasSuggestion { alias: "glg".to_string(), command: "git log".to_string(), reason: "Git log".to_string() }),
            "branch" => Some(AliasSuggestion { alias: "gb".to_string(), command: "git branch".to_string(), reason: "Git branch".to_string() }),
            _ => None,
        }
    }

    fn generate_docker_alias(&self, subcommand: &str) -> Option<AliasSuggestion> {
        match subcommand {
            "ps" => Some(AliasSuggestion { alias: "dps".to_string(), command: "docker ps".to_string(), reason: "Docker ps".to_string() }),
            "run" => Some(AliasSuggestion { alias: "dr".to_string(), command: "docker run".to_string(), reason: "Docker run".to_string() }),
            "build" => Some(AliasSuggestion { alias: "db".to_string(), command: "docker build".to_string(), reason: "Docker build".to_string() }),
            "exec" => Some(AliasSuggestion { alias: "de".to_string(), command: "docker exec".to_string(), reason: "Docker exec".to_string() }),
            "rm" => Some(AliasSuggestion { alias: "drm".to_string(), command: "docker rm".to_string(), reason: "Docker rm".to_string() }),
            "rmi" => Some(AliasSuggestion { alias: "drmi".to_string(), command: "docker rmi".to_string(), reason: "Docker rmi".to_string() }),
            _ => None,
        }
    }

    fn generate_npm_alias(&self, subcommand: &str) -> Option<AliasSuggestion> {
        match subcommand {
            "install" => Some(AliasSuggestion { alias: "ni".to_string(), command: "npm install".to_string(), reason: "NPM install".to_string() }),
            "run" => Some(AliasSuggestion { alias: "nr".to_string(), command: "npm run".to_string(), reason: "NPM run".to_string() }),
            "start" => Some(AliasSuggestion { alias: "ns".to_string(), command: "npm start".to_string(), reason: "NPM start".to_string() }),
            "test" => Some(AliasSuggestion { alias: "nt".to_string(), command: "npm test".to_string(), reason: "NPM test".to_string() }),
            "publish" => Some(AliasSuggestion { alias: "np".to_string(), command: "npm publish".to_string(), reason: "NPM publish".to_string() }),
            _ => None,
        }
    }

    fn generate_ssh_alias(&self, host: &str) -> Option<AliasSuggestion> {
        if let Some(short_name) = host.split('.').next() {
            if short_name.len() >= 2 {
                return Some(AliasSuggestion {
                    alias: short_name.to_string(),
                    command: format!("ssh {}", host),
                    reason: format!("SSH to {}", host),
                });
            }
        }
        None
    }

    fn generate_abbreviation_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            return suggestions;
        }
        let abbreviation: String = parts.iter().map(|p| p.chars().next().unwrap_or('x')).collect();
        if abbreviation.len() >= 2 && abbreviation.len() <= 4 {
            suggestions.push(AliasSuggestion {
                alias: abbreviation,
                command: command.to_string(),
                reason: "Abbreviation".to_string(),
            });
        }
        suggestions
    }

    fn generate_combined_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            return suggestions;
        }
        let tool = parts[0];
        let args = &parts[1..];
        if !args.is_empty() && args[0].len() >= 2 {
            let combined = format!("{}{}", tool.chars().next().unwrap_or('x'), args[0]);
            suggestions.push(AliasSuggestion { alias: combined, command: command.to_string(), reason: format!("{}-{} combination", tool, args[0]) });
        }
        if args.len() >= 2 && args[1].len() >= 2 {
            let combined = format!("{}{}", tool.chars().next().unwrap_or('x'), args[1]);
            suggestions.push(AliasSuggestion { alias: combined, command: command.to_string(), reason: format!("{}-{} combination", tool, args[1]) });
        }
        suggestions
    }

    fn generate_single_word_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() != 1 {
            return suggestions;
        }
        let tool = parts[0];
        if tool.starts_with("./") || tool.starts_with("../") {
            return suggestions;
        }
        if tool.len() > 3 {
            suggestions.push(AliasSuggestion { alias: tool.chars().take(3).collect(), command: command.to_string(), reason: "3-letter abbreviation".to_string() });
        }
        if tool.len() > 2 {
            suggestions.push(AliasSuggestion { alias: tool.chars().take(2).collect(), command: command.to_string(), reason: "2-letter abbreviation".to_string() });
        }
        if tool.len() > 2 {
            let fl = format!("{}{}", tool.chars().next().unwrap_or('x'), tool.chars().last().unwrap_or('x'));
            suggestions.push(AliasSuggestion { alias: fl, command: command.to_string(), reason: "First-last character".to_string() });
        }
        if tool.contains("git") {
            suggestions.push(AliasSuggestion { alias: "lg".to_string(), command: command.to_string(), reason: "LazyGit abbreviation".to_string() });
        }
        if tool.contains("docker") {
            suggestions.push(AliasSuggestion { alias: "dk".to_string(), command: command.to_string(), reason: "Docker abbreviation".to_string() });
        }
        if tool.contains("node") {
            suggestions.push(AliasSuggestion { alias: "nd".to_string(), command: command.to_string(), reason: "Node abbreviation".to_string() });
        }
        suggestions
    }

    fn generate_vowel_removal_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        let mut processed_parts = Vec::new();
        for word in &parts {
            let consonants: String = word.chars().filter(|c| !"aeiouAEIOU".contains(*c)).collect();
            let limited: String = consonants.chars().take(3).collect();
            if !limited.is_empty() { processed_parts.push(limited); }
        }
        if !processed_parts.is_empty() {
            let combined: String = processed_parts.join("");
            let final_alias: String = if combined.len() > 8 { combined.chars().take(8).collect() } else { combined };
            if final_alias.len() >= 2 && final_alias != command {
                suggestions.push(AliasSuggestion { alias: final_alias, command: command.to_string(), reason: "Vowel Removal".to_string() });
            }
        }
        suggestions
    }

    fn generate_truncated_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        let tool = parts[0];
        for len in 2..=tool.len().min(5) {
            let trunc: String = tool.chars().take(len).collect();
            if trunc != tool {
                suggestions.push(AliasSuggestion { alias: trunc, command: command.to_string(), reason: format!("Truncated to {} chars", len) });
            }
        }
        suggestions
    }

    fn has_conflicts(&self, alias: &str) -> bool {
        if alias.len() < 2 { return true; }
        if self.existing_aliases.contains(alias) { return true; }
        if self.system_commands.contains(alias) { return true; }
        false
    }

    fn get_priority(&self, suggestion: &AliasSuggestion) -> i32 {
        let mut priority = 0;
        match suggestion.reason.as_str() {
            r if r.contains("Git") || r.contains("Docker") || r.contains("NPM") || r.contains("SSH") => priority += 100,
            "Abbreviation" => priority += 90,
            "Vowel Removal" => priority += 80,
            r if r.contains("combination") => priority += 70,
            "Syllable-based" => priority += 65,
            r if r.contains("Remove prefix") || r.contains("Remove suffix") => priority += 60,
            r if r.contains("abbreviation") || r.contains("First-last") || r.contains("LazyGit") || r.contains("Docker") || r.contains("Node") => priority += 55,
            "Phonetic" => priority += 50,
            r if r.contains("Remove duplicates") || r.contains("Smart consonants") => priority += 45,
            "Keyboard pattern" => priority += 40,
            r if r.contains("Truncated") => priority += 35,
            _ => priority += 30,
        }
        priority += 10 - suggestion.alias.len() as i32;
        priority
    }

    fn generate_syllable_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        for word in &parts {
            if word.len() > 3 {
                let syllables = self.extract_syllables(word);
                if syllables.len() >= 2 {
                    let syllable_alias: String = syllables.iter().map(|s| s.chars().next().unwrap_or('x')).collect();
                    if syllable_alias.len() >= 2 && syllable_alias.len() <= 4 {
                        suggestions.push(AliasSuggestion { alias: syllable_alias, command: command.to_string(), reason: "Syllable-based".to_string() });
                    }
                }
            }
        }
        suggestions
    }

    fn generate_phonetic_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        for word in &parts {
            if word.len() > 2 {
                let phonetic = word.replace("ph", "f").replace("ck", "k").replace("qu", "kw").replace('x', "ks").replace("ch", "c").replace("sh", "s").replace("th", "t");
                if phonetic != *word && phonetic.len() >= 2 && phonetic.len() <= 6 {
                    suggestions.push(AliasSuggestion { alias: phonetic, command: command.to_string(), reason: "Phonetic".to_string() });
                }
            }
        }
        suggestions
    }

    fn generate_keyboard_pattern_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        for word in &parts {
            if word.len() > 2 {
                let kp = self.generate_keyboard_pattern(word);
                if kp.len() >= 2 && kp.len() <= 4 {
                    suggestions.push(AliasSuggestion { alias: kp, command: command.to_string(), reason: "Keyboard pattern".to_string() });
                }
            }
        }
        suggestions
    }

    fn generate_smart_prefix_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        for word in &parts {
            if word.len() > 3 {
                for prefix in &["un", "re", "pre", "post", "anti", "pro", "sub", "super", "inter"] {
                    if let Some(without) = word.strip_prefix(prefix) {
                        if without.len() >= 2 {
                            suggestions.push(AliasSuggestion { alias: without.to_string(), command: command.to_string(), reason: format!("Remove prefix '{}'", prefix) });
                        }
                    }
                }
                for suffix in &["ing", "ed", "er", "est", "ly", "tion", "sion", "ment"] {
                    if let Some(without) = word.strip_suffix(suffix) {
                        if without.len() >= 2 {
                            suggestions.push(AliasSuggestion { alias: without.to_string(), command: command.to_string(), reason: format!("Remove suffix '{}'", suffix) });
                        }
                    }
                }
            }
        }
        suggestions
    }

    fn generate_common_pattern_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        for word in &parts {
            if word.len() > 3 {
                let mut prev = '\0';
                let mut deduped = String::new();
                for c in word.chars() {
                    if c != prev { deduped.push(c); prev = c; }
                }
                if deduped.len() >= 2 && deduped != *word {
                    suggestions.push(AliasSuggestion { alias: deduped, command: command.to_string(), reason: "Remove duplicates".to_string() });
                }
                if word.len() > 4 {
                    let consonants: Vec<char> = word.chars().filter(|c| !"aeiouAEIOU".contains(*c)).collect();
                    if consonants.len() >= 3 {
                        let smart: String = consonants.iter().take(3).collect();
                        suggestions.push(AliasSuggestion { alias: smart, command: command.to_string(), reason: "Smart consonants".to_string() });
                    }
                }
            }
        }
        suggestions
    }

    fn extract_syllables(&self, word: &str) -> Vec<String> {
        let mut syllables = Vec::new();
        let mut current = String::new();
        let mut prev_vowel = false;
        for c in word.chars() {
            let is_vowel = "aeiouAEIOU".contains(c);
            if is_vowel {
                current.push(c);
                prev_vowel = true;
            } else {
                if prev_vowel && !current.is_empty() {
                    syllables.push(current.clone());
                    current.clear();
                }
                current.push(c);
                prev_vowel = false;
            }
        }
        if !current.is_empty() { syllables.push(current); }
        syllables
    }

    fn generate_keyboard_pattern(&self, word: &str) -> String {
        word.chars().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, c)| c).collect()
    }
}
