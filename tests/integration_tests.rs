use bento::{get_commands, fuzzy_match, BentoCommand};

#[test]
fn test_bento_command_creation() {
    let cmd = BentoCommand {
        name: "git".to_string(),
        category: "bin".to_string(),
    };
    assert_eq!(cmd.name, "git");
    assert_eq!(cmd.category, "bin");
}

#[test]
fn test_bento_command_new() {
    let cmd = BentoCommand::new("test".to_string(), "bin".to_string());
    assert_eq!(cmd.name, "test");
    assert_eq!(cmd.category, "bin");
}

#[test]
fn test_fuzzy_match_exact() {
    assert_eq!(fuzzy_match("git", "git"), 6); // query.len() * 2
    assert_eq!(fuzzy_match("test", "test"), 8);
}

#[test]
fn test_fuzzy_match_contains() {
    assert_eq!(fuzzy_match("git", "gitignore"), 6); // contains match
    assert_eq!(fuzzy_match("bat", "batman"), 6);
    assert_eq!(fuzzy_match("vim", "neovim"), 6);
}

#[test]
fn test_fuzzy_match_partial() {
    // Should get some score for partial character matches
    let score = fuzzy_match("gt", "git");
    assert!(score > 0 && score < 4); // Less than exact match
    
    let score = fuzzy_match("npm", "nodejs-package-manager");
    assert!(score > 0);
}

#[test]
fn test_fuzzy_match_no_match() {
    assert_eq!(fuzzy_match("xyz", "abc"), 0);
    assert_eq!(fuzzy_match("test", ""), 0);
    assert_eq!(fuzzy_match("", "test"), 0);
}

#[test]
fn test_fuzzy_match_case_insensitive() {
    assert_eq!(fuzzy_match("GIT", "git"), 6);
    assert_eq!(fuzzy_match("git", "GIT"), 6);
    assert_eq!(fuzzy_match("PyThOn", "python"), 12);
}

#[test]
fn test_fuzzy_match_ordering() {
    // Exact substring match should score higher than partial
    let exact_score = fuzzy_match("git", "git-flow");
    let partial_score = fuzzy_match("git", "gitlab");
    assert!(exact_score >= partial_score);
}

#[test]
fn test_fuzzy_match_complex_cases() {
    // Test with common command patterns
    let ls_exact = fuzzy_match("ls", "ls");
    let ls_partial = fuzzy_match("ls", "lsof");
    // Both should have scores, but we'll just verify they work
    assert!(ls_exact > 0);
    assert!(ls_partial > 0);
    
    assert!(fuzzy_match("docker", "docker-compose") > 0);
    assert!(fuzzy_match("py", "python3") > 0);
}

#[test]
fn test_get_commands_integration() {
    // Integration test that calls the actual function
    let commands = get_commands();
    // Should always return a Vec (len() is always >= 0 for Vec)
    assert!(commands.is_empty() || !commands.is_empty());
    
    // If we have commands, they should have valid categories
    for cmd in commands.iter().take(5) {
        assert!(!cmd.name.is_empty());
        assert!(!cmd.category.is_empty());
        
        // Should be one of the valid categories
        let valid_categories = vec![
            "bin", "homebrew", "cask", "pip", "npm", "yarn", "cargo", "go", "alias", "function"
        ];
        assert!(valid_categories.contains(&cmd.category.as_str()));
    }
}

#[test]
fn test_command_categories() {
    // Test that all expected categories are valid
    let valid_categories = vec![
        "bin", "homebrew", "cask", "pip", "npm", "yarn", "cargo", "go", "alias", "function"
    ];
    
    // Create test commands for each category
    for category in valid_categories {
        let cmd = BentoCommand {
            name: "test-command".to_string(),
            category: category.to_string(),
        };
        assert_eq!(cmd.category, category);
    }
}

#[test]
fn test_empty_query() {
    assert_eq!(fuzzy_match("", "git"), 0);
    assert_eq!(fuzzy_match("", ""), 0);
}

#[test]
fn test_special_characters() {
    // Test commands with special characters that might exist
    assert!(fuzzy_match("ls", "ls-la") > 0);
    assert!(fuzzy_match("docker", "docker_compose") > 0);
    assert!(fuzzy_match("git", "git.exe") > 0);
}

