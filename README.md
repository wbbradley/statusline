# statusline

A Rust CLI that reads [Claude Code](https://claude.ai/claude-code) session JSON from stdin and
prints a 2-line gruvbox-colored ANSI status bar.

```
[Opus]  ~/src/project  ctx: 145k  total: 230k  12m
⎇ main  +3 ~2  ↑1↓0  PR #42 ✓ approved  ● checks pass
```

## Install

```bash
cargo install --path .
```

## Usage

Pipe Claude Code's status JSON into `statusline`:

```bash
echo '{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/tmp/project"}}' | statusline
```

Missing or null fields are silently omitted — any valid JSON object works.

## Line 1

| Segment | Source | Color |
|---------|--------|-------|
| Model name | `model.display_name` | blue |
| Working directory | `workspace.current_dir` (tilde-contracted) | aqua |
| Context tokens | `context_window.current_usage` (input + cache) | yellow |
| Total tokens | `context_window.total_input_tokens` | yellow |
| Duration | `cost.total_duration_ms` | gray |

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
