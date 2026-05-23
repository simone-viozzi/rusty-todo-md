#!/usr/bin/env python3
"""Emit one self-contained subagent prompt per `tests/*.rs` candidate.

Each prompt is a single string with:
  - the subsumption rubric
  - a digest of the snapshot corpus (so the subagent can judge equivalence)
  - the candidate test's source + overlap data

Prompts are written to coverage/subagent-prompts/<name>.md so they can be
fed to subagents one by one, deterministically, without re-deriving any
context in the parent session.
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve()
EXPERIMENT_DIR = HERE.parents[1]  # docs/experiments/test-pruning-202/
ROOT = HERE.parents[4]            # repo root, retained as ROOT for path strings below
DATA = json.loads((EXPERIMENT_DIR / "overlap-data.json").read_text())
OUT = ROOT / "coverage" / "subagent-prompts"
OUT.mkdir(parents=True, exist_ok=True)

RUBRIC = """You are reviewing whether a Rust integration test under `tests/*.rs` is safe to delete because the snapshot test suite (`tests/snapshot_tests.rs` + the `tests/snapshots/*.snap` files) already catches every behavioral regression the candidate would catch.

This is a **strict subsumption check**: "would the snapshot suite fail whenever this candidate would fail?"

## Rubric

### When to DELETE
- Candidate asserts on the rendered `TODO.md` content **and** an equivalent fixture exists in the snapshot corpus that would change on the same regression.
- Candidate asserts on exit code = 0 *only* for a happy path also covered by snapshot fixtures.
- Candidate's unique-coverage lines are pure happy-path code already exercised by snapshots.

### When to KEEP
- Candidate asserts on **stderr content** or specific **error message text** — snapshots assert on stdout/file output, not stderr.
- Candidate asserts on **non-zero exit codes** or specific error paths — snapshots cover the happy path through the CLI.
- Candidate asserts on **filesystem side-effects** other than `TODO.md` content.
- Candidate exercises a **CLI flag combination** that no snapshot fixture exercises (custom `--markers`, custom `--todo-path`, glob excludes, etc.).
- Candidate exercises **git-state edge cases** that snapshots don't model (staged/unstaged distinctions, conflict markers, untracked files).
- Candidate asserts on **internal invariants** observable in the snapshot output only indirectly (a regression could pass the snapshot but break the invariant).
- High overlap ratio but the unique 5% is in a meaningful branch.
- **Any uncertainty** about subsumption-equivalence. Default to KEEP.

### Tie-breaker
When the candidate and a snapshot fixture cover the same scenario with different fixture data, lean KEEP unless the candidate's data adds nothing the snapshot's data doesn't (same shape, just different filenames/marker names).

## Output format
A single line: `DELETE` or `KEEP: <reason ≤80 chars>`. No preamble, no markdown.
"""

SNAP_DIR = ROOT / "tests" / "snapshots"
SNAPSHOT_TESTS_RS = ROOT / "tests" / "snapshot_tests.rs"


def _strip_snap_header(text: str) -> str:
    """Drop the `---\n…\n---\n` insta YAML header; return the asserted body."""
    if not text.startswith("---"):
        return text.rstrip("\n")
    parts = text.split("---\n", 2)
    if len(parts) < 3:
        return text.rstrip("\n")
    return parts[2].rstrip("\n")


def _parse_snapshot_tests() -> list[dict]:
    """Walk `tests/snapshot_tests.rs` and pull one entry per `#[test] fn`.

    Each entry carries the test's fixture name (the first `Scenario::new(..)`
    argument inside the body), the args list (best-effort), and the body
    text — enough for the digest builder to attach the matching .snap
    contents and to surface which integration test the snapshot mirrors
    (via the `Mirrors …` doc-comment convention).
    """
    text = SNAPSHOT_TESTS_RS.read_text()
    entries: list[dict] = []
    fn_re = re.compile(
        r"#\[test\][^\n]*\n((?:\s*//[^\n]*\n)*)\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)\s*\{",
        re.MULTILINE,
    )
    for m in fn_re.finditer(text):
        comments = m.group(1) or ""
        fn_name = m.group(2)
        # Find the function body by brace-balancing.
        i = m.end() - 1
        depth = 0
        start = m.end()
        body_end = None
        while i < len(text):
            c = text[i]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    body_end = i
                    break
            i += 1
        if body_end is None:
            continue
        body = text[start:body_end]
        # First "Scenario::new(\"<name>\")" → fixture name.
        fixture_m = re.search(r'Scenario::new\(\s*"([^"]+)"\s*\)', body)
        fixture = fixture_m.group(1) if fixture_m else fn_name
        # Collect any `.args([...])` arguments so the digest shows the flags.
        args_m = re.search(r"\.args\(\[([^\]]*)\]\)", body, re.DOTALL)
        args = ""
        if args_m:
            args = re.sub(r"\s+", " ", args_m.group(1).strip())
        comment_one_line = " ".join(
            line.strip().lstrip("/").strip() for line in comments.splitlines() if line.strip()
        )
        entries.append(
            {
                "fn": fn_name,
                "fixture": fixture,
                "args": args,
                "comment": comment_one_line,
            }
        )
    return entries


def _build_corpus_digest() -> str:
    """Read the live snapshot suite + .snap files and assemble a digest.

    The digest lists every snapshot scenario with the assertions it makes
    (todo_md body, stderr body, git_index body). The subagent compares
    these directly against the candidate's assertions.
    """
    entries = _parse_snapshot_tests()
    lines: list[str] = []
    lines.append("## Snapshot corpus — what the snapshot suite actually asserts on")
    lines.append("")
    lines.append(
        f"The harness in `tests/snapshot_tests.rs` runs the binary via `assert_cmd::Command::cargo_bin` "
        f"against a tempdir copy of each fixture under `tests/fixtures/snapshots/<fixture>/`. "
        f"Default args are `--markers TODO FIXME HACK --`; overrides are noted per scenario. "
        f"The captured `TODO.md` (or the literal sentinel `<no TODO.md generated>` when the binary writes nothing) "
        f"is compared to a checked-in `.snap` file; some scenarios additionally snapshot stderr "
        f"(env_logger output, with timestamps and `file:line` scrubbed) and/or the post-run "
        f"`git diff --cached --name-only`. **{len(entries)}** scenarios total."
    )
    lines.append("")
    for e in entries:
        lines.append(f"### `{e['fn']}` — fixture `{e['fixture']}`")
        if e["comment"]:
            lines.append(f"_{e['comment']}_")
        if e["args"]:
            lines.append(f"Args: `{e['args']}`")
        # Attach every .snap file matching this test fn.
        for snap in sorted(SNAP_DIR.glob(f"snapshot_tests__{e['fn']}*.snap")):
            kind = "todo_md"
            if "@" in snap.stem:
                kind = snap.stem.split("@", 1)[1]
            body = _strip_snap_header(snap.read_text())
            lines.append(f"<details><summary>{kind}</summary>")
            lines.append("")
            lines.append("```")
            lines.append(body)
            lines.append("```")
            lines.append("")
            lines.append("</details>")
        lines.append("")
    lines.append("## What the snapshot corpus still does NOT cover")
    lines.append(
        "- Merge-driver install / auto-install / self-heal-on-args-change "
        "(`install_merge_driver_*`, `auto_install_*`, `rebase_without_driver_*`)."
    )
    lines.append(
        "- Git index introspection beyond `--auto-add` "
        "(staged-vs-tracked distinctions, conflict-stage dedup)."
    )
    lines.append(
        "- Unreadable-file / corrupted-TODO.md fallback paths."
    )
    lines.append(
        "- The CLI no-files no-op exit (no fixture omits file args)."
    )
    return "\n".join(lines)


CORPUS_DIGEST = _build_corpus_digest()


def file_for_test(name: str, binary: str) -> tuple[str | None, str]:
    """Return (path, sanitized-rest) for a candidate's source file."""
    rest = name[len(binary) + 2 :]
    src_file = f"tests/{binary}.rs"
    return src_file, rest


