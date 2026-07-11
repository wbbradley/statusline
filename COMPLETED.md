# COMPLETED.md

- Added line-1 `🗒️ N` segment showing `PLAN.md` line count (or `🗒️ -` when
  missing/unreadable), placed between OS+hostname and context tokens, colored
  grey-blue. Introduced `src/plan.rs` with `plan_md_line_count`, unit + integration
  tests via `tempfile`, and README segment-table entry.
- Made the `🗒 N` segment queue-aware and ancestor-aware: `plan_md_line_count`
  now counts only the lines after a `## Next Up` marker (whole-file fallback
  when absent) and walks up parent directories to find `PLAN.md`, stopping at a
  `.git` boundary (file or dir) → `🗒 -`. Added helpers `count_plan_md`,
  `next_up_offset`, `count_lines` and 9 new/updated unit tests.

## Worktree indicator on line 1

### Summary of what was implemented

- Added `is_worktree: bool` to `GitInfo`, set from `git2`'s `Repository::is_worktree()` in
  `get_git_info`, plus a new `#[cfg(test)]` module in `src/git.rs` that builds a real repo + linked
  worktree and asserts detection both ways.
- Threaded `is_worktree` through `format_line1` / `format_line1_with_env`; `main.rs` computes it
  from `git_info` and passes it to both call sites.
- Line 1 now renders a `🌿 name` segment (grey-blue, with `←branch` when
  `worktree.original_branch` is known) next to the directory whenever the session is in a worktree.
  The trigger is `is_worktree || workspace.git_worktree || worktree` metadata. The name comes from
  `git_worktree` / `worktree.name`, else the worktree dir basename. When `worktree.original_cwd` is
  known the displayed directory is contracted back to that original repo root; otherwise
  `current_dir` is shown as-is. The `🗒` PLAN.md segment still keys on the raw `current_dir`.
- Three new `format.rs` unit tests (with-original, no-parent-branch, detected-only) plus mechanical
  regression updates to all existing `GitInfo` literals and `format_line1_with_env` call sites.
  Confirmed non-worktree line 1 is byte-for-byte identical to the prior release.
- Documented the new segment in `README.md`, added a `[0.3.11]` `CHANGELOG.md` entry, and bumped
  the version `0.3.10` → `0.3.11`.

### Original PLAN.md entry (verbatim)

> ### Worktree indicator on line 1
>
> Show, on line 1 next to the working directory, when the current session is running inside a git
> worktree — including which worktree and (when known) the branch it was forked from.
>
> **Detection (both sources).** Trigger off `git2`'s `Repository::is_worktree()` in `src/git.rs`
> (the repo is already opened at `git.rs:15`), so *any* linked worktree is caught — including one
> created by hand with `git worktree add`, not just Claude-Code-managed ones. Enrich the display
> with the richer input-JSON metadata when it is present:
>
> - `workspace.git_worktree` — worktree name (e.g. `"devin-29610"`); already parsed in `input.rs`.
> - top-level `worktree` (`Worktree` struct in `input.rs`) — `name`, `branch`, `original_cwd`,
>   `original_branch`, `path`.
>
> Plumb a worktree flag/metadata out of `get_git_info` (extend `GitInfo`, or thread the input
> `Worktree` through to `format_line1`). `main.rs` already has both `git_info` and `input` in scope,
> so `format_line1` can receive whatever worktree data it needs.
>
> **Display (line 1, near the directory).** When in a worktree, render:
>
> ```
> ~/src/langchainplus 🌿 devin-29610 ←main  macos host──🗒 3──145k
> ```
>
> - Worktree glyph + name: `🌿 devin-29610`. Name comes from `workspace.git_worktree` /
>   `worktree.name`; if neither is present (manual worktree detected only via `git2`), derive the
>   name from the worktree directory's basename.
> - Parent branch, when available: from `worktree.original_branch` (populated only ~1/3 of the time
>   in practice — 77/224 real cases). Prefix with a small "forked-from" marker. Recommended marker:
>   `←` (reads `←main`); the exploration mockups used `⑂`/`⑃`, but those OCR-fork glyphs render
>   unreliably — implementer's call, default to `←`. Omit this piece entirely when
>   `original_branch` is absent.
> - Color: use `GREY_BLUE` (secondary-info color, matching the os/host and PLAN segments) unless a
>   more distinct color reads better in practice.
> - Placement: the worktree segment attaches to the directory segment (before the `os host`
>   segment), joined the same way the existing line-1 segments are.
>
> **Directory contraction (the one behavioral change to confirm — see open question).** When in a
> Claude-managed worktree the raw `current_dir` is the ugly full path
> `~/src/langchainplus/.claude/worktrees/devin-29610`. When `worktree.original_cwd` is available,
> show the tilde-contracted `original_cwd` (`~/src/langchainplus`) as the directory instead, letting
> the `🌿 devin-29610` segment carry the worktree identity. When `original_cwd` is not available
> (manual worktree via `git2` only), keep the actual `current_dir` unchanged and simply append the
> `🌿 <derived-name>` marker.
>
> **Non-worktree behavior is unchanged.** Line 1 must be byte-for-byte identical to today when not
> in a worktree. Line 2 (branch/status) is untouched — `git2` already reports the worktree's checked
> -out branch correctly there.
>
> **Files:** `src/git.rs` (detection + `GitInfo`), `src/format.rs` (`format_line1` /
> `format_line1_with_env` rendering), possibly `src/main.rs` (thread worktree metadata through),
> `README.md` (document the new segment), `CHANGELOG.md` (new entry), version bump.
>
> **Acceptance criteria:**
>
> - Given input with `workspace.git_worktree` + `worktree` populated (use the real shape:
>   `current_dir` under `.claude/worktrees/`, `original_cwd` = repo root, `original_branch` present),
>   line 1 shows the contracted `original_cwd`, then `🌿 <name>`, then `←<original_branch>`.
> - Same but with `original_branch` absent → `🌿 <name>` with no parent-branch piece.
> - A git worktree with no worktree input fields (simulate a linked worktree on disk) is still
>   detected via `git2::Repository::is_worktree()` and shows `🌿 <dir-basename>`, with the directory
>   left as-is.
> - Not in a worktree → line 1 identical to current output (regression-guarded by existing
>   `format_line1` tests, which pass `git_worktree: None`).
> - New unit tests in `format.rs` cover each branch above; existing tests continue to pass.
> - `cargo fmt`, `cargo clippy`, and `cargo test` are clean.
>
> **Open question to resolve before implementing:**
>
> 1. Confirm the directory-contraction behavior: replace the shown path with `original_cwd`
>    (`~/src/langchainplus`) as drafted, or keep the full worktree path and only append the
>    `🌿` marker? The mockup you approved implied replacement; flagging because it changes the
>    existing directory-segment behavior.

