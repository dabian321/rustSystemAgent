use crate::config::Config;
use crate::llm::{ChatMessage, LlmClient};
use crate::tools;
use crate::tools::memory::MemoryManager;
use colored::Colorize;
use sysinfo::System;

pub struct Agent {
    llm: LlmClient,
    memory: MemoryManager,
    config: Config,
    system_info: String,
}

impl Agent {
    pub fn new(config: Config) -> Self {
        let llm = LlmClient::new(config.clone());
        let memory = MemoryManager::new(&config.data_dir);

        let mut sys = System::new_all();
        sys.refresh_all();
        let os_name = System::name().unwrap_or_else(|| "Unknown".into());
        let os_version = System::os_version().unwrap_or_else(|| "".into());
        let kernel = System::kernel_version().unwrap_or_else(|| "".into());
        let system_info = format!("{os_name} {os_version} (kernel {kernel}), Bash environment");

        memory.ensure_initialized(&system_info);

        Self {
            llm,
            memory,
            config,
            system_info,
        }
    }

    /// Run a single query through the agent loop and return the final text response.
    pub async fn run(&self, query: &str, conversation_history: Option<&[ChatMessage]>) -> Result<String, String> {
        let persistent_memory = self.memory.get_all_memory_content();

        let system_prompt = format!(
            "You are a helpful system agent running on {system_info}. \
             You have access to tools for executing shell commands, reading/writing files, \
             listing directories, managing a persistent memory store, web search (DuckDuckGo), and fetching web page content by URL.\n\n\
             Persistent Memory:\n{persistent_memory}\n\n\
             Rules:\n\
             - Use the terminal tool to execute commands compatible with the current OS.\n\
             - Always respond in the same language the user uses.\n\
             - Be concise and helpful.\n\
             - When the user asks you to remember something, use the remember_info tool.",
            system_info = self.system_info,
        );

        let mut messages: Vec<ChatMessage> = vec![ChatMessage {
            role: "system".into(),
            content: Some(system_prompt),
            tool_calls: None,
            tool_call_id: None,
        }];

        // Append conversation history if in interactive mode
        if let Some(history) = conversation_history {
            messages.extend_from_slice(history);
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: Some(query.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        let tool_defs = tools::tool_definitions();

        for iteration in 0..self.config.max_iterations {
            let response = self.llm.chat(&messages, &tool_defs).await?;

            if let Some(ref err) = response.error {
                return Err(format!("API error: {}", err.message));
            }

            let choice = response
                .choices
                .as_ref()
                .and_then(|c| c.first())
                .ok_or("No choices in API response")?;

            // If the model produced tool calls, execute them
            if let Some(ref tool_calls) = choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    // Add assistant message with tool_calls
                    messages.push(ChatMessage {
                        role: "assistant".into(),
                        content: choice.message.content.clone(),
                        tool_calls: Some(tool_calls.clone()),
                        tool_call_id: None,
                    });

                    for tc in tool_calls {
                        eprintln!(
                            "{}",
                            format!(
                                "  [{}/{}] Tool: {} ({})",
                                iteration + 1,
                                self.config.max_iterations,
                                tc.function.name,
                                truncate_str(&tc.function.arguments, 80)
                            )
                            .dimmed()
                        );

                        let result =
                            tools::dispatch_tool(&tc.function.name, &tc.function.arguments, &self.memory)
                                .await;

                        messages.push(ChatMessage {
                            role: "tool".into(),
                            content: Some(result),
                            tool_calls: None,
                            tool_call_id: Some(tc.id.clone()),
                        });
                    }

                    continue; // Next iteration
                }
            }

            // No tool calls → final text response
            let text = choice
                .message
                .content
                .clone()
                .unwrap_or_else(|| "(no response)".to_string());
            return Ok(text);
        }

        Err("Agent reached maximum iterations without a final response.".to_string())
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
