use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: u64,
    pub content: String,
    pub memory_type: String,
    pub importance: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryStore {
    entries: Vec<MemoryEntry>,
    next_id: u64,
    last_updated: String,
}

pub struct MemoryManager {
    file_path: PathBuf,
}

impl MemoryManager {
    pub fn new(data_dir: &std::path::Path) -> Self {
        let file_path = data_dir.join("memory.json");
        Self { file_path }
    }

    fn load(&self) -> MemoryStore {
        if self.file_path.exists() {
            match std::fs::read_to_string(&self.file_path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or(MemoryStore {
                    entries: Vec::new(),
                    next_id: 1,
                    last_updated: Utc::now().to_rfc3339(),
                }),
                Err(_) => MemoryStore {
                    entries: Vec::new(),
                    next_id: 1,
                    last_updated: Utc::now().to_rfc3339(),
                },
            }
        } else {
            MemoryStore {
                entries: Vec::new(),
                next_id: 1,
                last_updated: Utc::now().to_rfc3339(),
            }
        }
    }

    fn save(&self, store: &MemoryStore) {
        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(store) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }

    /// Ensure the memory store is initialized with system context.
    pub fn ensure_initialized(&self, system_info: &str) {
        let store = self.load();
        if store.entries.is_empty() {
            let mut store = store;
            let now = Utc::now().to_rfc3339();
            store.entries.push(MemoryEntry {
                id: store.next_id,
                content: format!("System Context: {system_info}"),
                memory_type: "system_context".to_string(),
                importance: "high".to_string(),
                timestamp: now.clone(),
            });
            store.next_id += 1;
            store.entries.push(MemoryEntry {
                id: store.next_id,
                content: "User Preferences: Language - Mixed (English/Chinese), Tone - Helpful and concise".to_string(),
                memory_type: "user_preferences".to_string(),
                importance: "medium".to_string(),
                timestamp: now,
            });
            store.next_id += 1;
            store.last_updated = Utc::now().to_rfc3339();
            self.save(&store);
        }
    }

    /// Get all memory content as a formatted string.
    pub fn get_all_memory_content(&self) -> String {
        let store = self.load();
        if store.entries.is_empty() {
            return "No memory content yet.".to_string();
        }
        let lines: Vec<String> = store
            .entries
            .iter()
            .map(|e| {
                format!(
                    "- {} (type: {}, time: {})",
                    e.content, e.memory_type, e.timestamp
                )
            })
            .collect();
        format!("# Memory Store\n\n{}", lines.join("\n"))
    }

    /// Add a new memory entry.
    pub fn add_memory(&self, content: &str, memory_type: &str, importance: &str) -> String {
        let mut store = self.load();
        store.entries.push(MemoryEntry {
            id: store.next_id,
            content: content.to_string(),
            memory_type: memory_type.to_string(),
            importance: importance.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        });
        store.next_id += 1;
        store.last_updated = Utc::now().to_rfc3339();
        self.save(&store);
        "Memory saved successfully.".to_string()
    }

    /// Search memory by keyword (simple substring matching).
    pub fn search_memory(&self, query: &str) -> String {
        let store = self.load();
        let query_lower = query.to_lowercase();
        let results: Vec<&MemoryEntry> = store
            .entries
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&query_lower))
            .collect();

        if results.is_empty() {
            return "No matching memory found.".to_string();
        }

        let lines: Vec<String> = results
            .iter()
            .enumerate()
            .map(|(i, e)| {
                format!(
                    "{}. [{}] {} (time: {})",
                    i + 1,
                    e.memory_type,
                    e.content,
                    e.timestamp
                )
            })
            .collect();

        format!("Found {} matching memories:\n\n{}", results.len(), lines.join("\n"))
    }

    /// Get memory statistics.
    pub fn get_stats(&self) -> String {
        let store = self.load();
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for entry in &store.entries {
            *type_counts.entry(entry.memory_type.clone()).or_insert(0) += 1;
        }

        let type_info: Vec<String> = type_counts
            .iter()
            .map(|(k, v)| format!("  {k}: {v}"))
            .collect();

        format!(
            "Memory Statistics:\n- Total entries: {}\n- Last updated: {}\n- Types:\n{}",
            store.entries.len(),
            store.last_updated,
            type_info.join("\n")
        )
    }

    /// Clear all memory.
    pub fn clear_memory(&self, confirm: &str) -> String {
        if confirm.trim().to_lowercase() != "confirm" {
            return "Please input 'confirm' to clear all memories.".to_string();
        }
        let store = MemoryStore {
            entries: Vec::new(),
            next_id: 1,
            last_updated: Utc::now().to_rfc3339(),
        };
        self.save(&store);
        "All memories cleared.".to_string()
    }

    /// Delete memories by type.
    pub fn delete_by_type(&self, memory_type: &str) -> String {
        let mut store = self.load();
        let before = store.entries.len();
        store.entries.retain(|e| e.memory_type != memory_type.trim());
        let deleted = before - store.entries.len();
        store.last_updated = Utc::now().to_rfc3339();
        self.save(&store);
        format!("Deleted {deleted} entries of type '{memory_type}'.")
    }
}
