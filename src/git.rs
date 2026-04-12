use git2::{BranchType, Repository, Status};

pub struct GitInfo {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
}

pub fn get_git_info(path: &str) -> Option<GitInfo> {
    let repo = Repository::discover(path).ok()?;
    let head = repo.head().ok()?;

    let branch = match head.shorthand() {
        Some(name) => name.to_string(),
        None => head.target()?.to_string()[..8].to_string(),
    };

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
        let local_branch = repo.find_branch(&branch, BranchType::Local).ok()?;
        let upstream = local_branch.upstream().ok()?;
        let local_oid = head.target()?;
        let upstream_oid = upstream.get().target()?;
        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid).ok()?;
        Some((ahead, behind, true))
    })()
    .unwrap_or((0, 0, false));

    Some(GitInfo {
        branch,
        staged,
        modified,
        ahead,
        behind,
        has_upstream,
    })
}
