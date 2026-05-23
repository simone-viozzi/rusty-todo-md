//! Snapshot tests over a small fixture corpus.
//!
//! Primary correctness signal for the extractor + TODO.md writer: each
//! fixture under `tests/fixtures/snapshots/<scenario>/` is copied into a
//! fresh git tempdir, the binary is run against the fixture files, and the
//! resulting `TODO.md` is compared to a checked-in `insta` snapshot. A
//! diff means observable output has changed — the test fails loudly and
//! the snapshot is reviewed (`cargo insta review`), not silently re-baselined.
//!
//! These tests deliberately exercise the binary via `assert_cmd`, mirroring
//! the pre-commit invocation shape. With `cargo-llvm-cov` instrumenting
//! subprocess execution, every line touched here counts toward coverage —
//! so this is the canonical place to assert *output* correctness, leaving
//! the unit tests to cover internal helpers.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::{tempdir, TempDir};

/// Absolute path to a fixture directory under `tests/fixtures/snapshots/`.
fn fixture_dir(scenario: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/snapshots")
        .join(scenario)
}

/// Copy every regular file from `src` into `dst` (one level deep — fixtures
/// are flat by convention).
fn copy_fixture_files(src: &Path, dst: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for entry in fs::read_dir(src).expect("read fixture dir") {
        let entry = entry.expect("read fixture entry");
        let path = entry.path();
        if path.is_file() {
            let name = entry.file_name().into_string().expect("utf-8 filename");
            fs::copy(&path, dst.join(&name)).expect("copy fixture file");
            names.push(name);
        }
    }
    names.sort();
    names
}

/// `git init` + minimal config + empty initial commit so HEAD exists. The
/// binary requires a real repo with a commit; nothing here cares about the
/// commit's content.
fn init_repo(at: &Path) {
    let run = |args: &[&str]| {
        let out = StdCommand::new("git")
            .current_dir(at)
            .args(args)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .output()
            .expect("git spawn");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(&["init", "-q", "-b", "main"]);
    run(&["config", "user.email", "t@t"]);
    run(&["config", "user.name", "t"]);
    run(&["commit", "-q", "--allow-empty", "-m", "init"]);
}

/// Drive the binary against one fixture and return the resulting `TODO.md`
/// contents (or a sentinel when the tool produced no file).
fn run_fixture(scenario: &str) -> (TempDir, String) {
    let temp = tempdir().expect("tempdir");
    let files = copy_fixture_files(&fixture_dir(scenario), temp.path());
    init_repo(temp.path());

    let mut cmd = Command::cargo_bin("rusty-todo-md").expect("locate binary");
    cmd.current_dir(temp.path())
        .args(["--markers", "TODO", "FIXME", "HACK", "--"]);
    for f in &files {
        cmd.arg(f);
    }
    cmd.assert().success();

    let todo_path = temp.path().join("TODO.md");
    let body = if todo_path.exists() {
        fs::read_to_string(&todo_path).expect("read TODO.md")
    } else {
        String::from("<no TODO.md generated>\n")
    };
    (temp, body)
}

#[test]
fn rust_basic() {
    let (_dir, out) = run_fixture("rust_basic");
    insta::assert_snapshot!(out);
}

#[test]
fn python_basic() {
    let (_dir, out) = run_fixture("python_basic");
    insta::assert_snapshot!(out);
}

#[test]
fn mixed_languages() {
    let (_dir, out) = run_fixture("mixed_languages");
    insta::assert_snapshot!(out);
}

#[test]
fn awkward_positions() {
    let (_dir, out) = run_fixture("awkward_positions");
    insta::assert_snapshot!(out);
}

#[test]
fn no_markers() {
    let (_dir, out) = run_fixture("no_markers");
    insta::assert_snapshot!(out);
}
