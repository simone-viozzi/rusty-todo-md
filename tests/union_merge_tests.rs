//! `TODO.md merge=union` + rebuild-on-non-canonical, end to end.
//!
//! Line numbers in TODO.md shift on every branch, so a rebase would stop on
//! TODO.md at nearly every replayed commit. With `merge=union` (built into
//! git) it never stops; the merged TODO.md is wrong but visibly so
//! (duplicate, out-of-order entries), and the next hook run rebuilds it.

use assert_cmd::Command as AssertCommand;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .env("GIT_EDITOR", "true")
        .output()
        .expect("git failed to spawn");
    assert!(
        out.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The hook as pre-commit runs it on the staged files.
fn hook(repo: &Path, files: &[&str]) {
    AssertCommand::new(assert_cmd::cargo::cargo_bin("rusty-todo-md"))
        .current_dir(repo)
        .arg("--")
        .args(files)
        .assert()
        .success();
}

fn commit(repo: &Path, files: &[&str], msg: &str) {
    hook(repo, files);
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", msg]);
}

#[test]
fn union_rebase_never_stops_and_next_run_rebuilds_todo_md() {
    let dir = tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main"]);
    fs::write(repo.join(".gitattributes"), "TODO.md merge=union\n").unwrap();
    let body: String = (1..=20).map(|i| format!("fn f{i}() {{}}\n")).collect();
    fs::write(
        repo.join("a.rs"),
        format!("// TODO: alpha\n{body}// TODO: omega\n"),
    )
    .unwrap();
    fs::write(repo.join("b.rs"), "fn b() {}\n").unwrap();
    commit(repo, &["a.rs", "b.rs"], "base");

    // Both branches move the same TODOs to different lines, so their
    // TODO.md edits conflict line by line.
    git(repo, &["checkout", "-q", "-b", "onto"]);
    fs::write(
        repo.join("a.rs"),
        format!("// top-1\n// top-2\n// TODO: alpha\n{body}// TODO: omega\n"),
    )
    .unwrap();
    commit(repo, &["a.rs"], "onto: shift");

    git(repo, &["checkout", "-q", "main"]);
    git(repo, &["checkout", "-q", "-b", "feat"]);
    fs::write(
        repo.join("a.rs"),
        format!("// TODO: alpha\n{body}// TODO: omega\n// TODO: added-by-feat\n"),
    )
    .unwrap();
    commit(repo, &["a.rs"], "feat: new todo");

    // Never stops on TODO.md (the source merges cleanly).
    git(repo, &["rebase", "onto"]);
    let merged = fs::read_to_string(repo.join("TODO.md")).unwrap();

    // Next commit touches an unrelated file; the run still rebuilds TODO.md.
    fs::write(repo.join("b.rs"), "fn b() {}\nfn c() {}\n").unwrap();
    hook(repo, &["b.rs"]);
    let rebuilt = fs::read_to_string(repo.join("TODO.md")).unwrap();

    AssertCommand::new(assert_cmd::cargo::cargo_bin("rusty-todo-md"))
        .current_dir(repo)
        .arg("--regenerate")
        .assert()
        .success();
    let canonical = fs::read_to_string(repo.join("TODO.md")).unwrap();

    assert_ne!(merged, canonical, "union output should need a rebuild");
    assert_eq!(rebuilt, canonical);
    assert!(canonical.contains("added-by-feat"), "{canonical}");
}
