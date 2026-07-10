use git2::{BranchType, Repository, Status};

pub struct GitInfo {
    pub branch: String,
    pub sha: Option<String>,
    pub staged: usize,
    pub modified: usize,
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
    pub origin_url: Option<String>,
    pub is_worktree: bool,
}

pub fn get_git_info(path: &str) -> Option<GitInfo> {
    let repo = Repository::discover(path).ok()?;
    let head = repo.head().ok();

    let branch = match head.as_ref() {
        Some(h) => match h.shorthand() {
            Some(name) => name.to_string(),
            None => h.target()?.to_string()[..8].to_string(),
        },
        None => repo
            .find_reference("HEAD")
            .ok()
            .and_then(|r| r.symbolic_target().map(|s| s.to_string()))
            .map(|s| s.strip_prefix("refs/heads/").unwrap_or(&s).to_string())
            .unwrap_or_else(|| "HEAD".to_string()),
    };

    let sha = head
        .as_ref()
        .and_then(|h| h.target())
        .map(|oid| oid.to_string()[..8].to_string());

    let statuses = repo.statuses(None).ok()?;
    let mut staged = 0usize;
    let mut modified = 0usize;
    for entry in statuses.iter() {
        let s = entry.status();
        if s.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }
        if s.intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE,
        ) {
            modified += 1;
        }
    }

    let (ahead, behind, has_upstream) = (|| {
        let head = head.as_ref()?;
        let local_branch = repo.find_branch(&branch, BranchType::Local).ok()?;
        let upstream = local_branch.upstream().ok()?;
        let local_oid = head.target()?;
        let upstream_oid = upstream.get().target()?;
        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid).ok()?;
        Some((ahead, behind, true))
    })()
    .unwrap_or((0, 0, false));

    let origin_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()));

    let is_worktree = repo.is_worktree();

    Some(GitInfo {
        branch,
        sha,
        staged,
        modified,
        ahead,
        behind,
        has_upstream,
        origin_url,
        is_worktree,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use tempfile::TempDir;

    fn init_with_commit(path: &std::path::Path) -> Repository {
        let repo = Repository::init(path).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        {
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
                .unwrap();
        }
        repo
    }

    #[test]
    fn detects_linked_worktree() {
        let dir = TempDir::new().unwrap();
        let repo = init_with_commit(dir.path());
        // Main working tree: not a linked worktree.
        let main_info = get_git_info(dir.path().to_str().unwrap()).unwrap();
        assert!(!main_info.is_worktree);
        // Linked worktree: detected.
        let wt_parent = TempDir::new().unwrap();
        let wt_path = wt_parent.path().join("wt");
        repo.worktree("wt", &wt_path, None).unwrap();
        let wt_info = get_git_info(wt_path.to_str().unwrap()).unwrap();
        assert!(wt_info.is_worktree);
    }
}