## Model-name statusline segment

### Summary of what was implemented

- Added a `model_segment` to `format_line1_with_env` in `src/format.rs`, built from
  `input.model.as_ref().and_then(|m| m.id.as_deref())` and rendered as bracketed
  plain text (`[claude-opus-4-6]`) with no `colored(...)` wrapper. It is prepended
  as the leftmost element of the `left` string, joined with the standard `sep(2)`
  divider. When `model` or `model.id` is `None` the segment is omitted and line 1
  renders exactly as before.
- Updated `test_format_line1_full` to set `id: Some("claude-opus-4-6")` and expect
  `[claude-opus-4-6]──<dir> macos myhost──🗒 3──145k`, and added
  `test_format_line1_no_model` asserting the stripped line does not lead with `[`.
- Documented the new segment as the first row of the "Line 1" table in `README.md`.

### Original PLAN.md entry (verbatim)

> ### Model-name statusline segment
>
> Render the current model as the leftmost segment on line 1, in brackets, using the
> model id. The input plumbing already exists — `StatusInput.model: Option<Model>`
> with `Model { id, display_name }` in `src/input.rs` — so this is a rendering-only
> change; no deserialization work is needed.
>
> Files to touch:
> - `src/format.rs`: in `format_line1_with_env` (the function that builds line 1),
>   add a `model_segment` built from `input.model.as_ref().and_then(|m| m.id.as_deref())`.
>   Render it as bracketed plain text, e.g. `[claude-opus-4-6]`, with minimal/no
>   color (do not wrap in `colored(...)` unless a neutral shade is clearly needed to
>   match the line). Place it as the **first (leftmost)** segment of the `left`
>   string, before the directory part, joined to the rest with the existing `sep()`
>   separator convention. Omit the segment entirely when `model` or `model.id` is
>   `None` — line 1 must still render correctly, matching the existing
>   `Option`-guarded segments.
> - `README.md`: add a row to the "Line 1" segment table (lines ~29-38) describing
>   the new segment, its source (`model.id`), and that it renders bracketed.
>
> Behavior / styling:
> - Show `model.id` only (e.g. `claude-opus-4-6`); do **not** fall back to
>   `display_name`. If `id` is absent, omit the segment.
> - Bracketed plain text `[<id>]`, leftmost on line 1.
> - Follow the `sep()` separator convention of neighboring segments so spacing/
>   dividers stay consistent.
>
> Acceptance criteria:
> - `cargo test` passes; update/extend the `format_line1` tests in `src/format.rs`
>   to assert the bracketed model id appears as the leftmost text of the
>   stripped-ANSI line 1.
> - Piping a real payload (e.g. the `test_full_json` fixture in `input.rs`) through
>   the binary shows the bracketed model id at the start of line 1.
> - `cargo fmt` is clean.
