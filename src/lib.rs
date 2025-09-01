use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BentoCommand {
    pub name: String,
    pub category: String,
}

impl BentoCommand {
    pub fn new(name: String, category: String) -> Self {
        Self { name, category }
    }
}

pub fn get_commands() -> Vec<BentoCommand> {
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

pub fn fuzzy_match(query: &str, target: &str) -> usize {
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
