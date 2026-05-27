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
