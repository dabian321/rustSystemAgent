use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_iterations: usize,
    pub data_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        // Try loading .env from the project root (next to the binary or workspace)
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));

        // Try multiple .env locations
        let env_paths = [
            Some(PathBuf::from("/home/yzb/lanchainAgent/.env")),
            exe_dir.map(|d| d.join(".env")),
            Some(PathBuf::from(".env")),
        ];

        for path in env_paths.iter().flatten() {
            if path.exists() {
                let _ = dotenvy::from_path(path);
                break;
            }
        }

        let api_key = std::env::var("OPENROUTER_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .map_err(|_| "Missing API key: set OPENROUTER_API_KEY or OPENAI_API_KEY".to_string())?;

        let base_url = if std::env::var("OPENROUTER_API_KEY").is_ok() {
            "https://openrouter.ai/api/v1".to_string()
        } else {
            std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
        };

        let model = std::env::var("MODEL_NAME")
            .unwrap_or_else(|_| "google/gemini-2.5-flash".to_string());

        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rust-system-agent");

        std::fs::create_dir_all(&data_dir).ok();

        Ok(Config {
            api_key,
            base_url,
            model,
            temperature: 0.0,
            max_iterations: 15,
            data_dir,
        })
    }
}
