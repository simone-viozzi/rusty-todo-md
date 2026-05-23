# Per-test coverage overlap vs snapshot suite (issue #202)

> **Generated file.** Rewritten by
> `docs/experiments/test-pruning-202/scripts/overlap_analysis.py`
> from the per-test JSON corpus under `coverage/per-test-json/`
> (produced by the sibling `extract_per_test_coverage.sh`). The PR
> only commits this report, not the JSON inputs.

Snapshot union covers **818 distinct (file, line) keys** in `src/`.

**Scope of measurement.** Only lines under `src/` are counted —
third-party code (tests/utils.rs, cargo registry, std) is excluded.
Per-test coverage is collected with `cargo llvm-cov` + `LLVM_PROFILE_FILE`
per process; subprocess coverage from `assert_cmd::Command::cargo_bin`
propagates correctly via the `%p` substitution in the profile path.

**Branch coverage.** Stable rustc 1.95 does not emit branch counts;
`cargo llvm-cov --branch` needs nightly. This pass uses line overlap only.

**Outcome.** After the snapshot corpus expanded from 5 → 23 fixtures (#207),
the subagent fan-out returned **DELETE for 11 of 32** `tests/*.rs` candidates
with overlap ≥ 0.70 — the integration tests those snapshots now subsume on
both stdout and stderr. See `triage-verdicts.md` for the full per-candidate
breakdown; the deleted tests are removed in this PR.

## Coverage of `src/` per test group

Three columns, each capped at the file's executable line count:

- **snap%** — `tests/snapshot_tests.rs` alone (the new primary signal).
- **intg%** — every other file under `tests/*.rs` (the integration
  suite this PR was meant to prune).
- **all%** — everything, including in-source `#[cfg(test)]` modules.

The gap `intg% − snap%` is exactly the slack the snapshot corpus
would need to grow to absorb before any `tests/*.rs` file becomes
a safe-delete candidate.

| file | snap% | intg% | all% | executable |
|---|---:|---:|---:|---:|
| `src/cli.rs` | 77% | 91% | 91% | 345 |
| `src/exclusion.rs` | 83% | 84% | 100% | 100 |
| `src/git_utils.rs` | 47% | 94% | 94% | 62 |
| `src/logger.rs` | 66% | 84% | 84% | 32 |
| `src/main.rs` | 100% | 100% | 100% | 6 |
| `src/merge_driver.rs` | 0% | 83% | 100% | 215 |
| `src/test_utils.rs` | 0% | 0% | 95% | 22 |
| `src/todo_extractor_internal/aggregator.rs` | 90% | 92% | 100% | 246 |
| `src/todo_extractor_internal/languages/common_syntax.rs` | 83% | 79% | 100% | 29 |
| `src/todo_extractor_internal/languages/dockerfile.rs` | 100% | 100% | 100% | 3 |
| `src/todo_extractor_internal/languages/go.rs` | 100% | 100% | 100% | 3 |
| `src/todo_extractor_internal/languages/js.rs` | 100% | 100% | 100% | 3 |
| `src/todo_extractor_internal/languages/markdown.rs` | 0% | 100% | 100% | 3 |
| `src/todo_extractor_internal/languages/python.rs` | 100% | 100% | 100% | 3 |
| `src/todo_extractor_internal/languages/rust.rs` | 100% | 100% | 100% | 3 |
| `src/todo_extractor_internal/languages/shell.rs` | 0% | 0% | 100% | 3 |
| `src/todo_extractor_internal/languages/sql.rs` | 0% | 0% | 100% | 3 |
| `src/todo_extractor_internal/languages/toml.rs` | 0% | 0% | 100% | 3 |
| `src/todo_extractor_internal/languages/yaml.rs` | 0% | 0% | 100% | 3 |
| `src/todo_md.rs` | 88% | 91% | 100% | 136 |
| `src/todo_md_internal.rs` | 84% | 84% | 100% | 38 |
| **total** | **64.9%** | **86.5%** | **96.7%** | **1261** |

Notes to avoid misreading the 0%/100% rows:

- A 0% in **snap%** does NOT mean nothing covers that code — it
  means snapshots don't. The in-source `dockerfile_tests` etc. do
  exercise their matching `languages/<x>.rs`, but in a separate
  test binary not shown in the snap column.
- Each `languages/<x>.rs` file shows **3 executable lines** because
  the snapshot/integration binaries compile the library *without*
  `#[cfg(test)]`, exposing only the production `impl CommentParser`
  body. Measuring against the in-source test binary instead would
  show ~120 lines per language file (production + test-mod).
- `shell/sql/toml/yaml.rs` are at intg% = 0% / all% = 100%: only
  their in-source unit tests reach them. Deleting those tests would
  leave those parsers with no test coverage at all.

## Distribution of overlap ratio across non-snapshot tests

| Bucket | Count |
|---|---|
| 1.00 (exactly) | 11 |
| 0.99–<1.00 | 0 |
| 0.95–0.99 | 15 |
| 0.90–0.95 | 0 |
| 0.80–0.90 | 62 |
| 0.70–0.80 | 25 |
| 0.50–0.70 | 15 |
| 0.25–0.50 | 10 |
| <0.25 | 9 |

Total non-snapshot tests measured: 147 (36 `tests/*.rs` + 111 in-source).

## tests/*.rs candidates (in scope for deletion in this PR)

| overlap | covered | overlap_lines | unique | top unique files | binary | test |
|---|---|---|---|---|---|---|
| 1.0000 | 549 | 549 | 0 |  | `multi_language_tests` | `multi_language_tests_test_go_with_mixed_comments` |
| 1.0000 | 547 | 547 | 0 |  | `multi_language_tests` | `multi_language_tests_test_js_with_fixme_markers` |
| 1.0000 | 534 | 534 | 0 |  | `multi_language_tests` | `multi_language_tests_test_mixed_language_todo_extraction` |
| 1.0000 | 531 | 531 | 0 |  | `multi_language_tests` | `multi_language_tests_test_dockerfile_with_multiple_markers` |
| 1.0000 | 522 | 522 | 0 |  | `merge_driver_tests` | `regenerate_advisory_printed_when_todo_md_has_conflict_markers` |
| 1.0000 | 468 | 468 | 0 |  | `merge_driver_tests` | `regenerate_wipes_conflict_markers` |
| 1.0000 | 299 | 299 | 0 |  | `merge_driver_tests` | `source_files_with_conflict_markers_are_skipped` |
| 1.0000 | 218 | 218 | 0 |  | `empty_todo_validation_tests` | `test_empty_todo_detection` |
| 1.0000 | 218 | 218 | 0 |  | `empty_todo_validation_tests` | `test_python_empty_todos` |
| 1.0000 | 207 | 207 | 0 |  | `empty_todo_validation_tests` | `test_valid_todo_detection` |
| 1.0000 | 199 | 199 | 0 |  | `empty_todo_validation_tests` | `test_extract_empty_todos_directly` |
| 0.9892 | 369 | 365 | 4 | src/logger.rs (4) | `cli_error_tests` | `test_run_cli_with_unreadable_file` |
| 0.9888 | 625 | 618 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_auto_add_functionality` |
| 0.9881 | 589 | 582 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_multiple_files_update` |
| 0.9880 | 582 | 575 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_multiple_runs_update` |
| 0.9880 | 582 | 575 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_update_todo_md_on_file_removal` |
| 0.9879 | 579 | 572 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_exclude_files_with_glob_patterns` |
| 0.9879 | 578 | 571 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_update_todo_md_on_file_change` |
| 0.9878 | 573 | 566 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `glob_exclude_tests` | `glob_exclude_tests_test_glob_multiple_exclude_patterns` |
| 0.9870 | 540 | 533 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_markers_arg_parsing` |
| 0.9870 | 539 | 532 | 7 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_process_files_list_single_run` |
| 0.9869 | 305 | 301 | 4 | src/logger.rs (4) | `cli_no_files_tests` | `test_run_cli_no_files` |
| 0.9859 | 568 | 560 | 8 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3); src/cli.rs (1) | `integration` | `integration_tests_test_markers_with_separator` |
| 0.9840 | 564 | 555 | 9 | src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3); src/exclusion.rs (2) | `glob_exclude_tests` | `glob_exclude_tests_test_glob_exclude_recursive_wildcard` |
| 0.9780 | 182 | 178 | 4 | src/logger.rs (4) | `cli_error_tests` | `test_run_cli_in_non_git_directory` |
| 0.9669 | 574 | 555 | 19 | src/todo_extractor_internal/aggregator.rs (7); src/logger.rs (6); src/todo_md.rs (5); src/git_utils.rs (1) | `cli_error_tests` | `test_sync_todo_file_fallback_mechanism` |
| 0.8718 | 39 | 34 | 5 | src/logger.rs (4); src/git_utils.rs (1) | `git_tests` | `test_get_tracked_files` |
| 0.8718 | 39 | 34 | 5 | src/logger.rs (4); src/git_utils.rs (1) | `git_tests` | `test_get_tracked_files_includes_staged_but_uncommitted` |
| 0.8500 | 40 | 34 | 6 | src/logger.rs (4); src/git_utils.rs (2) | `git_tests` | `test_get_tracked_files_deduplicates_conflict_stages` |
| 0.8017 | 832 | 667 | 165 | src/merge_driver.rs (129); src/cli.rs (28); src/todo_extractor_internal/aggregator.rs (4); src/todo_extractor_internal/languages/markdown.rs (3); src/git_utils.rs (1) | `merge_driver_tests` | `rebase_without_driver_conflicts_with_driver_clean` |
| 0.7781 | 730 | 568 | 162 | src/merge_driver.rs (144); src/cli.rs (18) | `merge_driver_tests` | `auto_install_flag_registers_driver_on_first_run_then_silent` |
| 0.7668 | 742 | 569 | 173 | src/merge_driver.rs (155); src/cli.rs (18) | `merge_driver_tests` | `auto_install_self_heals_on_args_change` |
| 0.5282 | 301 | 159 | 142 | src/merge_driver.rs (129); src/cli.rs (13) | `merge_driver_tests` | `install_merge_driver_writes_config_and_gitattributes` |
| 0.5016 | 317 | 159 | 158 | src/merge_driver.rs (145); src/cli.rs (13) | `merge_driver_tests` | `install_merge_driver_rewrites_block_on_drift` |
| 0.4969 | 320 | 159 | 161 | src/merge_driver.rs (148); src/cli.rs (13) | `merge_driver_tests` | `install_merge_driver_is_convergent` |
| 0.3800 | 50 | 19 | 31 | src/git_utils.rs (27); src/logger.rs (4) | `git_tests` | `test_get_staged_files` |

