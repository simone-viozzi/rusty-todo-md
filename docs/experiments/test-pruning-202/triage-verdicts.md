# Stage 3 subagent triage — verdicts

> **Hand-assembled file.** Each row was emitted verbatim by a Sonnet
> subagent invoked from the implementation session; the table here is a
> manual transcription of the 28 verdicts. Re-running the experiment
> would re-derive the verdicts but not regenerate this exact file. The
> per-candidate prompts the subagents read are reproducible from
> `scripts/build_subagent_prompts.py` against `overlap-data.json`.

One Sonnet subagent per `tests/*.rs` candidate with overlap ≥ 0.70.
Shared rubric: subsumption-review (KEEP bias by design — see the QA in
`arch-review` for why). Per-candidate prompts (generated at experiment
time under `coverage/subagent-prompts/`, not tracked) bundle the rubric,
a static snapshot-corpus digest, and the candidate test's source +
overlap data.

Result: **28 candidates reviewed, 0 DELETE, 28 KEEP.** No `tests/*.rs`
file is deleted in this PR. The honest reading is that the current
snapshot corpus — five fixtures, all happy-path, fixed `--markers TODO
FIXME HACK --` flags, no error paths, no flag combinations — does not
yet subsume any integration test. Every `tests/*.rs` candidate reaches
either an error path, a CLI flag combo, a multi-run / file-removal
update, merge-driver behavior, or an internal invariant that no
snapshot would fail on.

Calibration was done on `multi_language_tests::test_js_with_fixme_markers`
(line overlap 1.0000): the test asserts that a `/* FIXME: ...\n   ... */`
JS block-comment marker is captured AND joined with a single space, but
the snapshot corpus contains no fixture that asserts on multi-line
block-comment continuation in any language. A regression there would
not fail any snapshot. Verdict: KEEP. The rubric handled this exactly
as intended, so I proceeded to fan out the remaining 27 candidates.

## What every verdict tells us

The subagent verdicts converge on a small set of reasons:

| Reason | Where it shows up |
|---|---|
| stderr / error-message text | `cli_error_tests::*`, `merge_driver_tests::regenerate_advisory_*`, `merge_driver_tests::source_files_with_conflict_markers_are_skipped` |
| non-zero exit / error paths | `cli_error_tests::test_run_cli_in_non_git_directory`, `cli_error_tests::test_run_cli_with_unreadable_file` |
| custom `--markers` / `--todo-path` / `--auto-add` flags | `integration::test_markers_*`, `integration::test_process_files_list_single_run`, `integration::test_auto_add_functionality`, several `multi_language_tests::*` |
| `--exclude` / `--exclude-dir` glob handling | `glob_exclude_tests::*`, `integration::test_exclude_files_with_glob_patterns` |
| language parsers not in snapshot corpus (Go, Dockerfile, JSX) | `multi_language_tests::test_go_*`, `multi_language_tests::test_dockerfile_*`, `multi_language_tests::test_mixed_language_*` |
| multi-line block comment continuation | `multi_language_tests::test_js_with_fixme_markers` |
| second-run merge / file-removal / file-change update | `integration::test_multiple_runs_update`, `integration::test_update_todo_md_on_file_removal`, `integration::test_update_todo_md_on_file_change`, `integration::test_multiple_files_update` |
| empty-TODO validation (`validate_no_empty_todos`) | `empty_todo_validation_tests::*` |
| merge-driver install / reconcile / self-heal | `merge_driver_tests::auto_install_*`, `merge_driver_tests::regenerate_*` |
| stdout / git-config side-effects | `merge_driver_tests::auto_install_*` |

Each is a distinct gap in the snapshot corpus. Closing any of them
opens a deletion candidate; that work belongs in a follow-up that
expands the fixture set — explicitly **out of scope** for this PR per
the handover.

## Per-candidate verdicts

