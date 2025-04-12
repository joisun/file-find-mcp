# File Find MCP

A specialized Model Context Protocol (MCP) server for fast file search within a filesystem, built with Rust and powered by ripgrep.

## 🔍 Overview

File Find MCP is a tool that provides powerful search capabilities for files in a specified directory. It uses [ripgrep](https://github.com/BurntSushi/ripgrep) to perform high-performance searches through file content efficiently.

This project implements the Model Context Protocol (MCP), making it compatible with AI assistants and other systems that support the protocol.

## ✨ Features

- **High-performance search**: Uses ripgrep for extremely fast searches across directory structures
- **File content reader**: Read and display the content of specific text files
- **Smart file detection**: Automatically identifies text files and skips binary files
- **MCP integration**: Works with systems that support the Model Context Protocol
- **Fallback mechanism**: Uses pure Rust implementation when ripgrep command is not available


## 📋 Prerequisites

Before using this tool, make sure you have:

1. **Rust** installed - [Install Rust](https://www.rust-lang.org/tools/install)
2. **ripgrep** installed - On macOS, you can install it using Homebrew:
   ```bash
   brew install ripgrep
   ```

## 📦 Building

To build the project:

```bash
# Clone the repository
git clone https://github.com/yourusername/file-find-mcp.git
cd file-find-mcp

# Build in release mode
cargo build --release
```

The compiled binary will be available at `target/release/file-find-mcp`.

## ⚙️ Configuration

Add this to your MCP settings (in Cursor, Claude, or other MCP-compatible tools):

```json
{
  "mcpServers": {
    "file-find-mcp": {
      "command": "/path/to/your/file-find-mcp/target/release/file-find-mcp"
    }
  }
}
```

Replace `/path/to/your/file-find-mcp` with the actual path to your cloned repository.

## 🛠️ Available Tools

### Search Tool

- **Description**: Search for keywords in text files within a specified directory
- **Parameters**:
  - `directory`: Path to the directory to search
  - `keyword`: Keyword to search for

### File Content Reader Tool

- **Description**: Read and display the content of a specific file
- **Parameters**:
  - `file_path`: Path to the file to read

## 📄 License

MIT License

## 🙏 Acknowledgements

- [ripgrep](https://github.com/BurntSushi/ripgrep) for the lightning-fast search capabilities
- [RMCP](https://github.com/modelcontextprotocol/rust-sdk) for the Model Context Protocol implementation
- This project is forked from [file-search-mcp](https://github.com/Kurogoma4D/file-search-mcp) and modified to use ripgrep for improved search efficiency
