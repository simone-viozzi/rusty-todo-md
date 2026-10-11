//! TODO.md merge driver registration.
//!
//! Derive the expected state from the user's current CLI args, then write
//! `.git/config` and `.gitattributes` to match. Two callers:
//! [`install_driver`] (explicit `--install-merge-driver`, the maintainer's
//! one-time opt-in) writes both and always reports a summary;
//! [`reconcile`] (every pre-commit run) only fixes this clone's
//! `.git/config`, and only when the committed `.gitattributes` already
//! routes TODO.md to the driver.
//!
//! The `.gitattributes` rule lives in a `# BEGIN/END rusty-todo-md` block
//! that we rewrite as a unit. Rules outside the block are preserved
//! byte-for-byte; rules inside are canonical, so a hand-edit between the
//! markers will be reverted on the next install.
//!
//! Args are baked into the driver command (`--markers …`, `--exclude …`)
//! because git invokes the driver as a plain subprocess with no awareness
//! of CLI flags the user passed elsewhere — the registration has to be
//! self-contained. So is the binary's absolute path: pre-commit installs
//! the binary in a private virtualenv that is not on the PATH git's shell
//! sees.

use crate::MarkerConfig;
use git2::{AttrCheckFlags, Repository};
use std::path::{Path, PathBuf};

const BLOCK_BEGIN: &str = "# BEGIN rusty-todo-md (managed; do not edit between markers)";
const BLOCK_END: &str = "# END rusty-todo-md";
const DRIVER_NAME: &str = "rusty-todo-md TODO.md merge driver";
const CONFIG_KEY_NAME: &str = "merge.rusty-todo-md.name";
const CONFIG_KEY_DRIVER: &str = "merge.rusty-todo-md.driver";

/// Canonical state, computed purely from the user's current CLI args.
/// No I/O, no Repository — used both to write state and to compare
/// against existing state.
pub struct Expected {
    pub driver_command: String,
    pub gitattributes_block: String,
}

/// `exe` is the binary the driver should run, normally
/// `std::env::current_exe()` of the invocation doing the install.
pub fn build_expected(
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
    exe: &Path,
) -> Result<Expected, String> {
    if todo_path.is_absolute() {
        return Err(format!(
            "--todo-path {} is absolute; the merge driver needs a path relative to the repository root so the .gitattributes rule can match it",
            todo_path.display()
        ));
    }
    let driver_command = build_driver_command(
        markers,
        exclude_patterns,
        exclude_dir_patterns,
        todo_path,
        exe,
    );
    let pattern = quote_for_gitattributes(&todo_path.display().to_string());
    let gitattributes_block =
        format!("{BLOCK_BEGIN}\n{pattern} merge=rusty-todo-md\n{BLOCK_END}\n");
    Ok(Expected {
        driver_command,
        gitattributes_block,
    })
}

/// Outcome of an install. Captures only what the summary printer needs.
pub struct InstallSummary {
    pub driver_command: String,
    pub gitattributes_path: PathBuf,
    /// True when on-disk state already matched expected before this call —
    /// the writers were skipped entirely.
    pub was_in_sync: bool,
}

/// Whether this clone's `.git/config` registers exactly the expected
/// driver command (including the binary path).
fn config_matches(repo: &Repository, expected: &Expected) -> bool {
    repo.config()
        .ok()
        .and_then(|c| {
            Some((
                c.get_string(CONFIG_KEY_NAME).ok()?,
                c.get_string(CONFIG_KEY_DRIVER).ok()?,
            ))
        })
        .is_some_and(|(name, driver)| name == DRIVER_NAME && driver == expected.driver_command)
}

/// Whether the registration in this repo matches what `build_expected`
/// would install for the same args: both `.git/config` and the managed
/// `.gitattributes` block.
pub fn matches_expected(repo: &Repository, expected: &Expected) -> bool {
    let Some(workdir) = repo.workdir() else {
        return false;
    };
    if !config_matches(repo, expected) {
        return false;
    }
    let gitattributes = std::fs::read_to_string(workdir.join(".gitattributes")).unwrap_or_default();
    extract_block(&gitattributes).as_deref() == Some(expected.gitattributes_block.as_str())
}

