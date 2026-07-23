use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

/// The default config written on first run, also used as the template shown to
/// the user. Keep it in sync with the `Config` struct below.
const DEFAULT_CONFIG: &str = r#"[model]
provider    = "openai"            # openai | anthropic | gemini | groq | ollama | xai | deepseek | cohere | custom
name        = "gpt-5.6-luna"
base_url    = ""                  # optional; required for provider = "custom", else overrides the adapter default
api_key_env = "OPENAI_API_KEY"    # optional; env var the API key is read from

[params]
# temperature = 0.2               # optional; some reasoning models (e.g. gpt-5.x) only accept the default
# max_tokens = 512                # optional

[output]
copy = true                       # copy the answer to the clipboard (pbcopy / wl-copy / xclip)

[prompt]
system = """
You are a terminal assistant running on a {os} unix system.
Reply with only the single most direct shell command or answer.
No explanation, no markdown fences, no preamble.
"""
"#;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    #[serde(default)]
    pub params: Params,
    #[serde(default)]
    pub output: Output,
    pub prompt: Prompt,
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub name: String,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    api_key_env: Option<String>,
}

impl ModelConfig {
    /// Base URL, treating an empty string as unset.
    pub fn base_url(&self) -> Option<String> {
        non_empty(self.base_url.as_deref())
    }

    /// API-key env var name, treating an empty string as unset.
    pub fn api_key_env(&self) -> Option<String> {
        non_empty(self.api_key_env.as_deref())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Params {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    #[serde(default = "default_true")]
    pub copy: bool,
}

impl Default for Output {
    fn default() -> Self {
        Self { copy: true }
    }
}

#[derive(Debug, Deserialize)]
pub struct Prompt {
    pub system: String,
}

fn default_true() -> bool {
    true
}

fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// `$XDG_CONFIG_HOME/pls/pls.toml`, falling back to `~/.config/pls/pls.toml`.
pub fn config_path() -> Result<PathBuf> {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            let home = env::var_os("HOME").context("neither XDG_CONFIG_HOME nor HOME is set")?;
            PathBuf::from(home).join(".config")
        }
    };
    Ok(base.join("pls").join("pls.toml"))
}

/// Load the config, or write a default template and return an error prompting
/// the user to fill it in when none exists yet.
pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        fs::write(&path, DEFAULT_CONFIG)
            .with_context(|| format!("writing default config to {}", path.display()))?;
        anyhow::bail!(
            "wrote a default config to {}\nedit it and set the API key env var (e.g. export OPENAI_API_KEY=...), then re-run",
            path.display()
        );
    }

    let text =
        fs::read_to_string(&path).with_context(|| format!("reading config {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))
}
