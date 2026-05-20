//! Snapshot tests over a fixture corpus.
//!
//! Primary correctness signal for the extractor + TODO.md writer: each
//! fixture under `tests/fixtures/snapshots/<scenario>/` is copied into a
//! fresh git tempdir, the binary is run against it, and the resulting
//! `TODO.md` (plus, for some scenarios, stderr or the git index) is
//! compared to a checked-in `insta` snapshot. A diff means observable
//! output has changed — the test fails loudly and the snapshot is
//! reviewed (`cargo insta review`), not silently re-baselined.
//!
//! These tests deliberately exercise the binary via `assert_cmd`, mirroring
//! the pre-commit invocation shape. With `cargo-llvm-cov` instrumenting
//! subprocess execution, every line touched here counts toward coverage —
//! so this is the canonical place to assert *output* correctness, leaving
//! the unit tests to cover internal helpers.
//!
//! ## Harness shape
//!
//! Each scenario is a fresh `#[test] fn` that constructs a [`Scenario`]
//! and calls [`Scenario::run`]. The returned [`RunOutput`] is then fed to
//! `insta::assert_snapshot!` in the test function (not the helper, so insta
//! picks up the test name correctly for the `.snap` filename).
//!
//! Fixture directory layout:
//! - flat scenario: `tests/fixtures/snapshots/<name>/*` — all files are
//!   committed, then the binary runs once.
//! - multi-step scenario: `tests/fixtures/snapshots/<name>/step1/*` and
//!   `tests/fixtures/snapshots/<name>/step2/*`. `step1/` is committed and
//!   the binary runs; then `step2/` is overlaid (additions, modifications)
//!   and an optional `step2/.delete` lists files to remove; then the
//!   binary runs again. The snapshot is taken after the second run.

use assert_cmd::Command;
use std::collections::BTreeSet;
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

/// Recursively copy every regular file from `src` into `dst`, preserving
/// the relative directory structure. Returns the list of relative paths
/// copied (sorted, forward slashes).
fn copy_tree(src: &Path, dst: &Path) -> Vec<String> {
    let mut names = Vec::new();
    copy_tree_inner(src, dst, Path::new(""), &mut names);
    names.sort();
    names
}

