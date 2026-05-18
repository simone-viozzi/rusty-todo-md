#!/usr/bin/env python3
"""Per-test overlap analysis vs the snapshot test suite (issue #202).

Inputs: coverage/per-test-json/*.json (one per #[test], produced by
extract_per_test_coverage.sh — that script writes them at the repo root
under coverage/, not under this experiment directory).

For each test we build a set of (file, line) keys covered by that test
(line in src/* only — third-party code is uninteresting). The snapshot
test suite's union becomes the reference set. Per-test overlap is the
fraction of the candidate test's covered src lines that are also covered
by the snapshot union.

Outputs (rewritten in-place next to this script):
- ../overlap-report.md (human-readable, with distribution + tables)
- ../overlap-data.json (machine-readable, consumed by stage 3)
"""

from __future__ import annotations

import collections
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve()
EXPERIMENT_DIR = HERE.parents[1]  # docs/experiments/test-pruning-202/
REPO_ROOT = HERE.parents[4]       # repo root
JSON_DIR = REPO_ROOT / "coverage" / "per-test-json"
REPORT_PATH = EXPERIMENT_DIR / "overlap-report.md"
DATA_PATH = EXPERIMENT_DIR / "overlap-data.json"
SRC_PREFIX = str(REPO_ROOT / "src") + "/"
ROOT = REPO_ROOT  # backwards-compat alias used in path strings below

SNAPSHOT_PREFIX = "snapshot_tests__"


def covered_lines(file_obj: dict) -> set[tuple[str, int]]:
    """Walk segments and return the set of source lines actually executed.

    LLVM segments mark region boundaries: each segment carries a count
    that applies until the next segment. A line is considered covered
    when it falls inside any region with count > 0 and HasCount=True and
    is not a gap region.
    """
    filename = file_obj["filename"]
    if not filename.startswith(SRC_PREFIX):
        return set()
    short = filename[len(str(ROOT)) + 1 :]
    segs = file_obj.get("segments") or []
    if not segs:
        return set()
    lines: set[tuple[str, int]] = set()
    # segments are pre-sorted, but sort defensively.
    segs = sorted(segs, key=lambda s: (s[0], s[1]))
    for i, seg in enumerate(segs):
        line, _col, count, has_count, _is_region_entry, is_gap = seg
        if not has_count or is_gap or count == 0:
            continue
        # The region ends at the start of the next segment (exclusive).
        if i + 1 < len(segs):
            end_line = segs[i + 1][0]
            # If next segment starts on the same line, we still include
            # this line. If it's on a later line, we include up to but
            # not including that line.
            for ln in range(line, end_line + 1):
                lines.add((short, ln))
        else:
            lines.add((short, line))
    return lines


def load_test_coverage(path: pathlib.Path) -> set[tuple[str, int]]:
    data = json.loads(path.read_text())
    result: set[tuple[str, int]] = set()
    for export in data.get("data", []):
        for f in export.get("files", []):
            result |= covered_lines(f)
    return result


def parse_test_id(stem: str) -> tuple[str, str]:
    """Split a sanitized filename like `cli_error_tests__test_foo` into
    (binary_short_name, test_path). For in-source tests inside the
    library binary the prefix is `rusty_todo_md` and the rest is the
    module path with `_` instead of `::` (lossy, but informational)."""
    if "__" not in stem:
        return stem, "?"
    binary, _, rest = stem.partition("__")
    return binary, rest


