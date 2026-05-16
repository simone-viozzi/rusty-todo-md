use crate::MarkerConfig;
use git2::Repository;
use log::{debug, info};
use std::path::{Path, PathBuf};

/// Result of an install-merge-driver invocation, used to render a clear
/// summary of exactly what changed on disk and in `.git/config`.
pub struct InstallSummary {
    pub driver_name: String,
    pub driver_command: String,
    pub gitattributes_path: PathBuf,
    pub gitattributes_line: String,
    pub gitattributes_modified: bool,
}

/// Returns true iff `merge.rusty-todo-md.driver` is set in the repository's
/// config (any level). Used by the `--auto-install-merge-driver` opt-in to
/// avoid re-registering the driver on every invocation.
pub fn is_driver_registered(repo: &Repository) -> bool {
    match repo.config() {
        Ok(config) => config.get_string("merge.rusty-todo-md.driver").is_ok(),
        Err(_) => false,
    }
}

/// Build the `driver = ...` command string that git will invoke for TODO.md
/// merges. Bakes in non-default markers, exclusion patterns, and the TODO.md
/// path so the driver re-runs with the same configuration the user installed.
pub fn build_driver_command(
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
) -> String {
    let mut cmd = String::from("rusty-todo-md");

    let is_default_markers =
        markers.markers.len() == 1 && markers.markers[0].eq_ignore_ascii_case("TODO");
    if !is_default_markers && !markers.markers.is_empty() {
        cmd.push_str(" --markers");
        for m in &markers.markers {
            cmd.push(' ');
            cmd.push_str(m);
        }
    }

    for pat in exclude_patterns {
        cmd.push_str(" --exclude ");
        cmd.push_str(&shell_quote(pat));
    }
    for pat in exclude_dir_patterns {
        cmd.push_str(" --exclude-dir ");
        cmd.push_str(&shell_quote(pat));
    }

    if todo_path != Path::new("TODO.md") {
        cmd.push_str(" --todo-path ");
        cmd.push_str(&shell_quote(&todo_path.display().to_string()));
    }

    cmd.push_str(" --merge-driver %O %A %B");
    cmd
}

fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '.' | ',' | ':'))
    {
        return s.to_string();
    }
    let escaped = s.replace('\'', "'\\''");
    format!("'{escaped}'")
}

/// Write the driver registration to `.git/config` and append the merge
/// attribute to `.gitattributes`. Returns a structured summary of every
/// mutation so the caller can print exactly what changed.
pub fn install_driver(
    repo: &Repository,
    markers: &MarkerConfig,
    exclude_patterns: &[String],
    exclude_dir_patterns: &[String],
    todo_path: &Path,
) -> Result<InstallSummary, String> {
    let driver_command =
        build_driver_command(markers, exclude_patterns, exclude_dir_patterns, todo_path);
    let driver_name = "rusty-todo-md TODO.md merge driver".to_string();

    let mut config = repo
        .config()
        .map_err(|e| format!("failed to open git config: {e}"))?;

    config
        .set_str("merge.rusty-todo-md.name", &driver_name)
        .map_err(|e| format!("failed to write merge.rusty-todo-md.name: {e}"))?;
    config
        .set_str("merge.rusty-todo-md.driver", &driver_command)
        .map_err(|e| format!("failed to write merge.rusty-todo-md.driver: {e}"))?;

    info!("Wrote merge.rusty-todo-md.* keys to git config");

    let workdir = repo
        .workdir()
        .ok_or_else(|| "repository has no working directory".to_string())?;
    let gitattributes_path = workdir.join(".gitattributes");
    let gitattributes_line = format!("{} merge=rusty-todo-md", todo_path.display());

    let existing = std::fs::read_to_string(&gitattributes_path).unwrap_or_default();
    let already_present = existing
        .lines()
        .any(|l| l.trim() == gitattributes_line.trim());

    let mut gitattributes_modified = false;
    if !already_present {
        let mut new_content = existing.clone();
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&gitattributes_line);
        new_content.push('\n');
        std::fs::write(&gitattributes_path, new_content)
            .map_err(|e| format!("failed to write .gitattributes: {e}"))?;
        gitattributes_modified = true;
        debug!("Appended merge attribute to {gitattributes_path:?}");
    }

    Ok(InstallSummary {
        driver_name,
        driver_command,
        gitattributes_path,
        gitattributes_line,
        gitattributes_modified,
    })
}

/// Render the install summary in human-readable form. Used by both the
/// explicit `install-merge-driver` subcommand and the
/// `--auto-install-merge-driver` flag — the latter prints the same text to
/// ensure collaborators are never surprised by silent config mutation.
pub fn format_install_summary(summary: &InstallSummary) -> String {
    let mut out = String::new();
    out.push_str("rusty-todo-md: installed TODO.md merge driver.\n");
    out.push_str("  git config changes (local):\n");
    out.push_str(&format!(
        "    merge.rusty-todo-md.name   = {}\n",
        summary.driver_name
    ));
    out.push_str(&format!(
        "    merge.rusty-todo-md.driver = {}\n",
        summary.driver_command
    ));
    if summary.gitattributes_modified {
        out.push_str(&format!(
            "  appended to {}:\n    {}\n",
            summary.gitattributes_path.display(),
            summary.gitattributes_line
        ));
    } else {
        out.push_str(&format!(
            "  {} already contains: {}\n",
            summary.gitattributes_path.display(),
            summary.gitattributes_line
        ));
    }
    out.push_str("  to undo: git config --unset merge.rusty-todo-md.name && \\\n");
    out.push_str("           git config --unset merge.rusty-todo-md.driver\n");
    out
}
