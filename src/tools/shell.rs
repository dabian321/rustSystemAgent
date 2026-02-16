use colored::Colorize;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// Execute a shell command and return its output.
pub async fn execute_shell(input: &str) -> String {
    let command = input.trim();
    if command.is_empty() {
        return "Error: empty command".to_string();
    }

    eprintln!("{}", format!("\n> {command}").cyan());
    eprintln!("{}", "-".repeat(50).dimmed());

    let result = Command::new("/bin/bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match result {
        Ok(c) => c,
        Err(e) => return format!("Failed to spawn process: {e}"),
    };

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_end(&mut stdout_buf).await;
    }
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_end(&mut stderr_buf).await;
    }

    let status = match child.wait().await {
        Ok(s) => s,
        Err(e) => return format!("Failed to wait for process: {e}"),
    };

    let stdout_str = String::from_utf8_lossy(&stdout_buf);
    let stderr_str = String::from_utf8_lossy(&stderr_buf);

    // Print to terminal in real time
    if !stdout_str.is_empty() {
        eprint!("{stdout_str}");
    }
    if !stderr_str.is_empty() {
        eprint!("{}", stderr_str.yellow());
    }

    eprintln!("{}", "-".repeat(50).dimmed());

    let code = status.code().unwrap_or(-1);
    if code == 0 {
        eprintln!("{}", "Command succeeded".green());
        // Truncate very long outputs
        let out = stdout_str.to_string();
        if out.len() > 8000 {
            format!(
                "Command succeeded. Output (truncated):\n{}...\n[{} bytes total]",
                &out[..8000],
                out.len()
            )
        } else {
            format!("Command succeeded. Output:\n{out}")
        }
    } else {
        eprintln!("{}", format!("Command failed (exit code {code})").red());
        format!(
            "Command failed (exit code {code}). stdout:\n{stdout_str}\nstderr:\n{stderr_str}"
        )
    }
}