/// Whether `.gitattributes` routes `todo_path` to the driver. This is the
/// team-level opt-in: a maintainer commits the rule once, and every clone
/// that inherits it needs the driver registered. If the rule is present but
/// the driver isn't defined, git silently falls back to its text merge.
pub fn is_routed(repo: &Repository, todo_path: &Path) -> bool {
    matches!(
        repo.get_attr(todo_path, "merge", AttrCheckFlags::FILE_THEN_INDEX),
        Ok(Some("rusty-todo-md"))
    )
}

/// Per-run entry point. When `.gitattributes` routes TODO.md to the driver,
/// make this clone's `.git/config` register it with `exe` and the current
/// args. Never touches `.gitattributes` (a tracked file). Returns
/// `Ok(None)` when not routed or already in sync.
pub fn reconcile(
    repo: &Repository,
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
    exe: &Path,
) -> Result<Option<InstallSummary>, String> {
    if !is_routed(repo, todo_path) {
        return Ok(None);
    }
    let expected = build_expected(
        markers,
        exclude_patterns,
        exclude_dir_patterns,
        todo_path,
        exe,
    )?;
    if config_matches(repo, &expected) {
        return Ok(None);
    }
    write_config(repo, &expected)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| "repository has no working directory".to_string())?;
    Ok(Some(InstallSummary {
        driver_command: expected.driver_command,
        gitattributes_path: workdir.join(".gitattributes"),
        was_in_sync: false,
    }))
}

/// `--install-merge-driver` entry point. Always reports a summary (even
/// when already in sync) so a manual run shows the user what the current
/// registration looks like.
pub fn install_driver(
    repo: &Repository,
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
    exe: &Path,
) -> Result<InstallSummary, String> {
    let expected = build_expected(
        markers,
        exclude_patterns,
        exclude_dir_patterns,
        todo_path,
        exe,
    )?;
    let was_in_sync = matches_expected(repo, &expected);
    install_to_match(repo, &expected, was_in_sync)
}

fn write_config(repo: &Repository, expected: &Expected) -> Result<(), String> {
    let mut config = repo
        .config()
        .map_err(|e| format!("failed to open git config: {e}"))?;
    config
        .set_str(CONFIG_KEY_NAME, DRIVER_NAME)
        .map_err(|e| format!("failed to write {CONFIG_KEY_NAME}: {e}"))?;
    config
        .set_str(CONFIG_KEY_DRIVER, &expected.driver_command)
        .map_err(|e| format!("failed to write {CONFIG_KEY_DRIVER}: {e}"))
}

fn install_to_match(
    repo: &Repository,
    expected: &Expected,
    already_in_sync: bool,
) -> Result<InstallSummary, String> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| "repository has no working directory".to_string())?;
    let gitattributes_path = workdir.join(".gitattributes");

    if already_in_sync {
        return Ok(InstallSummary {
            driver_command: expected.driver_command.clone(),
            gitattributes_path,
            was_in_sync: true,
        });
    }

    write_config(repo, expected)?;

    let existing = std::fs::read_to_string(&gitattributes_path).unwrap_or_default();
    let new_gitattributes = rewrite_block(&existing, &expected.gitattributes_block);
    if new_gitattributes != existing {
        std::fs::write(&gitattributes_path, &new_gitattributes)
            .map_err(|e| format!("failed to write .gitattributes: {e}"))?;
    }

    Ok(InstallSummary {
        driver_command: expected.driver_command.clone(),
        gitattributes_path,
        was_in_sync: false,
    })
}

