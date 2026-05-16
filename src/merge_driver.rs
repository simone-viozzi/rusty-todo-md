//! TODO.md merge driver registration.
//!
//! Three responsibilities, all derived from one input — the user's current
//! invocation args — so that the registration in `.git/config` and the rule
//! in `.gitattributes` always reflect *this run's* configuration:
//!
//! 1. [`build_expected`] — pure function from args to the canonical
//!    "what should be installed" pair (driver command + gitattributes block).
//! 2. [`install_driver`] — write `.git/config` + `.gitattributes` to match
//!    the expected pair. Idempotent: same args twice = no net change.
//! 3. [`reconcile`] — auto-install entry point. Compare expected vs.
//!    currently-installed; install (and print the diff) only on mismatch.
//!
//! ### `.gitattributes` is owned by a managed block
//!
//! We delimit our rule with `# BEGIN/END rusty-todo-md` markers and rewrite
//! the block as a unit. Anything outside the block is never touched, so the
//! user can edit other rules freely. Anything inside is canonical: a user
//! edit between the markers will be reverted on the next install — the
//! marker comment warns about this.
//!
//! ### Why bake args into the driver command at install time
//!
//! Git invokes the merge driver as a plain process; it can't read CLI flags
//! the user passed elsewhere. The driver command in `.git/config` has to be
//! self-contained, including the marker list and exclusions, so that the
//! TODO.md the driver writes matches the TODO.md pre-commit writes.

use crate::MarkerConfig;
use git2::Repository;
use log::{debug, info};
use std::path::{Path, PathBuf};

const BLOCK_BEGIN: &str = "# BEGIN rusty-todo-md (managed; do not edit between markers)";
const BLOCK_END: &str = "# END rusty-todo-md";
const DRIVER_NAME: &str = "rusty-todo-md TODO.md merge driver";
const CONFIG_KEY_NAME: &str = "merge.rusty-todo-md.name";
const CONFIG_KEY_DRIVER: &str = "merge.rusty-todo-md.driver";

/// Canonical "what should be in `.git/config` and `.gitattributes`" pair,
/// computed purely from the user's current CLI args. No I/O, no Repository
/// — used both to write state and to compare against existing state.
#[derive(Debug)]
pub struct Expected {
    pub driver_name: String,
    pub driver_command: String,
    pub gitattributes_block: String,
    /// Pattern field of the single `.gitattributes` rule we manage —
    /// surfaced for display only.
    pub gitattributes_pattern: String,
}

/// Build the expected install state from the user's current args.
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
        driver_name: DRIVER_NAME.to_string(),
        driver_command,
        gitattributes_block,
        gitattributes_pattern: pattern,
    })
}

/// Per-key changes that happened during an install. Empty = nothing changed.
/// Used to make the install message accurate: we only print "wrote X" for
/// keys that actually moved.
pub struct InstallSummary {
    pub driver_name: String,
    pub driver_command: String,
    pub gitattributes_path: PathBuf,
    pub gitattributes_pattern: String,
    /// Lines from the user's previous block (without our markers) so we can
    /// say "WAS X, NOW Y" when self-healing rewrites the block.
    pub previous_block_body: Option<String>,
    pub config_name_changed: bool,
    pub config_driver_changed: bool,
    pub gitattributes_changed: bool,
}

impl InstallSummary {
    pub fn anything_changed(&self) -> bool {
        self.config_name_changed || self.config_driver_changed || self.gitattributes_changed
    }
}

/// Whether the registration currently in this repo matches what `build_expected`
/// would install for the same args. Used by auto-install to skip work when
/// nothing has drifted.
pub fn matches_expected(repo: &Repository, expected: &Expected) -> bool {
    let stored_name = repo
        .config()
        .ok()
        .and_then(|c| c.get_string(CONFIG_KEY_NAME).ok());
    if stored_name.as_deref() != Some(expected.driver_name.as_str()) {
        return false;
    }
    let stored_driver = repo
        .config()
        .ok()
        .and_then(|c| c.get_string(CONFIG_KEY_DRIVER).ok());
    if stored_driver.as_deref() != Some(expected.driver_command.as_str()) {
        return false;
    }
    let Some(workdir) = repo.workdir() else {
        return false;
    };
    let gitattributes = std::fs::read_to_string(workdir.join(".gitattributes")).unwrap_or_default();
    extract_block(&gitattributes).as_deref() == Some(expected.gitattributes_block.as_str())
}

