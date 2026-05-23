//! Integration tests for the TODO.md merge driver (issue #179).
//!
//! These tests drive the real binary through real git operations against a
//! temporary repository, because the entire point of the driver is its
//! behavior under `git rebase` / `git merge` — that can't be unit-tested.

use assert_cmd::Command as AssertCommand;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("rusty-todo-md")
}

fn git(repo: &Path, args: &[&str]) -> std::process::Output {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .expect("git failed to spawn");
    out
}

fn git_must(repo: &Path, args: &[&str]) -> std::process::Output {
    let out = git(repo, args);
    if !out.status.success() {
        panic!(
            "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    out
}

/// Initialize a fresh git repo with config + initial commit. Pinning to a
/// known initial-branch name keeps the test independent of the system git's
/// default ("main" vs "master").
fn init_repo() -> TempDir {
    let dir = tempdir().expect("tempdir");
    let p = dir.path();
    git_must(p, &["init", "-q", "-b", "main"]);
    git_must(p, &["config", "user.email", "t@t"]);
    git_must(p, &["config", "user.name", "t"]);
    // Empty initial commit so HEAD exists.
    git_must(p, &["commit", "-q", "--allow-empty", "-m", "init"]);
    dir
}

/// Run the binary inside `repo`, rewriting TODO.md from the given source
/// files. Mirrors the pre-commit invocation shape.
fn run_tool_on(repo: &Path, files: &[&str]) {
    let mut cmd = AssertCommand::new(bin());
    cmd.current_dir(repo).arg("--auto-add");
    if !files.is_empty() {
        cmd.arg("--");
        for f in files {
            cmd.arg(f);
        }
    }
    cmd.assert().success();
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).expect("read")
}

#[test]
fn install_merge_driver_writes_config_and_gitattributes() {
    let dir = init_repo();
    let repo = dir.path();

    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--install-merge-driver")
        .assert()
        .success()
        .stdout(predicates::str::contains("merge driver installed/updated"));

    let config = read(&repo.join(".git").join("config"));
    assert!(
        config.contains("[merge \"rusty-todo-md\"]"),
        "merge section missing in config:\n{config}"
    );
    assert!(
        config.contains("--merge-driver %O %A %B"),
        "driver command missing in config:\n{config}"
    );

    let attrs = read(&repo.join(".gitattributes"));
    assert!(
        attrs.contains("# BEGIN rusty-todo-md"),
        "managed-block begin marker missing:\n{attrs}"
    );
    assert!(
        attrs.contains("# END rusty-todo-md"),
        "managed-block end marker missing:\n{attrs}"
    );
    assert!(
        attrs.contains("TODO.md merge=rusty-todo-md"),
        "gitattributes rule missing:\n{attrs}"
    );
}

/// Install must be a fixed point on identical args: running twice produces
/// the same `.gitattributes` content byte-for-byte, and the second run
/// reports no changes. Pre-existing unrelated rules outside the block must
/// be preserved.
#[test]
fn install_merge_driver_is_convergent() {
    let dir = init_repo();
    let repo = dir.path();
    fs::write(
        repo.join(".gitattributes"),
        "*.png binary\n*.lock linguist-generated\n",
    )
    .unwrap();

    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--install-merge-driver")
        .assert()
        .success();
    let after_first = read(&repo.join(".gitattributes"));

    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--install-merge-driver")
        .assert()
        .success()
        .stdout(predicates::str::contains("already in sync"));
    let after_second = read(&repo.join(".gitattributes"));

    assert_eq!(
        after_first, after_second,
        "second install must be a no-op on disk; got:\nfirst:\n{after_first}\nsecond:\n{after_second}"
    );
    assert!(
        after_first.contains("*.png binary"),
        "pre-existing user rule must be preserved:\n{after_first}"
    );
    assert!(
        after_first.contains("*.lock linguist-generated"),
        "pre-existing user rule must be preserved:\n{after_first}"
    );
    let begin_count = after_first.matches("# BEGIN rusty-todo-md").count();
    assert_eq!(
        begin_count, 1,
        "exactly one managed block expected, got:\n{after_first}"
    );
}

/// Drift detection: if the user changes args (or someone hand-edits the
/// block), the next install rewrites the block in place and preserves
/// everything outside it.
#[test]
fn install_merge_driver_rewrites_block_on_drift() {
    let dir = init_repo();
    let repo = dir.path();

    // First install at TODO.md.
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--install-merge-driver")
        .assert()
        .success();

    // User pastes more rules around our block.
    let mut attrs = read(&repo.join(".gitattributes"));
    attrs.push_str("\n*.png binary\n");
    fs::write(repo.join(".gitattributes"), &attrs).unwrap();

    // Second install at a DIFFERENT path → the block must move to the new
    // path; the user's appended rule must remain.
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--todo-path")
        .arg("docs/TODOS.md")
        .arg("--install-merge-driver")
        .assert()
        .success();

    let attrs = read(&repo.join(".gitattributes"));
    assert!(
        attrs.contains("docs/TODOS.md merge=rusty-todo-md"),
        "new path's rule missing:\n{attrs}"
    );
    assert!(
        !attrs.contains("TODO.md merge=rusty-todo-md"),
        "old path's rule must be removed when the block is rewritten:\n{attrs}"
    );
    assert!(
        attrs.contains("*.png binary"),
        "user's outside-block rule must be preserved:\n{attrs}"
    );
}

#[test]
fn auto_install_flag_registers_driver_on_first_run_then_silent() {
    let dir = init_repo();
    let repo = dir.path();
    fs::write(repo.join("a.rs"), "// TODO: one\n").unwrap();
    git_must(repo, &["add", "."]);
    git_must(repo, &["commit", "-q", "-m", "src"]);

    // First run: registers the driver, prints loud summary.
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--auto-install-merge-driver")
        .arg("--")
        .arg("a.rs")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "reconciling merge driver registration",
        ));

    // Second run: driver already in sync, no reconcile message.
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--auto-install-merge-driver")
        .arg("--")
        .arg("a.rs")
        .assert()
        .success()
        .stdout(predicates::str::contains("reconciling merge driver registration").not());
}

