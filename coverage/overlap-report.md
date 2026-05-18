# Per-test coverage overlap vs snapshot suite (issue #202)

Snapshot union covers **574 distinct (file, line) keys** in `src/`.

**Scope of measurement.** Only lines under `src/` are counted —
third-party code (tests/utils.rs, cargo registry, std) is excluded.
Per-test coverage is collected with `cargo llvm-cov` + `LLVM_PROFILE_FILE`
per process; subprocess coverage from `assert_cmd::Command::cargo_bin`
propagates correctly via the `%p` substitution in the profile path.
Reproducer: `scripts/extract_per_test_coverage.sh`, then
`scripts/overlap_analysis.py`.

**Branch coverage.** Stable rustc 1.95 does not emit branch counts;
`cargo llvm-cov --branch` needs nightly. This pass uses line overlap only.
The QA framed branch as an enhancement on top of line overlap, not a
substitute. The false-flag rate is mitigated by the per-candidate
subagent review in stage 3 (see `triage-verdicts.md`).

**Outcome of stage 3 (preview):** the subagent fan-out returned KEEP for
all 28 `tests/*.rs` candidates with overlap ≥ 0.70. The snapshot corpus
(5 happy-path fixtures, fixed flags, no error paths) is too narrow to
subsume any integration test — every candidate reaches an error path,
flag combo, multi-run update, merge-driver path, or internal invariant
the snapshot suite does not assert on. No deletions in this PR; see
`triage-verdicts.md` for the full per-candidate breakdown.

## Distribution of overlap ratio across non-snapshot tests

| Bucket | Count |
|---|---|
| 1.00 (exactly) | 3 |
| 0.99–<1.00 | 0 |
| 0.95–0.99 | 5 |
| 0.90–0.95 | 9 |
| 0.80–0.90 | 19 |
| 0.70–0.80 | 46 |
| 0.50–0.70 | 21 |
| 0.25–0.50 | 25 |
| <0.25 | 19 |

Total non-snapshot tests measured: 147 (36 `tests/*.rs` + 111 in-source).

## tests/*.rs candidates (in scope for deletion in this PR)

