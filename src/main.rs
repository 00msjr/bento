use clap::{Arg, Command as ClapCommand};
use colored::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

struct BentoCommand {
    name: String,
    category: String,
}

fn get_commands() -> Vec<BentoCommand> {
    let mut commands = Vec::new();
    
    // Bin commands
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                            if let Ok(name) = entry.file_name().into_string() {
                                commands.push(BentoCommand {
                                    name,
                                    category: "bin".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Aliases
    if let Ok(output) = Command::new("bash").arg("-c").arg("alias").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(name) = line.split('=').next() {
                let clean_name = name.trim_start_matches("alias ");
                commands.push(BentoCommand {
                    name: clean_name.to_string(),
                    category: "alias".to_string(),
                });
            }
        }
    }
    
    // Functions (zsh/bash)
    if let Ok(output) = Command::new("bash").arg("-c").arg("declare -F").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(name) = line.split_whitespace().nth(2) {
                commands.push(BentoCommand {
                    name: name.to_string(),
                    category: "function".to_string(),
                });
            }
        }
    }
    
    // Package managers
    for (cmd, cat) in [("npm", "npm"), ("pip", "pip"), ("cargo", "cargo")] {
        if Command::new("which").arg(cmd).output().map_or(false, |o| o.status.success()) {
            commands.push(BentoCommand {
                name: cmd.to_string(),
                category: cat.to_string(),
            });
        }
    }
    
    commands
}

fn fuzzy_match(query: &str, target: &str) -> usize {
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    
    if target.contains(&query) {
        return query.len() * 2;
    }
    
    let mut score = 0;
    let mut target_chars = target.chars().peekable();
    
    for q_char in query.chars() {
        while let Some(&t_char) = target_chars.peek() {
            target_chars.next();
            if q_char == t_char {
                score += 1;
                break;
            }
        }
    }
    score
}

fn main() {
    let matches = ClapCommand::new("bento")
        .about("Command organizer")
        .arg(Arg::new("query").help("Command to search for"))
        .arg(Arg::new("aliases").short('a').long("aliases").help("Search aliases only").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("functions").short('f').long("functions").help("Search functions only").action(clap::ArgAction::SetTrue))
        .arg(Arg::new("bin").short('b').long("bin").help("Search bin commands only").action(clap::ArgAction::SetTrue))
        .get_matches();
    
    let commands = get_commands();
    
    if let Some(query) = matches.get_one::<String>("query") {
        let mut scored: Vec<_> = commands
            .iter()
            .filter(|cmd| {
                if matches.get_flag("aliases") { return cmd.category == "alias"; }
                if matches.get_flag("functions") { return cmd.category == "function"; }
                if matches.get_flag("bin") { return cmd.category == "bin"; }
                true
            })
            .map(|cmd| (fuzzy_match(query, &cmd.name), cmd))
            .filter(|(score, _)| *score > 0)
            .collect();
        
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        
        for (_, cmd) in scored.into_iter().take(10) {
            let color = match cmd.category.as_str() {
                "alias" => cmd.name.yellow(),
                "function" => cmd.name.green(),
                "bin" => cmd.name.blue(),
                "npm" | "pip" | "cargo" => cmd.name.magenta(),
                _ => cmd.name.white(),
            };
            println!("{} ({})", color, cmd.category.dimmed());
        }
    } else {
        let mut by_category: HashMap<String, Vec<&BentoCommand>> = HashMap::new();
        for cmd in &commands {
            by_category.entry(cmd.category.clone()).or_default().push(cmd);
        }
        
        for (category, mut cmds) in by_category {
            cmds.sort_by(|a, b| a.name.cmp(&b.name));
            println!("\n{}", category.to_uppercase().bold());
            for cmd in cmds.into_iter().take(20) {
                println!("  {}", cmd.name);
            }
        }
    }
}
