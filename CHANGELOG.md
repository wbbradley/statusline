# Changelog

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
