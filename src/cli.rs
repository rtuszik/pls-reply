use clap::Parser;

/// Ask an LLM for a quick terminal answer and print it.
#[derive(Parser, Debug)]
#[command(name = "pls", version, about, disable_help_subcommand = true)]
pub struct Cli {
    /// The question, e.g. `pls git command to show first commit`.
    /// If omitted, the query is read from stdin (or an interactive prompt).
    #[arg(trailing_var_arg = true)]
    pub query: Vec<String>,

    /// Override the model name from the config for this run
    #[arg(short, long)]
    pub model: Option<String>,

    /// Don't copy the answer to the clipboard
    #[arg(long)]
    pub no_copy: bool,

    /// Print a detailed latency profile to stderr
    #[arg(long, conflicts_with = "profile_json")]
    pub profile: bool,

    /// Print a JSON latency profile to stderr
    #[arg(long)]
    pub profile_json: bool,

    /// Print latency and token stats for this run (overrides config)
    #[arg(long)]
    pub stats: bool,

    /// Print one JSON statistics record to stderr instead of human-readable stats
    #[arg(long, conflicts_with = "stats")]
    pub stats_json: bool,
}

impl Cli {
    /// The query words joined into a single prompt string.
    pub fn query(&self) -> String {
        self.query.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_is_independent_of_stats() {
        for profile in ["--profile", "--profile-json"] {
            for stats in ["--stats", "--stats-json"] {
                assert!(Cli::try_parse_from(["pls", profile, stats, "hello"]).is_ok());
            }
        }
        assert!(Cli::try_parse_from(["pls", "--profile", "--profile-json", "hello"]).is_err());
    }
}
