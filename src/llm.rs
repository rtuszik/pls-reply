use anyhow::{Result, bail};
use futures::StreamExt;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatStreamEvent};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use std::io::Write;

use crate::config::{Config, ModelConfig};

/// Map a config `provider` string to a genai adapter. `custom` is treated as an
/// OpenAI-compatible endpoint (requires `base_url`).
fn adapter_kind(provider: &str) -> Result<AdapterKind> {
    Ok(match provider.to_ascii_lowercase().as_str() {
        "openai" | "custom" => AdapterKind::OpenAI,
        "anthropic" => AdapterKind::Anthropic,
        "gemini" => AdapterKind::Gemini,
        "groq" => AdapterKind::Groq,
        "ollama" => AdapterKind::Ollama,
        "cohere" => AdapterKind::Cohere,
        "xai" => AdapterKind::Xai,
        "deepseek" => AdapterKind::DeepSeek,
        other => bail!(
            "unknown provider '{other}' (expected: openai, anthropic, gemini, groq, ollama, cohere, xai, deepseek, custom)"
        ),
    })
}

/// Build a client whose resolver applies the config's `base_url` / `api_key_env`
/// overrides on top of the adapter's defaults.
fn build_client(model: &ModelConfig) -> Client {
    let base_url = model.base_url();
    let api_key_env = model.api_key_env();

    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |mut target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            if let Some(url) = &base_url {
                // genai joins path suffixes with reqwest's URL join, which drops
                // the last segment of a base that lacks a trailing slash (e.g.
                // `.../v1` + `chat/completions` -> `.../chat/completions`). Ensure
                // the trailing slash so a configured `/v1` path is preserved.
                let url = if url.ends_with('/') {
                    url.clone()
                } else {
                    format!("{url}/")
                };
                target.endpoint = Endpoint::from_owned(url);
            }
            if let Some(env) = &api_key_env {
                target.auth = AuthData::from_env(env.clone());
            }
            Ok(target)
        },
    );

    Client::builder()
        .with_service_target_resolver(resolver)
        .build()
}

/// Query the model, streaming the answer to stdout as it arrives, and return the
/// full accumulated text for downstream use (e.g. clipboard).
pub async fn ask(config: &Config, model_name: &str, query: &str, os: &str) -> Result<String> {
    let kind = adapter_kind(&config.model.provider)?;
    let model = ModelIden::new(kind, model_name.to_string());
    let client = build_client(&config.model);

    let system = config.prompt.system.replace("{os}", os);
    let chat_req = ChatRequest::new(vec![ChatMessage::system(system), ChatMessage::user(query)]);

    let mut options = ChatOptions::default();
    if let Some(t) = config.params.temperature {
        options = options.with_temperature(t);
    }
    if let Some(m) = config.params.max_tokens {
        options = options.with_max_tokens(m);
    }

    let response = client
        .exec_chat_stream(model, chat_req, Some(&options))
        .await?;

    let mut stream = response.stream;
    let mut full = String::new();
    let mut stdout = std::io::stdout();

    while let Some(event) = stream.next().await {
        if let ChatStreamEvent::Chunk(chunk) = event? {
            print!("{}", chunk.content);
            let _ = stdout.flush();
            full.push_str(&chunk.content);
        }
    }
    println!();

    Ok(full.trim().to_string())
}
