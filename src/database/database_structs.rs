use std::collections:: {BTreeSet, HashMap};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub command_list: BTreeSet<Command>,
    pub reverse_command_map: HashMap<String, Command>,
    pub total_num_commands: i32,
    pub total_score: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletedCommands {
    pub deleted_commands: BTreeSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub score: i32,
    pub last_access_time: i64,
    pub frequency: i32,
    pub length: i16,
    pub command_text: String,
    pub number_of_words: i8,
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by score descending
        match other.score.cmp(&self.score) {
            std::cmp::Ordering::Equal => self.command_text.cmp(&other.command_text), // tie-breaker
            ord => ord,
        }
    }
}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
