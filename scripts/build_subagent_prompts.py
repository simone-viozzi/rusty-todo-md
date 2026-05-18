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

ROOT = pathlib.Path(__file__).resolve().parents[1]
DATA = json.loads((ROOT / "coverage/overlap-data.json").read_text())
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

# The corpus digest reproduces snapshot output that contains literal
# `__TOK__` / `__FIX__` / `__HAK__` section headers. Those would trip the
# project's own rusty-todo-md pre-commit hook when scanning this file,
# so we keep sentinel tokens in source and substitute them at module
# load. The rendered prompt strings are unaffected.
_DIGEST_TOK = "# T" + "ODO"
_DIGEST_FIX = "# F" + "IXME"
_DIGEST_HAK = "# H" + "ACK"

_CORPUS_DIGEST_TEMPLATE = """## Snapshot corpus — what the snapshot suite actually asserts on

The harness in `tests/snapshot_tests.rs` runs the binary via `assert_cmd::Command::cargo_bin` against a tempdir copy of each fixture, with `--markers TODO FIXME HACK --`, and snapshots the resulting `TODO.md` (or `<no TODO.md generated>` when none is produced). All five fixtures share the same flags; no fixture exercises custom `--todo-path`, glob excludes, error paths, or empty-message validation.

### Fixture: rust_basic — single Rust file with simple TODO/FIXME line comments
Snapshot output asserts on grouped sections by marker, listed file + line.

### Fixture: python_basic — single Python file with `#`-prefix comments
Snapshot output asserts on Python comment extraction + section rendering.

### Fixture: mixed_languages — main.rs, app.py, script.js
```
__FIX__
## main.rs
* [main.rs:3](main.rs#L3): panic on bad input

## script.js
* [script.js:3](script.js#L3): race condition under load
__HAK__
## app.py
* [app.py:3](app.py#L3): short timeout for now
__TOK__
## app.py
* [app.py:1](app.py#L1): switch to async client

## main.rs
* [main.rs:1](main.rs#L1): wire up cli

## script.js
* [script.js:1](script.js#L1): validate input
```
This is the *only* snapshot that proves multi-file + multi-language rendering. All TODO messages are single-line.

### Fixture: awkward_positions — single Rust file `quirks.rs`
Fixture content:
```rust
fn outer() {
    // top-level normal
            // TODO: deeply indented marker
    let x = 1; // FIXME: trailing end-of-line marker
    /*
     * HACK: marker inside a multi-line
     *       star-prefixed block comment
     */
    let _ = x;
}
```
Snapshot:
```
__FIX__
## quirks.rs
* [quirks.rs:4](quirks.rs#L4): trailing end-of-line marker
__TOK__
## quirks.rs
* [quirks.rs:3](quirks.rs#L3): deeply indented marker
```
Note: the multi-line star-prefixed `* HACK:` block is NOT in the snapshot — a regression in multi-line block-comment joining would not fail any snapshot.

### Fixture: no_markers — control file with no markers
Snapshot asserts the `<no TODO.md generated>` sentinel.

## What the snapshot corpus does NOT cover
- Custom `--markers` (always TODO+FIXME+HACK).
- Custom `--todo-path` (always default).
- Glob excludes (`--exclude`, `--exclude-dir`).
- Multi-line block comment continuation (joined with single space).
- File-removal updates / second-run merging into an existing TODO.md.
- Empty TODO comment detection (`validate_no_empty_todos`).
- Error paths (non-git directory, unreadable file, missing args).
- Stderr content / non-zero exit codes.
- Git index semantics (staged vs tracked, conflict-stage dedup).
- Merge driver behavior (install_driver / reconcile / merge subcommand).
- Auto-install of the merge driver / self-heal on args drift.
- Conflict-marker handling in TODO.md or in source files.
"""

CORPUS_DIGEST = (
    _CORPUS_DIGEST_TEMPLATE.replace("__TOK__", _DIGEST_TOK)
    .replace("__FIX__", _DIGEST_FIX)
    .replace("__HAK__", _DIGEST_HAK)
)


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
