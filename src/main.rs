mod cli;
mod clipboard;
mod config;
mod llm;

use std::io::{self, BufRead, IsTerminal, Read, Write};

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load()?;

    let model_name = cli.model.as_deref().unwrap_or(&config.model.name);
    let query = resolve_query(&cli)?;

    let answer = llm::ask(&config, model_name, &query, os_name()).await?;

    if config.output.copy && !cli.no_copy && !answer.is_empty() {
        clipboard::copy(&answer);
    }

    Ok(())
}
