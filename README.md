# Domain Forge

**Domain name generation and availability checking tool**

A CLI tool that generates domain names and checks their availability in real-time.

## Features

- **Domain Generation**: Generate domain names using OpenAI, Anthropic, Gemini, or Ollama
- **Real-time Availability Checking**: Check domain availability using RDAP and WHOIS protocols
- **Beautiful Terminal UI**: Interactive multi-select interface with inquire
- **Simple & Fast**: Minimal setup, maximum productivity
- **Multi-Provider Support**: OpenAI, Anthropic, Google Gemini, and Ollama support

## Quick Start

### 1. Install

```bash
git clone https://github.com/voocel/domain-forge.git
cd domain-forge
cargo build --release
```

### 2. Set up API Key

Choose one of the supported providers:

```bash
# OpenAI (recommended)
export OPENAI_API_KEY="your-openai-api-key"

# Anthropic
export ANTHROPIC_API_KEY="your-anthropic-api-key"

# Gemini
export GEMINI_API_KEY="your-gemini-api-key"

# Ollama (local, no API key needed)
# Just make sure Ollama is running
```

### 3. Run

```bash
# Generate random domains (no input needed)
./target/release/domain-forge

# Generate domains for your idea
./target/release/domain-forge "AI-powered productivity app"
```

## How It Works

### Without Arguments (Random Generation)
```bash
./target/release/domain-forge
```
- Generates domains for a business idea
- Shows beautiful interactive selection interface
- Check availability for selected domains

### With Description
```bash
./target/release/domain-forge "sustainable fashion brand"
```
- Generates domains based on your description
- Interactive selection with multi-select
- Real-time availability checking

## Interactive Interface

The tool provides a beautiful terminal interface:

```
Domain Forge - Domain name generation
═══════════════════════════════════════════════════

✅ OpenAI provider configured
→ Generating domains for: "productivity tool"
⏳ Processing request...

Generated Domains:
═══════════════════
1. productiv.com (score: 85%)
   Analysis: Short, memorable domain name

2. taskforge.io (score: 92%)
   Analysis: Combines task management concept

? Select domains to check availability:
❯ ◯ 🔄 Generate more options
  ◯ productiv.com (85%)
  ◯ taskforge.io (92%)
  ◯ ✅ Check all domains
```

## Configuration

### Environment Variables

```bash
# API Keys (choose one or more)
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
export GEMINI_API_KEY="your-key"

# Optional: Custom models
export OPENAI_MODEL="gpt-4.1-mini"
export ANTHROPIC_MODEL="claude-4-sonnet"
export GEMINI_MODEL="gemini-2.5-flash"
export OLLAMA_MODEL="deepseek-r1"
```

### Supported Providers

| Provider | Models | Notes |
|----------|--------|-------|
| **OpenAI** | gpt-4.1, gpt-4.1-mini, o3, o4-mini | Recommended |
| **Anthropic** | claude-3.7-sonnet, claude-4-sonnet | Alternative option |
| **Gemini** | gemini-2.5-pro, gemini-2.5-flash | Cost-effective |
| **Ollama** | deepseek-r1, deepseek-v3, qwen3 | Local deployment |

## Examples

### Startup Ideas
```bash
./target/release/domain-forge "fintech mobile app"
./target/release/domain-forge "sustainable energy platform"
./target/release/domain-forge "AI-powered healthcare"
```

### Creative Projects
```bash
./target/release/domain-forge "indie game studio"
./target/release/domain-forge "digital art marketplace"
./target/release/domain-forge "music streaming service"
```

## Project Structure

```
domain-forge/
├── src/
│   ├── main.rs           # Main program entry
│   ├── lib.rs            # Library entry
│   ├── types.rs          # Type definitions
│   ├── error.rs          # Error handling
│   ├── domain/           # Domain checking module
│   │   ├── mod.rs
│   │   ├── checker.rs
│   │   ├── rdap.rs
│   │   ├── whois.rs
│   │   └── validator.rs
│   └── llm/              # LLM module
│       ├── mod.rs
│       ├── generator.rs
│       └── providers/    # Each provider in separate file
│           ├── mod.rs
│           ├── openai.rs
│           ├── anthropic.rs
│           ├── gemini.rs
│           └── ollama.rs
├── Cargo.toml            # Dependencies configuration
├── README.md             # Project documentation
└── demo.sh               # Demo script
```

## Development

### Build from Source
```bash
git clone https://github.com/voocel/domain-forge.git
cd domain-forge
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Check Code
```bash
cargo check
cargo clippy
```

## Requirements

- Rust 1.70+
- API key for at least one supported provider
- Internet connection for domain checking

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

Apache License - see [LICENSE](LICENSE) for details.

---

**Made with Rust**
