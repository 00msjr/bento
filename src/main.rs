use bento::{fuzzy_match, get_commands};
use clap::{Arg, Command as ClapCommand};
use colored::*;

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
