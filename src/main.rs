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

    // Bin commands from PATH
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

    // Homebrew packages
    if let Ok(output) = Command::new("brew").arg("list").arg("--formula").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let name = line.trim();
            if !name.is_empty() {
                commands.push(BentoCommand {
                    name: name.to_string(),
                    category: "homebrew".to_string(),
                });
            }
        }
    }

    // Homebrew casks
    if let Ok(output) = Command::new("brew").arg("list").arg("--cask").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let name = line.trim();
            if !name.is_empty() {
                commands.push(BentoCommand {
                    name: name.to_string(),
                    category: "cask".to_string(),
                });
            }
        }
    }

    // Python packages (pip)
    if let Ok(output) = Command::new("pip")
        .arg("list")
        .arg("--format=freeze")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(name) = line.split("==").next() {
                commands.push(BentoCommand {
                    name: name.to_string(),
                    category: "pip".to_string(),
                });
            }
        }
    }

    // Node packages (npm global)
    if let Ok(output) = Command::new("npm")
        .arg("list")
        .arg("-g")
        .arg("--depth=0")
        .arg("--parseable")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(name) = line.split('/').last() {
                if name != "lib" && !name.is_empty() {
                    commands.push(BentoCommand {
                        name: name.to_string(),
                        category: "npm".to_string(),
                    });
                }
            }
        }
    }

    // Yarn global packages
    if let Ok(output) = Command::new("yarn").arg("global").arg("list").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("info ") && line.contains("@") {
                if let Some(name) = line.split("@").next() {
                    let clean_name = name.trim_start_matches("info ");
                    commands.push(BentoCommand {
                        name: clean_name.to_string(),
                        category: "yarn".to_string(),
                    });
                }
            }
        }
    }

    // Rust packages (cargo)
    if let Ok(output) = Command::new("cargo").arg("install").arg("--list").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.starts_with(" ") && line.contains(" v") {
                if let Some(name) = line.split(" v").next() {
                    commands.push(BentoCommand {
                        name: name.to_string(),
                        category: "cargo".to_string(),
                    });
                }
            }
        }
    }

    // Go packages
    if let Ok(output) = Command::new("go").arg("list").arg("-m").arg("all").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(name) = line.split_whitespace().next() {
                if name.contains("/") {
                    if let Some(pkg_name) = name.split("/").last() {
                        commands.push(BentoCommand {
                            name: pkg_name.to_string(),
                            category: "go".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Aliases - try multiple shell methods
    for cmd in ["zsh -c 'alias'", "bash -c 'alias'", "sh -c 'alias'"] {
        if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains('=') && !line.trim().is_empty() {
                    if let Some(name) = line.split('=').next() {
                        let clean_name = name.trim_start_matches("alias ").trim();
                        if !clean_name.is_empty() && !clean_name.starts_with('-') {
                            commands.push(BentoCommand {
                                name: clean_name.to_string(),
                                category: "alias".to_string(),
                            });
                        }
                    }
                }
            }
            if !stdout.is_empty() {
                break;
            }
        }
    }

    // Functions - try multiple methods
    for cmd in [
        "zsh -c 'print -l ${(k)functions}'",
        "bash -c 'declare -F | cut -d\" \" -f3'",
        "zsh -c 'functions | grep \"^[a-zA-Z]\" | cut -d\" \" -f1'",
    ] {
        if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim();
                if !name.is_empty() && !name.contains(' ') && !name.starts_with('_') {
                    commands.push(BentoCommand {
                        name: name.to_string(),
                        category: "function".to_string(),
                    });
                }
            }
            if !stdout.is_empty() {
                break;
            }
        }
    }

    // Direct shell execution for current shell
    if let Ok(shell) = env::var("SHELL") {
        // Get aliases from current shell
        if let Ok(output) = Command::new(&shell)
            .arg("-i")
            .arg("-c")
            .arg("alias")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains('=') && !line.trim().is_empty() {
                    if let Some(name) = line.split('=').next() {
                        let clean_name = name
                            .trim_start_matches("alias ")
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"');
                        if !clean_name.is_empty()
                            && clean_name
                                .chars()
                                .all(|c| c.is_alphanumeric() || "_-~.".contains(c))
                        {
                            commands.push(BentoCommand {
                                name: clean_name.to_string(),
                                category: "alias".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Get functions from current shell
        if shell.contains("zsh") {
            if let Ok(output) = Command::new(&shell)
                .arg("-i")
                .arg("-c")
                .arg("print -l ${(k)functions}")
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let name = line.trim();
                    if !name.is_empty()
                        && !name.starts_with('_')
                        && name
                            .chars()
                            .all(|c| c.is_alphanumeric() || "_-".contains(c))
                    {
                        commands.push(BentoCommand {
                            name: name.to_string(),
                            category: "function".to_string(),
                        });
                    }
                }
            }
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
        .arg(
            Arg::new("alias")
                .long("alias")
                .help("Search aliases only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("function")
                .long("function")
                .help("Search functions only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bin")
                .long("bin")
                .help("Search bin commands only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("homebrew")
                .long("homebrew")
                .help("Search homebrew commands only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pip")
                .long("pip")
                .help("Search pip packages only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("npm")
                .long("npm")
                .help("Search npm packages only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("cargo")
                .long("cargo")
                .help("Search cargo packages only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("cask")
                .long("cask")
                .help("Search homebrew casks only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("yarn")
                .long("yarn")
                .help("Search yarn packages only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("go")
                .long("go")
                .help("Search go packages only")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let commands = get_commands();

    if let Some(query) = matches.get_one::<String>("query") {
        let mut scored: Vec<_> = commands
            .iter()
            .filter(|cmd| {
                if matches.get_flag("alias") {
                    return cmd.category == "alias";
                }
                if matches.get_flag("function") {
                    return cmd.category == "function";
                }
                if matches.get_flag("bin") {
                    return cmd.category == "bin";
                }
                if matches.get_flag("homebrew") {
                    return cmd.category == "homebrew";
                }
                if matches.get_flag("cask") {
                    return cmd.category == "cask";
                }
                if matches.get_flag("pip") {
                    return cmd.category == "pip";
                }
                if matches.get_flag("npm") {
                    return cmd.category == "npm";
                }
                if matches.get_flag("yarn") {
                    return cmd.category == "yarn";
                }
                if matches.get_flag("cargo") {
                    return cmd.category == "cargo";
                }
                if matches.get_flag("go") {
                    return cmd.category == "go";
                }
                true
            })
            .map(|cmd| (fuzzy_match(query, &cmd.name), cmd))
            .filter(|(score, _)| *score > 0)
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        for (_, cmd) in scored.into_iter().take(15) {
            let color = match cmd.category.as_str() {
                "alias" => cmd.name.yellow(),
                "function" => cmd.name.green(),
                "bin" => cmd.name.blue(),
                "homebrew" => cmd.name.cyan(),
                "cask" => cmd.name.bright_cyan(),
                "pip" => cmd.name.red(),
                "npm" => cmd.name.magenta(),
                "yarn" => cmd.name.bright_magenta(),
                "cargo" => cmd.name.bright_red(),
                "go" => cmd.name.bright_blue(),
                _ => cmd.name.white(),
            };
            println!("{} ({})", color, cmd.category.dimmed());
        }
    } else {
        println!("{}", "🍱 Bento - Command Organizer".bold().cyan());
        println!(
            "\nSearch and organize all your commands, packages, aliases, and functions in one place.\n"
        );

        println!("{}", "Usage:".bold());
        println!("  bento <query>           Search all commands");
        println!("  bento --homebrew git    Search homebrew packages");
        println!("  bento --alias ls        Search aliases only");
        println!("  bento --pip django      Search pip packages");

        println!("\n{}", "Filters:".bold());
        println!("  --bin        Binary commands");
        println!("  --homebrew   Homebrew formulae");
        println!("  --cask       Homebrew casks");
        println!("  --pip        Python packages");
        println!("  --npm        NPM packages");
        println!("  --yarn       Yarn packages");
        println!("  --cargo      Rust packages");
        println!("  --go         Go packages");
        println!("  --alias      Shell aliases");
        println!("  --function   Shell functions");

        println!("\n{}", "Examples:".bold());
        println!("  bento git               # Find all git-related commands");
        println!("  bento --homebrew bat    # Search homebrew for 'bat'");
        println!("  bento --alias my        # Find aliases containing 'my'");

        let total: usize = commands.len();
        println!(
            "\n{} {} commands available across all sources",
            "📦".cyan(),
            total.to_string().bold()
        );
    }
}
