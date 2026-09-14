mod cli;
mod clipboard;
mod config;
mod llm;
mod profile;

use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::time::Instant;

use anyhow::{Result, bail};
use clap::Parser;

use cli::Cli;

/// The OS name substituted into `{os}` in the system prompt.
fn os_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unix"
    }
}

/// Resolve the query from args, falling back to stdin. Reading from stdin lets
/// the query contain shell metacharacters (backticks, `$`, quotes) that the
/// shell would otherwise expand before they reach argv.
fn resolve_query(cli: &Cli) -> Result<String> {
    if !cli.query.is_empty() {
        return Ok(cli.query());
    }

    let stdin = io::stdin();
    let query = if stdin.is_terminal() {
        eprint!("ask> ");
        io::stderr().flush().ok();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        line
    } else {
        let mut buf = String::new();
        stdin.lock().read_to_string(&mut buf)?;
        buf
    };

    let query = query.trim().to_string();
    if query.is_empty() {
        bail!("no query provided (pass it as arguments, pipe it in, or type it at the prompt)");
    }
    Ok(query)
}

fn main() -> Result<()> {
    let start = Instant::now();
    let cli = Cli::parse();
    let mut profile = profile::Profile::new(cli.profile || cli.profile_json, start);
    profile.enter("runtime setup");
    let result = (|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(run(&cli, &mut profile))
    })();
    profile.report(result.is_ok(), cli.profile_json);
    result
}

async fn run(cli: &Cli, profile: &mut profile::Profile) -> Result<()> {
    profile.enter("config");
    let config = config::load()?;

    let model_name = cli.model.as_deref().unwrap_or(&config.model.name);
    profile.enter(if cli.query.is_empty() {
        "input wait"
    } else {
        "query arguments"
    });
    let query = resolve_query(cli)?;
    let start = Instant::now();

    let stats = if cli.stats_json {
        llm::Stats::Json
    } else if config.output.stats || cli.stats {
        llm::Stats::Human
    } else {
        llm::Stats::Off
    };
    profile.enter("request preparation");
    let answer = llm::ask(
        &config,
        model_name,
        &query,
        os_name(),
        stats,
        start,
        profile,
    )
    .await?;

    if config.output.copy && !cli.no_copy && !answer.is_empty() {
        profile.enter("clipboard");
        clipboard::copy(&answer);
    }

    profile.enter("cleanup");
    Ok(())
}
