pub mod file;
pub mod memory;
pub mod shell;

use crate::llm::{FunctionDefinition, ToolDefinition};
use memory::MemoryManager;
use serde_json::json;

/// Build the list of tool definitions sent to the LLM.
pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "terminal".into(),
                description: "Execute a shell command in bash and return the output. Use this to run system commands, view files, manage processes, etc.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        }
                    },
                    "required": ["command"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "read_file".into(),
                description: "Read the content of a file given its path.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "write_file".into(),
                description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file to write"
                        },
                        "text": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["file_path", "text"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "list_directory".into(),
                description: "List files and directories in a given path.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory path to list (defaults to current directory)"
                        }
                    },
                    "required": []
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "remember_info".into(),
                description: "Save important information to persistent memory. Use when the user asks you to remember something.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The information to remember"
                        }
                    },
                    "required": ["content"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "search_memory".into(),
                description: "Search for previously stored information in memory by keyword.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query or keywords"
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "memory_stats".into(),
                description: "Get statistics about the memory database.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "clear_memory".into(),
                description: "Clear all memories. Input 'confirm' to proceed.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "confirm": {
                            "type": "string",
                            "description": "Must be 'confirm' to proceed"
                        }
                    },
                    "required": ["confirm"]
                }),
            },
        },
        ToolDefinition {
            tool_type: "function".into(),
            function: FunctionDefinition {
                name: "delete_memory_type".into(),
                description: "Delete all memories of a specific type.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "memory_type": {
                            "type": "string",
                            "description": "The memory type to delete (e.g. 'user_note', 'system_context')"
                        }
                    },
                    "required": ["memory_type"]
                }),
            },
        },
    ]
}

/// Dispatch a tool call to the appropriate handler and return the result string.
pub async fn dispatch_tool(
    name: &str,
    arguments: &str,
    memory: &MemoryManager,
) -> String {
    let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();

    match name {
        "terminal" => {
            let cmd = args["command"].as_str().unwrap_or(arguments);
            shell::execute_shell(cmd).await
        }
        "read_file" => {
            let path = args["file_path"].as_str().unwrap_or(arguments);
            file::read_file(path).await
        }
        "write_file" => {
            // Pass the full JSON so write_file can parse it
            let input = json!({
                "file_path": args["file_path"].as_str().unwrap_or(""),
                "text": args["text"].as_str().unwrap_or("")
            });
            file::write_file(&input.to_string()).await
        }
        "list_directory" => {
            let path = args["path"].as_str().unwrap_or(".");
            file::list_directory(path).await
        }
        "remember_info" => {
            let content = args["content"].as_str().unwrap_or(arguments);
            memory.add_memory(content, "user_note", "medium")
        }
        "search_memory" => {
            let query = args["query"].as_str().unwrap_or(arguments);
            memory.search_memory(query)
        }
        "memory_stats" => memory.get_stats(),
        "clear_memory" => {
            let confirm = args["confirm"].as_str().unwrap_or("");
            memory.clear_memory(confirm)
        }
        "delete_memory_type" => {
            let mt = args["memory_type"].as_str().unwrap_or(arguments);
            memory.delete_by_type(mt)
        }
        _ => format!("Unknown tool: {name}"),
    }
}
