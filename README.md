# Rust System Agent

A fast, lightweight system agent written in Rust. Interact with your system using natural language — execute commands, read/write files, and manage persistent memory, all powered by LLM tool-calling.

## Features

- **Single query mode (`rsa`)** — ask a question, get a result
- **Interactive session mode (`rasi`)** — multi-turn conversation with context history
- **Tool-calling agent loop** — the LLM decides which tools to invoke, iterating until a final answer
- **9 built-in tools**: shell execution, file read/write, directory listing, persistent memory (remember / search / stats / clear / delete)
- **OpenRouter / OpenAI compatible** — works with any model that supports function calling
- **Auto OS detection** — generates shell commands appropriate for the host system
- **Fast startup** — single static binary, no runtime dependencies

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- An API key for [OpenRouter](https://openrouter.ai/) or OpenAI

### Build

```bash
git clone https://github.com/dabian321/rustSystemAgent.git
cd rustSystemAgent
cargo build --release
```

The binary is at `target/release/rsa`.

### Configure

Create a `.env` file in the project root (or anywhere the binary can find it):

```env
# OpenRouter API Key (recommended)
OPENROUTER_API_KEY=sk-or-v1-your-key-here

# Or use OpenAI directly
# OPENAI_API_KEY=sk-your-key-here

# Model (default: google/gemini-2.5-flash)
MODEL_NAME=google/gemini-2.5-flash
```

### Install

Copy the binary to a directory in your PATH:

```bash
cp target/release/rsa ~/.local/bin/
```

Optionally, add a shell alias for interactive mode in `~/.bashrc`:

```bash
alias rasi='rsa interactive'
```

Then reload: `source ~/.bashrc`

## Usage

### Single Query

```bash
rsa '列出当前目录文件'
rsa 'show system memory and CPU info'
rsa '创建一个 hello.py 文件，内容是打印 Hello World'
rsa 'remember I prefer Python for scripting'
```

### Interactive Session

```bash
rasi
# or
rsa interactive
```

In interactive mode you get a `>>` prompt with:
- Multi-turn conversation with context awareness (keeps last 10 exchanges)
- readline support (arrow keys, history, Ctrl+R search)
- Type `quit` or `exit` (or Ctrl+C / Ctrl+D) to leave

### Example

```
$ rsa '显示系统信息'

Rust System Agent

Query: 显示系统信息

Processing...

  [1/15] Tool: terminal ({"command":"uname -a && free -h"})

> uname -a && free -h
--------------------------------------------------
Linux myhost 6.8.0-94-generic #94-Ubuntu ...
               total        used        free
Mem:            15Gi       4.5Gi       1.7Gi
--------------------------------------------------
Command succeeded

Response:
您的系统信息：
- OS: Linux 6.8.0-94-generic (Ubuntu)
- 总内存: 15Gi, 已用: 4.5Gi, 可用: 10Gi

Time: 5.99s
```

## Built-in Tools

| Tool | Description |
|------|-------------|
| `terminal` | Execute shell commands with real-time output |
| `read_file` | Read file contents |
| `write_file` | Write/create files |
| `list_directory` | List directory contents |
| `remember_info` | Save information to persistent memory |
| `search_memory` | Search memory by keyword |
| `memory_stats` | Show memory database statistics |
| `clear_memory` | Clear all memories (requires confirmation) |
| `delete_memory_type` | Delete memories by type |

## Project Structure

```
src/
├── main.rs          # CLI entry point (clap), single + interactive modes
├── config.rs        # Configuration loading from .env
├── llm.rs           # OpenAI-compatible chat completions client
├── agent.rs         # Agent loop: system prompt → LLM → tool calls → iterate
└── tools/
    ├── mod.rs       # Tool definitions (JSON Schema) + dispatch
    ├── shell.rs     # Shell command execution (async, streamed output)
    ├── file.rs      # File read / write / list directory
    └── memory.rs    # JSON-based persistent memory store
```

## How It Works

1. The user's query is sent to the LLM along with tool definitions (OpenAI function-calling format)
2. If the LLM responds with tool calls, each tool is executed and results are fed back
3. This loop continues (up to 15 iterations) until the LLM produces a final text response
4. In interactive mode, conversation history is preserved across turns for context awareness

## License

GPL3
