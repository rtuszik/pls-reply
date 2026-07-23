use std::io::Write;
use std::process::{Command, Stdio};

/// Copy `text` to the system clipboard by shelling out to the platform tool.
///
/// Shelling out (rather than an in-process crate) keeps the copied text alive
/// after we exit, notably on Linux/X11 where an owning process must stay
/// running for the selection to persist; `xclip`/`wl-copy` daemonize for us.
///
/// Returns `true` if a clipboard tool accepted the text, `false` if none was
/// found (in which case the answer is still printed to stdout).
pub fn copy(text: &str) -> bool {
    let candidates: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
        &[("pbcopy", &[])]
    } else {
        &[
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ]
    };

    for (cmd, args) in candidates {
        let child = Command::new(cmd)
            .args(*args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let Ok(mut child) = child else { continue };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
            // stdin drops here, sending EOF so the tool finishes reading.
        }
        let _ = child.wait();
        return true;
    }
    false
}