| overlap | covered | overlap_lines | unique | top unique files | binary | test |
|---|---|---|---|---|---|---|
| 1.0000 | 547 | 547 | 0 |  | `multi_language_tests` | `multi_language_tests_test_js_with_fixme_markers` |
| 1.0000 | 207 | 207 | 0 |  | `empty_todo_validation_tests` | `test_valid_todo_detection` |
| 1.0000 | 199 | 199 | 0 |  | `empty_todo_validation_tests` | `test_extract_empty_todos_directly` |
| 0.9891 | 549 | 543 | 6 | src/todo_extractor_internal/languages/go.rs (3); src/todo_extractor_internal/aggregator.rs (3) | `multi_language_tests` | `multi_language_tests_test_go_with_mixed_comments` |
| 0.9888 | 534 | 528 | 6 | src/todo_extractor_internal/languages/go.rs (3); src/todo_extractor_internal/aggregator.rs (3) | `multi_language_tests` | `multi_language_tests_test_mixed_language_todo_extraction` |
| 0.9774 | 531 | 519 | 12 | src/todo_extractor_internal/aggregator.rs (9); src/todo_extractor_internal/languages/dockerfile.rs (3) | `multi_language_tests` | `multi_language_tests_test_dockerfile_with_multiple_markers` |
| 0.9519 | 540 | 514 | 26 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_markers_arg_parsing` |
| 0.9518 | 539 | 513 | 26 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_process_files_list_single_run` |
| 0.9450 | 218 | 206 | 12 | src/cli.rs (12) | `empty_todo_validation_tests` | `test_empty_todo_detection` |
| 0.9450 | 218 | 206 | 12 | src/cli.rs (12) | `empty_todo_validation_tests` | `test_python_empty_todos` |
| 0.9246 | 305 | 282 | 23 | src/logger.rs (23) | `cli_no_files_tests` | `test_run_cli_no_files` |
| 0.9214 | 369 | 340 | 29 | src/logger.rs (25); src/todo_extractor_internal/aggregator.rs (3); src/cli.rs (1) | `cli_error_tests` | `test_run_cli_with_unreadable_file` |
| 0.9138 | 522 | 477 | 45 | src/todo_md.rs (16); src/git_utils.rs (15); src/cli.rs (14) | `merge_driver_tests` | `regenerate_advisory_printed_when_todo_md_has_conflict_markers` |
| 0.9137 | 568 | 519 | 49 | src/logger.rs (23); src/cli.rs (23); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_markers_with_separator` |
| 0.9096 | 564 | 513 | 51 | src/exclusion.rs (25); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3) | `glob_exclude_tests` | `glob_exclude_tests_test_glob_exclude_recursive_wildcard` |
| 0.9040 | 573 | 518 | 55 | src/exclusion.rs (29); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3) | `glob_exclude_tests` | `glob_exclude_tests_test_glob_multiple_exclude_patterns` |
| 0.9017 | 468 | 422 | 46 | src/cli.rs (31); src/git_utils.rs (15) | `merge_driver_tests` | `regenerate_wipes_conflict_markers` |
| 0.8883 | 582 | 517 | 65 | src/todo_md.rs (38); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3); src/cli.rs (1) | `integration` | `integration_tests_test_multiple_runs_update` |
| 0.8883 | 582 | 517 | 65 | src/todo_md.rs (38); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3); src/cli.rs (1) | `integration` | `integration_tests_test_update_todo_md_on_file_removal` |
| 0.8875 | 578 | 513 | 65 | src/todo_md.rs (38); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3); src/cli.rs (1) | `integration` | `integration_tests_test_update_todo_md_on_file_change` |
| 0.8862 | 589 | 522 | 67 | src/todo_md.rs (40); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3); src/cli.rs (1) | `integration` | `integration_tests_test_multiple_files_update` |
| 0.8860 | 579 | 513 | 66 | src/exclusion.rs (40); src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_exclude_files_with_glob_patterns` |
| 0.8516 | 182 | 155 | 27 | src/logger.rs (25); src/cli.rs (2) | `cli_error_tests` | `test_run_cli_in_non_git_directory` |
| 0.8464 | 625 | 529 | 96 | src/todo_md.rs (38); src/cli.rs (24); src/logger.rs (23); src/git_utils.rs (8); src/todo_extractor_internal/aggregator.rs (3) | `integration` | `integration_tests_test_auto_add_functionality` |
| 0.8449 | 574 | 485 | 89 | src/logger.rs (25); src/todo_md.rs (21); src/todo_extractor_internal/aggregator.rs (16); src/git_utils.rs (16); src/cli.rs (11) | `cli_error_tests` | `test_sync_todo_file_fallback_mechanism` |
| 0.8395 | 299 | 251 | 48 | src/cli.rs (30); src/git_utils.rs (15); src/todo_extractor_internal/aggregator.rs (3) | `merge_driver_tests` | `source_files_with_conflict_markers_are_skipped` |
| 0.7247 | 730 | 529 | 201 | src/merge_driver.rs (144); src/todo_md.rs (38); src/cli.rs (19) | `merge_driver_tests` | `auto_install_flag_registers_driver_on_first_run_then_silent` |
| 0.7143 | 742 | 530 | 212 | src/merge_driver.rs (155); src/todo_md.rs (38); src/cli.rs (19) | `merge_driver_tests` | `auto_install_self_heals_on_args_change` |
| 0.6466 | 832 | 538 | 294 | src/merge_driver.rs (129); src/cli.rs (82); src/todo_md.rs (38); src/git_utils.rs (24); src/todo_extractor_internal/aggregator.rs (18) | `merge_driver_tests` | `rebase_without_driver_conflicts_with_driver_clean` |
| 0.5282 | 301 | 159 | 142 | src/merge_driver.rs (129); src/cli.rs (13) | `merge_driver_tests` | `install_merge_driver_writes_config_and_gitattributes` |
| 0.5016 | 317 | 159 | 158 | src/merge_driver.rs (145); src/cli.rs (13) | `merge_driver_tests` | `install_merge_driver_rewrites_block_on_drift` |
| 0.4969 | 320 | 159 | 161 | src/merge_driver.rs (148); src/cli.rs (13) | `merge_driver_tests` | `install_merge_driver_is_convergent` |
| 0.0000 | 50 | 0 | 50 | src/git_utils.rs (27); src/logger.rs (23) | `git_tests` | `test_get_staged_files` |
| 0.0000 | 40 | 0 | 40 | src/logger.rs (23); src/git_utils.rs (17) | `git_tests` | `test_get_tracked_files_deduplicates_conflict_stages` |
| 0.0000 | 39 | 0 | 39 | src/logger.rs (23); src/git_utils.rs (16) | `git_tests` | `test_get_tracked_files` |
| 0.0000 | 39 | 0 | 39 | src/logger.rs (23); src/git_utils.rs (16) | `git_tests` | `test_get_tracked_files_includes_staged_but_uncommitted` |