/// Shown on stderr when git runs the driver but neither the registered
/// binary nor a `rusty-todo-md` on PATH exists (e.g. pre-commit
/// garbage-collected the virtualenv). Running the hook once re-registers
/// the driver with the current binary.
const DRIVER_NOT_FOUND: &str = "rusty-todo-md: TODO.md merge driver binary not found. Run: pre-commit run rusty-todo-md --all-files (or rusty-todo-md --install-merge-driver), then retry.";

/// Build the `driver = ...` command. Bakes in non-default markers,
/// exclusion patterns, and the TODO.md path so the driver runs with the
/// same configuration the user installed.
///
/// Git runs the command through `sh` (Git for Windows ships one), so it can
/// resolve the binary the way pre-commit's own `.git/hooks` scripts do: the
/// absolute path first, then `rusty-todo-md` on PATH, else a clear error
/// instead of a bare "command not found".
fn build_driver_command(
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
    exe: &Path,
) -> String {
    let exe = exe.display().to_string();
    // `sh` on Windows handles forward slashes in drive paths reliably.
    let exe = if cfg!(windows) {
        exe.replace('\\', "/")
    } else {
        exe
    };
    let mut cmd = format!(
        "p={}; [ -x \"$p\" ] || p=$(command -v rusty-todo-md) || {{ echo {} >&2; exit 1; }}; \"$p\"",
        quote_for_shell(&exe),
        quote_for_shell(DRIVER_NOT_FOUND),
    );

    // Always emit --markers when non-empty. We previously omitted it for the
    // "default" case (single marker case-insensitively equal to "TODO") to
    // keep the command shorter, but marker matching is case-sensitive
    // downstream — `--markers todo` is a real configuration choice that has
    // to be preserved verbatim in the driver command, otherwise the merge
    // driver would extract using the wrong marker name.
    if !markers.markers.is_empty() {
        cmd.push_str(" --markers");
        for m in &markers.markers {
            cmd.push(' ');
            cmd.push_str(&quote_for_shell(m));
        }
    }
    for pat in exclude_patterns {
        cmd.push_str(" --exclude ");
        cmd.push_str(&quote_for_shell(pat));
    }
    for pat in exclude_dir_patterns {
        cmd.push_str(" --exclude-dir ");
        cmd.push_str(&quote_for_shell(pat));
    }
    if todo_path != Path::new("TODO.md") {
        cmd.push_str(" --todo-path ");
        cmd.push_str(&quote_for_shell(&todo_path.display().to_string()));
    }
    cmd.push_str(" --merge-driver %O %A %B");
    cmd
}

/// Shell-quote for the command string in `.git/config`. Git parses that
/// string by splitting on whitespace and respecting standard shell quoting.
fn quote_for_shell(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '.' | ',' | ':'))
    {
        return s.to_string();
    }
    let escaped = s.replace('\'', "'\\''");
    format!("'{escaped}'")
}

/// Quote a path for use as the pattern field of a `.gitattributes` rule.
///
/// gitattributes treats `*`, `?`, `[` as glob metacharacters; an unescaped
/// `?` in `weird?file.md` would match a literal `?` *or* any single
/// character, which is not what we want for a fixed filename. We always
/// escape glob metacharacters with a backslash. We additionally
/// double-quote the pattern when it contains whitespace, comment-leading
/// `#`, or characters that need backslash escaping (`"`, `\`), since
/// gitattributes parses unquoted whitespace as the field separator.
fn quote_for_gitattributes(pattern: &str) -> String {
    let needs_quoting = pattern.is_empty()
        || pattern
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '\\' | '#'));
    let mut body = String::with_capacity(pattern.len() + 2);
    for c in pattern.chars() {
        match c {
            '*' | '?' | '[' | '\\' | '"' => {
                body.push('\\');
                body.push(c);
            }
            _ => body.push(c),
        }
    }
    if needs_quoting {
        format!("\"{body}\"")
    } else {
        body
    }
}

