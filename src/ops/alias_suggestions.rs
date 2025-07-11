use std::collections::HashSet;
use std::env;

#[derive(Debug, Clone)]
pub struct AliasSuggestion {
    pub alias: String,
    pub command: String,
    pub reason: String,
}

pub struct AliasSuggester {
    existing_aliases: HashSet<String>,
    system_commands: HashSet<String>,
}

impl AliasSuggester {
    pub fn new(alias_file_path: &str) -> Self {
        let existing_aliases = Self::load_existing_aliases(alias_file_path);
        let system_commands = Self::load_system_commands();
        
        Self {
            existing_aliases,
            system_commands,
        }
    }

    fn load_existing_aliases(alias_file_path: &str) -> HashSet<String> {
        use crate::ops::alias_ops::get_aliases;
        
        let aliases = get_aliases(alias_file_path);
        aliases.into_iter().map(|(alias, _)| alias).collect()
    }

    fn load_system_commands() -> HashSet<String> {
        let mut commands = HashSet::new();
        
        // Get PATH from environment
        if let Ok(path) = env::var("PATH") {
            for path_dir in path.split(':') {
                if let Ok(entries) = std::fs::read_dir(path_dir) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() {
                                // Check if file is executable (check execute permissions)
                                use std::os::unix::fs::PermissionsExt;
                                if metadata.permissions().mode() & 0o111 != 0 {
                                    if let Some(name) = entry.file_name().to_str() {
                                        commands.insert(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Get shell aliases using the user's current shell from $SHELL
        if let Ok(shell_path) = std::env::var("SHELL") {
            if let Ok(output) = std::process::Command::new(shell_path)
                .arg("-i")
                .arg("-c")
                .arg("alias")
                .output() {
                if let Ok(alias_output) = String::from_utf8(output.stdout) {
                    for line in alias_output.lines() {
                        if let Some(alias_name) = line.split('=').next() {
                            // Remove any quotes and whitespace
                            let clean_alias = alias_name.trim_matches('"').trim_matches('\'').trim();
                            if !clean_alias.is_empty() {
                                commands.insert(clean_alias.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        commands
    }

    pub fn suggest_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        
        // Generate different types of suggestions
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
        
        // Filter out conflicts and sort by priority
        suggestions.retain(|s| !self.has_conflicts(&s.alias));
        
        // Remove duplicates based on alias name
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.alias.clone()));
        
        suggestions.sort_by(|a, b| self.get_priority(b).cmp(&self.get_priority(a)));
        
        // Show all suggestions (no top 3 limit)
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

        // Handle relative path commands (e.g., ./target/debug/alman)
        if tool.starts_with("./") || tool.starts_with("../") {
            suggestions.extend(self.generate_relative_path_aliases(command, tool, args));
            return suggestions;
        }

        // Generate tool-specific semantic aliases
        if let Some(semantic_alias) = self.generate_tool_specific_alias(tool, args) {
            suggestions.push(semantic_alias);
        }

        // Generate general semantic aliases
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
        
        // Extract the executable name from the path
        let executable_name = tool.split('/').last().unwrap_or(tool);
        
        // Generate alias based on executable name
        if let Some(name) = executable_name.strip_suffix(".exe") {
            // For Windows executables
            suggestions.push(AliasSuggestion {
                alias: name.to_string(),
                command: command.to_string(),
                reason: "Executable name".to_string(),
            });
        } else {
            // For Unix executables
            suggestions.push(AliasSuggestion {
                alias: executable_name.to_string(),
                command: command.to_string(),
                reason: "Executable name".to_string(),
            });
        }

        // Generate abbreviation based on executable name
        if executable_name.len() > 2 {
            let abbrev = executable_name.chars().take(3).collect::<String>();
            suggestions.push(AliasSuggestion {
                alias: abbrev,
                command: command.to_string(),
                reason: "Executable abbreviation".to_string(),
            });
        }

        // If there are arguments, create a more specific alias
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
            "status" => Some(AliasSuggestion {
                alias: "gs".to_string(),
                command: "git status".to_string(),
                reason: "Git status".to_string(),
            }),
            "add" => {
                if !remaining_args.is_empty() && remaining_args[0] == "." {
                    Some(AliasSuggestion {
                        alias: "gaa".to_string(),
                        command: "git add .".to_string(),
                        reason: "Git add all".to_string(),
                    })
                } else {
                    Some(AliasSuggestion {
                        alias: "ga".to_string(),
                        command: "git add".to_string(),
                        reason: "Git add".to_string(),
                    })
                }
            }
            "commit" => {
                if remaining_args.len() >= 2 && remaining_args[0] == "-m" {
                    Some(AliasSuggestion {
                        alias: "gcm".to_string(),
                        command: format!("git commit -m \"{}\"", remaining_args[1]),
                        reason: "Git commit with message".to_string(),
                    })
                } else {
                    Some(AliasSuggestion {
                        alias: "gc".to_string(),
                        command: "git commit".to_string(),
                        reason: "Git commit".to_string(),
                    })
                }
            }
            "checkout" => {
                if !remaining_args.is_empty() && remaining_args[0] == "-b" {
                    Some(AliasSuggestion {
                        alias: "gcb".to_string(),
                        command: format!("git checkout -b {}", remaining_args[1]),
                        reason: "Git checkout new branch".to_string(),
                    })
                } else {
                    Some(AliasSuggestion {
                        alias: "gco".to_string(),
                        command: "git checkout".to_string(),
                        reason: "Git checkout".to_string(),
                    })
                }
            }
            "push" => Some(AliasSuggestion {
                alias: "gp".to_string(),
                command: "git push".to_string(),
                reason: "Git push".to_string(),
            }),
            "pull" => Some(AliasSuggestion {
                alias: "gl".to_string(),
                command: "git pull".to_string(),
                reason: "Git pull".to_string(),
            }),
            "log" => Some(AliasSuggestion {
                alias: "glg".to_string(),
                command: "git log".to_string(),
                reason: "Git log".to_string(),
            }),
            "branch" => Some(AliasSuggestion {
                alias: "gb".to_string(),
                command: "git branch".to_string(),
                reason: "Git branch".to_string(),
            }),
            _ => None,
        }
    }

    fn generate_docker_alias(&self, subcommand: &str) -> Option<AliasSuggestion> {
        match subcommand {
            "ps" => Some(AliasSuggestion {
                alias: "dps".to_string(),
                command: "docker ps".to_string(),
                reason: "Docker ps".to_string(),
            }),
            "run" => Some(AliasSuggestion {
                alias: "dr".to_string(),
                command: "docker run".to_string(),
                reason: "Docker run".to_string(),
            }),
            "build" => Some(AliasSuggestion {
                alias: "db".to_string(),
                command: "docker build".to_string(),
                reason: "Docker build".to_string(),
            }),
            "exec" => Some(AliasSuggestion {
                alias: "de".to_string(),
                command: "docker exec".to_string(),
                reason: "Docker exec".to_string(),
            }),
            "rm" => Some(AliasSuggestion {
                alias: "drm".to_string(),
                command: "docker rm".to_string(),
                reason: "Docker rm".to_string(),
            }),
            "rmi" => Some(AliasSuggestion {
                alias: "drmi".to_string(),
                command: "docker rmi".to_string(),
                reason: "Docker rmi".to_string(),
            }),
            _ => None,
        }
    }

    fn generate_npm_alias(&self, subcommand: &str) -> Option<AliasSuggestion> {
        match subcommand {
            "install" => Some(AliasSuggestion {
                alias: "ni".to_string(),
                command: "npm install".to_string(),
                reason: "NPM install".to_string(),
            }),
            "run" => Some(AliasSuggestion {
                alias: "nr".to_string(),
                command: "npm run".to_string(),
                reason: "NPM run".to_string(),
            }),
            "start" => Some(AliasSuggestion {
                alias: "ns".to_string(),
                command: "npm start".to_string(),
                reason: "NPM start".to_string(),
            }),
            "test" => Some(AliasSuggestion {
                alias: "nt".to_string(),
                command: "npm test".to_string(),
                reason: "NPM test".to_string(),
            }),
            "publish" => Some(AliasSuggestion {
                alias: "np".to_string(),
                command: "npm publish".to_string(),
                reason: "NPM publish".to_string(),
            }),
            _ => None,
        }
    }

    fn generate_ssh_alias(&self, host: &str) -> Option<AliasSuggestion> {
        // Generate alias based on hostname
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

        // Take first letter of each word
        let abbreviation: String = parts.iter()
            .map(|part| part.chars().next().unwrap_or('x'))
            .collect();

        if abbreviation.len() >= 2 && abbreviation.len() <= 4 {
            suggestions.push(AliasSuggestion {
                alias: abbreviation.clone(),
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

        // Generate tool + first arg combinations
        if !args.is_empty() {
            let first_arg = args[0];
            if first_arg.len() >= 2 {
                let combined = format!("{}{}", tool.chars().next().unwrap_or('x'), first_arg);
                suggestions.push(AliasSuggestion {
                    alias: combined,
                    command: command.to_string(),
                    reason: format!("{}-{} combination", tool, first_arg),
                });
            }
        }

        // Generate tool + second arg combinations
        if args.len() >= 2 {
            let second_arg = args[1];
            if second_arg.len() >= 2 {
                let combined = format!("{}{}", tool.chars().next().unwrap_or('x'), second_arg);
                suggestions.push(AliasSuggestion {
                    alias: combined,
                    command: command.to_string(),
                    reason: format!("{}-{} combination", tool, second_arg),
                });
            }
        }

        suggestions
    }

    fn generate_single_word_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        // Only handle single-word commands
        if parts.len() != 1 {
            return suggestions;
        }
        
        let tool = parts[0];
        
        // Skip if it's a relative path command
        if tool.starts_with("./") || tool.starts_with("../") {
            return suggestions;
        }
        
        // Generate various aliases for single-word commands
        if tool.len() > 3 {
            // Take first 3 characters
            let abbrev = tool.chars().take(3).collect::<String>();
            suggestions.push(AliasSuggestion {
                alias: abbrev,
                command: command.to_string(),
                reason: "3-letter abbreviation".to_string(),
            });
        }
        
        // Take first 2 characters if tool is longer than 2
        if tool.len() > 2 {
            let abbrev = tool.chars().take(2).collect::<String>();
            suggestions.push(AliasSuggestion {
                alias: abbrev,
                command: command.to_string(),
                reason: "2-letter abbreviation".to_string(),
            });
        }
        
        // Generate alias with first and last character
        if tool.len() > 2 {
            let first = tool.chars().next().unwrap_or('x');
            let last = tool.chars().last().unwrap_or('x');
            let fl_alias = format!("{}{}", first, last);
            suggestions.push(AliasSuggestion {
                alias: fl_alias,
                command: command.to_string(),
                reason: "First-last character".to_string(),
            });
        }
        

        
        // For compound words like "lazygit", try to extract meaningful parts
        if tool.contains("git") {
            suggestions.push(AliasSuggestion {
                alias: "lg".to_string(),
                command: command.to_string(),
                reason: "LazyGit abbreviation".to_string(),
            });
        }
        
        if tool.contains("docker") {
            suggestions.push(AliasSuggestion {
                alias: "dk".to_string(),
                command: command.to_string(),
                reason: "Docker abbreviation".to_string(),
            });
        }
        
        if tool.contains("node") {
            suggestions.push(AliasSuggestion {
                alias: "nd".to_string(),
                command: command.to_string(),
                reason: "Node abbreviation".to_string(),
            });
        }
        
        suggestions
    }

    fn generate_vowel_removal_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        
        // Process each word: remove vowels and limit to 3 consonants per word
        let mut processed_parts = Vec::new();
        for word in &parts {
            let consonants: String = word.chars()
                .filter(|c| !"aeiouAEIOU".contains(*c))
                .collect();
            
            // Limit to 3 consonants per word
            let limited = consonants.chars().take(3).collect::<String>();
            if !limited.is_empty() {
                processed_parts.push(limited);
            }
        }
        
        // Combine all processed parts
        if !processed_parts.is_empty() {
            let combined: String = processed_parts.join("");
            
            // Truncate if too long (max 8 characters total)
            let final_alias = if combined.len() > 8 {
                combined.chars().take(8).collect::<String>()
            } else {
                combined
            };
            
            // Only add if it's different from the original command and has at least 2 characters
            if final_alias.len() >= 2 && final_alias != command {
                suggestions.push(AliasSuggestion {
                    alias: final_alias,
                    command: command.to_string(),
                    reason: "Vowel Removal".to_string(),
                });
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
            let trunc = tool.chars().take(len).collect::<String>();
            if trunc != tool {
                suggestions.push(AliasSuggestion {
                    alias: trunc,
                    command: command.to_string(),
                    reason: format!("Truncated to {} chars", len),
                });
            }
        }
        suggestions
    }



    fn has_conflicts(&self, alias: &str) -> bool {
        // Check if alias already exists in our alias files
        if self.existing_aliases.contains(alias) {
            return true;
        }

        // Check if alias conflicts with system command
        if self.system_commands.contains(alias) {
            return true;
        }

        // Check if alias is too short (likely to conflict)
        if alias.len() < 2 {
            return true;
        }

        false
    }

    fn get_priority(&self, suggestion: &AliasSuggestion) -> i32 {
        let mut priority = 0;
        
        // Priority based on suggestion type (higher number = higher priority)
        match suggestion.reason.as_str() {
            // Semantic aliases (tool-specific) - highest priority
            reason if reason.contains("Git") || reason.contains("Docker") || reason.contains("NPM") || reason.contains("SSH") => {
                priority += 100;
            }
            // Abbreviation aliases - second priority
            "Abbreviation" => {
                priority += 90;
            }
            // Vowel removal - third priority
            "Vowel Removal" => {
                priority += 80;
            }
            // Combined aliases - fourth priority
            reason if reason.contains("combination") => {
                priority += 70;
            }
            // Syllable-based - fifth priority
            "Syllable-based" => {
                priority += 65;
            }
            // Smart prefixes/suffixes - sixth priority
            reason if reason.contains("Remove prefix") || reason.contains("Remove suffix") => {
                priority += 60;
            }
            // Single word aliases - seventh priority
            reason if reason.contains("abbreviation") || reason.contains("First-last") || reason.contains("LazyGit") || reason.contains("Docker") || reason.contains("Node") => {
                priority += 55;
            }
            // Phonetic aliases - eighth priority
            "Phonetic" => {
                priority += 50;
            }
            // Common patterns - ninth priority
            reason if reason.contains("Remove duplicates") || reason.contains("Smart consonants") => {
                priority += 45;
            }
            // Keyboard patterns - tenth priority
            "Keyboard pattern" => {
                priority += 40;
            }
            // Truncated aliases - lowest priority
            reason if reason.contains("Truncated") => {
                priority += 35;
            }
            _ => {
                priority += 30; // Default priority for unknown types
            }
        }
        
        // Higher priority for shorter aliases within the same type
        priority += 10 - suggestion.alias.len() as i32;
        
        priority
    }

    fn generate_syllable_aliases(&self, command: &str) -> Vec<AliasSuggestion> {
        let mut suggestions = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return suggestions; }
        
        for word in &parts {
            if word.len() > 3 {
                // Extract syllables based on vowel-consonant patterns
                let syllables = self.extract_syllables(word);
                if syllables.len() >= 2 {
                    // Take first letter of each syllable
                    let syllable_alias: String = syllables.iter()
                        .map(|s| s.chars().next().unwrap_or('x'))
                        .collect();
                    
                    if syllable_alias.len() >= 2 && syllable_alias.len() <= 4 {
                        suggestions.push(AliasSuggestion {
                            alias: syllable_alias,
                            command: command.to_string(),
                            reason: "Syllable-based".to_string(),
                        });
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
                // Common phonetic substitutions
                let phonetic = word
                    .replace("ph", "f")
                    .replace("ck", "k")
                    .replace("qu", "kw")
                    .replace("x", "ks")
                    .replace("ch", "c")
                    .replace("sh", "s")
                    .replace("th", "t");
                
                if phonetic != *word && phonetic.len() >= 2 && phonetic.len() <= 6 {
                    suggestions.push(AliasSuggestion {
                        alias: phonetic,
                        command: command.to_string(),
                        reason: "Phonetic".to_string(),
                    });
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
                // Common keyboard patterns (adjacent keys)
                let keyboard_pattern = self.generate_keyboard_pattern(word);
                if keyboard_pattern.len() >= 2 && keyboard_pattern.len() <= 4 {
                    suggestions.push(AliasSuggestion {
                        alias: keyboard_pattern,
                        command: command.to_string(),
                        reason: "Keyboard pattern".to_string(),
                    });
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
                // Common prefixes
                let prefixes = ["un", "re", "pre", "post", "anti", "pro", "sub", "super", "inter"];
                for prefix in &prefixes {
                    if word.starts_with(prefix) {
                        let without_prefix = &word[prefix.len()..];
                        if without_prefix.len() >= 2 {
                            suggestions.push(AliasSuggestion {
                                alias: without_prefix.to_string(),
                                command: command.to_string(),
                                reason: format!("Remove prefix '{}'", prefix),
                            });
                        }
                    }
                }
                
                // Common suffixes
                let suffixes = ["ing", "ed", "er", "est", "ly", "tion", "sion", "ment"];
                for suffix in &suffixes {
                    if word.ends_with(suffix) {
                        let without_suffix = &word[..word.len() - suffix.len()];
                        if without_suffix.len() >= 2 {
                            suggestions.push(AliasSuggestion {
                                alias: without_suffix.to_string(),
                                command: command.to_string(),
                                reason: format!("Remove suffix '{}'", suffix),
                            });
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
                // Double letter patterns (keep one)
                let mut prev_char = '\0';
                let mut deduplicated = String::new();
                for c in word.chars() {
                    if c != prev_char {
                        deduplicated.push(c);
                        prev_char = c;
                    }
                }
                
                if deduplicated.len() >= 2 && deduplicated != *word {
                    suggestions.push(AliasSuggestion {
                        alias: deduplicated,
                        command: command.to_string(),
                        reason: "Remove duplicates".to_string(),
                    });
                }
                
                // Keep only consonants in specific positions
                if word.len() > 4 {
                    let consonants: Vec<char> = word.chars()
                        .filter(|c| !"aeiouAEIOU".contains(*c))
                        .collect();
                    
                    if consonants.len() >= 3 {
                        let smart_consonants: String = consonants.iter()
                            .take(3)
                            .collect();
                        
                        suggestions.push(AliasSuggestion {
                            alias: smart_consonants,
                            command: command.to_string(),
                            reason: "Smart consonants".to_string(),
                        });
                    }
                }
            }
        }
        
        suggestions
    }

    fn extract_syllables(&self, word: &str) -> Vec<String> {
        let mut syllables = Vec::new();
        let mut current_syllable = String::new();
        let mut prev_vowel = false;
        
        for c in word.chars() {
            let is_vowel = "aeiouAEIOU".contains(c);
            
            if is_vowel {
                current_syllable.push(c);
                prev_vowel = true;
            } else {
                if prev_vowel && !current_syllable.is_empty() {
                    // End of syllable
                    syllables.push(current_syllable.clone());
                    current_syllable.clear();
                }
                current_syllable.push(c);
                prev_vowel = false;
            }
        }
        
        if !current_syllable.is_empty() {
            syllables.push(current_syllable);
        }
        
        syllables
    }

    fn generate_keyboard_pattern(&self, word: &str) -> String {
        // Simple keyboard pattern: take every other character
        word.chars()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, c)| c)
            .collect()
    }
} 
