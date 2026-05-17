//! TODO.md merge driver registration.
//!
//! Derive the expected state from the user's current CLI args, then write
//! `.git/config` and `.gitattributes` to match. Two callers:
//! [`install_driver`] (explicit `--install-merge-driver`) always reports a
//! summary; [`reconcile`] (auto-install) skips silently when state already
//! matches and only writes on drift.
//!
//! The `.gitattributes` rule lives in a `# BEGIN/END rusty-todo-md` block
//! that we rewrite as a unit. Rules outside the block are preserved
//! byte-for-byte; rules inside are canonical, so a hand-edit between the
//! markers will be reverted on the next install.
//!
//! Args are baked into the driver command (`--markers …`, `--exclude …`)
//! because git invokes the driver as a plain subprocess with no awareness
//! of CLI flags the user passed elsewhere — the registration has to be
//! self-contained.

use crate::MarkerConfig;
use git2::Repository;
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

pub fn build_expected(
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
) -> Result<Expected, String> {
    if todo_path.is_absolute() {
        return Err(format!(
            "--todo-path {} is absolute; the merge driver needs a path relative to the repository root so the .gitattributes rule can match it",
            todo_path.display()
        ));
    }
    let driver_command =
        build_driver_command(markers, exclude_patterns, exclude_dir_patterns, todo_path);
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

/// Whether the registration in this repo matches what `build_expected`
/// would install for the same args. Used by `reconcile` to skip work when
/// nothing has drifted.
pub fn matches_expected(repo: &Repository, expected: &Expected) -> bool {
    let Some(workdir) = repo.workdir() else {
        return false;
    };
    let config_ok = repo
        .config()
        .ok()
        .and_then(|c| {
            Some((
                c.get_string(CONFIG_KEY_NAME).ok()?,
                c.get_string(CONFIG_KEY_DRIVER).ok()?,
            ))
        })
        .is_some_and(|(name, driver)| name == DRIVER_NAME && driver == expected.driver_command);
    if !config_ok {
        return false;
    }
    let gitattributes = std::fs::read_to_string(workdir.join(".gitattributes")).unwrap_or_default();
    extract_block(&gitattributes).as_deref() == Some(expected.gitattributes_block.as_str())
}

/// Auto-install entry point. Installs only if state has drifted from
/// expected; safe to call on every invocation. Returns `Ok(None)` when
/// nothing changed.
pub fn reconcile(
    repo: &Repository,
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
) -> Result<Option<InstallSummary>, String> {
    let expected = build_expected(markers, exclude_patterns, exclude_dir_patterns, todo_path)?;
    if matches_expected(repo, &expected) {
        return Ok(None);
    }
    Ok(Some(install_to_match(repo, &expected, false)?))
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
) -> Result<InstallSummary, String> {
    let expected = build_expected(markers, exclude_patterns, exclude_dir_patterns, todo_path)?;
    let was_in_sync = matches_expected(repo, &expected);
    install_to_match(repo, &expected, was_in_sync)
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

    let mut config = repo
        .config()
        .map_err(|e| format!("failed to open git config: {e}"))?;
    config
        .set_str(CONFIG_KEY_NAME, DRIVER_NAME)
        .map_err(|e| format!("failed to write {CONFIG_KEY_NAME}: {e}"))?;
    config
        .set_str(CONFIG_KEY_DRIVER, &expected.driver_command)
        .map_err(|e| format!("failed to write {CONFIG_KEY_DRIVER}: {e}"))?;

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

/// Build the `driver = ...` command. Bakes in non-default markers,
/// exclusion patterns, and the TODO.md path so the driver runs with the
/// same configuration the user installed.
fn build_driver_command(
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
) -> String {
    let mut cmd = String::from("rusty-todo-md");

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
        let result = build_expected(&markers, &[], &[], Path::new("/abs/TODO.md"));
        let Err(msg) = result else {
            panic!("expected Err");
        };
        assert!(msg.contains("absolute"), "got: {msg}");
    }

    #[test]
    fn build_expected_quotes_path_with_specials() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string()]);
        let expected = build_expected(&markers, &[], &[], Path::new("docs/my todos.md")).unwrap();
        assert!(
            expected
                .gitattributes_block
                .contains("\"docs/my todos.md\""),
            "block: {}",
            expected.gitattributes_block
        );
    }

    // ---- in-process tests against a real git2::Repository -----------------
    //
    // These exist because the end-to-end integration tests in
    // tests/merge_driver_tests.rs spawn the binary as a subprocess (via
    // assert_cmd), which `cargo tarpaulin` cannot instrument. The merge
    // driver code runs in that subprocess and shows as uncovered. The tests
    // below exercise the same public surface in-process so coverage
    // reflects the work the integration tests already do end-to-end.

    fn default_markers() -> MarkerConfig {
        MarkerConfig::normalized(vec!["TODO".to_string()])
    }

    fn fresh_repo() -> (tempfile::TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn install_driver_writes_state_first_run() {
        let (dir, repo) = fresh_repo();
        let summary =
            install_driver(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        assert!(!summary.was_in_sync);

        let cfg = repo.config().unwrap();
        assert_eq!(cfg.get_string(CONFIG_KEY_NAME).unwrap(), DRIVER_NAME);
        assert!(cfg
            .get_string(CONFIG_KEY_DRIVER)
            .unwrap()
            .contains("--merge-driver %O %A %B"));

        let attrs = std::fs::read_to_string(dir.path().join(".gitattributes")).unwrap();
        assert!(attrs.contains(BLOCK_BEGIN));
        assert!(attrs.contains("TODO.md merge=rusty-todo-md"));
    }

    #[test]
    fn install_driver_is_a_fixed_point() {
        let (dir, repo) = fresh_repo();
        install_driver(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        let after_first = std::fs::read_to_string(dir.path().join(".gitattributes")).unwrap();
        let summary =
            install_driver(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        assert!(summary.was_in_sync);
        let after_second = std::fs::read_to_string(dir.path().join(".gitattributes")).unwrap();
        assert_eq!(after_first, after_second);
    }

    #[test]
    fn reconcile_is_none_when_state_matches() {
        let (_dir, repo) = fresh_repo();
        install_driver(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        let result = reconcile(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn reconcile_writes_when_config_drifted() {
        let (_dir, repo) = fresh_repo();
        install_driver(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        repo.config()
            .unwrap()
            .set_str(CONFIG_KEY_DRIVER, "stale-cmd")
            .unwrap();
        let result = reconcile(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        let summary = result.expect("expected drift to trigger an install");
        assert!(!summary.was_in_sync);
        assert_eq!(
            repo.config()
                .unwrap()
                .get_string(CONFIG_KEY_DRIVER)
                .unwrap(),
            summary.driver_command
        );
    }

    #[test]
    fn install_driver_rewrites_block_on_path_change() {
        let (dir, repo) = fresh_repo();
        install_driver(&repo, &default_markers(), &[], &[], Path::new("TODO.md")).unwrap();
        install_driver(
            &repo,
            &default_markers(),
            &[],
            &[],
            Path::new("docs/TODOS.md"),
        )
        .unwrap();
        let attrs = std::fs::read_to_string(dir.path().join(".gitattributes")).unwrap();
        assert_eq!(
            attrs.matches("merge=rusty-todo-md").count(),
            1,
            "exactly one managed rule expected; got:\n{attrs}"
        );
        assert!(attrs.contains("docs/TODOS.md merge=rusty-todo-md"));
    }

    #[test]
    fn build_driver_command_emits_all_args() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
        let cmd = build_driver_command(
            &markers,
            &["*.log".to_string()],
            &["vendor".to_string()],
            Path::new("docs/T.md"),
        );
        assert!(cmd.contains("--markers TODO FIXME"));
        assert!(cmd.contains("--exclude '*.log'"));
        assert!(cmd.contains("--exclude-dir vendor"));
        assert!(cmd.contains("--todo-path docs/T.md"));
        assert!(cmd.ends_with("--merge-driver %O %A %B"));
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