/// Pull the `# BEGIN rusty-todo-md` ... `# END rusty-todo-md` block out of
/// `.gitattributes`, including both marker lines. Returns `None` if no
/// block is present.
fn extract_block(content: &str) -> Option<String> {
    let mut iter = content.lines().enumerate();
    let begin = iter.find(|(_, l)| l.trim() == BLOCK_BEGIN)?.0;
    let end = iter.find(|(_, l)| l.trim() == BLOCK_END)?.0;
    let lines: Vec<&str> = content.lines().collect();
    let mut block = lines[begin..=end].join("\n");
    block.push('\n');
    Some(block)
}

/// Replace the managed block in `content` with `new_block`, or append a
/// fresh block if no managed block exists. Anything outside the block is
/// preserved byte-for-byte (modulo a leading newline if we have to insert
/// one to separate the appended block from the existing tail).
fn rewrite_block(content: &str, new_block: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let begin = lines.iter().position(|l| l.trim() == BLOCK_BEGIN);
    let end = lines.iter().position(|l| l.trim() == BLOCK_END);

    if let (Some(b), Some(e)) = (begin, end) {
        if e >= b {
            let before = lines[..b].join("\n");
            let after = lines[e + 1..].join("\n");
            let mut out = String::new();
            if !before.is_empty() {
                out.push_str(&before);
                out.push('\n');
            }
            out.push_str(new_block);
            if !after.is_empty() {
                out.push_str(&after);
                if !after.ends_with('\n') {
                    out.push('\n');
                }
            }
            return out;
        }
    }

    let mut out = content.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(new_block);
    out
}

/// Render the install summary. Two states: "already in sync" (silent
/// confirmation that the current registration matches the user's args) or
/// "installed/updated" (driver command + undo hint).
pub fn format_install_summary(s: &InstallSummary) -> String {
    let header = if s.was_in_sync {
        "rusty-todo-md: merge driver already in sync."
    } else {
        "rusty-todo-md: merge driver installed/updated."
    };
    format!(
        "{header}\n  {CONFIG_KEY_DRIVER} = {}\n  managed block in {}\n  to undo: git config --unset {CONFIG_KEY_NAME} && git config --unset {CONFIG_KEY_DRIVER} && remove the `# BEGIN rusty-todo-md` .. `# END rusty-todo-md` block from {}\n",
        s.driver_command,
        s.gitattributes_path.display(),
        s.gitattributes_path.display(),
    )
}

