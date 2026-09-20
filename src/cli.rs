#[derive(usage::Cli, Debug)]
#[usage(
    bin = "pls",
    version,
    unknown_flags = "error",
    args_override_self = false
)]
pub struct Cli {
    #[usage(subcommand)]
    pub command: Option<Command>,

    #[usage(short = 'm', long, global)]
    pub model: Option<String>,

    #[usage(long, global)]
    pub no_copy: bool,

    #[usage(long, global, conflicts("--profile-json"))]
    pub profile: bool,

    #[usage(long, global)]
    pub profile_json: bool,

    #[usage(long, global)]
    pub stats: bool,

    #[usage(long, global, conflicts("--stats"))]
    pub stats_json: bool,
}

#[derive(usage::Subcommands, Debug, PartialEq, Eq)]
pub enum Command {
    Ask,

    Commit,
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn bare_invocation_selects_implicit_ask() {
        let cli = Cli::parse_from(&[]).unwrap();
        assert_eq!(cli.command, None);
    }

    #[test]
    fn parses_commands() {
        let ask = Cli::parse_from(&[OsStr::new("ask")]).unwrap();
        assert_eq!(ask.command, Some(Command::Ask));

        let commit = Cli::parse_from(&[OsStr::new("commit")]).unwrap();
        assert_eq!(commit.command, Some(Command::Commit));
    }

    #[test]
    fn arbitrary_root_words_are_rejected() {
        let argv = [OsStr::new("git"), OsStr::new("show")];
        assert!(Cli::parse_from(&argv).is_err());
    }

    #[test]
    fn commands_reject_query_arguments() {
        let argv = [OsStr::new("ask"), OsStr::new("git"), OsStr::new("show")];
        assert!(Cli::parse_from(&argv).is_err());
    }

    #[test]
    fn global_flags_work_before_and_after_commands() {
        for argv in [
            [OsStr::new("--no-copy"), OsStr::new("commit")],
            [OsStr::new("commit"), OsStr::new("--no-copy")],
        ] {
            let cli = Cli::parse_from(&argv).unwrap();
            assert!(cli.no_copy);
            assert_eq!(cli.command, Some(Command::Commit));
        }
    }

    #[test]
    fn profile_is_independent_of_stats() {
        for profile in ["--profile", "--profile-json"] {
            for stats in ["--stats", "--stats-json"] {
                let argv = [OsStr::new(profile), OsStr::new(stats), OsStr::new("ask")];
                assert!(Cli::parse_from(&argv).is_ok());
            }
        }
        let argv = [
            OsStr::new("--profile"),
            OsStr::new("--profile-json"),
            OsStr::new("ask"),
        ];
        assert!(Cli::parse_from(&argv).is_err());
    }

    #[test]
    fn stats_formats_conflict() {
        let argv = [
            OsStr::new("--stats"),
            OsStr::new("--stats-json"),
            OsStr::new("ask"),
        ];
        assert!(Cli::parse_from(&argv).is_err());
    }

    #[test]
    fn unknown_flags_are_rejected() {
        let argv = [OsStr::new("--definitely-invalid")];
        assert!(Cli::parse_from(&argv).is_err());
    }
}
