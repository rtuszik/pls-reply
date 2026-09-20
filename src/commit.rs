use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

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

/// Build the model input from the current repository's staged changes.
pub fn staged_changes_prompt() -> Result<String> {
    staged_changes_prompt_in(Path::new("."))
}

fn staged_changes_prompt_in(cwd: &Path) -> Result<String> {
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

    let diff = String::from_utf8_lossy(&output.stdout);
    Ok(format!(
        "Generate a Conventional Commit message for exactly the staged changes below.\n\n<staged-diff>\n{diff}</staged-diff>"
    ))
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
        let error = staged_changes_prompt_in(&repo.path).unwrap_err();
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

        let prompt = staged_changes_prompt_in(&repo.path).unwrap();
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

        let error = staged_changes_prompt_in(&repo.path).unwrap_err();
        assert!(error.to_string().contains("staged diff is too large"));
    }
}
