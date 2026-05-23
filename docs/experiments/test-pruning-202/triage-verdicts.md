# Stage 3 subagent triage — verdicts (re-run against expanded snapshot corpus)

> **Hand-assembled file.** Each row was emitted verbatim by a Sonnet
> subagent invoked from the implementation session; the table here is a
> manual transcription of the 32 verdicts. Re-running the experiment
> would re-derive the verdicts but not regenerate this exact file. The
> per-candidate prompts the subagents read are reproducible from
> `scripts/build_subagent_prompts.py` against `overlap-data.json`.

One Sonnet subagent per `tests/*.rs` candidate with overlap ≥ 0.70.
Shared rubric: strict subsumption review with a KEEP bias (uncertainty
defaults to KEEP). Per-candidate prompts (generated at experiment time
under `coverage/subagent-prompts/`, not tracked) bundle the rubric, a
**data-driven** snapshot-corpus digest (one section per `#[test] fn` in
`tests/snapshot_tests.rs`, with the actual `.snap` body inlined), and
the candidate test's source + overlap data.

Result: **32 candidates reviewed, 11 DELETE, 21 KEEP.** The 11 deleted
tests are removed in this PR. The 21 retained tests assert on
something the snapshot corpus still can't reach.

## Why the result flipped (vs. the first pass)

The first pass against a 5-fixture corpus returned 28 KEEP / 0 DELETE
because the corpus did not exercise error paths, custom flags, glob
excludes, multi-language parsers, second-run merging, or conflict
markers. PR #207 added all of those (corpus went from 5 → 23 fixtures
and snapshot-only `src/` coverage from 46% → 65%). The expanded corpus
now subsumes — on both stdout and stderr — every `tests/*.rs`
candidate that asserts only on those user-visible behaviors. What's
left in the KEEP column is exactly the set of behaviors snapshots
*still* don't model.

## What the KEEP verdicts converge on

| Reason | Where it shows up |
|---|---|
| Internal-API invariants (calls library functions directly) | `empty_todo_validation_tests::*` (the surviving three call `validate_no_empty_todos` directly with field-level assertions on the returned items) |
| Stderr/exit-code combinations the snapshot fixture doesn't cover end-to-end | `cli_error_tests::*` (non-git directory, unreadable file, corrupted-TODO.md fallback) |
| CLI no-files no-op exit | `cli_no_files_tests::test_run_cli_no_files` |
| Git-index introspection beyond `--auto-add` | `git_tests::*`, `integration::test_auto_add_functionality` (asserts on `is_wt_new` / `is_index_new` index bits no snapshot captures) |
| Multi-step second-run / file-removal with different flag combos | `integration::test_multiple_runs_update`, `test_multiple_files_update`, `test_update_todo_md_on_file_removal` |
| `--exclude` glob combos the snapshots only sample | `glob_exclude_tests::*`, `integration::test_exclude_files_with_glob_patterns` |
| Merge-driver install / auto-install / self-heal / rebase-without-driver | all surviving `merge_driver_tests::*` |
| Per-entry exclusion assertion (skip logic at item granularity) | `merge_driver_tests::source_files_with_conflict_markers_are_skipped` |

Each is a residual gap in the snapshot corpus. Closing any of them
opens further deletion candidates, but that work is **out of scope**
for this PR.

## Per-candidate verdicts

