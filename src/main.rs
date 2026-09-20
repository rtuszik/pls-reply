mod cli;
mod clipboard;
mod commit;
mod config;
mod llm;
mod profile;

use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Instant;

use anyhow::{Result, bail};

use cli::{Cli, Command};

fn os_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unix"
    }
}

fn resolve_query() -> Result<String> {
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();
    if interactive {
        eprint!("ask> ");
        io::stderr().flush().ok();
    }
    read_query(stdin.lock(), interactive)
}

fn read_query(mut input: impl BufRead, single_line: bool) -> Result<String> {
    let query = if single_line {
        let mut line = String::new();
        input.read_line(&mut line)?;
        line
    } else {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        buf
    };

    let query = query.trim().to_string();
    if query.is_empty() {
        bail!("no query provided (pipe it in or type it at the prompt)");
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
    let (system_prompt, query) = match cli.command.as_ref() {
        None | Some(Command::Ask) => {
            profile.enter("input wait");
            (config.prompt.system.as_str(), resolve_query()?)
        }
        Some(Command::Commit) => {
            profile.enter("staged diff");
            (
                config.prompt.commit.as_str(),
                commit::staged_changes_prompt()?,
            )
        }
    };
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
        llm::Prompt {
            system: system_prompt,
            user: &query,
        },
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn piped_query_reads_all_lines_verbatim() {
        let input = "explain `git show`\nand $HOME\n";
        assert_eq!(read_query(Cursor::new(input), false).unwrap(), input.trim());
    }

    #[test]
    fn interactive_query_reads_one_line() {
        let input = "first question\nsecond question\n";
        assert_eq!(
            read_query(Cursor::new(input), true).unwrap(),
            "first question"
        );
    }

    #[test]
    fn empty_query_is_rejected() {
        let error = read_query(Cursor::new(" \n"), false).unwrap_err();
        assert_eq!(
            error.to_string(),
            "no query provided (pipe it in or type it at the prompt)"
        );
    }
}
