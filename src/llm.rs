use anstyle::{AnsiColor, Color, Effects, Style};
use anyhow::{Result, anyhow};
use futures::StreamExt;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatStreamEvent, ReasoningEffort, Usage};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use std::io::Write;
use std::time::{Duration, Instant};

use crate::config::{Config, ModelConfig};

#[derive(Clone, Copy, PartialEq)]
pub enum Stats {
    Off,
    Human,
    Json,
}

pub struct Prompt<'a> {
    pub system: &'a str,
    pub user: &'a str,
}

fn adapter_kind(provider: &str) -> Result<AdapterKind> {
    let provider = provider.to_ascii_lowercase();
    if provider == "custom" {
        return Ok(AdapterKind::OpenAI);
    }

    AdapterKind::from_lower_str(&provider).ok_or_else(|| anyhow!("unknown provider '{provider}'"))
}

fn build_client(model: &ModelConfig) -> Client {
    let base_url = model.base_url();
    let api_key = model.api_key();
    let api_key_env = model.api_key_env();

    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |mut target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            if let Some(url) = &base_url {
                let url = if url.ends_with('/') {
                    url.clone()
                } else {
                    format!("{url}/")
                };
                target.endpoint = Endpoint::from_owned(url);
            }
            if let Some(key) = &api_key {
                target.auth = AuthData::from_single(key.clone());
            } else if let Some(env) = &api_key_env {
                target.auth = AuthData::from_env(env.clone());
            }
            Ok(target)
        },
    );

    Client::builder()
        .with_service_target_resolver(resolver)
        .build()
}

pub async fn ask(
    config: &Config,
    model_name: &str,
    prompt: Prompt<'_>,
    os: &str,
    stats: Stats,
    start: Instant,
    profile: &mut crate::profile::Profile,
) -> Result<String> {
    let kind = adapter_kind(&config.model.provider)?;
    let model = ModelIden::new(kind, model_name.to_string());
    let client = build_client(&config.model);

    let system = prompt.system.replace("{os}", os);
    let chat_req = ChatRequest::new(vec![
        ChatMessage::system(system),
        ChatMessage::user(prompt.user),
    ]);

    let mut options = ChatOptions::default();
    if let Some(t) = config.params.temperature {
        options = options.with_temperature(t);
    }
    if let Some(m) = config.params.max_tokens {
        options = options.with_max_tokens(m);
    }
    if let Some(effort) = &config.params.reasoning_effort {
        let effort = effort
            .parse::<ReasoningEffort>()
            .map_err(|_| anyhow::anyhow!("invalid reasoning_effort '{effort}'"))?;
        options = options.with_reasoning_effort(effort);
    }
    if stats != Stats::Off {
        options = options.with_capture_usage(true);
    }

    profile.enter("request to first content");
    let request_start = Instant::now();
    let response = client
        .exec_chat_stream(model, chat_req, Some(&options))
        .await?;

    let mut stream = response.stream;
    let mut full = String::new();
    let mut stdout = std::io::stdout();
    let mut first_token: Option<Instant> = None;
    let mut usage: Option<Usage> = None;

    while let Some(event) = stream.next().await {
        match event? {
            ChatStreamEvent::Chunk(chunk) => {
                if !chunk.content.is_empty() {
                    profile.content();
                }
                let output_start = profile.clock();
                let flushed = write!(stdout, "{}", chunk.content).and_then(|()| stdout.flush());
                profile.flushed(output_start, !chunk.content.is_empty() && flushed.is_ok());
                flushed?;
                if !chunk.content.is_empty() {
                    first_token.get_or_insert_with(Instant::now);
                }
                full.push_str(&chunk.content);
            }
            ChatStreamEvent::End(end) => usage = end.captured_usage,
            _ => {}
        }
    }
    let request_elapsed = request_start.elapsed();
    profile.stream_finished();
    writeln!(stdout)?;

    let ttft = first_token.map(|t| t.duration_since(start));
    match stats {
        Stats::Off => {}
        Stats::Human => print_stats(start.elapsed(), request_elapsed, usage.as_ref()),
        Stats::Json => eprintln!(
            "{}",
            serde_json::json!({
                "type": "pls_stats",
                "schema_version": 1,
                "provider": config.model.provider,
                "model": model_name,
                "elapsed_ms": start.elapsed().as_secs_f64() * 1000.0,
                "ttft_ms": ttft.map(|d| d.as_secs_f64() * 1000.0),
                "request_ttft_ms": first_token.map(|t| t.duration_since(request_start).as_secs_f64() * 1000.0),
                "input_tokens": usage.as_ref().and_then(|u| u.prompt_tokens),
                "cached_input_tokens": usage.as_ref().and_then(|u| u.prompt_tokens_details.as_ref()).and_then(|d| d.cached_tokens),
                "output_tokens": usage.as_ref().and_then(|u| u.completion_tokens),
                "params": {
                    "temperature": config.params.temperature,
                    "max_tokens": config.params.max_tokens,
                    "reasoning_effort": config.params.reasoning_effort,
                },
            })
        ),
    }

    Ok(full.trim().to_string())
}

