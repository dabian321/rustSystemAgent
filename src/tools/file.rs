use std::path::Path;
use tokio::fs;

/// Read a file and return its contents.
pub async fn read_file(path: &str) -> String {
    let path = path.trim();
    match fs::read_to_string(path).await {
        Ok(content) => {
            if content.len() > 50_000 {
                format!(
                    "File content (truncated, {} bytes total):\n{}...",
                    content.len(),
                    &content[..50_000]
                )
            } else {
                format!("File content:\n{content}")
            }
        }
        Err(e) => format!("Failed to read file '{path}': {e}"),
    }
}

/// Write content to a file. Input should be JSON: {"file_path": "...", "text": "..."}.
pub async fn write_file(input: &str) -> String {
    #[derive(serde::Deserialize)]
    struct WriteInput {
        file_path: String,
        text: String,
    }

    let parsed: WriteInput = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return format!("Invalid JSON input: {e}. Expected: {{\"file_path\": \"...\", \"text\": \"...\"}}"),
    };

    // Create parent directories if needed
    if let Some(parent) = Path::new(&parsed.file_path).parent() {
        let _ = fs::create_dir_all(parent).await;
    }

    match fs::write(&parsed.file_path, &parsed.text).await {
        Ok(()) => format!("Successfully wrote to: {}", parsed.file_path),
        Err(e) => format!("Failed to write file '{}': {e}", parsed.file_path),
    }
}

/// List directory contents.
pub async fn list_directory(path: &str) -> String {
    let path = if path.trim().is_empty() { "." } else { path.trim() };

    let mut entries = match fs::read_dir(path).await {
        Ok(e) => e,
        Err(e) => return format!("Failed to list directory '{path}': {e}"),
    };

    let mut items = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
        let prefix = if is_dir { "[DIR] " } else { "[FILE]" };
        items.push(format!("{prefix} {name}"));
    }

    items.sort();
    format!("Directory listing ({path}):\n{}", items.join("\n"))
}