## in-source #[cfg(test)] candidates (FLAG-ONLY — parked for post-#190)

| overlap | covered | overlap_lines | unique | top unique files | binary | test |
|---|---|---|---|---|---|---|
| 0.8865 | 185 | 164 | 21 | src/todo_extractor_internal/aggregator.rs (11); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_fixme_with_colon` |
| 0.8811 | 185 | 163 | 22 | src/todo_extractor_internal/aggregator.rs (12); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_fixme_without_colon` |
| 0.8706 | 170 | 148 | 22 | src/todo_extractor_internal/aggregator.rs (12); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_ignore_todo_not_at_beginning` |
| 0.8571 | 203 | 174 | 29 | src/todo_extractor_internal/aggregator.rs (19); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_mixed_markers` |
| 0.8554 | 242 | 207 | 35 | src/todo_extractor_internal/aggregator.rs (21); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_marker_prefilter_lets_marker_bearing_file_through` |
| 0.8511 | 47 | 40 | 7 | src/todo_extractor_internal/languages/go.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_ignore_non_comment_go` |
| 0.8511 | 47 | 40 | 7 | src/todo_extractor_internal/languages/js.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_ignore_non_comment_js` |
| 0.8511 | 47 | 40 | 7 | src/todo_extractor_internal/languages/rust.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_ignore_non_comment_rust` |
| 0.8400 | 200 | 168 | 32 | src/todo_extractor_internal/aggregator.rs (22); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_mixed_marker_configurations` |
| 0.8341 | 229 | 191 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_dockerfile_no_extension` |
| 0.8333 | 72 | 60 | 12 | src/exclusion.rs (12) | `rusty_todo_md` | `exclusion_tests_test_filter_excluded_files` |
| 0.8333 | 48 | 40 | 8 | src/todo_extractor_internal/languages/go.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_extract_go_comments` |
| 0.8333 | 48 | 40 | 8 | src/todo_extractor_internal/languages/js.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_extract_js_comments` |
| 0.8333 | 48 | 40 | 8 | src/todo_extractor_internal/languages/rust.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_extract_rust_comments` |
| 0.8304 | 230 | 191 | 39 | src/test_utils.rs (20); src/todo_extractor_internal/languages/dockerfile.rs (11); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_single_comment` |
| 0.8296 | 223 | 185 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_go_extension` |
| 0.8281 | 221 | 183 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_js_extension` |
| 0.8281 | 221 | 183 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_jsx_extension` |
| 0.8269 | 52 | 43 | 9 | src/exclusion.rs (9) | `rusty_todo_md` | `exclusion_tests_test_last_match_wins` |
| 0.8265 | 219 | 181 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_display_todo_output` |
| 0.8265 | 219 | 181 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_rust_extension` |
| 0.8248 | 234 | 193 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/languages/markdown.rs (13); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_markdown_markdown_tests_test_markdown_html_comment` |
| 0.8240 | 233 | 192 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4); src/todo_extractor_internal/languages/markdown.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_markdown_extension` |
| 0.8221 | 253 | 208 | 45 | src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (17); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_nested_block_comments` |
| 0.8217 | 230 | 189 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (17); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_stop_merge_on_unindented_line` |
| 0.8217 | 230 | 189 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/languages/python.rs (13); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_extract_python_todo` |
| 0.8214 | 28 | 23 | 5 | src/exclusion.rs (5) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_exclude_dir` |
| 0.8210 | 229 | 188 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4); src/todo_extractor_internal/languages/sql.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_sql_extension` |
| 0.8210 | 229 | 188 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4); src/todo_extractor_internal/languages/toml.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_toml_extension` |
| 0.8210 | 229 | 188 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/languages/sql.rs (13); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_sql_sql_tests_test_sql_line_comment` |
| 0.8205 | 39 | 32 | 7 | src/exclusion.rs (7) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_multiple` |
| 0.8202 | 228 | 187 | 41 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4); src/todo_extractor_internal/languages/shell.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_sh_extension` |
| 0.8175 | 252 | 206 | 46 | src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (18); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_block_comment` |
| 0.8174 | 230 | 188 | 42 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (18); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_merge_multiline_todo_with_todo_in_str` |
| 0.8166 | 229 | 187 | 42 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (15); src/logger.rs (4); src/todo_extractor_internal/languages/yaml.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_yaml_extension` |
| 0.8166 | 229 | 187 | 42 | src/test_utils.rs (20); src/todo_extractor_internal/languages/shell.rs (14); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_shell_shell_tests_test_sh_single_comment` |
| 0.8160 | 250 | 204 | 46 | src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (18); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_block_comment` |
| 0.8158 | 228 | 186 | 42 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (18); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_todo_with_line_number` |
| 0.8155 | 233 | 190 | 43 | src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (15); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_ignore_string_literals` |
| 0.8147 | 259 | 211 | 48 | src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_package_comments` |
| 0.8146 | 205 | 167 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/languages/python.rs (11); src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_ignore_non_todo_python` |
| 0.8140 | 258 | 210 | 48 | src/todo_extractor_internal/languages/go.rs (20); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_mixed_comments` |
| 0.8139 | 231 | 188 | 43 | src/test_utils.rs (20); src/todo_extractor_internal/languages/toml.rs (15); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_toml_toml_tests_test_toml_single_comment` |
| 0.8136 | 236 | 192 | 44 | src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (16); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_multiline_todo` |
| 0.8130 | 230 | 187 | 43 | src/test_utils.rs (20); src/todo_extractor_internal/languages/yaml.rs (14); src/todo_extractor_internal/aggregator.rs (5); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_single_comment` |
| 0.8125 | 256 | 208 | 48 | src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_mixed_comments` |
| 0.8120 | 234 | 190 | 44 | src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (16); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_multiline_todo` |
| 0.8114 | 228 | 185 | 43 | src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (15); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_ignore_string_literals` |
| 0.8103 | 232 | 188 | 44 | src/todo_extractor_internal/aggregator.rs (20); src/test_utils.rs (20); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_merge_multiline_todo` |
| 0.8093 | 236 | 191 | 45 | src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (17); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_jsx_syntax` |
| 0.8089 | 225 | 182 | 43 | src/test_utils.rs (20); src/todo_extractor_internal/languages/python.rs (15); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_python_single_line` |
| 0.8087 | 230 | 186 | 44 | src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (16); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_single_line_comment` |
| 0.8077 | 26 | 21 | 5 | src/exclusion.rs (5) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_exclude` |
| 0.8070 | 228 | 184 | 44 | src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (16); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_single_line_comment` |
| 0.8062 | 227 | 183 | 44 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (20); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_multiple_consecutive_todos` |
| 0.8052 | 231 | 186 | 45 | src/test_utils.rs (20); src/todo_extractor_internal/languages/rust.rs (17); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_rust_single_line` |
| 0.8000 | 245 | 196 | 49 | src/todo_extractor_internal/languages/dockerfile.rs (22); src/test_utils.rs (20); src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_multiline_run_with_todo` |
| 0.8000 | 25 | 20 | 5 | src/todo_extractor_internal/languages/common_syntax.rs (5) | `rusty_todo_md` | `todo_extractor_internal_languages_common_syntax_tests_test_strip_markers_with_indent` |
| 0.7992 | 244 | 195 | 49 | src/todo_extractor_internal/languages/python.rs (21); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_python_docstring` |
| 0.7991 | 234 | 187 | 47 | src/test_utils.rs (20); src/todo_extractor_internal/languages/yaml.rs (18); src/todo_extractor_internal/aggregator.rs (5); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_quoted_strings` |
| 0.7969 | 256 | 204 | 52 | src/todo_extractor_internal/languages/rust.rs (21); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4); src/todo_extractor_internal/languages/common_syntax.rs (3) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_rust_block_doc` |
| 0.7931 | 29 | 23 | 6 | src/todo_extractor_internal/languages/common_syntax.rs (6) | `rusty_todo_md` | `todo_extractor_internal_languages_common_syntax_tests_test_strip_markers_different_markers` |
| 0.7908 | 239 | 189 | 50 | src/todo_extractor_internal/aggregator.rs (37); src/test_utils.rs (10); src/todo_extractor_internal/languages/common_syntax.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_mixed_markers_complex` |
| 0.7893 | 242 | 191 | 51 | src/todo_extractor_internal/languages/yaml.rs (22); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (5); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_multiple_comments` |
| 0.7857 | 238 | 187 | 51 | src/todo_extractor_internal/languages/yaml.rs (22); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (5); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_ignore_triple_quoted_strings` |
| 0.7857 | 28 | 22 | 6 | src/exclusion.rs (6) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_exclude_dir_with_slash` |
| 0.7840 | 250 | 196 | 54 | src/todo_extractor_internal/languages/python.rs (26); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_python_docstring_multiple_todos` |
| 0.7764 | 161 | 125 | 36 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_display_no_todos` |
| 0.7750 | 160 | 124 | 36 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_empty_input_no_todos` |
| 0.7716 | 197 | 152 | 45 | src/todo_md.rs (31); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_tests_test_sync_todo_file_filters_nonexistent_files` |
| 0.7669 | 163 | 125 | 38 | src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_false_positive_detection` |
| 0.7652 | 264 | 202 | 62 | src/todo_extractor_internal/languages/dockerfile.rs (34); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_mixed_comment_styles` |
| 0.7647 | 17 | 13 | 4 | src/exclusion.rs (4) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_invalid_pattern` |
| 0.7500 | 72 | 54 | 18 | src/exclusion.rs (18) | `rusty_todo_md` | `exclusion_tests_test_should_exclude_exclude_dir_flag` |
| 0.7419 | 31 | 23 | 8 | src/todo_extractor_internal/languages/common_syntax.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_common_syntax_tests_test_strip_markers` |
| 0.7410 | 166 | 123 | 43 | src/todo_md.rs (29); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_tests_test_sync_todo_file` |
| 0.7344 | 64 | 47 | 17 | src/exclusion.rs (17) | `rusty_todo_md` | `exclusion_tests_test_should_exclude_files` |
| 0.7324 | 71 | 52 | 19 | src/exclusion.rs (19) | `rusty_todo_md` | `exclusion_tests_test_should_exclude_directories` |
| 0.7266 | 278 | 202 | 76 | src/todo_extractor_internal/languages/dockerfile.rs (49); src/test_utils.rs (20); src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (3) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_multiple_todos_and_markers` |
| 0.7160 | 81 | 58 | 23 | src/todo_extractor_internal/aggregator.rs (11); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_extract_marked_items_from_file_nonexistent_file` |
| 0.7085 | 295 | 209 | 86 | src/todo_extractor_internal/languages/rust.rs (58); src/test_utils.rs (20); src/logger.rs (4); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_large_rust_file_scenario` |
| 0.6897 | 87 | 60 | 27 | src/todo_extractor_internal/aggregator.rs (13); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_extract_marked_items_from_file_unsupported_extension` |
| 0.6639 | 119 | 79 | 40 | src/todo_extractor_internal/aggregator.rs (28); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_marker_prefilter_skips_large_marker_free_file` |
| 0.6588 | 85 | 56 | 29 | src/todo_extractor_internal/languages/yaml.rs (16); src/test_utils.rs (10); src/logger.rs (2); src/todo_extractor_internal/aggregator.rs (1) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_direct_parser` |
| 0.6235 | 85 | 53 | 32 | src/test_utils.rs (19); src/todo_extractor_internal/aggregator.rs (11); src/logger.rs (2) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_invalid_extension` |
| 0.5882 | 34 | 20 | 14 | src/todo_md_internal.rs (14) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_scanned_file_removal` |
| 0.5692 | 65 | 37 | 28 | src/todo_md_internal.rs (16); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_keeps_existing_items_when_new_empty` |
| 0.5571 | 70 | 39 | 31 | src/todo_md_internal.rs (17); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_no_duplicates` |
| 0.5567 | 97 | 54 | 43 | src/todo_md.rs (33); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_tests_test_read_todo_file_with_markdown_parser` |
| 0.5424 | 59 | 32 | 27 | src/todo_md_internal.rs (15); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_md_internal_tests_test_add_item` |
| 0.5421 | 107 | 58 | 49 | src/todo_extractor_internal/aggregator.rs (37); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_extract_marked_items_from_file_permission_denied` |
| 0.5000 | 84 | 42 | 42 | src/todo_md_internal.rs (30); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_sorting_order` |
| 0.5000 | 84 | 42 | 42 | src/todo_md_internal.rs (30); src/test_utils.rs (10); src/logger.rs (2) | `rusty_todo_md` | `todo_md_internal_tests_test_to_sorted_vec` |
| 0.5000 | 78 | 39 | 39 | src/todo_md_internal.rs (25); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_adds_missing_items` |
| 0.4815 | 81 | 39 | 42 | src/todo_md_internal.rs (28); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_multiple_files` |
| 0.4643 | 84 | 39 | 45 | src/todo_md_internal.rs (31); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_replaces_existing_items` |
| 0.4535 | 86 | 39 | 47 | src/todo_md_internal.rs (33); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_collections` |
| 0.3750 | 8 | 3 | 5 | src/exclusion.rs (5) | `rusty_todo_md` | `exclusion_tests_test_normalize_pattern` |
| 0.3571 | 14 | 5 | 9 | src/todo_extractor_internal/aggregator.rs (9) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_content_may_contain_marker_basic` |
| 0.3488 | 86 | 30 | 56 | src/todo_md.rs (46); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_tests_test_write_todo_file_sectioned` |
| 0.3197 | 122 | 39 | 83 | src/todo_md_internal.rs (69); src/test_utils.rs (10); src/logger.rs (4) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_complex_replacement` |
| 0.2800 | 25 | 7 | 18 | src/merge_driver.rs (18) | `rusty_todo_md` | `merge_driver_tests_build_expected_rejects_absolute_todo_path` |
| 0.1228 | 57 | 7 | 50 | src/merge_driver.rs (50) | `rusty_todo_md` | `merge_driver_tests_build_driver_command_emits_all_args` |
| 0.0921 | 76 | 7 | 69 | src/merge_driver.rs (69) | `rusty_todo_md` | `merge_driver_tests_build_expected_quotes_path_with_specials` |
| 0.0000 | 26 | 0 | 26 | src/merge_driver.rs (26) | `rusty_todo_md` | `merge_driver_tests_format_install_summary_renders_both_states` |
| 0.0000 | 22 | 0 | 22 | src/merge_driver.rs (22) | `rusty_todo_md` | `merge_driver_tests_quote_for_gitattributes_quotes_when_whitespace_or_specials` |
| 0.0000 | 21 | 0 | 21 | src/merge_driver.rs (21) | `rusty_todo_md` | `merge_driver_tests_quote_for_gitattributes_escapes_glob_metacharacters` |
| 0.0000 | 16 | 0 | 16 | src/merge_driver.rs (16) | `rusty_todo_md` | `merge_driver_tests_quote_for_gitattributes_unquoted_when_safe` |
| 0.0000 | 14 | 0 | 14 | src/merge_driver.rs (14) | `rusty_todo_md` | `merge_driver_tests_quote_for_shell_quotes_specials_and_escapes_single_quotes` |
| 0.0000 | 14 | 0 | 14 | src/test_utils.rs (10); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_basic_framework` |
| 0.0000 | 11 | 0 | 11 | src/merge_driver.rs (11) | `rusty_todo_md` | `merge_driver_tests_quote_for_shell_passes_safe_strings_through` |
