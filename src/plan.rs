use std::path::Path;

pub fn plan_md_line_count(dir: &Path) -> Option<usize> {
    let path = dir.join("PLAN.md");
    let meta = std::fs::metadata(&path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let contents = std::fs::read_to_string(&path).ok()?;
    if contents.is_empty() {
        return Some(0);
    }
    let mut n = contents.matches('\n').count();
    if !contents.ends_with('\n') {
        n += 1;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn missing_file_returns_none() {
        let dir = TempDir::new().unwrap();
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
        assert_eq!(plan_md_line_count(dir.path()), None);
    }
}