/// Drift through pre-commit args: simulates the scenario where someone
/// changes `--markers` in `.pre-commit-config.yaml`. The next auto-install
/// run detects the mismatch in `.git/config` and rewrites the driver
/// command; the run after that is silent again.
#[test]
fn auto_install_self_heals_on_args_change() {
    let dir = init_repo();
    let repo = dir.path();
    fs::write(repo.join("a.rs"), "// TODO: one\n").unwrap();
    git_must(repo, &["add", "."]);
    git_must(repo, &["commit", "-q", "-m", "src"]);

    // First run: defaults (TODO only).
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--auto-install-merge-driver")
        .arg("--")
        .arg("a.rs")
        .assert()
        .success();
    let driver_v1 = read(&repo.join(".git").join("config"));
    assert!(
        driver_v1.contains("--markers TODO"),
        "v1 driver should bake the default marker verbatim:\n{driver_v1}"
    );

    // Second run: pre-commit args changed to include FIXME.
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--markers")
        .arg("TODO")
        .arg("FIXME")
        .arg("--auto-install-merge-driver")
        .arg("--")
        .arg("a.rs")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "reconciling merge driver registration",
        ));
    let driver_v2 = read(&repo.join(".git").join("config"));
    assert!(
        driver_v2.contains("--markers TODO FIXME"),
        "v2 driver should bake the new markers:\n{driver_v2}"
    );

    // Third run with same v2 args: silent.
    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--markers")
        .arg("TODO")
        .arg("FIXME")
        .arg("--auto-install-merge-driver")
        .arg("--")
        .arg("a.rs")
        .assert()
        .success()
        .stdout(predicates::str::contains("reconciling merge driver registration").not());
}

#[test]
fn source_files_with_conflict_markers_are_skipped() {
    let dir = init_repo();
    let repo = dir.path();
    // a source file containing a conflict marker — must not produce garbled TODOs.
    fs::write(
        repo.join("a.rs"),
        "// TODO: real\n<<<<<<< HEAD\n// TODO: ours-side\n=======\n// TODO: theirs-side\n>>>>>>> b\n",
    )
    .unwrap();
    git_must(repo, &["add", "."]);
    git_must(repo, &["commit", "-q", "-m", "src"]);

    AssertCommand::new(bin())
        .current_dir(repo)
        .arg("--regenerate")
        .assert()
        .success()
        .stderr(predicates::str::contains("contains conflict markers"));

    let todo = read(&repo.join("TODO.md"));
    assert!(
        !todo.contains("ours-side") && !todo.contains("theirs-side") && !todo.contains("real"),
        "skipped file's TODOs must not appear in TODO.md; got:\n{todo}"
    );
}