fn copy_tree_inner(src: &Path, dst: &Path, rel: &Path, names: &mut Vec<String>) {
    for entry in fs::read_dir(src).expect("read fixture dir") {
        let entry = entry.expect("read fixture entry");
        let path = entry.path();
        let file_name = entry.file_name();
        let rel_child = rel.join(&file_name);
        if path.is_dir() {
            fs::create_dir_all(dst.join(&rel_child)).expect("create fixture subdir");
            copy_tree_inner(&path, dst, &rel_child, names);
        } else if path.is_file() {
            // `.delete` is a manifest, not a payload to copy.
            if file_name == ".delete" {
                continue;
            }
            if let Some(parent) = rel_child.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(dst.join(parent)).expect("create fixture parent dir");
                }
            }
            fs::copy(&path, dst.join(&rel_child)).expect("copy fixture file");
            names.push(rel_child.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// `git init` + minimal config + empty initial commit so HEAD exists. The
/// binary requires a real repo with a commit; nothing here cares about the
/// commit's content.
fn init_repo(at: &Path) {
    git(at, &["init", "-q", "-b", "main"]);
    git(at, &["config", "user.email", "t@t"]);
    git(at, &["config", "user.name", "t"]);
    git(at, &["commit", "-q", "--allow-empty", "-m", "init"]);
}

fn git(at: &Path, args: &[&str]) {
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
}

fn git_output(at: &Path, args: &[&str]) -> String {
    let out = StdCommand::new("git")
        .current_dir(at)
        .args(args)
        .output()
        .expect("git spawn");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// One snapshot scenario. Defaults match the original five-fixture harness:
/// single-step, init a git repo, commit fixture files, run with
/// `--markers TODO FIXME HACK --` followed by the fixture file names, and
/// expect a successful exit.
pub struct Scenario {
    name: &'static str,
    args: Vec<String>,
    /// If `true`, append the names of the fixture files (relative to the
    /// repo root) after `args`. If `false`, `args` is the complete CLI.
    pass_files: bool,
    expect_failure: bool,
    init_git: bool,
    /// `true` when the fixture is split into `step1/` and `step2/` subdirs.
    multi_step: bool,
    /// Snapshot `git diff --cached --name-only` (sorted) after the run.
    capture_git_index: bool,
    /// Path (relative to the repo root) to read for the `todo_md` snapshot.
    /// Defaults to `TODO.md`; override when the scenario passes a custom
    /// `--todo-path` so the captured content actually reflects what the
    /// binary wrote.
    todo_path: &'static str,
}

impl Scenario {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            args: vec![
                "--markers".into(),
                "TODO".into(),
                "FIXME".into(),
                "HACK".into(),
                "--".into(),
            ],
            pass_files: true,
            expect_failure: false,
            init_git: true,
            multi_step: false,
            capture_git_index: false,
            todo_path: "TODO.md",
        }
    }

    pub fn todo_path(mut self, path: &'static str) -> Self {
        self.todo_path = path;
        self
    }

    /// Override the CLI args (everything before the file list, unless
    /// `pass_files(false)` is also set in which case this is the full CLI).
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// If set, the fixture file names are NOT appended after `args`. Use
    /// when the scenario passes its inputs implicitly (e.g. `--regenerate`,
    /// or scenarios that exercise tracked-files discovery).
    pub fn no_file_args(mut self) -> Self {
        self.pass_files = false;
        self
    }

    pub fn expect_failure(mut self) -> Self {
        self.expect_failure = true;
        self
    }

    /// Don't `git init` the tempdir. Required for fixtures that assert the
    /// "not a git repository" error path.
    pub fn no_git(mut self) -> Self {
        self.init_git = false;
        self
    }

    pub fn multi_step(mut self) -> Self {
        self.multi_step = true;
        self
    }

    pub fn capture_git_index(mut self) -> Self {
        self.capture_git_index = true;
        self
    }

    /// Drive the binary against the fixture and return the captured output.
    pub fn run(self) -> RunOutput {
        let temp = tempdir().expect("tempdir");

        let (step1_dir, step2_dir) = if self.multi_step {
            (
                fixture_dir(self.name).join("step1"),
                Some(fixture_dir(self.name).join("step2")),
            )
        } else {
            (fixture_dir(self.name), None)
        };

        let initial_files = copy_tree(&step1_dir, temp.path());

        if self.init_git {
            init_repo(temp.path());
            if !initial_files.is_empty() {
                git(temp.path(), &["add", "-A"]);
                git(temp.path(), &["commit", "-q", "-m", "fixture"]);
            }
        }

        let run1_files = if self.pass_files {
            filter_source_files(&initial_files, self.todo_path)
        } else {
            Vec::new()
        };
        run_binary(temp.path(), &self.args, &run1_files, self.expect_failure);

        if let Some(step2) = step2_dir {
            apply_overlay(temp.path(), &step2);
            let run2_files = if self.pass_files {
                // Pass all current top-level files (excluding TODO.md and
                // .git) — mirrors how pre-commit would invoke us on the
                // changed file set.
                let all = gather_run2_files(temp.path(), &initial_files);
                filter_source_files(&all, self.todo_path)
            } else {
                Vec::new()
            };
            run_binary(temp.path(), &self.args, &run2_files, self.expect_failure);
        }

        // The most-recent binary invocation's stderr is stashed in
        // `LAST_STDERR` by `run_binary`; pull it out for the snapshot.
        let stderr = LAST_STDERR.with(|s| s.borrow().clone());

        let todo_path = temp.path().join(self.todo_path);
        let todo_md = if todo_path.exists() {
            fs::read_to_string(&todo_path).expect("read TODO.md")
        } else {
            String::from("<no TODO.md generated>\n")
        };

        let git_index = if self.capture_git_index {
            let raw = git_output(temp.path(), &["diff", "--cached", "--name-only"]);
            let mut lines: Vec<&str> = raw.lines().collect();
            lines.sort();
            Some(lines.join("\n") + "\n")
        } else {
            None
        };

        RunOutput {
            _temp: temp,
            todo_md,
            stderr,
            git_index,
        }
    }
}

thread_local! {
    /// Holds the stderr captured from the most recent binary invocation
    /// inside `Scenario::run`. Tests are single-threaded per `#[test] fn`,
    /// so a thread-local is sufficient and avoids threading a return value
    /// through the multi-step branch.
    static LAST_STDERR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Drop entries from the fixture's file list that aren't meant to be
/// scanned: the TODO.md file the binary will write to, and the
/// `.gitattributes` infrastructure file. Anything else (source files,
/// pre-shipped configuration, etc.) is passed through.
fn filter_source_files(files: &[String], todo_path: &str) -> Vec<String> {
    files
        .iter()
        .filter(|f| f.as_str() != todo_path && f.as_str() != ".gitattributes")
        .cloned()
        .collect()
}

fn run_binary(cwd: &Path, args: &[String], files: &[String], expect_failure: bool) {
    let mut cmd = Command::cargo_bin("rusty-todo-md").expect("locate binary");
    cmd.current_dir(cwd);
    for a in args {
        cmd.arg(a);
    }
    for f in files {
        cmd.arg(f);
    }
    let assert = cmd.assert();
    let assert = if expect_failure {
        assert.failure()
    } else {
        assert.success()
    };
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    LAST_STDERR.with(|s| *s.borrow_mut() = stderr);
}

/// Apply a `step2/` overlay onto the working tree at `cwd`:
/// - every regular file under `step2/` overwrites the corresponding file
///   (parent dirs created on demand);
/// - if `step2/.delete` exists, every non-empty line is treated as a
///   path relative to the repo root and the file is removed.
fn apply_overlay(cwd: &Path, step2: &Path) {
    if !step2.exists() {
        return;
    }
    let _ = copy_tree(step2, cwd);

    let delete_manifest = step2.join(".delete");
    if delete_manifest.exists() {
        let contents = fs::read_to_string(&delete_manifest).expect("read .delete");
        for line in contents.lines() {
            let rel = line.trim();
            if rel.is_empty() || rel.starts_with('#') {
                continue;
            }
            let target = cwd.join(rel);
            if target.exists() {
                fs::remove_file(&target).expect("apply .delete");
            }
        }
    }
}

/// After the step2 overlay, collect the set of fixture-payload paths
/// currently present so we can invoke the binary on them. We pass every
/// file that was in step1 (regardless of whether step2 deleted or modified
/// it — pre-commit also re-passes deleted paths) PLUS any new files step2
/// introduced.
fn gather_run2_files(cwd: &Path, initial_files: &[String]) -> Vec<String> {
    let mut set: BTreeSet<String> = initial_files.iter().cloned().collect();
    // Sweep top-level + nested files except .git, .gitattributes, TODO.md.
    sweep(cwd, Path::new(""), &mut set);
    set.into_iter().collect()
}

fn sweep(cwd: &Path, rel: &Path, set: &mut BTreeSet<String>) {
    let dir = cwd.join(rel);
    for entry in fs::read_dir(&dir).expect("sweep read_dir") {
        let entry = entry.expect("sweep entry");
        let path = entry.path();
        let name = entry.file_name();
        if name == ".git" || name == "TODO.md" || name == ".gitattributes" {
            continue;
        }
        let rel_child = rel.join(&name);
        if path.is_dir() {
            sweep(cwd, &rel_child, set);
        } else if path.is_file() {
            set.insert(rel_child.to_string_lossy().replace('\\', "/"));
        }
    }
}

pub struct RunOutput {
    /// Holds the tempdir alive until the test reads from it; the tempdir
    /// is dropped (and the directory removed) when `RunOutput` is dropped.
    _temp: TempDir,
    pub todo_md: String,
    pub stderr: String,
    /// `git diff --cached --name-only` (sorted), only set when
    /// `capture_git_index()` was requested.
    pub git_index: Option<String>,
}

// ---------------------------------------------------------------------------
// Baseline (pre-existing) scenarios
// ---------------------------------------------------------------------------

// These five tests bind the captured `TODO.md` to a local named `out`
// (rather than `result.todo_md`) so the `expression: out` metadata in the
// pre-existing `.snap` files stays byte-identical after the harness migration.

#[test]
fn rust_basic() {
    let out = Scenario::new("rust_basic").run().todo_md;
    insta::assert_snapshot!(out);
}

#[test]
fn python_basic() {
    let out = Scenario::new("python_basic").run().todo_md;
    insta::assert_snapshot!(out);
}

#[test]
fn mixed_languages() {
    let out = Scenario::new("mixed_languages").run().todo_md;
    insta::assert_snapshot!(out);
}

#[test]
fn awkward_positions() {
    let out = Scenario::new("awkward_positions").run().todo_md;
    insta::assert_snapshot!(out);
}

#[test]
fn no_markers() {
    let out = Scenario::new("no_markers").run().todo_md;
    insta::assert_snapshot!(out);
}

// ---------------------------------------------------------------------------
// Reason-class 1 + 2: stderr / non-zero exit / error paths
// (see docs/experiments/test-pruning-202/triage-verdicts.md)
// ---------------------------------------------------------------------------

#[test]
fn non_git_directory_fails() {
    // No `--markers` here; the binary should fail to open the repo before
    // it ever scans a file. Mirrors `cli_error_tests::test_run_cli_in_non_git_directory`.
    let out = Scenario::new("non_git_directory_fails")
        .args(["--todo-path", "TODO.md"])
        .no_git()
        .expect_failure()
        .run();
    insta::with_settings!({snapshot_suffix => "stderr"}, {
        let stderr = scrub_stderr(&out.stderr);
        insta::assert_snapshot!(stderr);
    });
}

#[test]
fn empty_todo_marker_fails() {
    // A `TODO:` with no message must fail validation and print the
    // `empty TODO comment found` advisory. Mirrors
    // `empty_todo_validation_tests::test_empty_todo_detection`.
    let out = Scenario::new("empty_todo_marker_fails")
        .expect_failure()
        .run();
    insta::assert_snapshot!(out.todo_md);
    insta::with_settings!({snapshot_suffix => "stderr"}, {
        let stderr = scrub_stderr(&out.stderr);
        insta::assert_snapshot!(stderr);
    });
}

#[test]
fn empty_python_todo_marker_fails() {
    // Same as above but in Python — exercises a different parser path.
    let out = Scenario::new("empty_python_todo_marker_fails")
        .expect_failure()
        .run();
    insta::assert_snapshot!(out.todo_md);
    insta::with_settings!({snapshot_suffix => "stderr"}, {
        let stderr = scrub_stderr(&out.stderr);
        insta::assert_snapshot!(stderr);
    });
}

// ---------------------------------------------------------------------------
// Reason-class 3 + 4: custom flags + glob exclusions
// ---------------------------------------------------------------------------

#[test]
fn custom_markers_only_todo() {
    // Only `TODO` is registered as a marker — `FIXME` / `HACK` lines must
    // be ignored. Mirrors `integration::test_markers_arg_parsing`.
    let out = Scenario::new("custom_markers_only_todo")
        .args(["--markers", "TODO", "--"])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn custom_todo_path() {
    // `--todo-path` writes to a non-default location. The fixture seeds an
    // empty `todos/` directory placeholder so the parent path exists; the
    // harness reads from the custom location to verify the binary wrote
    // there. Mirrors `integration::test_markers_arg_parsing` /
    // `test_process_files_list_single_run`.
    let out = Scenario::new("custom_todo_path")
        .args(["--todo-path", "MY_TODO.md", "--markers", "TODO", "--"])
        .todo_path("MY_TODO.md")
        .run()
        .todo_md;
    insta::assert_snapshot!(out);
}

#[test]
fn auto_add_stages_todo_md() {
    // `--auto-add` should stage the generated TODO.md. Snapshot the
    // resulting `git diff --cached --name-only`. Mirrors
    // `integration::test_auto_add_functionality`.
    let out = Scenario::new("auto_add_stages_todo_md")
        .args(["--auto-add", "--markers", "TODO", "--"])
        .capture_git_index()
        .run();
    insta::assert_snapshot!(out.todo_md);
    insta::with_settings!({snapshot_suffix => "git_index"}, {
        let git_index = out.git_index.as_deref().unwrap_or("").to_string();
        insta::assert_snapshot!(git_index);
    });
}

#[test]
fn exclude_glob_recursive() {
    // `--exclude src/**` skips everything under src/. Mirrors
    // `glob_exclude_tests::test_glob_exclude_recursive_wildcard`.
    let out = Scenario::new("exclude_glob_recursive")
        .args(["--exclude", "src/**", "--markers", "TODO", "--"])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn exclude_dir_flag() {
    // `--exclude-dir build` skips the build/ directory. Mirrors
    // `integration::test_exclude_files_with_glob_patterns`.
    let out = Scenario::new("exclude_dir_flag")
        .args([
            "--exclude",
            "src/",
            "--exclude-dir",
            "build",
            "--markers",
            "TODO",
            "--",
        ])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

// ---------------------------------------------------------------------------
// Reason-class 5 + 6: language parsers and multi-line block-comment joining
// ---------------------------------------------------------------------------

#[test]
fn go_with_block_comments() {
    // Go single-line + multi-line block comment with multi-line marker
    // continuation. Mirrors `multi_language_tests::test_go_with_mixed_comments`.
    let out = Scenario::new("go_with_block_comments")
        .args(["--markers", "TODO", "FIXME", "--"])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn dockerfile_multiple_markers() {
    // Dockerfile parser with TODO/FIXME/HACK. Mirrors
    // `multi_language_tests::test_dockerfile_with_multiple_markers`.
    let out = Scenario::new("dockerfile_multiple_markers")
        .args(["--markers", "TODO", "FIXME", "HACK", "--"])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn jsx_with_markers() {
    // JSX file (.jsx extension) routes through the JS parser. Mirrors
    // the JSX portion of `multi_language_tests::test_mixed_language_todo_extraction`.
    let out = Scenario::new("jsx_with_markers")
        .args(["--markers", "TODO", "FIXME", "--"])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn js_block_comment_continuation() {
    // `/* FIXME: line1\n   line2 */` must be captured AND joined with a
    // single space. Mirrors `multi_language_tests::test_js_with_fixme_markers`.
    let out = Scenario::new("js_block_comment_continuation")
        .args(["--markers", "TODO", "FIXME", "--"])
        .run();
    insta::assert_snapshot!(out.todo_md);
}

// ---------------------------------------------------------------------------
// Reason-class 7: second-run merge / file-change / file-removal
// ---------------------------------------------------------------------------

#[test]
fn second_run_message_changes() {
    // First run sees "Initial implementation", second sees "Updated".
    // The old entry must be replaced. Mirrors
    // `integration::test_update_todo_md_on_file_change`.
    let out = Scenario::new("second_run_message_changes")
        .args(["--markers", "TODO", "--"])
        .multi_step()
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn second_run_file_no_longer_has_todo() {
    // First run has a TODO; second run rewrites the file to no longer
    // contain one. The file's section must be removed. Mirrors
    // `integration::test_update_todo_md_on_file_removal`.
    let out = Scenario::new("second_run_file_no_longer_has_todo")
        .args(["--markers", "TODO", "--"])
        .multi_step()
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn second_run_file_deleted() {
    // step2 deletes one of two files via `.delete` manifest; the deleted
    // file's section must be removed but the other file's must stay.
    // Mirrors `integration::test_multiple_files_update`.
    let out = Scenario::new("second_run_file_deleted")
        .args(["--markers", "TODO", "--"])
        .multi_step()
        .run();
    insta::assert_snapshot!(out.todo_md);
}

// ---------------------------------------------------------------------------
// Reason-class 9 (partial): conflict markers in pre-shipped files
// (auto_install_* merge-driver fixtures are EXCLUDED by design)
// ---------------------------------------------------------------------------

#[test]
fn todo_md_conflict_markers_emit_advisory() {
    // Pre-shipped TODO.md contains literal `<<<<<<<` markers. The default
    // scan must emit the "detected conflict markers in TODO.md" advisory
    // on stderr. The binary recovers via the sync-fallback rescan and
    // exits successfully, so we don't assert a failure code — only that
    // the advisory text reaches stderr. Mirrors
    // `merge_driver_tests::regenerate_advisory_printed_when_todo_md_has_conflict_markers`.
    let out = Scenario::new("todo_md_conflict_markers_emit_advisory")
        .args(["--"])
        .run();
    insta::with_settings!({snapshot_suffix => "stderr"}, {
        let stderr = scrub_stderr(&out.stderr);
        insta::assert_snapshot!(stderr);
    });
}

#[test]
fn regenerate_wipes_todo_md_conflict_markers() {
    // `--regenerate` rebuilds TODO.md from scratch from tracked source
    // files, wiping any prior conflict markers. Mirrors
    // `merge_driver_tests::regenerate_wipes_conflict_markers`.
    let out = Scenario::new("regenerate_wipes_todo_md_conflict_markers")
        .args(["--regenerate", "--markers", "TODO", "FIXME"])
        .no_file_args()
        .run();
    insta::assert_snapshot!(out.todo_md);
}

#[test]
fn source_file_with_conflict_markers_is_skipped() {
    // A source file containing `<<<<<<<` must be skipped (with a stderr
    // notice) so its half-merged TODOs don't appear in TODO.md. Mirrors
    // `merge_driver_tests::source_files_with_conflict_markers_are_skipped`.
    let out = Scenario::new("source_file_with_conflict_markers_is_skipped")
        .args(["--regenerate", "--markers", "TODO"])
        .no_file_args()
        .run();
    insta::assert_snapshot!(out.todo_md);
    insta::with_settings!({snapshot_suffix => "stderr"}, {
        let stderr = scrub_stderr(&out.stderr);
        insta::assert_snapshot!(stderr);
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalize captured stderr so the snapshot is stable across runs.
///
/// `env_logger` formats messages as
/// `2026-05-20T00:01:02Z LEVEL [crate::mod - src/file.rs:LINE] message`.
/// Both the timestamp and the source `file:line` shift unpredictably
/// (the timestamp every second; the line whenever the source file is
/// edited), so we replace them with stable placeholders. The level and
/// the human-readable message text are preserved verbatim.
fn scrub_stderr(s: &str) -> String {
    let ts_re = regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z").unwrap();
    let line_re = regex::Regex::new(r"(\.rs):\d+\]").unwrap();
    let mut out = ts_re.replace_all(s, "<TS>").into_owned();
    out = line_re.replace_all(&out, "$1:<LINE>]").into_owned();
    out
}