| overlap | candidate | verdict |
|---|---|---|
| 1.0000 | `multi_language_tests::test_go_with_mixed_comments` | DELETE |
| 1.0000 | `multi_language_tests::test_js_with_fixme_markers` | DELETE |
| 1.0000 | `multi_language_tests::test_mixed_language_todo_extraction` | DELETE |
| 1.0000 | `multi_language_tests::test_dockerfile_with_multiple_markers` | DELETE |
| 1.0000 | `merge_driver_tests::regenerate_advisory_printed_when_todo_md_has_conflict_markers` | DELETE |
| 1.0000 | `merge_driver_tests::regenerate_wipes_conflict_markers` | DELETE |
| 1.0000 | `merge_driver_tests::source_files_with_conflict_markers_are_skipped` | KEEP: snapshot only asserts the "all TODOs skipped" output; candidate uniquely checks per-entry exclusion logic |
| 1.0000 | `empty_todo_validation_tests::test_empty_todo_detection` | KEEP: asserts directly on `validate_no_empty_todos` return value and error text, not via CLI/snapshot |
| 1.0000 | `empty_todo_validation_tests::test_python_empty_todos` | KEEP: asserts on internal `validate_no_empty_todos` API and error text not covered by CLI snapshot stderr path |
| 1.0000 | `empty_todo_validation_tests::test_valid_todo_detection` | DELETE |
| 1.0000 | `empty_todo_validation_tests::test_extract_empty_todos_directly` | KEEP: asserts internal item count and per-item fields (marker, line_number) not visible in snapshot output |
| 0.9892 | `cli_error_tests::test_run_cli_with_unreadable_file` | KEEP: asserts stderr warning text for unreadable-file path — not covered by any snapshot |
| 0.9888 | `integration::test_auto_add_functionality` | KEEP: asserts git index state (`is_wt_new`, `is_index_new`) not captured by snapshot `git_index` diff |
| 0.9881 | `integration::test_multiple_files_update` | KEEP: second-run update + removal scenario not covered by any snapshot fixture |
| 0.9880 | `integration::test_multiple_runs_update` | KEEP: `second_run_message_changes` covers update only; candidate also tests removal (run 3) and the no-TODO-remains assertion |
| 0.9880 | `integration::test_update_todo_md_on_file_removal` | KEEP: `second_run_file_no_longer_has_todo` uses `--markers TODO` but candidate uses default markers with custom `--todo-path` — different CLI path |
| 0.9879 | `integration::test_exclude_files_with_glob_patterns` | KEEP: `--exclude` + `--exclude-dir` combo; `exclude_dir_flag` snapshot uses different flags/files |
| 0.9879 | `integration::test_update_todo_md_on_file_change` | DELETE |
| 0.9878 | `glob_exclude_tests::test_glob_multiple_exclude_patterns` | KEEP: multiple `--exclude` flags combo not exercised by any snapshot fixture |
| 0.9870 | `integration::test_markers_arg_parsing` | DELETE |
| 0.9870 | `integration::test_process_files_list_single_run` | DELETE |
| 0.9869 | `cli_no_files_tests::test_run_cli_no_files` | KEEP: CLI no-files no-op exit path explicitly listed as not covered by snapshot corpus |
| 0.9859 | `integration::test_markers_with_separator` | DELETE |
| 0.9840 | `glob_exclude_tests::test_glob_exclude_recursive_wildcard` | KEEP: exercises `src/**` recursive-wildcard with multi-level nesting not present in snapshot fixture |
| 0.9780 | `cli_error_tests::test_run_cli_in_non_git_directory` | KEEP: snapshot covers stderr text but not the non-zero exit code; candidate explicitly checks `.failure()` |
| 0.9669 | `cli_error_tests::test_sync_todo_file_fallback_mechanism` | KEEP: corrupted-TODO.md fallback path not covered by any snapshot fixture |
| 0.8718 | `git_tests::test_get_tracked_files` | KEEP: git-state invariant (no double slashes in paths) not covered by any snapshot fixture |
| 0.8718 | `git_tests::test_get_tracked_files_includes_staged_but_uncommitted` | KEEP: git-index staged-but-uncommitted path in `git_utils.rs` not exercised by snapshots |
| 0.8500 | `git_tests::test_get_tracked_files_deduplicates_conflict_stages` | KEEP: conflict-stage dedup in index not modeled by any snapshot fixture |
| 0.8017 | `merge_driver_tests::rebase_without_driver_conflicts_with_driver_clean` | KEEP: merge-driver git-state edge case not covered by any snapshot fixture |
| 0.7781 | `merge_driver_tests::auto_install_flag_registers_driver_on_first_run_then_silent` | KEEP: `--auto-install-merge-driver` flag and idempotent stdout behaviour not in snapshot corpus |
| 0.7668 | `merge_driver_tests::auto_install_self_heals_on_args_change` | KEEP: stdout reconciliation message + git config args update not covered by snapshots |

## What was deleted

11 test functions across 4 files. Per-binary breakdown after deletion:

| binary | tests before | tests after | deleted |
|---|---:|---:|---:|
| `tests/cli_error_tests.rs` | 3 | 3 | 0 |
| `tests/cli_no_files_tests.rs` | 1 | 1 | 0 |
| `tests/empty_todo_validation_tests.rs` | 4 | 3 | 1 (`test_valid_todo_detection`) |
| `tests/git_tests.rs` | 4 | 4 | 0 |
| `tests/glob_exclude_tests.rs` | 2 | 2 | 0 |
| `tests/integration.rs` | 9 | 5 | 4 (`test_markers_arg_parsing`, `test_markers_with_separator`, `test_process_files_list_single_run`, `test_update_todo_md_on_file_change`) |
| `tests/merge_driver_tests.rs` | 9 | 7 | 2 (`regenerate_advisory_*`, `regenerate_wipes_conflict_markers`) |
| `tests/multi_language_tests.rs` | 4 | — | **whole file removed** |
| **total** | **36** | **25** | **11** |

`tests/snapshot_tests.rs` (23 tests) is untouched — it's the new
primary signal, not in scope for deletion.

## In-source `#[cfg(test)]` candidates — still parked

The handover puts in-source tests on FLAG-ONLY status: deletion is
deferred until after #190 (aggregator decomposition) lands. The data
is captured (every in-source test ran with LLVM coverage, every JSON
is in `coverage/per-test-json/`), but the eyeball step is still
parked along with the deletion decision.