/// Auto-install entry point. Computes the expected state from the current
/// args and installs only if it doesn't already match — making this safe to
/// call on every pre-commit invocation. Returns `Ok(None)` when nothing
/// changed, `Ok(Some(summary))` when state was updated.
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
    Ok(Some(install_inner(repo, &expected)?))
}

/// Public install entry point used by the explicit `--install-merge-driver`
/// mode. Always writes (no skip-when-matching), but the value-level writes
/// underneath are idempotent — running it twice still produces a summary
/// that says "nothing changed" the second time.
pub fn install_driver(
    repo: &Repository,
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
) -> Result<InstallSummary, String> {
    let expected = build_expected(markers, exclude_patterns, exclude_dir_patterns, todo_path)?;
    install_inner(repo, &expected)
}

fn install_inner(repo: &Repository, expected: &Expected) -> Result<InstallSummary, String> {
    let mut config = repo
        .config()
        .map_err(|e| format!("failed to open git config: {e}"))?;

    let stored_name = config.get_string(CONFIG_KEY_NAME).ok();
    let stored_driver = config.get_string(CONFIG_KEY_DRIVER).ok();

    let config_name_changed = stored_name.as_deref() != Some(expected.driver_name.as_str());
    let config_driver_changed = stored_driver.as_deref() != Some(expected.driver_command.as_str());

    if config_name_changed {
        config
            .set_str(CONFIG_KEY_NAME, &expected.driver_name)
            .map_err(|e| format!("failed to write {CONFIG_KEY_NAME}: {e}"))?;
    }
    if config_driver_changed {
        config
            .set_str(CONFIG_KEY_DRIVER, &expected.driver_command)
            .map_err(|e| format!("failed to write {CONFIG_KEY_DRIVER}: {e}"))?;
    }

    info!("merge.rusty-todo-md.* config in sync");

    let workdir = repo
        .workdir()
        .ok_or_else(|| "repository has no working directory".to_string())?;
    let gitattributes_path = workdir.join(".gitattributes");
    let existing = std::fs::read_to_string(&gitattributes_path).unwrap_or_default();

    let previous_block_body = extract_block(&existing).map(|block| {
        block
            .lines()
            .filter(|l| !(l.trim() == BLOCK_BEGIN || l.trim() == BLOCK_END))
            .collect::<Vec<_>>()
            .join("\n")
    });

    let new_gitattributes = rewrite_block(&existing, &expected.gitattributes_block);
    let gitattributes_changed = new_gitattributes != existing;
    if gitattributes_changed {
        std::fs::write(&gitattributes_path, &new_gitattributes)
            .map_err(|e| format!("failed to write .gitattributes: {e}"))?;
        debug!("Wrote managed block to {gitattributes_path:?}");
    }

    Ok(InstallSummary {
        driver_name: expected.driver_name.clone(),
        driver_command: expected.driver_command.clone(),
        gitattributes_path,
        gitattributes_pattern: expected.gitattributes_pattern.clone(),
        previous_block_body,
        config_name_changed,
        config_driver_changed,
        gitattributes_changed,
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

/// Render the install summary in human-readable form, surfacing only what
/// actually changed. Used by both the explicit `--install-merge-driver`
/// mode and the `--auto-install-merge-driver` reconciler so collaborators
/// see a precise account of any state mutation.
pub fn format_install_summary(summary: &InstallSummary) -> String {
    let mut out = String::new();
    if !summary.anything_changed() {
        out.push_str("rusty-todo-md: merge driver already in sync; no changes.\n");
        out.push_str(&format!("  current driver: {}\n", summary.driver_command));
        return out;
    }

    out.push_str("rusty-todo-md: merge driver registration updated.\n");
    out.push_str("  git config changes (local):\n");
    out.push_str(&format!(
        "    {CONFIG_KEY_NAME}   = {} {}\n",
        summary.driver_name,
        if summary.config_name_changed {
            "(changed)"
        } else {
            "(unchanged)"
        }
    ));
    out.push_str(&format!(
        "    {CONFIG_KEY_DRIVER} = {} {}\n",
        summary.driver_command,
        if summary.config_driver_changed {
            "(changed)"
        } else {
            "(unchanged)"
        }
    ));
    if summary.gitattributes_changed {
        if let Some(prev) = &summary.previous_block_body {
            if !prev.is_empty() {
                out.push_str(&format!(
                    "  rewrote managed block in {}; previous body:\n",
                    summary.gitattributes_path.display()
                ));
                for line in prev.lines() {
                    out.push_str(&format!("    - {line}\n"));
                }
            } else {
                out.push_str(&format!(
                    "  rewrote managed block in {}\n",
                    summary.gitattributes_path.display()
                ));
            }
        } else {
            out.push_str(&format!(
                "  wrote managed block to {}\n",
                summary.gitattributes_path.display()
            ));
        }
        out.push_str(&format!(
            "    {} merge=rusty-todo-md\n",
            summary.gitattributes_pattern
        ));
    } else {
        out.push_str(&format!(
            "  {} managed block unchanged\n",
            summary.gitattributes_path.display()
        ));
    }
    out.push_str(&format!(
        "  to undo:\n    git config --unset {CONFIG_KEY_NAME}\n    git config --unset {CONFIG_KEY_DRIVER}\n    # then delete the `# BEGIN rusty-todo-md` .. `# END rusty-todo-md` block from {}\n",
        summary.gitattributes_path.display()
    ));
    out
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
    fn rewrite_block_appends_when_absent() {
        let before = "*.png binary\n";
        let block = format!("{BLOCK_BEGIN}\nTODO.md merge=rusty-todo-md\n{BLOCK_END}\n");
        let after = rewrite_block(before, &block);
        assert!(after.contains("*.png binary"));
        assert!(after.contains(BLOCK_BEGIN));
        assert!(after.ends_with(&format!("{BLOCK_END}\n")));
    }

    #[test]
    fn rewrite_block_replaces_when_present() {
        let initial = format!(
            "*.png binary\n{BLOCK_BEGIN}\nold TODO.md merge=rusty-todo-md\n{BLOCK_END}\n*.lock linguist-generated\n"
        );
        let new_block = format!("{BLOCK_BEGIN}\nnew/path.md merge=rusty-todo-md\n{BLOCK_END}\n");
        let after = rewrite_block(&initial, &new_block);
        assert!(after.contains("*.png binary"));
        assert!(after.contains("*.lock linguist-generated"));
        assert!(after.contains("new/path.md merge=rusty-todo-md"));
        assert!(!after.contains("old TODO.md merge=rusty-todo-md"));
    }

    #[test]
    fn rewrite_block_is_idempotent() {
        let block = format!("{BLOCK_BEGIN}\nTODO.md merge=rusty-todo-md\n{BLOCK_END}\n");
        let first = rewrite_block("", &block);
        let second = rewrite_block(&first, &block);
        assert_eq!(first, second);
    }

    #[test]
    fn extract_block_returns_full_block() {
        let content = format!(
            "*.png binary\n{BLOCK_BEGIN}\nTODO.md merge=rusty-todo-md\n{BLOCK_END}\nother stuff\n"
        );
        let block = extract_block(&content).unwrap();
        assert!(block.starts_with(BLOCK_BEGIN));
        assert!(block.contains("TODO.md merge=rusty-todo-md"));
        assert!(block.trim_end().ends_with(BLOCK_END));
    }

    #[test]
    fn build_expected_rejects_absolute_todo_path() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string()]);
        let result = build_expected(&markers, &[], &[], Path::new("/abs/TODO.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("absolute"));
    }

    #[test]
    fn build_expected_emits_quoted_path_in_block() {
        let markers = MarkerConfig::normalized(vec!["TODO".to_string()]);
        let expected = build_expected(&markers, &[], &[], Path::new("docs/my todos.md")).unwrap();
        assert!(expected
            .gitattributes_block
            .contains("\"docs/my todos.md\""));
    }
}
