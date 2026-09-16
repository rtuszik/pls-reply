/// Ask an LLM for a quick terminal answer and print it.
#[derive(usage::Cli, Debug)]
#[usage(
    bin = "pls",
    version,
    unknown_flags = "error",
    args_override_self = false
)]
pub struct Cli {
    /// The question, e.g. `pls git command to show first commit`.
    /// If omitted, the query is read from stdin (or an interactive prompt).
    #[usage(trailing_var_arg)]
    pub query: Vec<String>,

    /// Override the model name from the config for this run
    #[usage(short = 'm', long)]
    pub model: Option<String>,

    /// Don't copy the answer to the clipboard
    #[usage(long)]
    pub no_copy: bool,

    /// Print a detailed latency profile to stderr
    #[usage(long, conflicts("--profile-json"))]
    pub profile: bool,

    /// Print a JSON latency profile to stderr
    #[usage(long)]
    pub profile_json: bool,

    /// Print latency and token stats for this run (overrides config)
    #[usage(long)]
    pub stats: bool,

    /// Print one JSON statistics record to stderr instead of human-readable stats
    #[usage(long, conflicts("--stats"))]
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
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn profile_is_independent_of_stats() {
        for profile in ["--profile", "--profile-json"] {
            for stats in ["--stats", "--stats-json"] {
                let argv = [OsStr::new(profile), OsStr::new(stats), OsStr::new("hello")];
                assert!(Cli::parse_from(&argv).is_ok());
            }
        }
        let argv = [
            OsStr::new("--profile"),
            OsStr::new("--profile-json"),
            OsStr::new("hello"),
        ];
        assert!(Cli::parse_from(&argv).is_err());
    }

    #[test]
    fn stats_formats_conflict() {
        let argv = [
            OsStr::new("--stats"),
            OsStr::new("--stats-json"),
            OsStr::new("hello"),
        ];
        assert!(Cli::parse_from(&argv).is_err());
    }

    #[test]
    fn trailing_query_keeps_option_looking_words() {
        let argv = [
            OsStr::new("--no-copy"),
            OsStr::new("explain"),
            OsStr::new("--help"),
        ];
        let cli = Cli::parse_from(&argv).unwrap();

        assert!(cli.no_copy);
        assert_eq!(cli.query, ["explain", "--help"]);
    }

    #[test]
    fn unknown_flags_before_the_query_are_rejected() {
        let argv = [OsStr::new("--definitely-invalid")];
        assert!(Cli::parse_from(&argv).is_err());
    }
}