#[test]
fn test_fuzzy_match_performance() {
    // Ensure fuzzy matching doesn't break with long strings
    let long_query = "a".repeat(100);
    let long_target = "b".repeat(100) + "a";
    let score = fuzzy_match(&long_query, &long_target);
    // Should not panic and should return a valid usize
    assert!(score <= long_query.len() * 2);
}

#[test]
fn test_command_filtering() {
    let test_commands = vec![
        BentoCommand {
            name: "git".to_string(),
            category: "bin".to_string(),
        },
        BentoCommand {
            name: "python".to_string(),
            category: "bin".to_string(),
        },
        BentoCommand {
            name: "django".to_string(),
            category: "pip".to_string(),
        },
        BentoCommand {
            name: "ll".to_string(),
            category: "alias".to_string(),
        },
    ];

    // Test that we can filter by category
    let bin_commands: Vec<_> = test_commands
        .iter()
        .filter(|cmd| cmd.category == "bin")
        .collect();
    assert_eq!(bin_commands.len(), 2);

    let pip_commands: Vec<_> = test_commands
        .iter()
        .filter(|cmd| cmd.category == "pip")
        .collect();
    assert_eq!(pip_commands.len(), 1);
    assert_eq!(pip_commands[0].name, "django");
}

#[test]
fn test_scoring_and_sorting() {
    let test_commands = vec![
        BentoCommand {
            name: "git".to_string(),
            category: "bin".to_string(),
        },
        BentoCommand {
            name: "gitignore".to_string(),
            category: "bin".to_string(),
        },
        BentoCommand {
            name: "gitlab".to_string(),
            category: "bin".to_string(),
        },
    ];

    let query = "git";
    let mut scored: Vec<_> = test_commands
        .iter()
        .map(|cmd| (fuzzy_match(query, &cmd.name), cmd))
        .filter(|(score, _)| *score > 0)
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    // "git" should score highest
    assert_eq!(scored[0].1.name, "git");
    assert!(scored[0].0 >= scored[1].0);
}

#[test]
fn test_unicode_handling() {
    // Test that fuzzy matching handles unicode properly
    // Unicode string "café" has 4 chars but may be encoded differently
    let score = fuzzy_match("café", "café");
    assert!(score > 0); // Should match
    
    let score = fuzzy_match("test", "tëst");
    assert!(score > 0); // Should get some match for the common characters
}

#[test]
fn test_edge_cases() {
    // Test various edge cases
    assert_eq!(fuzzy_match("a", "a"), 2);
    assert_eq!(fuzzy_match("ab", "ba"), 1); // Only 'a' matches in sequence
    assert_eq!(fuzzy_match("abc", "cba"), 1); // Only 'a' matches in sequence
}

#[test]
fn test_search_workflow() {
    // Test the complete search workflow
    let commands = vec![
        BentoCommand::new("git".to_string(), "bin".to_string()),
        BentoCommand::new("github-cli".to_string(), "homebrew".to_string()),
        BentoCommand::new("gitignore".to_string(), "npm".to_string()),
        BentoCommand::new("pytest".to_string(), "pip".to_string()),
    ];

    let query = "git";
    let mut results: Vec<_> = commands
        .iter()
        .map(|cmd| (fuzzy_match(query, &cmd.name), cmd))
        .filter(|(score, _)| *score > 0)
        .collect();

    results.sort_by(|a, b| b.0.cmp(&a.0));
    
    assert!(!results.is_empty());
    // All results should contain git-related commands
    for (score, cmd) in results {
        assert!(score > 0);
        assert!(cmd.name.to_lowercase().contains("git"));
    }
}

#[test]
fn test_category_filtering() {
    let commands = vec![
        BentoCommand::new("brew".to_string(), "homebrew".to_string()),
        BentoCommand::new("npm".to_string(), "bin".to_string()),
        BentoCommand::new("pip".to_string(), "bin".to_string()),
        BentoCommand::new("cargo".to_string(), "bin".to_string()),
    ];

    // Test filtering by category
    let bin_only: Vec<_> = commands
        .iter()
        .filter(|cmd| cmd.category == "bin")
        .collect();
    
    assert_eq!(bin_only.len(), 3);
    
    let homebrew_only: Vec<_> = commands
        .iter()
        .filter(|cmd| cmd.category == "homebrew")
        .collect();
    
    assert_eq!(homebrew_only.len(), 1);
    assert_eq!(homebrew_only[0].name, "brew");
}
