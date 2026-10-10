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

enum Failure {
    BeforeOutput(anyhow::Error),
    AfterOutput(anyhow::Error),
}

struct Answer {
    text: String,
    usage: Option<Usage>,
    request_start: Instant,
    request_elapsed: Duration,
    first_token: Option<Instant>,
}

fn chat_options(config: &Config, stats: Stats) -> Result<ChatOptions> {
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
    Ok(options)
}

fn candidates<'a>(config: &'a Config, model_name: &'a str) -> Vec<(&'a ModelConfig, &'a str)> {
    std::iter::once((&config.model, model_name))
        .chain(config.fallback.iter().map(|m| (m, m.name.as_str())))
        .collect()
}

async fn attempt(
    model: &ModelConfig,
    model_name: &str,
    chat_req: ChatRequest,
    options: &ChatOptions,
    profile: &mut crate::profile::Profile,
) -> Result<Answer, Failure> {
    let kind = adapter_kind(&model.provider).map_err(Failure::BeforeOutput)?;
    let iden = ModelIden::new(kind, model_name.to_string());
    let client = build_client(model);

    profile.enter("request to first content");
    let request_start = Instant::now();
    let response = client
        .exec_chat_stream(iden, chat_req, Some(options))
        .await
        .map_err(|e| Failure::BeforeOutput(e.into()))?;

    let mut stream = response.stream;
    let mut full = String::new();
    let mut stdout = std::io::stdout();
    let mut first_token: Option<Instant> = None;
    let mut usage: Option<Usage> = None;

    while let Some(event) = stream.next().await {
        let event = event.map_err(|e| {
            if first_token.is_none() {
                Failure::BeforeOutput(e.into())
            } else {
                Failure::AfterOutput(e.into())
            }
        })?;
        match event {
            ChatStreamEvent::Chunk(chunk) => {
                if !chunk.content.is_empty() {
                    profile.content();
                }
                let output_start = profile.clock();
                let flushed = write!(stdout, "{}", chunk.content).and_then(|()| stdout.flush());
                profile.flushed(output_start, !chunk.content.is_empty() && flushed.is_ok());
                flushed.map_err(|e| Failure::AfterOutput(e.into()))?;
                if !chunk.content.is_empty() {
                    first_token.get_or_insert_with(Instant::now);
                }
                full.push_str(&chunk.content);
            }
            ChatStreamEvent::End(end) => usage = end.captured_usage,
            _ => {}
        }
    }

    Ok(Answer {
        text: full,
        usage,
        request_start,
        request_elapsed: request_start.elapsed(),
        first_token,
    })
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
    let system = prompt.system.replace("{os}", os);
    let chat_req = ChatRequest::new(vec![
        ChatMessage::system(system),
        ChatMessage::user(prompt.user),
    ]);
    let options = chat_options(config, stats)?;

    let candidates = candidates(config, model_name);
    let mut last_error = None;
    let mut answer = None;
    for (i, &(model, name)) in candidates.iter().enumerate() {
        if let Some(error) = last_error.take() {
            let (prev_model, prev_name) = candidates[i - 1];
            print_fallback(
                &prev_model.provider,
                prev_name,
                &error,
                &model.provider,
                name,
            );
        }
        match attempt(model, name, chat_req.clone(), &options, profile).await {
            Ok(a) => {
                answer = Some((model, name, a));
                break;
            }
            Err(Failure::BeforeOutput(e)) => last_error = Some(e),
            Err(Failure::AfterOutput(e)) => return Err(e),
        }
    }
    let Some((model, model_name, answer)) = answer else {
        return Err(last_error.expect("at least one model is configured"));
    };

    profile.stream_finished();
    writeln!(std::io::stdout())?;

    let ttft = answer.first_token.map(|t| t.duration_since(start));
    let usage = answer.usage.as_ref();
    match stats {
        Stats::Off => {}
        Stats::Human => print_stats(start.elapsed(), answer.request_elapsed, usage),
        Stats::Json => eprintln!(
            "{}",
            serde_json::json!({
                "type": "pls_stats",
                "schema_version": 1,
                "provider": model.provider,
                "model": model_name,
                "elapsed_ms": start.elapsed().as_secs_f64() * 1000.0,
                "ttft_ms": ttft.map(|d| d.as_secs_f64() * 1000.0),
                "request_ttft_ms": answer.first_token.map(|t| t.duration_since(answer.request_start).as_secs_f64() * 1000.0),
                "input_tokens": usage.and_then(|u| u.prompt_tokens),
                "cached_input_tokens": usage.and_then(|u| u.prompt_tokens_details.as_ref()).and_then(|d| d.cached_tokens),
                "output_tokens": usage.and_then(|u| u.completion_tokens),
                "params": {
                    "temperature": config.params.temperature,
                    "max_tokens": config.params.max_tokens,
                    "reasoning_effort": config.params.reasoning_effort,
                },
            })
        ),
    }

    Ok(answer.text.trim().to_string())
}

fn print_fallback(
    provider: &str,
    model: &str,
    error: &anyhow::Error,
    next_provider: &str,
    next_model: &str,
) {
    const WARN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
    let (w, wr) = (WARN.render(), WARN.render_reset());
    anstream::eprintln!(
        "{w}warning:{wr} {provider}/{model} failed: {error:#}\n{w}falling back to{wr} {next_provider}/{next_model}"
    );
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

    fn config_with_fallbacks(fallbacks: &str) -> Config {
        let text = format!(
            "[model]\nprovider = \"openai\"\nname = \"primary\"\n\n[prompt]\nsystem = \"s\"\n{fallbacks}"
        );
        toml::from_str(&text).unwrap()
    }

    #[test]
    fn candidates_without_fallbacks_is_primary_only() {
        let config = config_with_fallbacks("");
        let names: Vec<_> = candidates(&config, "primary").iter().map(|c| c.1).collect();
        assert_eq!(names, ["primary"]);
    }

    #[test]
    fn candidates_try_primary_then_fallbacks_in_order() {
        let config = config_with_fallbacks(
            "[[fallback]]\nprovider = \"anthropic\"\nname = \"a\"\n[[fallback]]\nprovider = \"groq\"\nname = \"b\"\n",
        );
        let list = candidates(&config, "override");
        let got: Vec<_> = list
            .iter()
            .map(|(m, n)| (m.provider.as_str(), *n))
            .collect();
        assert_eq!(
            got,
            [("openai", "override"), ("anthropic", "a"), ("groq", "b")]
        );
    }

    #[tokio::test]
    async fn unknown_fallback_provider_fails_before_output() {
        let config =
            config_with_fallbacks("[[fallback]]\nprovider = \"not-a-provider\"\nname = \"x\"\n");
        let mut profile = crate::profile::Profile::default();
        let result = attempt(
            &config.fallback[0],
            "x",
            ChatRequest::new(vec![]),
            &ChatOptions::default(),
            &mut profile,
        )
        .await;
        assert!(matches!(result, Err(Failure::BeforeOutput(_))));
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