fn print_stats(elapsed: Duration, request_elapsed: Duration, usage: Option<&Usage>) {
    anstream::eprintln!("{}", format_stats(elapsed, request_elapsed, usage));
}

fn format_stats(elapsed: Duration, request_elapsed: Duration, usage: Option<&Usage>) -> String {
    const NUM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
    const DIM: Style = Style::new().effects(Effects::DIMMED);
    let (n, nr) = (NUM.render(), NUM.render_reset());
    let (d, dr) = (DIM.render(), DIM.render_reset());

    let mut line = format!("{n}{:.2}{nr}{d}s{dr}", elapsed.as_secs_f64());

    if let Some(tokens) = usage.and_then(|u| u.completion_tokens) {
        line += &format!("{d} · {dr}{n}{tokens}{nr}{d} tok{dr}");

        let request_secs = request_elapsed.as_secs_f64();
        if request_secs > 0.0 {
            let tps = f64::from(tokens) / request_secs;
            line += &format!("{d} · {dr}{n}{tps:.0}{nr}{d} effective tok/s{dr}");
        }
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_adapters_from_genai_registry() {
        assert_eq!(
            adapter_kind("open_router").unwrap(),
            AdapterKind::OpenRouter
        );
    }

    #[test]
    fn provider_names_are_case_insensitive() {
        assert_eq!(adapter_kind("OpenAI").unwrap(), AdapterKind::OpenAI);
    }

    #[test]
    fn custom_uses_openai_adapter() {
        assert_eq!(adapter_kind("custom").unwrap(), AdapterKind::OpenAI);
    }

    #[test]
    fn rejects_unknown_provider() {
        let error = adapter_kind("not-a-provider").unwrap_err();
        assert_eq!(error.to_string(), "unknown provider 'not-a-provider'");
    }

    fn usage_with(completion_tokens: i32) -> Usage {
        Usage {
            completion_tokens: Some(completion_tokens),
            ..Default::default()
        }
    }

    fn plain(line: &str) -> String {
        anstream::adapter::strip_str(line).to_string()
    }

    #[test]
    fn full_line_strips_to_plain_text() {
        let line = format_stats(
            Duration::from_millis(1340),
            Duration::from_millis(1240),
            Some(&usage_with(284)),
        );
        assert_eq!(plain(&line), "1.34s · 284 tok · 229 effective tok/s");
    }

    #[test]
    fn styling_present_before_strip() {
        let line = format_stats(
            Duration::from_millis(1340),
            Duration::from_millis(1240),
            Some(&usage_with(284)),
        );
        assert!(
            line.contains('\u{1b}'),
            "expected ANSI escapes in styled line"
        );
        assert!(line.contains("36"), "expected cyan foreground code");
        assert!(
            !plain(&line).contains('\u{1b}'),
            "stripped line must be clean"
        );
    }

    #[test]
    fn latency_only_when_usage_missing() {
        let line = format_stats(
            Duration::from_millis(1340),
            Duration::from_millis(1240),
            None,
        );
        assert_eq!(plain(&line), "1.34s");
    }

    #[test]
    fn omits_throughput_for_zero_request_duration() {
        let line = format_stats(
            Duration::from_millis(500),
            Duration::ZERO,
            Some(&usage_with(42)),
        );
        assert_eq!(plain(&line), "0.50s · 42 tok");
    }

    #[test]
    fn throughput_includes_wait_for_first_content() {
        let line = format_stats(
            Duration::from_millis(856),
            Duration::from_millis(850),
            Some(&usage_with(350)),
        );
        assert_eq!(plain(&line), "0.86s · 350 tok · 412 effective tok/s");
    }
}
