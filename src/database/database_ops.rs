
// use core::time;
// use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};
use super::database_structs::{Command, Database, DeletedCommands};

fn get_score(command: &Command) -> i32 {
    let now = SystemTime::now();
    let current_time: i64 = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let time_difference: i64 = current_time - command.last_access_time;
    let mult: f64;
    if time_difference <= 3600 {
        mult=4.0;
    }
    else if time_difference <= 86400 {
        mult=2.0;
    }
    else if time_difference <= 604800 {
        mult=0.5;
    }
    else {
        mult=0.25;
    }
    let length = command.length as f64;
    let frequency = command.frequency as f64;

    (mult * length.powf(3.0 / 5.0) * frequency) as i32
    
    
}

impl Database {
    pub fn add_command(&mut self, command_str: String, deleted_commands: &DeletedCommands) {
        if !deleted_commands.deleted_commands.contains(&command_str) {
            if let Some(existing_command) = self.reverse_command_map.get(&command_str) {
                let mut updated_command = existing_command.clone();
                self.total_score -= updated_command.score as i64;
                self.total_num_commands -= 1;
                updated_command.add();

                self.command_list.remove(existing_command);
                self.reverse_command_map.remove(&command_str);

                self.command_list.insert(updated_command.clone());
                self.reverse_command_map.insert(command_str.clone(), updated_command);

                self.total_num_commands += 1;
                self.total_score += self.reverse_command_map.get(&command_str).unwrap().score as i64;

            } else {
                let new_command: Command = Command::new(command_str.clone());
                if new_command.length <= 5 && new_command.number_of_words == 1 {
                    return; // Ignore commands that are too short and single-word
                }
                self.command_list.insert(new_command.clone());
                self.reverse_command_map.insert(command_str.clone(), new_command);
                self.total_num_commands += 1;
                self.total_score += self.reverse_command_map.get(&command_str).unwrap().score as i64;
            }
            let threshold: i64 = 10000; // Much higher threshold
            if self.total_score > threshold {
                self.score_reset();
            }
            
        }
    }

    pub fn remove_command(&mut self, command_str: &String, deleted_commands: &mut DeletedCommands) {
        // first check if already exists in DeletedCommands
        if !deleted_commands.deleted_commands.contains(command_str) {
            deleted_commands
                .deleted_commands
                .insert(command_str.clone());
            // then remove from command list, first get command from reverse map
            let command_to_remove = self.reverse_command_map.get(command_str);
            // if not null, remove it fromm both
            if let Some(cmd) = command_to_remove {
                self.command_list.remove(cmd);
                self.total_num_commands -= 1;
                self.total_score -= cmd.score as i64;
                self.reverse_command_map.remove(command_str);
            }
        }
    }

    pub fn update_db(&mut self) {
    // Update each command in place
    let keys: Vec<String> = self.reverse_command_map.keys().cloned().collect();
    for key in keys {
        if let Some(command) = self.reverse_command_map.get_mut(&key) {
            self.total_score -= command.score as i64;
            command.update();
            self.total_score += command.score as i64;

            // Remove and re-insert in command_list to update ordering if needed
            self.command_list.remove(command);
            self.command_list.insert(command.clone());
        }
    }
    let threshold: i64 = 10000; // Much higher threshold
    if self.total_score > threshold {
        self.score_reset();
    }
}

    pub fn get_top_commands(&mut self, n: Option<usize>) -> Vec<&Command> {
        let n = n.unwrap_or(5);
        self.command_list.iter().take(n).collect()
    }



    pub fn score_reset(&mut self){
        //iterate through the set & map and reduce the freq of each string by 50% and delete the strings with 0 freq
        let mut to_remove = Vec::new();
        let mut num:i32 =0;
        let mut sc:i64=0;
        for (key, value) in self.reverse_command_map.iter_mut() {
            value.frequency = (value.frequency as f32 * 0.5).round() as i32; // Reduce by 50% instead of 90%
            self.total_score -= value.score as i64;
            value.score = get_score(&value);
            sc+= value.score as i64;
            if value.frequency < 1 {
                self.total_num_commands-=1;
                
                to_remove.push(key.clone());
            }
            else {num+=1;}
        }
        self.total_score = sc;
        self.total_num_commands = num;
        for key in &to_remove {
            self.reverse_command_map.remove(key);
        }
        let old_set = std::mem::take(&mut self.command_list);

        for mut cmd in old_set {
            cmd.frequency = (cmd.frequency as f32 * 0.5).round() as i32; // Reduce by 50% instead of 90%
            cmd.score=get_score(&cmd); 
            if cmd.frequency>0 {self.command_list.insert(cmd);}
        }
        
        
    }

}

impl Command {
    pub fn new(command_text: String) -> Self {
        let length: i16 = command_text.split_whitespace().map(|s| s.len()).sum::<usize>() as i16;
        //map transforms the strings into their lengths
        let number_of_words: i8 = command_text.split_whitespace().count() as i8;
        let frequency: i32 = 1;
        let now = SystemTime::now();
        let last_access_time: i64 = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64; // or set as needed

        // Create a temporary Command to calculate the score
        let temp_command = Command {
            last_access_time,
            frequency,
            length,
            score: 0, // placeholder
            command_text: command_text.clone(),
            number_of_words,
        };

        let score = get_score(&temp_command);

        Command {
            last_access_time,
            frequency,
            length,
            score,
            command_text,
            number_of_words,
        }
    }
    pub fn add(&mut self) {
        // here we update the last_access time, frequency, and score
        let now = SystemTime::now();
        self.last_access_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64; // placeholder
        self.frequency += 1;
        self.score = get_score(self);
    }
    pub fn update (&mut self) {
        // just update with time and score
        self.score = get_score(self);
    }
}