/// End-to-end rebase reproducer — the scenario the design exists to fix.
///
/// Two branches both insert lines above an existing TODO, shifting its line
/// number. Without the driver, git's text merge sees the same TODO.md line
/// edited differently on each side → conflict. With the driver, the merge
/// regenerates TODO.md from working-tree source and resolves cleanly.
#[test]
fn rebase_without_driver_conflicts_with_driver_clean() {
    let scenario = |with_driver: bool| -> Vec<String> {
        let dir = init_repo();
        let repo = dir.path();

        // Base: a source file with two TODOs, plus a generated TODO.md.
        fs::write(
            repo.join("a.rs"),
            "// TODO: alpha\nfn a(){}\nfn b(){}\n// TODO: beta\n",
        )
        .unwrap();
        run_tool_on(repo, &["a.rs"]);
        git_must(repo, &["add", "."]);
        git_must(repo, &["commit", "-q", "-m", "base"]);

        if with_driver {
            AssertCommand::new(bin())
                .current_dir(repo)
                .arg("--install-merge-driver")
                .assert()
                .success();
            // The installed driver uses the bare command name `rusty-todo-md`,
            // resolved via PATH. In tests we can't assume PATH points at the
            // freshly-built test binary (a stale system install often shadows
            // it), so rewrite the driver registration to the absolute path of
            // the test binary.
            let bin_str = bin().display().to_string();
            git_must(
                repo,
                &[
                    "config",
                    "merge.rusty-todo-md.driver",
                    &format!("{bin_str} --merge-driver %O %A %B"),
                ],
            );
            // Re-commit .gitattributes so it's on both branches.
            git_must(repo, &["add", ".gitattributes"]);
            git_must(repo, &["commit", "-q", "-m", "add gitattr"]);
        }

        // Branch feat-a: insert lines above alpha, shift everything down.
        git_must(repo, &["checkout", "-q", "-b", "feat-a"]);
        fs::write(
            repo.join("a.rs"),
            "// new-a-1\n// new-a-2\n// TODO: alpha\nfn a(){}\nfn b(){}\n// TODO: beta\n// TODO: gamma-a\n",
        )
        .unwrap();
        run_tool_on(repo, &["a.rs"]);
        git_must(repo, &["add", "."]);
        git_must(repo, &["commit", "-q", "-m", "feat-a"]);

        // Branch feat-b: from main, insert different lines.
        git_must(repo, &["checkout", "-q", "main"]);
        git_must(repo, &["checkout", "-q", "-b", "feat-b"]);
        fs::write(
            repo.join("a.rs"),
            "// new-b-1\n// TODO: alpha\nfn a(){}\nfn b(){}\n// TODO: beta\n// TODO: gamma-b\n",
        )
        .unwrap();
        run_tool_on(repo, &["a.rs"]);
        git_must(repo, &["add", "."]);
        git_must(repo, &["commit", "-q", "-m", "feat-b"]);

        // Rebase feat-b onto feat-a. Without the driver, this conflicts on
        // TODO.md (and on a.rs — that's a real semantic conflict we must
        // resolve manually to let the test proceed).
        let rebase = git(repo, &["rebase", "feat-a"]);
        let mut events = Vec::new();
        if !rebase.status.success() {
            events.push(String::from_utf8_lossy(&rebase.stderr).to_string());
            // Resolve the source-file conflict manually so we can observe the
            // TODO.md outcome.
            fs::write(
                repo.join("a.rs"),
                "// new-a-1\n// new-a-2\n// new-b-1\n// TODO: alpha\nfn a(){}\nfn b(){}\n// TODO: beta\n// TODO: gamma-a\n// TODO: gamma-b\n",
            )
            .unwrap();

            // If TODO.md was conflicted, resolve it manually too (this is what
            // the user has to do today, in the absence of the driver).
            let status = git(repo, &["status", "--porcelain"]);
            let status_str = String::from_utf8_lossy(&status.stdout).to_string();
            if status_str
                .lines()
                .any(|l| l.starts_with("UU ") && l.contains("TODO.md"))
            {
                events.push("TODO.md was UU (conflicted)".to_string());
                // Regenerate via the tool to canonicalize.
                AssertCommand::new(bin())
                    .current_dir(repo)
                    .arg("--regenerate")
                    .assert()
                    .success();
            }
            git_must(repo, &["add", "."]);
            // GIT_EDITOR=true makes `git rebase --continue` skip the commit
            // message editor.
            let cont = Command::new("git")
                .current_dir(repo)
                .args(["rebase", "--continue"])
                .env("GIT_EDITOR", "true")
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .expect("git rebase --continue failed to spawn");
            if !cont.status.success() {
                events.push(format!(
                    "rebase --continue failed: {}",
                    String::from_utf8_lossy(&cont.stderr)
                ));
                git(repo, &["rebase", "--abort"]);
            }
        }
        events
    };

    let without = scenario(false);
    let with = scenario(true);

    let saw_todo_conflict_without = without.iter().any(|e| e.contains("TODO.md was UU"));
    let saw_todo_conflict_with = with.iter().any(|e| e.contains("TODO.md was UU"));

    assert!(
        saw_todo_conflict_without,
        "expected TODO.md conflict without driver; events were: {without:?}"
    );
    assert!(
        !saw_todo_conflict_with,
        "did not expect TODO.md conflict with driver installed; events were: {with:?}"
    );
}

// `predicates` re-exports we need at the top level — silence unused import
// warning by referencing them through `use`-trick.
#[allow(unused_imports)]
use predicates::prelude::PredicateBooleanExt;