/// Render what [`reconcile`] changed. Reconcile only writes this clone's
/// `.git/config`; the `.gitattributes` routing is the team's committed
/// opt-in, so it isn't offered for undoing here.
pub fn format_reconcile_summary(s: &InstallSummary) -> String {
    format!(
        "rusty-todo-md: registered the TODO.md merge driver in this clone's .git/config ({} routes TODO.md to it).\n  {CONFIG_KEY_DRIVER} = {}\n",
        s.gitattributes_path.display(),
        s.driver_command,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_for_gitattributes_unquoted_when_safe() {
        assert_eq!(quote_for_gitattributes("TODO.md"), "TODO.md");
        assert_eq!(quote_for_gitattributes("docs/TODOS.md"), "docs/TODOS.md");
    }

    #[test]
    fn quote_for_gitattributes_quotes_when_whitespace_or_specials() {
        assert_eq!(quote_for_gitattributes("my todos.md"), "\"my todos.md\"");
        assert_eq!(quote_for_gitattributes("a\\b"), "\"a\\\\b\"");
        assert_eq!(quote_for_gitattributes("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote_for_gitattributes("# weird.md"), "\"# weird.md\"");
    }

    #[test]
    fn quote_for_gitattributes_escapes_glob_metacharacters() {
        // `*`, `?`, `[` are gitattributes glob meta — must be backslash-
        // escaped so they match literally and don't expand to a pattern.
        assert_eq!(quote_for_gitattributes("file?.md"), "file\\?.md");
        assert_eq!(quote_for_gitattributes("a*b.md"), "a\\*b.md");
        assert_eq!(quote_for_gitattributes("[brackets].md"), "\\[brackets].md");
    }

    #[test]
    fn build_expected_rejects_absolute_todo_path() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string()]);
        let result = build_expected(
            &markers,
            &[],
            &[],
            Path::new("/abs/TODO.md"),
            Path::new("/bin/rtm"),
        );
        let Err(msg) = result else {
            panic!("expected Err");
        };
        assert!(msg.contains("absolute"), "got: {msg}");
    }

    #[test]
    fn build_expected_quotes_path_with_specials() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string()]);
        let expected = build_expected(
            &markers,
            &[],
            &[],
            Path::new("docs/my todos.md"),
            Path::new("/bin/rtm"),
        )
        .unwrap();
        assert!(
            expected
                .gitattributes_block
                .contains("\"docs/my todos.md\""),
            "block: {}",
            expected.gitattributes_block
        );
    }

    #[test]
    fn build_driver_command_emits_all_args() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
        let cmd = build_driver_command(
            &markers,
            &["*.log".to_string()],
            &["vendor".to_string()],
            Path::new("docs/T.md"),
            Path::new("/opt/venv/bin/rusty-todo-md"),
        );
        assert!(cmd.starts_with("p=/opt/venv/bin/rusty-todo-md; [ -x \"$p\" ] || "));
        assert!(cmd.contains("--markers TODO FIXME"));
        assert!(cmd.contains("--exclude '*.log'"));
        assert!(cmd.contains("--exclude-dir vendor"));
        assert!(cmd.contains("--todo-path docs/T.md"));
        assert!(cmd.ends_with("--merge-driver %O %A %B"));
    }

    #[test]
    fn build_driver_command_quotes_exe_path_with_spaces() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string()]);
        let cmd = build_driver_command(
            &markers,
            &[],
            &[],
            Path::new("TODO.md"),
            Path::new("/home/a user/venv/bin/rusty-todo-md"),
        );
        assert!(
            cmd.starts_with("p='/home/a user/venv/bin/rusty-todo-md'; "),
            "cmd: {cmd}"
        );
    }

    #[test]
    fn quote_for_shell_passes_safe_strings_through() {
        assert_eq!(quote_for_shell("TODO"), "TODO");
        assert_eq!(quote_for_shell("docs/file.md"), "docs/file.md");
        assert_eq!(quote_for_shell("a-b_c.d"), "a-b_c.d");
    }

    #[test]
    fn quote_for_shell_quotes_specials_and_escapes_single_quotes() {
        assert_eq!(quote_for_shell("a b"), "'a b'");
        assert_eq!(quote_for_shell("*.log"), "'*.log'");
        assert_eq!(quote_for_shell("a'b"), "'a'\\''b'");
        assert_eq!(quote_for_shell(""), "''");
    }

    #[test]
    fn format_reconcile_summary_reports_config_only() {
        let out = format_reconcile_summary(&InstallSummary {
            driver_command: "p=/v/rusty-todo-md; \"$p\" --merge-driver %O %A %B".into(),
            gitattributes_path: PathBuf::from("/x/.gitattributes"),
            was_in_sync: false,
        });
        assert!(out.contains(".git/config"), "{out}");
        assert!(out.contains("--merge-driver %O %A %B"), "{out}");
        // The routing block is the team's opt-in; never suggest removing it.
        assert!(!out.contains("remove the"), "{out}");
    }

    #[test]
    fn format_install_summary_renders_both_states() {
        let path = PathBuf::from("/x/.gitattributes");
        let in_sync = InstallSummary {
            driver_command: "rusty-todo-md --merge-driver %O %A %B".into(),
            gitattributes_path: path.clone(),
            was_in_sync: true,
        };
        let out = format_install_summary(&in_sync);
        assert!(out.contains("already in sync"));
        assert!(out.contains("rusty-todo-md --merge-driver"));

        let installed = InstallSummary {
            was_in_sync: false,
            ..in_sync
        };
        let out = format_install_summary(&installed);
        assert!(out.contains("installed/updated"));
        assert!(out.contains("to undo"));
    }
}
