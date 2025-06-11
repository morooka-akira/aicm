# AI Context Management Tool (aicm) 🦀

A unified CLI tool built in Rust to automatically generate context files for multiple AI coding agents from a single configuration.

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/morooka-akira/ai-context-management/rust.yml?branch=main)](https://github.com/morooka-akira/ai-context-management/actions)

[Installation](#installation) • [Quick Start](#quick-start) • [Configuration](#configuration) • [Testing](#testing) • [Development](#development)

</div>

## ✨ Overview

**aicm** streamlines AI-assisted development by centralizing context management for popular AI coding tools. Instead of maintaining separate configuration files for each tool, define your project context once and let aicm generate the appropriate formats for all your AI assistants.

### 🎯 Supported Tools

- **✅ Cursor**: `.cursor/rules/*.mdc` files (with split_config support)
- **✅ Cline**: `.clinerules/*.md` files  
- **✅ GitHub Copilot**: `.github/instructions/*.instructions.md` or `.github/copilot-instructions.md` (with applyTo options)
- **✅ Claude Code**: `CLAUDE.md`
- **✅ OpenAI Codex**: `AGENTS.md`

## 🚀 Installation

### Using Cargo (Recommended)

```bash
# Install from crates.io (coming soon)
cargo install aicm

# Install directly from GitHub
cargo install --git https://github.com/morooka-akira/ai-context-management

# Local build and install
git clone https://github.com/morooka-akira/ai-context-management
cd ai-context-management
cargo install --path .
```

### Requirements

- Rust 1.70.0 or higher
- Cargo (installed with Rust)

## ⚡ Quick Start

```bash
# Initialize configuration in your project
aicm init

# Edit the configuration file
vim ai-context.yaml

# Generate context files for all enabled agents
aicm generate

# Generate for a specific agent only
aicm generate --agent cursor

# Validate your configuration
aicm validate
```

## 📖 Configuration

### Basic Configuration

Create an `ai-context.yaml` file in your project root:

```yaml
# ai-context.yaml
version: "1.0"
output_mode: split         # merged | split
include_filenames: false   # Include file name headers in merged mode
base_docs_dir: ./ai-docs

# Simple agent configuration
agents:
  cursor: true
  cline: false
  github: true
  claude: true
  codex: false
```

### Advanced Configuration

```yaml
version: "1.0"
output_mode: split
include_filenames: false
base_docs_dir: ./ai-context

agents:
  # Advanced Cursor configuration with split_config
  cursor:
    enabled: true
    output_mode: split
    include_filenames: true
    split_config:
      rules:
        - file_patterns: ["*project*", "*overview*"]
          alwaysApply: true
        - file_patterns: ["*architecture*", "*design*"] 
          globs: ["**/*.rs", "**/*.ts"]
        - file_patterns: ["*development*", "*rules*"]
          description: "Development guidelines and coding standards"
        - file_patterns: ["*troubleshoot*", "*debug*"]
          manual: true

  # GitHub Copilot with applyTo options
  github:
    enabled: true
    output_mode: split
    split_config:
      rules:
        - file_patterns: ["*backend*", "*api*"]
          apply_to: ["**/*.rs", "**/*.toml"]
        - file_patterns: ["*frontend*", "*ui*"]
          apply_to: ["**/*.ts", "**/*.tsx"]

  # Simple configurations
  claude: true
  cline: false
  codex: false
```

### External Configuration Files

Use the `--config` / `-c` option to specify alternative configuration files:

```bash
# Use custom configuration
aicm generate --config production.yaml
aicm generate -c ./configs/staging.yaml

# Combine with specific agent
aicm generate --agent cursor --config custom.yaml
```

## 🏗️ Project Structure

```
your-project/
├── ai-context/              # Documentation directory (base_docs_dir)
│   ├── 01-project-overview.md
│   ├── 02-architecture.md
│   ├── 03-development-rules.md
│   └── 04-api-reference.md
├── ai-context.yaml          # Configuration file
├── src/
│   └── main.rs
└── Cargo.toml
```

## 📤 Generated Output

### Cursor
```
.cursor/rules/
├── project-overview.mdc      # alwaysApply: true
├── architecture.mdc          # globs: ["**/*.rs"]
└── development-rules.mdc     # description: "..."
```

### GitHub Copilot
```
.github/instructions/
├── backend.instructions.md   # applyTo: "**/*.rs,**/*.toml"
└── frontend.instructions.md  # applyTo: "**/*.ts,**/*.tsx"
```

### Other Agents
```
.clinerules/context.md        # Cline (merged)
CLAUDE.md                     # Claude Code (merged)
AGENTS.md                     # OpenAI Codex (merged)
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test config

# Run with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# Integration tests
cargo test --test integration_test
```

## 🛠️ Development

### Setup

```bash
git clone https://github.com/morooka-akira/ai-context-management
cd ai-context-management
cargo build
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check all targets
cargo clippy --all-targets --all-features
```

### Architecture

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library entry point
├── config/                 # Configuration management
├── core/                   # Core functionality
├── agents/                 # Agent implementations
└── types/                  # Type definitions
```

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run `cargo fmt` and `cargo clippy`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to your branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Add comprehensive tests for new features
- Update documentation for user-facing changes
- Run the full test suite before submitting
- Use conventional commit messages

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

This project is built with excellent Rust ecosystem tools:

- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [tokio](https://github.com/tokio-rs/tokio) - Asynchronous runtime
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling

## 📞 Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/morooka-akira/ai-context-management/issues)
- 💡 **Feature Requests**: [GitHub Issues](https://github.com/morooka-akira/ai-context-management/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/morooka-akira/ai-context-management/discussions)

---

<div align="center">

Made with ❤️ for the AI-assisted development community

</div>