| overlap | candidate | verdict |
|---|---|---|
| 1.0000 | `multi_language_tests::test_js_with_fixme_markers` | KEEP: snapshot corpus never asserts multi-line JS block-comment continuation is captured and joined; a regression there would not fail any snapshot. |
| 1.0000 | `empty_todo_validation_tests::test_valid_todo_detection` | KEEP: asserts on `validate_no_empty_todos` internal invariant not covered by any snapshot fixture. |
| 1.0000 | `empty_todo_validation_tests::test_extract_empty_todos_directly` | KEEP: asserts on empty-message internal invariant not observable from TODO.md snapshot output |
| 0.9891 | `multi_language_tests::test_go_with_mixed_comments` | KEEP: exercises Go parser + custom --markers + custom --todo-path + multi-line block comment joining, none snapshot-covered |
| 0.9888 | `multi_language_tests::test_mixed_language_todo_extraction` | KEEP: exercises Go + JSX languages and custom --todo-path, none covered by snapshots |
| 0.9774 | `multi_language_tests::test_dockerfile_with_multiple_markers` | KEEP: exercises Dockerfile language parser + custom --markers/--todo-path, both outside snapshot corpus |
| 0.9519 | `integration::test_markers_arg_parsing` | KEEP: exercises custom --todo-path flag, which no snapshot fixture covers. |
| 0.9518 | `integration::test_process_files_list_single_run` | KEEP: uses custom `--todo-path` flag not exercised by any snapshot fixture. |
| 0.9450 | `empty_todo_validation_tests::test_empty_todo_detection` | KEEP: asserts on stderr error-message text and validate_no_empty_todos path not covered by snapshots |
| 0.9450 | `empty_todo_validation_tests::test_python_empty_todos` | KEEP: asserts on empty-comment error paths and stderr-like error text not covered by snapshots |
| 0.9246 | `cli_no_files_tests::test_run_cli_no_files` | KEEP: asserts exit-code 0 on no-files path; no snapshot exercises missing-file-args branch. |
| 0.9214 | `cli_error_tests::test_run_cli_with_unreadable_file` | KEEP: asserts stderr text and non-zero-success error path not covered by any snapshot fixture |
| 0.9138 | `merge_driver_tests::regenerate_advisory_printed_when_todo_md_has_conflict_markers` | KEEP: asserts stderr text ("detected conflict markers in TODO.md") — not covered by snapshots |
| 0.9137 | `integration::test_markers_with_separator` | KEEP: exercises `--` separator CLI parsing; no snapshot covers custom `--todo-path` + separator combo. |
| 0.9096 | `glob_exclude_tests::test_glob_exclude_recursive_wildcard` | KEEP: exercises glob-exclude CLI flag + recursive wildcard logic not covered by any snapshot fixture |
| 0.9040 | `glob_exclude_tests::test_glob_multiple_exclude_patterns` | KEEP: exercises --exclude flag with multiple patterns; glob-exclude logic not covered by any snapshot fixture. |
| 0.9017 | `merge_driver_tests::regenerate_wipes_conflict_markers` | KEEP: exercises --regenerate flag + conflict-marker git-state edge case; no snapshot covers this |
| 0.8883 | `integration::test_multiple_runs_update` | KEEP: second-run merge & file-removal update paths not covered by any snapshot fixture |
| 0.8883 | `integration::test_update_todo_md_on_file_removal` | KEEP: second-run merging / file-removal update path explicitly excluded from snapshot corpus |
| 0.8875 | `integration::test_update_todo_md_on_file_change` | KEEP: exercises second-run merging into an existing TODO.md, not covered by snapshots |
| 0.8862 | `integration::test_multiple_files_update` | KEEP: exercises second-run merge/update and file-removal from TODO.md — not covered by snapshots |
| 0.8860 | `integration::test_exclude_files_with_glob_patterns` | KEEP: exercises --exclude/--exclude-dir flags and src/exclusion.rs lines not covered by snapshots |
| 0.8516 | `cli_error_tests::test_run_cli_in_non_git_directory` | KEEP: asserts non-zero exit + stderr error text; error paths not covered by snapshots. |
| 0.8464 | `integration::test_auto_add_functionality` | KEEP: asserts --auto-add git-staging side-effect and --auto-add CLI flag, not covered by snapshots |
| 0.8449 | `cli_error_tests::test_sync_todo_file_fallback_mechanism` | KEEP: exercises corrupted-TODO.md fallback path and file-removal/second-run merging, not covered by snapshots. |
| 0.8395 | `merge_driver_tests::source_files_with_conflict_markers_are_skipped` | KEEP: asserts stderr text ("contains conflict markers") and non-zero-exit-free skip logic not covered by snapshots |
| 0.7247 | `merge_driver_tests::auto_install_flag_registers_driver_on_first_run_then_silent` | KEEP: asserts stdout content ("reconciling…" text) and silent-on-second-run; neither path is snapshot-covered. |
| 0.7143 | `merge_driver_tests::auto_install_self_heals_on_args_change` | KEEP: asserts stdout text + git config content for merge-driver self-heal; not covered by snapshots. |

## In-source `#[cfg(test)]` candidates — parked, no subagent fan-out this PR

The handover puts in-source tests on FLAG-ONLY status: deletion is
deferred until after #190 (aggregator decomposition) lands, when a
re-measurement on the post-refactor code will show which in-source
tests actually caught regressions. Running 111 in-source uniqueness
subagents now would consume agent budget producing verdicts that the
post-#190 second pass plans to re-derive against fresh code. The
overlap-report table itself *is* the flag-only list — entries with
ratio ≥ 0.70 are the candidates the second pass will visit.

In other words, the data was captured (every in-source test ran with
LLVM coverage, every JSON is in `coverage/per-test-json/`), but the
eyeball step is parked along with the deletion decision.