def main() -> int:
    if not JSON_DIR.exists():
        print(f"missing {JSON_DIR}", file=sys.stderr)
        return 1

    per_test: dict[str, set[tuple[str, int]]] = {}
    for p in sorted(JSON_DIR.glob("*.json")):
        per_test[p.stem] = load_test_coverage(p)

    snapshot_tests = {name for name in per_test if name.startswith(SNAPSHOT_PREFIX)}
    if not snapshot_tests:
        print("no snapshot tests found", file=sys.stderr)
        return 1

    cov_snapshot: set[tuple[str, int]] = set()
    for name in snapshot_tests:
        cov_snapshot |= per_test[name]

    # Group candidates by scope:
    #   - tests/*.rs  : binary != rusty_todo_md and != snapshot_tests
    #   - in-source   : binary == rusty_todo_md
    integration: list[dict] = []
    in_source: list[dict] = []
    for name, cov in per_test.items():
        if name in snapshot_tests:
            continue
        binary, test_path = parse_test_id(name)
        if not cov:
            entry = {
                "name": name,
                "binary": binary,
                "test_path": test_path,
                "covered_lines": 0,
                "overlap_lines": 0,
                "overlap_ratio": None,
                "unique_lines": [],
                "top_unique_files": [],
            }
        else:
            shared = cov & cov_snapshot
            unique = cov - cov_snapshot
            unique_by_file: dict[str, int] = collections.Counter()
            for f, _ln in unique:
                unique_by_file[f] += 1
            top_unique = sorted(
                unique_by_file.items(), key=lambda kv: -kv[1]
            )[:5]
            entry = {
                "name": name,
                "binary": binary,
                "test_path": test_path,
                "covered_lines": len(cov),
                "overlap_lines": len(shared),
                "overlap_ratio": round(len(shared) / len(cov), 4),
                "unique_lines": sorted(
                    f"{f}:{ln}" for f, ln in unique
                ),
                "top_unique_files": [
                    {"file": f, "unique_lines": n} for f, n in top_unique
                ],
            }
        if binary == "rusty_todo_md":
            in_source.append(entry)
        else:
            integration.append(entry)

    integration.sort(
        key=lambda e: (-(e["overlap_ratio"] or 0.0), -e["covered_lines"])
    )
    in_source.sort(
        key=lambda e: (-(e["overlap_ratio"] or 0.0), -e["covered_lines"])
    )

    histogram = collections.Counter()
    bucket_labels = [
        (1.0, "1.00 (exactly)"),
        (0.99, "0.99–<1.00"),
        (0.95, "0.95–0.99"),
        (0.90, "0.90–0.95"),
        (0.80, "0.80–0.90"),
        (0.70, "0.70–0.80"),
        (0.50, "0.50–0.70"),
        (0.25, "0.25–0.50"),
        (0.0, "<0.25"),
    ]

    def bucket(r: float | None) -> str:
        if r is None:
            return "n/a"
        for thresh, label in bucket_labels:
            if r >= thresh:
                return label
        return "<0.25"

    all_candidates = integration + in_source
    for e in all_candidates:
        histogram[bucket(e["overlap_ratio"])] += 1

    DATA_PATH.write_text(
        json.dumps(
            {
                "snapshot_union_size": len(cov_snapshot),
                "snapshot_tests": sorted(snapshot_tests),
                "integration": integration,
                "in_source": in_source,
            },
            indent=2,
        )
    )

    def fmt_ratio(r: float | None) -> str:
        return "n/a" if r is None else f"{r:.4f}"

    def fmt_top(top: list[dict]) -> str:
        if not top:
            return ""
        return "; ".join(f"{t['file']} ({t['unique_lines']})" for t in top)

    # Coverage of src/ by three groups, capped by executable line count
    # so the segment-expansion proxy doesn't double-count:
    #   1. snapshot-only (tests/snapshot_tests.rs)
    #   2. integration-only (tests/*.rs minus snapshot_tests.rs)
    #   3. all tests (also includes in-source #[cfg(test)] modules)
    INTG_PREFIXES = (
        "cli_error_tests",
        "cli_no_files_tests",
        "empty_todo_validation_tests",
        "git_tests",
        "glob_exclude_tests",
        "integration",
        "merge_driver_tests",
        "multi_language_tests",
    )
    snap_lines_by_file: dict[str, set[int]] = {}
    intg_lines_by_file: dict[str, set[int]] = {}
    all_lines_by_file: dict[str, set[int]] = {}
    exec_lines_by_file: dict[str, int] = {}
    for p in sorted(JSON_DIR.glob("*.json")):
        d = json.loads(p.read_text())
        is_snap = p.name.startswith("snapshot_tests__")
        is_intg = p.name.startswith(INTG_PREFIXES)
        for f in d["data"][0]["files"]:
            fn = f["filename"]
            if not fn.startswith(SRC_PREFIX):
                continue
            short = fn[len(str(REPO_ROOT)) + 1 :]
            exec_lines_by_file[short] = f["summary"]["lines"]["count"]
            file_lines = {ln for (_f, ln) in covered_lines(f) if _f == short}
            all_lines_by_file.setdefault(short, set()).update(file_lines)
            if is_snap:
                snap_lines_by_file.setdefault(short, set()).update(file_lines)
            if is_intg:
                intg_lines_by_file.setdefault(short, set()).update(file_lines)

    lines: list[str] = []
    lines.append("# Per-test coverage overlap vs snapshot suite (issue #202)")
    lines.append("")
    lines.append("> **Generated file.** Rewritten by")
    lines.append("> `docs/experiments/test-pruning-202/scripts/overlap_analysis.py`")
    lines.append("> from the per-test JSON corpus under `coverage/per-test-json/`")
    lines.append("> (produced by the sibling `extract_per_test_coverage.sh`). The PR")
    lines.append("> only commits this report, not the JSON inputs.")
    lines.append("")
    lines.append(
        f"Snapshot union covers **{len(cov_snapshot)} distinct (file, line) keys** in `src/`."
    )
    lines.append("")
    lines.append("**Scope of measurement.** Only lines under `src/` are counted —")
    lines.append("third-party code (tests/utils.rs, cargo registry, std) is excluded.")
    lines.append("Per-test coverage is collected with `cargo llvm-cov` + `LLVM_PROFILE_FILE`")
    lines.append("per process; subprocess coverage from `assert_cmd::Command::cargo_bin`")
    lines.append("propagates correctly via the `%p` substitution in the profile path.")
    lines.append("")
    lines.append("**Branch coverage.** Stable rustc 1.95 does not emit branch counts;")
    lines.append("`cargo llvm-cov --branch` needs nightly. This pass uses line overlap only.")
    lines.append("")
    lines.append("**Outcome of stage 3 (preview):** the subagent fan-out returned KEEP for")
    lines.append("all 28 `tests/*.rs` candidates with overlap ≥ 0.70. The snapshot corpus")
    lines.append("(5 happy-path fixtures, fixed flags, no error paths) is too narrow to")
    lines.append("subsume any integration test. No deletions in this PR; see")
    lines.append("`triage-verdicts.md` for the full per-candidate breakdown.")
    lines.append("")
    lines.append("## Coverage of `src/` per test group")
    lines.append("")
    lines.append("Three columns, each capped at the file's executable line count:")
    lines.append("")
    lines.append("- **snap%** — `tests/snapshot_tests.rs` alone (the new primary signal).")
    lines.append("- **intg%** — every other file under `tests/*.rs` (the integration")
    lines.append("  suite this PR was meant to prune).")
    lines.append("- **all%** — everything, including in-source `#[cfg(test)]` modules.")
    lines.append("")
    lines.append("The gap `intg% − snap%` is exactly the slack the snapshot corpus")
    lines.append("would need to grow to absorb before any `tests/*.rs` file becomes")
    lines.append("a safe-delete candidate.")
    lines.append("")
    lines.append("| file | snap% | intg% | all% | executable |")
    lines.append("|---|---:|---:|---:|---:|")
    snap_total = intg_total = all_total = exec_total = 0
    for fn in sorted(exec_lines_by_file):
        tot = exec_lines_by_file[fn]
        s = min(len(snap_lines_by_file.get(fn, set())), tot)
        i = min(len(intg_lines_by_file.get(fn, set())), tot)
        a = min(len(all_lines_by_file.get(fn, set())), tot)

        def pct(x: int) -> str:
            return f"{100.0 * x / tot:.0f}%" if tot else "-"

        lines.append(f"| `{fn}` | {pct(s)} | {pct(i)} | {pct(a)} | {tot} |")
        snap_total += s
        intg_total += i
        all_total += a
        exec_total += tot

    def total_pct(x: int) -> str:
        return f"{100.0 * x / exec_total:.1f}%" if exec_total else "-"

    lines.append(
        f"| **total** | **{total_pct(snap_total)}** | **{total_pct(intg_total)}** | "
        f"**{total_pct(all_total)}** | **{exec_total}** |"
    )
    lines.append("")
    lines.append("Notes to avoid misreading the 0%/100% rows:")
    lines.append("")
    lines.append("- A 0% in **snap%** does NOT mean nothing covers that code — it")
    lines.append("  means snapshots don't. The in-source `dockerfile_tests` etc. do")
    lines.append("  exercise their matching `languages/<x>.rs`, but in a separate")
    lines.append("  test binary not shown in the snap column.")
    lines.append("- Each `languages/<x>.rs` file shows **3 executable lines** because")
    lines.append("  the snapshot/integration binaries compile the library *without*")
    lines.append("  `#[cfg(test)]`, exposing only the production `impl CommentParser`")
    lines.append("  body. Measuring against the in-source test binary instead would")
    lines.append("  show ~120 lines per language file (production + test-mod).")
    lines.append("- `shell/sql/toml/yaml.rs` are at intg% = 0% / all% = 100%: only")
    lines.append("  their in-source unit tests reach them. Deleting those tests would")
    lines.append("  leave those parsers with no test coverage at all.")
    lines.append("")
    lines.append("## Distribution of overlap ratio across non-snapshot tests")
    lines.append("")
    lines.append("| Bucket | Count |")
    lines.append("|---|---|")
    for _t, label in bucket_labels:
        lines.append(f"| {label} | {histogram.get(label, 0)} |")
    if "n/a" in histogram:
        lines.append(f"| n/a (no src/ lines covered) | {histogram['n/a']} |")
    lines.append("")
    lines.append(
        f"Total non-snapshot tests measured: {len(all_candidates)} "
        f"({len(integration)} `tests/*.rs` + {len(in_source)} in-source)."
    )
    lines.append("")

    def emit_table(title: str, rows: list[dict]) -> None:
        lines.append(f"## {title}")
        lines.append("")
        lines.append(
            "| overlap | covered | overlap_lines | unique | top unique files | binary | test |"
        )
        lines.append(
            "|---|---|---|---|---|---|---|"
        )
        for r in rows:
            unique_count = (
                r["covered_lines"] - r["overlap_lines"]
                if r["overlap_ratio"] is not None
                else 0
            )
            lines.append(
                "| {ratio} | {cov} | {ov} | {uniq} | {top} | `{bin}` | `{tp}` |".format(
                    ratio=fmt_ratio(r["overlap_ratio"]),
                    cov=r["covered_lines"],
                    ov=r["overlap_lines"],
                    uniq=unique_count,
                    top=fmt_top(r["top_unique_files"]),
                    bin=r["binary"],
                    tp=r["test_path"],
                )
            )
        lines.append("")

    emit_table(
        "tests/*.rs candidates (in scope for deletion in this PR)", integration
    )
    emit_table(
        "in-source #[cfg(test)] candidates (FLAG-ONLY — parked for post-#190)",
        in_source,
    )

    REPORT_PATH.write_text("\n".join(lines))
    print(f"wrote {REPORT_PATH} and {DATA_PATH}")
    print(
        f"snapshot union size: {len(cov_snapshot)} lines | "
        f"{len(integration)} integration + {len(in_source)} in-source candidates"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
