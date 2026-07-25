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

    /// Print latency and token stats for this run (overrides config)
    #[arg(long)]
    pub stats: bool,
}

impl Cli {
    /// The query words joined into a single prompt string.
    pub fn query(&self) -> String {
        self.query.join(" ")
    }
}
