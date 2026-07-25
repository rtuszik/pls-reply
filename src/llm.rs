use anstyle::{AnsiColor, Color, Effects, Style};
use anyhow::{Result, bail};
use futures::StreamExt;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatStreamEvent, ReasoningEffort, Usage};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use std::io::Write;
use std::time::{Duration, Instant};

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
    let api_key = model.api_key();
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
            // A literal key in the config wins over the env-var name; either
            // overrides the adapter's default env lookup.
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

/// Query the model, streaming the answer to stdout as it arrives, and return the
/// full accumulated text for downstream use (e.g. clipboard). When `stats` is
/// set, a latency / token-throughput line is printed to stderr after the answer.
pub async fn ask(
    config: &Config,
    model_name: &str,
    query: &str,
    os: &str,
    stats: bool,
) -> Result<String> {
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
    if let Some(effort) = &config.params.reasoning_effort {
        let effort = effort
            .parse::<ReasoningEffort>()
            .map_err(|_| anyhow::anyhow!("invalid reasoning_effort '{effort}'"))?;
        options = options.with_reasoning_effort(effort);
    }
    // Token usage is only collected during streaming when explicitly captured.
    if stats {
        options = options.with_capture_usage(true);
    }

    let start = Instant::now();
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
                first_token.get_or_insert_with(Instant::now);
                print!("{}", chunk.content);
                let _ = stdout.flush();
                full.push_str(&chunk.content);
            }
            // Usage arrives on the terminal event, only when capture is enabled.
            ChatStreamEvent::End(end) => usage = end.captured_usage,
            _ => {}
        }
    }
    println!();

    if stats {
        let ttft = first_token.map(|t| t.duration_since(start));
        print_stats(start.elapsed(), ttft, usage.as_ref());
    }

    Ok(full.trim().to_string())
}

/// Print a dim `latency · tokens · throughput` line to stderr, with the numbers
/// accented. `anstream` strips the styling automatically when stderr is not a
/// terminal or `NO_COLOR` is set.
fn print_stats(elapsed: Duration, ttft: Option<Duration>, usage: Option<&Usage>) {
    anstream::eprintln!("{}", format_stats(elapsed, ttft, usage));
}

/// Build the styled stats line. Numbers are cyan, units/separators dim. The ANSI
/// codes are always emitted here; stripping for non-terminals is left to the
/// writer (`anstream`). Kept pure and separate from I/O so it can be tested.
fn format_stats(elapsed: Duration, ttft: Option<Duration>, usage: Option<&Usage>) -> String {
    const NUM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
    const DIM: Style = Style::new().effects(Effects::DIMMED);
    let (n, nr) = (NUM.render(), NUM.render_reset());
    let (d, dr) = (DIM.render(), DIM.render_reset());

    let mut line = format!("{n}{:.2}{nr}{d}s{dr}", elapsed.as_secs_f64());

    if let Some(tokens) = usage.and_then(|u| u.completion_tokens) {
        line += &format!("{d} · {dr}{n}{tokens}{nr}{d} tok{dr}");

        // Throughput over the generation window (excludes time-to-first-token).
        let gen_secs = ttft
            .map(|t| elapsed.saturating_sub(t))
            .unwrap_or(elapsed)
            .as_secs_f64();
        if gen_secs > 0.0 {
            let tps = f64::from(tokens) / gen_secs;
            line += &format!("{d} · {dr}{n}{tps:.0}{nr}{d} tok/s{dr}");
        }
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Usage with the given completion token count; other fields default/None.
    fn usage_with(completion_tokens: i32) -> Usage {
        Usage {
            completion_tokens: Some(completion_tokens),
            ..Default::default()
        }
    }

    /// The plain-text form is what a non-terminal / `NO_COLOR` writer emits,
    /// since `anstream` strips styling with this same adapter.
    fn plain(line: &str) -> String {
        anstream::adapter::strip_str(line).to_string()
    }

    #[test]
    fn full_line_strips_to_plain_text() {
        // 284 tokens over a 1.24s generation window (1.34s total - 0.10s ttft)
        // -> 284 / 1.24 = 229.03 -> "229" tok/s.
        let line = format_stats(
            Duration::from_millis(1340),
            Some(Duration::from_millis(100)),
            Some(&usage_with(284)),
        );
        assert_eq!(plain(&line), "1.34s · 284 tok · 229 tok/s");
    }

    #[test]
    fn styling_present_before_strip() {
        let line = format_stats(
            Duration::from_millis(1340),
            Some(Duration::from_millis(100)),
            Some(&usage_with(284)),
        );
        // Raw line carries ANSI escapes (cyan = SGR 36); stripping removes them.
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
            Some(Duration::from_millis(100)),
            None,
        );
        assert_eq!(plain(&line), "1.34s");
    }

    #[test]
    fn omits_throughput_for_zero_generation_window() {
        // ttft == elapsed leaves a zero-length generation window: no tok/s, but
        // the token count still shows.
        let d = Duration::from_millis(500);
        let line = format_stats(d, Some(d), Some(&usage_with(42)));
        assert_eq!(plain(&line), "0.50s · 42 tok");
    }

    #[test]
    fn throughput_uses_total_when_ttft_unknown() {
        // No first-token timestamp -> fall back to total elapsed: 200 / 2.00 = 100.
        let line = format_stats(Duration::from_secs(2), None, Some(&usage_with(200)));
        assert_eq!(plain(&line), "2.00s · 200 tok · 100 tok/s");
    }
}