def fn_candidates_in_file(path: pathlib.Path) -> list[tuple[str, int]]:
    """Find all `#[test] fn <name>(` definitions in `path` (any depth)."""
    out: list[tuple[str, int]] = []
    text = path.read_text()
    for m in re.finditer(
        r"#\[test\][^\n]*\n\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        text,
    ):
        out.append((m.group(1), m.start()))
    return out


def find_test_in_file(path: pathlib.Path, rest: str) -> str | None:
    """Match the sanitized test ID `rest` against actual `#[test] fn ...`
    bodies in `path`. We accept either an exact match against the
    rightmost segment of `rest` (split on `_`) or any progressively
    shorter suffix until one resolves to a unique #[test] fn."""
    candidates = fn_candidates_in_file(path)
    by_name = {n: i for n, i in candidates}
    # Try the full sanitized name first.
    if rest in by_name:
        return rest
    # Walk progressively shorter suffixes (strip one underscore-segment
    # at a time from the left) — this handles `mod foo { fn test_x(); }`
    # collapsing to the sanitized name `foo_test_x`.
    parts = rest.split("_")
    for i in range(1, len(parts)):
        candidate = "_".join(parts[i:])
        if candidate in by_name:
            return candidate
    return None


def extract_test_source(path: pathlib.Path, test_fn: str) -> str | None:
    text = path.read_text()
    pattern = re.compile(
        r"(#\[test\][^\n]*\n\s*)?fn\s+" + re.escape(test_fn) + r"\s*\([^)]*\)\s*\{",
        re.MULTILINE,
    )
    m = pattern.search(text)
    if not m:
        return None
    start = m.start()
    i = m.end() - 1
    depth = 0
    while i < len(text):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start : i + 1]
        i += 1
    return None


def candidate_prompt(entry: dict) -> str:
    binary = entry["binary"]
    name = entry["name"]
    test_path = entry["test_path"]
    src_file, rest = file_for_test(name, binary)
    src_path = ROOT / src_file if src_file else None

    body = None
    test_fn = rest
    if src_path and src_path.exists():
        resolved = find_test_in_file(src_path, rest)
        if resolved:
            test_fn = resolved
        body = extract_test_source(src_path, test_fn)
    if body is None:
        body = f"// could not extract source for {test_fn}"

    top_unique = entry.get("top_unique_files", [])
    unique_summary = (
        "; ".join(f"{t['file']} ({t['unique_lines']} lines)" for t in top_unique)
        or "none"
    )

    return f"""{RUBRIC}

{CORPUS_DIGEST}

## Candidate

- Test source file: `{src_file}`
- Test function: `{test_fn}`
- Line overlap with snapshot union: **{entry['overlap_ratio']:.4f}** ({entry['overlap_lines']} of {entry['covered_lines']} covered src lines also covered by snapshots)
- Top files with unique (non-snapshot-covered) lines: {unique_summary}

### Candidate source

```rust
{body}
```

Return your verdict on a single line, exactly: `DELETE` or `KEEP: <reason>`. Nothing else.
"""


def main() -> int:
    written = 0
    for entry in DATA["integration"]:
        if (entry["overlap_ratio"] or 0.0) < 0.70:
            continue
        prompt = candidate_prompt(entry)
        out = OUT / f"{entry['name']}.md"
        out.write_text(prompt)
        written += 1
    print(f"wrote {written} per-candidate prompts to {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
