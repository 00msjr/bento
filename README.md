# 🍱 Bento

A unified command organizer that searches across all your installed commands, packages, aliases, and functions in one place.

## Description

Bento is a Rust-based command-line tool that provides a centralized search interface for discovering commands across your entire system. Instead of trying to remember whether that utility you installed was a homebrew formula, an npm package, or a shell alias, Bento searches everything simultaneously and presents results with clear categorization and color coding. It supports fuzzy matching, making it easy to find commands even when you only remember part of the name.

Bento searches across:
- System binaries in your PATH
- Homebrew packages (formulae and casks)
- Python packages (pip)
- Node.js packages (npm and yarn)
- Rust packages (cargo)
- Go modules
- Shell aliases
- Shell functions

## Getting Started

### Dependencies

* **Operating System**: macOS or Linux (Unix-like system required for file permissions check)
* **Rust**: Version 1.70.0 or higher
* **Optional package managers** (Bento will gracefully skip any that aren't installed):
  * Homebrew (`brew`)
  * Python pip (`pip`)
  * Node.js npm (`npm`)
  * Yarn (`yarn`)
  * Cargo (comes with Rust)
  * Go (`go`)

### Installing

#### Option 1: Install from crates.io (Recommended)
```bash
cargo install bento
```

#### Option 2: Install from GitHub using Cargo
```bash
cargo install --git https://github.com/00msjr/bento.git
```

#### Option 3: Build from source
1. **Clone the repository**
```bash
git clone https://github.com/00msjr/bento.git
cd bento
```

2. **Build from source using Cargo**
```bash
cargo build --release
```

3. **Install the binary**
```bash
# Option 1: Install to cargo bin directory
cargo install --path .

# Option 2: Copy to a location in your PATH
sudo cp target/release/bento /usr/local/bin/
```

4. **Verify installation**
```bash
bento --help
```

### Executing program

**Basic search across all sources:**
```bash
bento git
```

**Search with category filters:**
```bash
# Search only homebrew packages
bento --homebrew python

# Search only aliases
bento --alias ls

# Search only npm packages
bento --npm react
```

**View available commands and statistics:**
```bash
bento
```

**Example outputs:**
- Yellow text: Aliases
- Green text: Functions
- Blue text: Binary commands
- Cyan text: Homebrew formulae
- Bright cyan: Homebrew casks
- Red text: Python packages
- Magenta text: NPM packages
- Bright magenta: Yarn packages
- Bright red: Cargo packages
- Bright blue: Go packages

## Help

**Common issues and solutions:**

1. **No aliases or functions showing up**
   - Bento attempts to read from multiple shells (zsh, bash, sh)
   - Ensure your shell configuration files are properly sourced
   - Try running with your specific shell: `SHELL=/bin/zsh bento`

2. **Permission denied errors**
   - Ensure the binary has execute permissions: `chmod +x /usr/local/bin/bento`

3. **Package manager commands not found**
   - Bento gracefully skips package managers that aren't installed
   - Install the relevant package manager to search its packages

4. **Too many results**
   - Use category filters to narrow down results
   - Results are limited to top 15 matches by fuzzy search score

For more help:
```bash
bento --help
```

## Authors

Contributors and maintainers:
* [@00msjr](https://github.com/00msjr)

## License

This project is licensed under the MIT License - see the LICENSE.md file for details

## Acknowledgments

* Inspired by the need for a unified command discovery tool
* Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
* Uses [colored](https://github.com/mackwic/colored) for terminal colors
* Special thanks to [@dompizzie](https://twitter.com/dompizzie) for the README template
