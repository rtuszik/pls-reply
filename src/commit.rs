//! `pls commit`: wrap the staged Git diff in a prompt for a Conventional Commit message.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};

use crate::config::ScanConfig;

pub const DEFAULT_SYSTEM_PROMPT: &str = "\
You write Conventional Commit messages from staged Git changes.
Output only the commit message, with no Markdown fences or commentary.
Use the form type(scope): description, omitting the scope when it is not useful.
Use an imperative, lowercase description with no trailing period and keep the subject at most 72 characters.
Do not add a body to the messages.
Represent breaking changes with ! before the colon.
Do not invent changes, motivations, issue references, or breaking behavior.
";

const MAX_DIFF_BYTES: usize = 256 * 1024;
/// Match Git's standard compact context while ignoring user-specific diff settings.
const DIFF_CONTEXT_LINES: usize = 3;

/// Read the current repository's staged changes.
pub fn staged_diff() -> Result<Vec<u8>> {
    staged_diff_in(Path::new("."))
}

fn staged_diff_in(cwd: &Path) -> Result<Vec<u8>> {
    let unified = format!("--unified={DIFF_CONTEXT_LINES}");
    let output = Command::new("git")
        .current_dir(cwd)
        .args([
            "diff",
            "--cached",
            "--no-ext-diff",
            "--no-textconv",
            "--find-renames",
            "--stat",
            "--patch",
        ])
        .arg(unified)
        .output()
        .context("running git diff for staged changes")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("reading staged changes: {}", stderr.trim());
    }
    if output.stdout.is_empty() {
        bail!("no staged changes");
    }
    if output.stdout.len() > MAX_DIFF_BYTES {
        bail!(
            "staged diff is too large ({} KiB; limit is {} KiB); stage a smaller change",
            output.stdout.len().div_ceil(1024),
            MAX_DIFF_BYTES / 1024
        );
    }

    Ok(output.stdout)
}

/// Wrap a staged diff in the model input.
pub fn prompt(diff: &[u8]) -> String {
    let diff = String::from_utf8_lossy(diff);
    format!(
        "Generate a Conventional Commit message for exactly the staged changes below.\n\n<staged-diff>\n{diff}</staged-diff>"
    )
}