## in-source #[cfg(test)] candidates (FLAG-ONLY — parked for post-#190)

| overlap | covered | overlap_lines | unique | top unique files | binary | test |
|---|---|---|---|---|---|---|
| 0.8865 | 185 | 164 | 21 | src/todo_extractor_internal/aggregator.rs (11); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_fixme_with_colon` |
| 0.8811 | 185 | 163 | 22 | src/todo_extractor_internal/aggregator.rs (12); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_fixme_without_colon` |
| 0.8706 | 170 | 148 | 22 | src/todo_extractor_internal/aggregator.rs (12); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_ignore_todo_not_at_beginning` |
| 0.8571 | 203 | 174 | 29 | src/todo_extractor_internal/aggregator.rs (19); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_mixed_markers` |
| 0.8511 | 47 | 40 | 7 | src/todo_extractor_internal/languages/js.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_ignore_non_comment_js` |
| 0.8511 | 47 | 40 | 7 | src/todo_extractor_internal/languages/rust.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_ignore_non_comment_rust` |
| 0.8400 | 200 | 168 | 32 | src/todo_extractor_internal/aggregator.rs (22); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_mixed_marker_configurations` |
| 0.8333 | 48 | 40 | 8 | src/todo_extractor_internal/languages/js.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_extract_js_comments` |
| 0.8333 | 48 | 40 | 8 | src/todo_extractor_internal/languages/rust.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_extract_rust_comments` |
| 0.8000 | 25 | 20 | 5 | src/todo_extractor_internal/languages/common_syntax.rs (5) | `rusty_todo_md` | `todo_extractor_internal_languages_common_syntax_tests_test_strip_markers_with_indent` |
| 0.7931 | 29 | 23 | 6 | src/todo_extractor_internal/languages/common_syntax.rs (6) | `rusty_todo_md` | `todo_extractor_internal_languages_common_syntax_tests_test_strip_markers_different_markers` |
| 0.7908 | 239 | 189 | 50 | src/todo_extractor_internal/aggregator.rs (37); src/test_utils.rs (10); src/todo_extractor_internal/languages/common_syntax.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_mixed_markers_complex` |
| 0.7872 | 47 | 37 | 10 | src/todo_extractor_internal/languages/go.rs (10) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_ignore_non_comment_go` |
| 0.7769 | 242 | 188 | 54 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (21); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_marker_prefilter_lets_marker_bearing_file_through` |
| 0.7708 | 48 | 37 | 11 | src/todo_extractor_internal/languages/go.rs (11) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_extract_go_comments` |
| 0.7421 | 221 | 164 | 57 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_js_extension` |
| 0.7421 | 221 | 164 | 57 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_jsx_extension` |
| 0.7419 | 31 | 23 | 8 | src/todo_extractor_internal/languages/common_syntax.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_common_syntax_tests_test_strip_markers` |
| 0.7400 | 250 | 185 | 65 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (18); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_block_comment` |
| 0.7397 | 219 | 162 | 57 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_display_todo_output` |
| 0.7397 | 219 | 162 | 57 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_rust_extension` |
| 0.7391 | 230 | 170 | 60 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (17) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_stop_merge_on_unindented_line` |
| 0.7391 | 230 | 170 | 60 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/python.rs (13); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_extract_python_todo` |
| 0.7383 | 256 | 189 | 67 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (20); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_mixed_comments` |
| 0.7348 | 230 | 169 | 61 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (18) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_merge_multiline_todo_with_todo_in_str` |
| 0.7325 | 228 | 167 | 61 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (18) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_todo_with_line_number` |
| 0.7308 | 234 | 171 | 63 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (16); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_multiline_todo` |
| 0.7288 | 236 | 172 | 64 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (17); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_jsx_syntax` |
| 0.7284 | 232 | 169 | 63 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (20); src/test_utils.rs (20) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_merge_multiline_todo` |
| 0.7281 | 228 | 166 | 62 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (15); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_ignore_string_literals` |
| 0.7244 | 225 | 163 | 62 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/python.rs (15); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_python_single_line` |
| 0.7237 | 228 | 165 | 63 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/js.rs (16); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_js_js_tests_test_js_single_line_comment` |
| 0.7233 | 253 | 183 | 70 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (20); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_nested_block_comments` |
| 0.7229 | 231 | 167 | 64 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/rust.rs (17); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_rust_single_line` |
| 0.7227 | 256 | 185 | 71 | src/logger.rs (23); src/todo_extractor_internal/languages/rust.rs (21); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4); src/todo_extractor_internal/languages/common_syntax.rs (3) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_rust_block_doc` |
| 0.7225 | 227 | 164 | 63 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (20) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_multiple_consecutive_todos` |
| 0.7220 | 205 | 148 | 57 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/python.rs (11); src/todo_extractor_internal/aggregator.rs (3) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_ignore_non_todo_python` |
| 0.7213 | 244 | 176 | 68 | src/logger.rs (23); src/todo_extractor_internal/languages/python.rs (21); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_python_docstring` |
| 0.7193 | 228 | 164 | 64 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (18); src/todo_extractor_internal/languages/shell.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_sh_extension` |
| 0.7183 | 252 | 181 | 71 | src/logger.rs (23); src/todo_extractor_internal/languages/go.rs (21); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_block_comment` |
| 0.7181 | 259 | 186 | 73 | src/logger.rs (23); src/todo_extractor_internal/languages/go.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_package_comments` |
| 0.7175 | 223 | 160 | 63 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (17); src/todo_extractor_internal/languages/go.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_go_extension` |
| 0.7171 | 258 | 185 | 73 | src/todo_extractor_internal/languages/go.rs (23); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_mixed_comments` |
| 0.7162 | 229 | 164 | 65 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (19); src/todo_extractor_internal/languages/toml.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_toml_extension` |
| 0.7162 | 229 | 164 | 65 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/shell.rs (14); src/todo_extractor_internal/aggregator.rs (8) | `rusty_todo_md` | `todo_extractor_internal_languages_shell_shell_tests_test_sh_single_comment` |
| 0.7100 | 231 | 164 | 67 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/toml.rs (15); src/todo_extractor_internal/aggregator.rs (9) | `rusty_todo_md` | `todo_extractor_internal_languages_toml_toml_tests_test_toml_single_comment` |
| 0.7082 | 233 | 165 | 68 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (18); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_ignore_string_literals` |
| 0.7080 | 250 | 177 | 73 | src/todo_extractor_internal/languages/python.rs (26); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_python_python_tests_test_python_docstring_multiple_todos` |
| 0.7076 | 236 | 167 | 69 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (19); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_multiline_todo` |
| 0.7031 | 229 | 161 | 68 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (22); src/test_utils.rs (20); src/todo_extractor_internal/languages/sql.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_sql_extension` |
| 0.7031 | 229 | 161 | 68 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (22); src/test_utils.rs (20); src/todo_extractor_internal/languages/yaml.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_yaml_extension` |
| 0.7031 | 229 | 161 | 68 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/sql.rs (13); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_sql_sql_tests_test_sql_line_comment` |
| 0.7000 | 230 | 161 | 69 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/go.rs (19); src/todo_extractor_internal/aggregator.rs (7) | `rusty_todo_md` | `todo_extractor_internal_languages_go_go_tests_test_go_single_line_comment` |
| 0.7000 | 230 | 161 | 69 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/yaml.rs (14); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_single_comment` |
| 0.6987 | 229 | 160 | 69 | src/todo_extractor_internal/aggregator.rs (23); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/dockerfile.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_dockerfile_no_extension` |
| 0.6966 | 234 | 163 | 71 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (15); src/todo_extractor_internal/languages/markdown.rs (13) | `rusty_todo_md` | `todo_extractor_internal_languages_markdown_markdown_tests_test_markdown_html_comment` |
| 0.6957 | 230 | 160 | 70 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/dockerfile.rs (14); src/todo_extractor_internal/aggregator.rs (13) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_single_comment` |
| 0.6953 | 233 | 162 | 71 | src/todo_extractor_internal/aggregator.rs (25); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/markdown.rs (3) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_valid_markdown_extension` |
| 0.6880 | 234 | 161 | 73 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/languages/yaml.rs (18); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_quoted_strings` |
| 0.6818 | 242 | 165 | 77 | src/logger.rs (23); src/todo_extractor_internal/languages/yaml.rs (22); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_multiple_comments` |
| 0.6765 | 238 | 161 | 77 | src/logger.rs (23); src/todo_extractor_internal/languages/yaml.rs (22); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_ignore_triple_quoted_strings` |
| 0.6735 | 245 | 165 | 80 | src/todo_extractor_internal/languages/dockerfile.rs (25); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_multiline_run_with_todo` |
| 0.6584 | 161 | 106 | 55 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_display_no_todos` |
| 0.6562 | 160 | 105 | 55 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_empty_input_no_todos` |
| 0.6503 | 163 | 106 | 57 | src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (14) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_false_positive_detection` |
| 0.6477 | 264 | 171 | 93 | src/todo_extractor_internal/languages/dockerfile.rs (37); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (13) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_mixed_comment_styles` |
| 0.6441 | 295 | 190 | 105 | src/todo_extractor_internal/languages/rust.rs (58); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_languages_rust_rust_tests_test_large_rust_file_scenario` |
| 0.6265 | 166 | 104 | 62 | src/todo_md.rs (29); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_tests_test_sync_todo_file` |
| 0.6151 | 278 | 171 | 107 | src/todo_extractor_internal/languages/dockerfile.rs (52); src/logger.rs (23); src/test_utils.rs (20); src/todo_extractor_internal/aggregator.rs (12) | `rusty_todo_md` | `todo_extractor_internal_languages_dockerfile_dockerfile_tests_test_dockerfile_multiple_todos_and_markers` |
| 0.5882 | 34 | 20 | 14 | src/todo_md_internal.rs (14) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_scanned_file_removal` |
| 0.5882 | 17 | 10 | 7 | src/exclusion.rs (7) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_invalid_pattern` |
| 0.5000 | 72 | 36 | 36 | src/exclusion.rs (36) | `rusty_todo_md` | `exclusion_tests_test_filter_excluded_files` |
| 0.4721 | 197 | 93 | 104 | src/todo_md.rs (71); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_tests_test_sync_todo_file_filters_nonexistent_files` |
| 0.4615 | 52 | 24 | 28 | src/exclusion.rs (28) | `rusty_todo_md` | `exclusion_tests_test_last_match_wins` |
| 0.4615 | 26 | 12 | 14 | src/exclusion.rs (14) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_exclude` |
| 0.4353 | 85 | 37 | 48 | src/logger.rs (21); src/todo_extractor_internal/languages/yaml.rs (16); src/test_utils.rs (10); src/todo_extractor_internal/aggregator.rs (1) | `rusty_todo_md` | `todo_extractor_internal_languages_yaml_yaml_tests_test_yaml_direct_parser` |
| 0.4286 | 28 | 12 | 16 | src/exclusion.rs (16) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_exclude_dir` |
| 0.4286 | 28 | 12 | 16 | src/exclusion.rs (16) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_exclude_dir_with_slash` |
| 0.4198 | 81 | 34 | 47 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (14); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_extract_marked_items_from_file_nonexistent_file` |
| 0.4118 | 119 | 49 | 70 | src/todo_extractor_internal/aggregator.rs (39); src/logger.rs (21); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_marker_prefilter_skips_large_marker_free_file` |
| 0.3750 | 64 | 24 | 40 | src/exclusion.rs (40) | `rusty_todo_md` | `exclusion_tests_test_should_exclude_files` |
| 0.3750 | 8 | 3 | 5 | src/exclusion.rs (5) | `rusty_todo_md` | `exclusion_tests_test_normalize_pattern` |
| 0.3678 | 87 | 32 | 55 | src/logger.rs (23); src/todo_extractor_internal/aggregator.rs (22); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_extract_marked_items_from_file_unsupported_extension` |
| 0.3571 | 14 | 5 | 9 | src/todo_extractor_internal/aggregator.rs (9) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_content_may_contain_marker_basic` |
| 0.3488 | 86 | 30 | 56 | src/todo_md.rs (46); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_tests_test_write_todo_file_sectioned` |
| 0.3380 | 71 | 24 | 47 | src/exclusion.rs (47) | `rusty_todo_md` | `exclusion_tests_test_should_exclude_directories` |
| 0.3333 | 72 | 24 | 48 | src/exclusion.rs (48) | `rusty_todo_md` | `exclusion_tests_test_should_exclude_exclude_dir_flag` |
| 0.3178 | 107 | 34 | 73 | src/todo_extractor_internal/aggregator.rs (40); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_extract_marked_items_from_file_permission_denied` |
| 0.3077 | 39 | 12 | 27 | src/exclusion.rs (27) | `rusty_todo_md` | `exclusion_tests_test_build_exclusion_matcher_multiple` |
| 0.2941 | 85 | 25 | 60 | src/logger.rs (21); src/todo_extractor_internal/aggregator.rs (20); src/test_utils.rs (19) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_invalid_extension` |
| 0.2857 | 70 | 20 | 50 | src/logger.rs (23); src/todo_md_internal.rs (17); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_no_duplicates` |
| 0.2800 | 25 | 7 | 18 | src/merge_driver.rs (18) | `rusty_todo_md` | `merge_driver_tests_build_expected_rejects_absolute_todo_path` |
| 0.2769 | 65 | 18 | 47 | src/logger.rs (21); src/todo_md_internal.rs (16); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_keeps_existing_items_when_new_empty` |
| 0.2738 | 84 | 23 | 61 | src/todo_md_internal.rs (30); src/logger.rs (21); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_sorting_order` |
| 0.2738 | 84 | 23 | 61 | src/todo_md_internal.rs (30); src/logger.rs (21); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_to_sorted_vec` |
| 0.2564 | 78 | 20 | 58 | src/todo_md_internal.rs (25); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_adds_missing_items` |
| 0.2469 | 81 | 20 | 61 | src/todo_md_internal.rs (28); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_multiple_files` |
| 0.2381 | 84 | 20 | 64 | src/todo_md_internal.rs (31); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_replaces_existing_items` |
| 0.2326 | 86 | 20 | 66 | src/todo_md_internal.rs (33); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_collections` |
| 0.2203 | 59 | 13 | 46 | src/logger.rs (21); src/todo_md_internal.rs (15); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_add_item` |
| 0.1856 | 97 | 18 | 79 | src/todo_md.rs (69); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_tests_test_read_todo_file_with_markdown_parser` |
| 0.1639 | 122 | 20 | 102 | src/todo_md_internal.rs (69); src/logger.rs (23); src/test_utils.rs (10) | `rusty_todo_md` | `todo_md_internal_tests_test_merge_complex_replacement` |
| 0.1228 | 57 | 7 | 50 | src/merge_driver.rs (50) | `rusty_todo_md` | `merge_driver_tests_build_driver_command_emits_all_args` |
| 0.0921 | 76 | 7 | 69 | src/merge_driver.rs (69) | `rusty_todo_md` | `merge_driver_tests_build_expected_quotes_path_with_specials` |
| 0.0000 | 26 | 0 | 26 | src/merge_driver.rs (26) | `rusty_todo_md` | `merge_driver_tests_format_install_summary_renders_both_states` |
| 0.0000 | 22 | 0 | 22 | src/merge_driver.rs (22) | `rusty_todo_md` | `merge_driver_tests_quote_for_gitattributes_quotes_when_whitespace_or_specials` |
| 0.0000 | 21 | 0 | 21 | src/merge_driver.rs (21) | `rusty_todo_md` | `merge_driver_tests_quote_for_gitattributes_escapes_glob_metacharacters` |
| 0.0000 | 16 | 0 | 16 | src/merge_driver.rs (16) | `rusty_todo_md` | `merge_driver_tests_quote_for_gitattributes_unquoted_when_safe` |
| 0.0000 | 14 | 0 | 14 | src/merge_driver.rs (14) | `rusty_todo_md` | `merge_driver_tests_quote_for_shell_quotes_specials_and_escapes_single_quotes` |
| 0.0000 | 14 | 0 | 14 | src/test_utils.rs (10); src/todo_extractor_internal/aggregator.rs (4) | `rusty_todo_md` | `todo_extractor_internal_aggregator_aggregator_tests_test_basic_framework` |
| 0.0000 | 11 | 0 | 11 | src/merge_driver.rs (11) | `rusty_todo_md` | `merge_driver_tests_quote_for_shell_passes_safe_strings_through` |
