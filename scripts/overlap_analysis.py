#!/usr/bin/env python3
"""Per-test overlap analysis vs the snapshot test suite.

Inputs: coverage/per-test-json/*.json (one per #[test], produced by
extract_per_test_coverage.sh).

For each test we build a set of (file, line) keys covered by that test
(line in src/* only — third-party code is uninteresting). The snapshot
test suite's union becomes the reference set. Per-test overlap is the
fraction of the candidate test's covered src lines that are also covered
by the snapshot union.

Outputs:
- coverage/overlap-report.md (human-readable, with distribution + tables)
- coverage/overlap-data.json (machine-readable, consumed by stage 3)
"""

from __future__ import annotations

import collections
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
JSON_DIR = ROOT / "coverage" / "per-test-json"
REPORT_PATH = ROOT / "coverage" / "overlap-report.md"
DATA_PATH = ROOT / "coverage" / "overlap-data.json"
SRC_PREFIX = str(ROOT / "src") + "/"

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

    lines: list[str] = []
    lines.append("# Per-test coverage overlap vs snapshot suite (issue #202)")
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
    lines.append("Reproducer: `scripts/extract_per_test_coverage.sh`, then")
    lines.append("`scripts/overlap_analysis.py`.")
    lines.append("")
    lines.append("**Branch coverage.** Stable rustc 1.95 does not emit branch counts;")
    lines.append("`cargo llvm-cov --branch` needs nightly. This pass uses line overlap only.")
    lines.append("The QA framed branch as an enhancement on top of line overlap, not a")
    lines.append("substitute. The false-flag rate is mitigated by the per-candidate")
    lines.append("subagent review in stage 3 (see `triage-verdicts.md`).")
    lines.append("")
    lines.append("**Outcome of stage 3 (preview):** the subagent fan-out returned KEEP for")
    lines.append("all 28 `tests/*.rs` candidates with overlap ≥ 0.70. The snapshot corpus")
    lines.append("(5 happy-path fixtures, fixed flags, no error paths) is too narrow to")
    lines.append("subsume any integration test — every candidate reaches an error path,")
    lines.append("flag combo, multi-run update, merge-driver path, or internal invariant")
    lines.append("the snapshot suite does not assert on. No deletions in this PR; see")
    lines.append("`triage-verdicts.md` for the full per-candidate breakdown.")
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
