# ground-truth-cli-rust

High-performance Rust reimplementation of the `ground-truth-cli` Model Context Protocol (MCP) server. Generates `.assistant_rules.toon` files to provide context-aware behavioral constraints for AI assistants.

## Features
- **Agent-Native:** Minimal initial token footprint.
- **Dynamic Ranking:** Injects only the top 3-5 most critical rules based on intent.
- **Lazy-Load Guidance:** Expert developer guidance loaded only on demand.
- **Auto-Discovery:** Detects language (Rust, TypeScript) and framework/architecture.

## Installation
```bash
cargo install ground-truth-cli-rust
```

## Setup (Claude Desktop)
Add the server to your MCP settings (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "ground-truth": {
      "command": "ground-truth-cli-rust",
      "args": []
    }
  }
}
```

## Tools
- `gt_status`: Returns current project state and framework detection.
- `gt_exec_scan`: Scans project and generates `.assistant_rules.toon`.
  - **Args:**
    - `path` (optional): Directory to scan. Default: `.`
    - `intent` (optional): Current task intent (e.g., "debugging", "refactoring") to rank rules.
    - `include_guidance` (optional): `true` to append expert developer guidance. Default: `false`.

## Building
```bash
git clone https://github.com/benjamesmurray/ground-truth-cli-rust
cd ground-truth-cli-rust
cargo build --release
```
