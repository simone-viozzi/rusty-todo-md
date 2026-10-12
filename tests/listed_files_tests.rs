//! Each run rescans the files TODO.md lists, end to end.
//!
//! Line numbers in TODO.md shift on every branch, so a rebase would stop on
//! TODO.md at nearly every replayed commit. With `merge=union` (built into
//! git) it never stops; the merged TODO.md is wrong (old and new entries
//! side by side), and the next hook run rebuilds it by rescanning every
//! file it lists. Only tracked files are rescanned: TODO.md is committed
//! input and must not make the hook read anything else.

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

/// Union can also produce output that is already sorted: if one side moves
/// a TODO from line 10 to 11 and the other to 12, union keeps the line-11
/// entry followed by the line-12 one. Nothing about the entries themselves
/// looks wrong; the rebuild must still happen.
#[test]
fn union_leftovers_in_sorted_order_are_rebuilt() {
    let dir = tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main"]);
    fs::write(repo.join(".gitattributes"), "TODO.md merge=union\n").unwrap();
    let head: String = (1..=4).map(|i| format!("fn h{i}() {{}}\n")).collect();
    let mid: String = (1..=4).map(|i| format!("fn m{i}() {{}}\n")).collect();
    fs::write(repo.join("a.rs"), format!("{head}{mid}// TODO: alpha\n")).unwrap();
    fs::write(repo.join("b.rs"), "fn b() {}\n").unwrap();
    commit(repo, &["a.rs", "b.rs"], "base");

    // Non-overlapping source edits that both move `alpha` down: one line at
    // the top on `onto`, two lines in the middle on `feat`.
    git(repo, &["checkout", "-q", "-b", "onto"]);
    fs::write(
        repo.join("a.rs"),
        format!("// top\n{head}{mid}// TODO: alpha\n"),
    )
    .unwrap();
    commit(repo, &["a.rs"], "onto: +1 line");

    git(repo, &["checkout", "-q", "main"]);
    git(repo, &["checkout", "-q", "-b", "feat"]);
    fs::write(
        repo.join("a.rs"),
        format!("{head}// mid-1\n// mid-2\n{mid}// TODO: alpha\n"),
    )
    .unwrap();
    commit(repo, &["a.rs"], "feat: +2 lines");

    git(repo, &["rebase", "onto"]);
    fs::write(repo.join("b.rs"), "fn b() {}\nfn c() {}\n").unwrap();
    hook(repo, &["b.rs"]);
    let rebuilt = fs::read_to_string(repo.join("TODO.md")).unwrap();

    assert_eq!(
        rebuilt.matches("alpha").count(),
        1,
        "stale union entries survived:\n{rebuilt}"
    );
    assert!(rebuilt.contains("[a.rs:12](a.rs#L12): alpha"), "{rebuilt}");
}

/// The CLI accepts absolute paths (manual runs). They are recorded
/// repository-relative, so a later run for another file still finds the
/// first file in the index and keeps its entries.
#[test]
fn absolute_paths_are_recorded_relative_and_kept() {
    let dir = tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main"]);
    fs::write(repo.join("a.rs"), "// TODO: alpha\n").unwrap();
    fs::write(repo.join("b.rs"), "// TODO: beta\n").unwrap();
    git(repo, &["add", "a.rs", "b.rs"]);
    git(repo, &["commit", "-q", "-m", "src"]);

    let absolute = repo.join("a.rs").canonicalize().unwrap();
    hook(repo, &[absolute.to_str().unwrap()]);
    hook(repo, &["b.rs"]);

    let todo = fs::read_to_string(repo.join("TODO.md")).unwrap();
    assert!(todo.contains("* [a.rs:1](a.rs#L1): alpha"), "{todo}");
    assert!(todo.contains("* [b.rs:1](b.rs#L1): beta"), "{todo}");
}

/// A crafted TODO.md must not make the hook read files outside what git
/// tracks: not a path outside the repository, and not an untracked file
/// inside it. Otherwise their TODO lines would be copied into TODO.md.
#[test]
fn listed_files_outside_the_tracked_set_are_not_read() {
    let parent = tempdir().unwrap();
    fs::write(
        parent.path().join("outside.rs"),
        "// TODO: outside secret\n",
    )
    .unwrap();
    let repo = parent.path().join("repo");
    fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    fs::write(repo.join("a.rs"), "// TODO: tracked\n").unwrap();
    fs::write(
        repo.join("TODO.md"),
        "# TODOs\n\n## TODO\n### a.rs\n* [a.rs:1](a.rs#L1): tracked\n\n### ../outside.rs\n* [../outside.rs:1](../outside.rs#L1): x\n\n### secret.rs\n* [secret.rs:1](secret.rs#L1): x\n",
    )
    .unwrap();
    git(&repo, &["add", "a.rs", "TODO.md"]);
    git(&repo, &["commit", "-q", "-m", "crafted TODO.md"]);
    fs::write(repo.join("secret.rs"), "// TODO: untracked secret\n").unwrap();

    fs::write(repo.join("a.rs"), "// TODO: tracked\nfn a() {}\n").unwrap();
    hook(&repo, &["a.rs"]);

    let todo = fs::read_to_string(repo.join("TODO.md")).unwrap();
    assert!(todo.contains("tracked"), "{todo}");
    assert!(
        !todo.contains("outside secret") && !todo.contains("untracked secret"),
        "{todo}"
    );
}
