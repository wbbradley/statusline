# statusline

A Rust CLI that reads [Claude Code](https://claude.ai/claude-code) session JSON from stdin and
prints a 2-line gruvbox-colored ANSI status bar.

```
╭─~/src/project──145k────────────────────────────────────╮
╰─⎇ main──+3─~2──↑1↓0──PR #42──✓ approved──● checks pass─╯
```

## Install

```bash
cargo install --path .
```

## Usage

Pipe Claude Code's status JSON into `statusline`:

```bash
echo '{"workspace":{"current_dir":"/tmp/project"}}' | statusline
```

Missing or null fields are silently omitted — any valid JSON object works.

Each invocation appends the parsed input as a compact JSON line to `~/statusline.log`.

## Line 1

| Segment | Source | Color |
|---------|--------|-------|
| Working directory | `workspace.current_dir` (tilde-contracted) | aqua |
| OS + Hostname | `target_os` + `gethostname()` (e.g. `macOS myhost`) | grey-blue |
| Context tokens | `context_window.current_usage` (input + cache) | orange |

## Line 2

| Segment | Source | Color |
|---------|--------|-------|
| Branch | git HEAD via libgit2 | green |
| Staged count (+N) | git index status | green |
| Modified count (~N) | git worktree status | yellow |
| Ahead/behind (↑N↓N) | upstream tracking | orange |
| PR number | `gh pr view` (cached) | blue |
| Review decision | `gh pr view` (cached) | green/red/yellow |
| CI checks | `gh pr view` (cached) | green/red/yellow |

PR data is cached in `~/.cache/statusline/cache.db` (SQLite, 60s TTL). If `gh` is not installed or
there is no PR for the current branch, the PR segment is silently omitted.

## Dependencies

- [git2](https://crates.io/crates/git2) — branch, status, ahead/behind
- [rusqlite](https://crates.io/crates/rusqlite) (bundled) — PR cache
- [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) — JSON parsing
- [dirs](https://crates.io/crates/dirs) — cache directory resolution
