#!/usr/bin/env bash
# Per-test coverage extraction (one-shot experiment for issue #202).
# Reads coverage/binaries.list (one instrumented test-binary path per
# line, relative to the repo root) and runs each contained #[test]
# individually with an isolated LLVM_PROFILE_FILE pattern. The pattern
# uses %p so subprocess coverage (assert_cmd::Command::cargo_bin) lands
# in distinct profraw files; all profraws are merged, and llvm-cov
# export is invoked with both the test binary AND the rusty-todo-md
# binary as objects so source files reached only via subprocess are
# visible. Output JSONs feed scripts/overlap_analysis.py.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
OUT_ROOT="$REPO_ROOT/coverage"
RAW_DIR="$OUT_ROOT/per-test-raw"
DATA_DIR="$OUT_ROOT/per-test"
JSON_DIR="$OUT_ROOT/per-test-json"
MAIN_BIN="$REPO_ROOT/target/debug/rusty-todo-md"
mkdir -p "$RAW_DIR" "$DATA_DIR" "$JSON_DIR"

if [[ ! -x "$MAIN_BIN" ]]; then
  echo "missing instrumented main binary at $MAIN_BIN" >&2
  exit 1
fi
if [[ ! -s "$OUT_ROOT/binaries.list" ]]; then
  echo "coverage/binaries.list is missing or empty" >&2
  exit 1
fi

count=0
while IFS= read -r bin; do
  [[ -z "$bin" ]] && continue
  [[ ! -x "$bin" ]] && { echo "skip non-exec: $bin" >&2; continue; }
  bin_base="${bin##*/}"
  bin_short="${bin_base%-*}"
  while IFS= read -r tname; do
    [[ -z "$tname" ]] && continue
    safe="${bin_short}__${tname//::/_}"
    safe="$(printf '%s' "$safe" | tr -c 'A-Za-z0-9._-' '_')"
    test_raw_dir="$RAW_DIR/$safe"
    profdata="$DATA_DIR/${safe}.profdata"
    json="$JSON_DIR/${safe}.json"
    if [[ -s "$json" ]]; then
      continue
    fi
    rm -rf "$test_raw_dir"
    mkdir -p "$test_raw_dir"
    # %p = pid, %16m = continuous-mode binary hash. Together these give
    # one profraw per (process, binary) pair, which is what we want.
    LLVM_PROFILE_FILE="$test_raw_dir/cov-%p-%16m.profraw" \
      "$bin" --exact "$tname" --test-threads=1 --quiet \
      > "$test_raw_dir/stdout" 2> "$test_raw_dir/stderr" || {
        echo "TEST FAILED: $bin_short :: $tname (continuing)" >&2
      }
    shopt -s nullglob
    raws=("$test_raw_dir"/*.profraw)
    shopt -u nullglob
    if (( ${#raws[@]} == 0 )); then
      echo "NO PROFRAW: $bin_short :: $tname" >&2
      continue
    fi
    llvm-profdata merge -sparse "${raws[@]}" -o "$profdata"
    llvm-cov export --format=text \
      --instr-profile="$profdata" "$bin" \
      --object "$MAIN_BIN" \
      --sources "$ROOT/src" \
      > "$json"
    count=$((count + 1))
  done < <("$bin" --list --format=terse 2>/dev/null | sed -n 's/: test$//p')
done < "$OUT_ROOT/binaries.list"

echo "wrote $count per-test json files into $JSON_DIR"
