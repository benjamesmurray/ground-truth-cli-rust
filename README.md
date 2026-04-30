# Ground Truth CLI (Rust)

A high-performance Rust reimplementation of the `ground-truth-cli` Model Context Protocol (MCP) server.

## Overview

`ground-truth-cli-rust` is an Agent-Native MCP server designed to synthesize "Ground Truth" rules for AI coding assistants. It scans your project's context (dependencies, language, architecture) and generates a `.assistant_rules.toon` file that provides rigid behavioral constraints for the AI.

## Features

- **Agent-Native Design:** Optimized for minimal initial token footprint.
- **TOON Format:** Outputs rules in Token-Oriented Object Notation to reduce context usage.
- **Auto-Discovery:** Automatically detects languages (Rust, TypeScript) and architectural domains (Tokio, Next.js, Fastify, etc.).
- **Embedded Rules:** Permanent rule files are embedded directly into the binary for a single-binary distribution.

## Installation

```bash
cargo install ground-truth-cli-rust
```

## Building from Source

```bash
git clone https://github.com/benjamesmurray/ground-truth-cli-rust
cd ground-truth-cli-rust
cargo build --release
```

## Usage as MCP Server

Add this to your MCP settings (e.g., in Claude Desktop configuration):

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

### Available Tools

- `gt_status`: Returns the current project state and detection results.
- `gt_exec_scan`: Runs the scanner and generates the `.assistant_rules.toon` file.

## License

MIT
