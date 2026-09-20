use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("../pls.example.toml");

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
    #[serde(default)]
    api_key: Option<String>,
}

impl ModelConfig {
    pub fn base_url(&self) -> Option<String> {
        non_empty(self.base_url.as_deref())
    }

    pub fn api_key_env(&self) -> Option<String> {
        non_empty(self.api_key_env.as_deref())
    }

    pub fn api_key(&self) -> Option<String> {
        non_empty(self.api_key.as_deref())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Params {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    #[serde(default = "default_true")]
    pub copy: bool,
    #[serde(default)]
    pub stats: bool,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            copy: true,
            stats: false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Prompt {
    pub system: String,
    #[serde(default = "default_commit_prompt")]
    pub commit: String,
}

fn default_true() -> bool {
    true
}

fn default_commit_prompt() -> String {
    crate::commit::DEFAULT_SYSTEM_PROMPT.to_owned()
}

fn non_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

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

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        write_default_config(&path)
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

fn write_default_config(path: &Path) -> Result<()> {
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(path)?;
    file.write_all(DEFAULT_CONFIG.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_CONFIG: &str = r#"
[model]
provider = "openai"
name = "test"

[prompt]
system = "ask prompt"
"#;

    #[test]
    fn existing_config_uses_the_default_commit_prompt() {
        let config: Config = toml::from_str(BASE_CONFIG).unwrap();
        assert_eq!(config.prompt.commit, crate::commit::DEFAULT_SYSTEM_PROMPT);
    }

    #[test]
    fn custom_commit_prompt_is_loaded() {
        let text = format!("{BASE_CONFIG}commit = \"custom commit prompt\"\n");
        let config: Config = toml::from_str(&text).unwrap();
        assert_eq!(config.prompt.commit, "custom commit prompt");
    }

    #[test]
    fn example_config_matches_the_default_commit_prompt() {
        let config: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(config.prompt.commit, crate::commit::DEFAULT_SYSTEM_PROMPT);
    }
}