/// Pipe the exact diff sent to the model through the secret scanner; fail closed.
pub fn scan(diff: &[u8], config: &ScanConfig) -> Result<()> {
    let Some((program, args)) = config.command.split_first() else {
        bail!("commit.scan.command is empty");
    };
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            bail!("secret scanner `{program}` not found; install it or set commit.scan.command")
        }
        Err(error) => {
            return Err(error).with_context(|| format!("starting secret scanner `{program}`"));
        }
    };

    let mut stdin = child.stdin.take().expect("stdin is piped");
    let (written, output) = thread::scope(|scope| {
        // Feed stdin concurrently so a chatty scanner cannot deadlock on a full pipe.
        let writer = scope.spawn(move || stdin.write_all(diff));
        let output = child.wait_with_output();
        (writer.join().expect("stdin writer panicked"), output)
    });
    let output = output.with_context(|| format!("running secret scanner `{program}`"))?;

    match output.status.code() {
        Some(0) => match written {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                bail!("secret scanner `{program}` exited before reading the whole diff")
            }
            Err(error) => Err(error).context("writing staged diff to secret scanner"),
        },
        Some(code) if code == config.leak_exit_code => {
            let mut stderr = io::stderr().lock();
            stderr.write_all(&output.stdout).ok();
            stderr.write_all(&output.stderr).ok();
            bail!("secret scanner found possible secrets in the staged diff; nothing was sent");
        }
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "secret scanner `{program}` failed ({}): {}",
                output.status,
                stderr.trim()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    static NEXT_REPO: AtomicUsize = AtomicUsize::new(0);

    struct TestRepo {
        path: std::path::PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            let id = NEXT_REPO.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("pls-commit-test-{}-{id}", std::process::id()));
            fs::create_dir(&path).unwrap();
            git(&path, &["init", "--quiet"]);
            Self { path }
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).unwrap();
        }
    }

    fn git(cwd: &Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .current_dir(cwd)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn rejects_an_empty_index() {
        let repo = TestRepo::new();
        let error = staged_diff_in(&repo.path).unwrap_err();
        assert_eq!(error.to_string(), "no staged changes");
    }

    #[test]
    fn includes_only_staged_changes() {
        let repo = TestRepo::new();
        let staged = repo.path.join("staged.txt");
        fs::write(&staged, "staged content\n").unwrap();
        git(&repo.path, &["add", "staged.txt"]);
        fs::write(&staged, "staged content\nprivate unstaged content\n").unwrap();
        fs::write(
            repo.path.join("untracked.txt"),
            "private untracked content\n",
        )
        .unwrap();

        let prompt = prompt(&staged_diff_in(&repo.path).unwrap());
        assert!(prompt.contains("staged.txt"));
        assert!(prompt.contains("staged content"));
        assert!(!prompt.contains("untracked.txt"));
        assert!(!prompt.contains("private unstaged content"));
        assert!(!prompt.contains("private untracked content"));
    }

    #[test]
    fn rejects_an_oversized_diff() {
        let repo = TestRepo::new();
        fs::write(repo.path.join("large.txt"), "x".repeat(MAX_DIFF_BYTES)).unwrap();
        git(&repo.path, &["add", "large.txt"]);

        let error = staged_diff_in(&repo.path).unwrap_err();
        assert!(error.to_string().contains("staged diff is too large"));
    }

    fn scanner(command: &[&str]) -> ScanConfig {
        ScanConfig {
            enabled: true,
            command: command.iter().map(|arg| arg.to_string()).collect(),
            leak_exit_code: 42,
        }
    }

    fn sh(script: &str) -> ScanConfig {
        scanner(&["sh", "-c", script])
    }

    #[test]
    fn scanner_receives_the_exact_diff() {
        let repo = TestRepo::new();
        let received = repo.path.join("received");
        let config = scanner(&["sh", "-c", "cat > \"$0\"", received.to_str().unwrap()]);
        let diff = b"-removed secret\n+added line\n";

        scan(diff, &config).unwrap();
        assert_eq!(fs::read(&received).unwrap(), diff);
    }

    #[test]
    fn clean_scan_passes() {
        scan(b"diff", &sh("cat > /dev/null")).unwrap();
    }

    #[test]
    fn leak_exit_code_blocks_the_diff() {
        let error = scan(b"diff", &sh("cat > /dev/null; echo finding; exit 42")).unwrap_err();
        assert!(error.to_string().contains("found possible secrets"));
    }

    #[test]
    fn other_exit_codes_fail_closed() {
        let error = scan(b"diff", &sh("cat > /dev/null; echo boom >&2; exit 2")).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("secret scanner `sh` failed"), "{message}");
        assert!(message.contains("boom"), "{message}");
    }

    #[test]
    fn signal_termination_fails_closed() {
        let error = scan(b"diff", &sh("kill -9 $$")).unwrap_err();
        assert!(error.to_string().contains("secret scanner `sh` failed"));
    }

    #[test]
    fn scanner_that_skips_input_fails_closed() {
        let diff = vec![b'x'; MAX_DIFF_BYTES];
        let error = scan(&diff, &sh("exit 0")).unwrap_err();
        assert!(error.to_string().contains("before reading the whole diff"));
    }

    #[test]
    fn missing_scanner_is_reported() {
        let error = scan(b"diff", &scanner(&["pls-missing-secret-scanner"])).unwrap_err();
        assert_eq!(
            error.to_string(),
            "secret scanner `pls-missing-secret-scanner` not found; install it or set commit.scan.command"
        );
    }

    #[test]
    fn empty_scanner_command_is_rejected() {
        let error = scan(b"diff", &scanner(&[])).unwrap_err();
        assert_eq!(error.to_string(), "commit.scan.command is empty");
    }
}
