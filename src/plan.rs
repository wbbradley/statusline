use std::path::Path;

pub fn plan_md_line_count(dir: &Path) -> Option<usize> {
    let mut current = Some(dir);
    while let Some(d) = current {
        let path = d.join("PLAN.md");
        if let Ok(meta) = std::fs::metadata(&path)
            && meta.is_file()
        {
            // Found a regular-file PLAN.md: use it and stop (even if the
            // read fails, we do not keep walking).
            return count_plan_md(&path);
        }
        // PLAN.md exists but is not a regular file (e.g. a directory): do not
        // match it; fall through to the .git boundary check.
        // .git as a file (worktrees/submodules) or directory ends the search.
        if d.join(".git").exists() {
            return None;
        }
        current = d.parent();
    }
    None
}

fn count_plan_md(path: &Path) -> Option<usize> {
    let contents = std::fs::read_to_string(path).ok()?;
    let counted = match next_up_offset(&contents) {
        Some(offset) => &contents[offset..],
        None => contents.as_str(),
    };
    Some(count_lines(counted))
}

/// Byte offset of the start of the line immediately following the first
/// `## Next Up` marker line (trimmed of surrounding whitespace), if present.
fn next_up_offset(contents: &str) -> Option<usize> {
    let mut offset = 0;
    for line in contents.split_inclusive('\n') {
        if line.trim() == "## Next Up" {
            return Some(offset + line.len());
        }
        offset += line.len();
    }
    None
}

/// Existing trailing-newline-aware line count: empty -> 0; otherwise number of
/// '\n' plus one if the slice does not end in '\n'.
fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }
    let mut n = s.matches('\n').count();
    if !s.ends_with('\n') {
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn missing_file_returns_none() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        assert_eq!(plan_md_line_count(dir.path()), None);
    }

    #[test]
    fn empty_file_returns_zero() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(0));
    }

    #[test]
    fn newline_terminated_lines() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "a\nb\nc\n").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(3));
    }

    #[test]
    fn no_trailing_newline() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "a\nb\nc").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(3));
    }

    #[test]
    fn directory_named_plan_md_returns_none() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("PLAN.md")).unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        assert_eq!(plan_md_line_count(dir.path()), None);
    }

    #[test]
    fn marker_counts_lines_after() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("PLAN.md"),
            "# PLAN.md\n\n## Next Up\n\ntask one\ntask two\n",
        )
        .unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(3));
    }

    #[test]
    fn marker_absent_counts_whole_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "# Notes\na\nb\n").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(3));
    }

    #[test]
    fn marker_with_surrounding_whitespace_matches() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "# h\n  ## Next Up   \nx\n").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(1));
    }

    #[test]
    fn empty_queue_after_marker_is_zero() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "# PLAN.md\n\n## Next Up\n").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(0));
    }

    #[test]
    fn blank_line_after_marker_counted() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "## Next Up\n\n").unwrap();
        assert_eq!(plan_md_line_count(dir.path()), Some(1));
    }

    #[test]
    fn ancestor_directory_discovery() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "a\nb\n").unwrap();
        let deep = dir.path().join("sub").join("deep");
        fs::create_dir_all(&deep).unwrap();
        assert_eq!(plan_md_line_count(&deep), Some(2));
    }

    #[test]
    fn git_dir_boundary_short_circuits() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "a\nb\n").unwrap();
        let repo = dir.path().join("repo");
        let sub = repo.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir(repo.join(".git")).unwrap();
        assert_eq!(plan_md_line_count(&sub), None);
    }

    #[test]
    fn git_file_boundary_short_circuits() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("PLAN.md"), "a\nb\n").unwrap();
        let wt = dir.path().join("wt");
        fs::create_dir(&wt).unwrap();
        fs::write(wt.join(".git"), "gitdir: /somewhere\n").unwrap();
        assert_eq!(plan_md_line_count(&wt), None);
    }
}
