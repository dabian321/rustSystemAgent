mod agent;
mod config;
mod llm;
mod tools;

use agent::Agent;
use clap::{Parser, Subcommand};
use colored::Colorize;
use config::Config;
use llm::ChatMessage;
use rustyline::DefaultEditor;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "rsa", version, about = "Rust System Agent - 智能系统代理")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Direct query (when no subcommand is used)
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive session mode with conversation history
    #[command(name = "interactive", alias = "i")]
    Interactive,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {e}", "Error:".red().bold());
            std::process::exit(1);
        }
    };

    match cli.command {
        Some(Commands::Interactive) => {
            run_interactive(config).await;
        }
        None => {
            let query = cli.query.join(" ");
            if query.trim().is_empty() {
                print_usage();
                return;
            }
            run_single(config, &query).await;
        }
    }
}

fn print_usage() {
    eprintln!("{}", "Rust System Agent - 智能系统助手".blue().bold());
    eprintln!("用法:");
    eprintln!("  {} <your request>          Single query", "rsa".green());
    eprintln!(
        "  {} interactive              Interactive session",
        "rsa".green()
    );
    eprintln!();
    eprintln!("示例:");
    eprintln!("  rsa '列出当前目录文件'");
    eprintln!("  rsa '显示系统信息'");
    eprintln!("  rsa interactive");
}

async fn run_single(config: Config, query: &str) {
    let start = Instant::now();
    eprintln!("{}", "\nRust System Agent\n".blue().bold());
    eprintln!("{} {}\n", "Query:".cyan(), query);

    let agent = Agent::new(config);

    eprintln!("{}", "Processing...\n".yellow());
    match agent.run(query, None).await {
        Ok(response) => {
            eprintln!("{}", "\nResponse:\n".green().bold());
            println!("{response}");
            let elapsed = start.elapsed().as_secs_f64();
            eprintln!("{}", format!("\nTime: {elapsed:.2}s\n").dimmed());
        }
        Err(e) => {
            eprintln!("{} {e}", "Error:".red().bold());
            std::process::exit(1);
        }
    }
}

async fn run_interactive(config: Config) {
    eprintln!("{}", "\nRust System Agent - Interactive Mode".blue().bold());
    eprintln!(
        "{}",
        "Type 'quit' or 'exit' to leave.\n".dimmed()
    );

    let agent = Agent::new(config);
    let mut conversation_history: Vec<ChatMessage> = Vec::new();

    let mut rl = match DefaultEditor::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} Failed to init readline: {e}", "Error:".red().bold());
            std::process::exit(1);
        }
    };

    loop {
        let readline = rl.readline(&format!("{} ", ">>".cyan()));
        match readline {
            Ok(line) => {
                let query = line.trim().to_string();
                if query.is_empty() {
                    continue;
                }
                if query == "quit" || query == "exit" {
                    eprintln!("{}", "Goodbye!".yellow());
                    break;
                }

                let _ = rl.add_history_entry(&query);

                eprintln!("{}", "Processing...\n".yellow());
                let start = Instant::now();

                match agent.run(&query, Some(&conversation_history)).await {
                    Ok(response) => {
                        eprintln!("{}", "Response:".green().bold());
                        println!("{response}");
                        let elapsed = start.elapsed().as_secs_f64();
                        eprintln!("{}", format!("({elapsed:.2}s)\n").dimmed());

                        // Record in conversation history
                        conversation_history.push(ChatMessage {
                            role: "user".into(),
                            content: Some(query),
                            tool_calls: None,
                            tool_call_id: None,
                        });
                        conversation_history.push(ChatMessage {
                            role: "assistant".into(),
                            content: Some(response),
                            tool_calls: None,
                            tool_call_id: None,
                        });

                        // Keep last 20 messages (10 exchanges)
                        if conversation_history.len() > 20 {
                            conversation_history = conversation_history
                                [conversation_history.len() - 20..]
                                .to_vec();
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {e}\n", "Error:".red().bold());
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof) => {
                eprintln!("{}", "Goodbye!".yellow());
                break;
            }
            Err(e) => {
                eprintln!("{} {e}", "Error:".red().bold());
                break;
            }
        }
    }
}
