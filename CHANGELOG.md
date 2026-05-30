# Changelog

## [0.3.10] - 2026-05-30

### Changed

- PLAN.md line-count segment is now queue-aware: when a `## Next Up` marker is present, only lines
  after that marker are counted (whole-file fallback when the marker is absent)
- PLAN.md is now discovered by walking up parent directories from the workspace, short-circuiting
  to no segment at a `.git` boundary, so the segment works from any subdirectory of a project

## [0.3.9] - 2026-05-21

### Added

- New line-1 segment shows the line count of `PLAN.md` in the workspace directory as `🗒 N`
  (grey-blue), placed between the OS+hostname segment and the right-aligned context tokens.
  Renders `🗒 -` when `PLAN.md` is missing or unreadable. Suppressed only when no workspace
  directory is resolved. No caching — the file is read on every render.

### Fixed

- Notepad glyph no longer renders with an extra cell of space on the first paint in some
  terminals; the U+FE0F variation selector has been dropped from the emitted glyph, which
  also aligns `visible_width` with the terminal's 1-cell rendering

## [0.3.8] - 2026-05-09

### Fixed

- Branches with no PR no longer re-run `gh pr view` on every invocation — the negative result is
  now cached, so the statusline stops hammering `gh` when the current branch has no PR
- When a TTL-boundary re-fetch failed, the cached `fetched_at` was not updated, causing every
  subsequent invocation to retry; the cache row is now always rewritten on a fetch attempt

### Changed

- PR cache TTL extended from 60 seconds to 5 minutes
- PR cache now busts early when the local HEAD SHA changes, so a fresh push picks up new PR state
  without waiting for the TTL
- PR cache schema migrated from `pr_cache` to `pr_cache_v2` (adds a `sha` column); the old table
  is dropped automatically on first run

## [0.3.7] - 2026-05-07

### Added

- Show the first 8 hex digits of the git commit SHA after the branch name on line 2

### Fixed

- Update line 1 unit-test fixtures to match the lowercased OS rendering introduced in 0.3.6

## [0.3.6] - 2026-04-27

### Changed

- Render the OS name in lowercase on line 1 (e.g. `macos myhost` instead of `macOS myhost`)

## [0.3.5] - 2026-04-27

### Changed

- On line 1, render the OS name before the hostname (e.g. `macOS myhost` instead of `myhost macOS`)

## [0.3.4] - 2026-04-21

### Changed

- Show the hostname on all platforms (previously Linux-only) and append the OS name (`macOS`, `Linux`, etc.) after it on line 1

## [0.3.3] - 2026-04-19

### Added

- On Linux, show the hostname after the working directory on line 1 in a darkish grey-blue

## [0.3.2] - 2026-04-14

### Fixed

- Render line 2 in repositories with no commits yet (unborn branch); branch name is read from the symbolic `HEAD` ref and ahead/behind is skipped
- Always emit the second framed line even outside git repositories, so the status bar shape is consistent (empty line 2 renders as just the bottom border)

## [0.3.1] - 2026-04-14

### Added

- Append each parsed stdin payload as a compact JSON line to `~/statusline.log`

## [0.3.0] - 2026-04-12

### Breaking Changes

- Removed total input tokens and session duration segments from line 1
- Output frame changed from `│ ... │` with space padding to `╭─...─╮` / `╰─...─╯` with `─` separators

### Changed

- Context token count displayed in orange without `ctx:` prefix
- Segment separators are now horizontal bar characters (`──`) instead of spaces
- Border color changed to `#978771`

### Removed

- Total input tokens segment from line 1
- Session duration segment from line 1
- `GRAY` color constant and `format_duration` helper

## [0.2.0] - 2026-04-12

### Breaking Changes

- Removed model name display (`[Opus]`, `[Sonnet]`, etc.) from line 1
- Output lines are now wrapped in a box-drawing frame (`│ ... │`) with equal-width padding and a dark background color

### Added

- Box-drawing frame around status lines with consistent width padding
- Dark background color applied to the status bar for contrast against terminal backgrounds

### Changed

- Main output logic refactored to collect lines and batch-frame them for aligned borders

### Removed

- Model name segment from the first status line

## [0.1.0] - 2026-04-11

### Added

- Read Claude Code session JSON from stdin and display a 2-line gruvbox-colored ANSI status bar
- Line 1: model name, working directory (tilde-contracted), context/total token counts, and session duration
- Line 2: git branch name, staged/modified file counts, and ahead/behind remote tracking info via libgit2
- GitHub PR number, review state, and CI check status appended to line 2 when a PR exists for the current branch via `gh` CLI
- SQLite cache for PR lookups with a 60-second TTL
- Graceful error handling on malformed input or stdin failures
- Suppress blank output when no displayable data is present
