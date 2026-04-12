use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;
use serde::Deserialize;

#[derive(Deserialize)]
struct GhPrData {
    number: u64,
    #[serde(rename = "reviewDecision")]
    review_decision: String,
    #[serde(rename = "statusCheckRollup", default)]
    status_check_rollup: Vec<CheckItem>,
}

#[derive(Deserialize)]
struct CheckItem {
    conclusion: Option<String>,
    state: Option<String>,
}

pub struct PrInfo {
    pub number: u64,
    pub review_decision: ReviewDecision,
    pub checks: ChecksStatus,
}

pub enum ReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
    None,
}

pub enum ChecksStatus {
    Pass,
    Fail,
    Pending,
    None,
}

const CACHE_TTL_SECS: i64 = 60;

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn open_cache_db() -> Option<Connection> {
    let cache_dir = dirs::cache_dir()?.join("statusline");
    std::fs::create_dir_all(&cache_dir).ok()?;
    let db_path = cache_dir.join("cache.db");
    let conn = Connection::open(db_path).ok()?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pr_cache(
            repo TEXT,
            branch TEXT,
            data TEXT,
            fetched_at INTEGER,
            PRIMARY KEY(repo, branch)
        )",
    )
    .ok()?;
    Some(conn)
}

pub fn repo_slug(origin_url: &str) -> Option<String> {
    let url = origin_url.strip_suffix(".git").unwrap_or(origin_url);
    if let Some(rest) = url.strip_prefix("git@github.com:")
        && rest.contains('/')
    {
        return Some(rest.to_string());
    }
    if let Some(rest) = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        && rest.contains('/')
    {
        return Some(rest.to_string());
    }
    Option::None
}

fn fetch_from_gh(slug: &str, branch: &str) -> Option<String> {
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            branch,
            "--repo",
            slug,
            "--json",
            "number,reviewDecision,statusCheckRollup",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return Option::None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn convert_gh_data(data: &GhPrData) -> PrInfo {
    let review_decision = match data.review_decision.as_str() {
        "APPROVED" => ReviewDecision::Approved,
        "CHANGES_REQUESTED" => ReviewDecision::ChangesRequested,
        "REVIEW_REQUIRED" => ReviewDecision::ReviewRequired,
        _ => ReviewDecision::None,
    };

    let checks = if data.status_check_rollup.is_empty() {
        ChecksStatus::None
    } else {
        let any_fail = data.status_check_rollup.iter().any(|c| {
            c.conclusion.as_deref() == Some("FAILURE")
                || c.state.as_deref() == Some("FAILURE")
                || c.state.as_deref() == Some("ERROR")
        });
        if any_fail {
            ChecksStatus::Fail
        } else {
            let all_pass = data.status_check_rollup.iter().all(|c| {
                c.conclusion.as_deref() == Some("SUCCESS") || c.state.as_deref() == Some("SUCCESS")
            });
            if all_pass {
                ChecksStatus::Pass
            } else {
                ChecksStatus::Pending
            }
        }
    };

    PrInfo {
        number: data.number,
        review_decision,
        checks,
    }
}

pub fn get_pr_info(origin_url: &str, branch: &str) -> Option<PrInfo> {
    let slug = repo_slug(origin_url)?;
    let conn = open_cache_db()?;

    let cached: Option<(String, i64)> = conn
        .query_row(
            "SELECT data, fetched_at FROM pr_cache WHERE repo = ?1 AND branch = ?2",
            [&slug, branch],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let json_str = if let Some((data, fetched_at)) = cached {
        if now_secs() - fetched_at < CACHE_TTL_SECS {
            data
        } else {
            match fetch_from_gh(&slug, branch) {
                Some(fresh) => {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO pr_cache(repo, branch, data, fetched_at) VALUES (?1, ?2, ?3, ?4)",
                        (&slug, branch, &fresh, now_secs()),
                    );
                    fresh
                }
                Option::None => data,
            }
        }
    } else {
        let fresh = fetch_from_gh(&slug, branch)?;
        let _ = conn.execute(
            "INSERT OR REPLACE INTO pr_cache(repo, branch, data, fetched_at) VALUES (?1, ?2, ?3, ?4)",
            (&slug, branch, &fresh, now_secs()),
        );
        fresh
    };

    let gh_data: GhPrData = serde_json::from_str(&json_str).ok()?;
    Some(convert_gh_data(&gh_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_slug_ssh() {
        assert_eq!(
            repo_slug("git@github.com:user/repo.git"),
            Some("user/repo".to_string())
        );
    }

    #[test]
    fn test_repo_slug_https() {
        assert_eq!(
            repo_slug("https://github.com/user/repo.git"),
            Some("user/repo".to_string())
        );
    }

    #[test]
    fn test_repo_slug_https_no_git() {
        assert_eq!(
            repo_slug("https://github.com/user/repo"),
            Some("user/repo".to_string())
        );
    }

    #[test]
    fn test_repo_slug_invalid() {
        assert_eq!(repo_slug("not-a-url"), Option::None);
    }

    #[test]
    fn test_convert_gh_data_approved_pass() {
        let data = GhPrData {
            number: 42,
            review_decision: "APPROVED".to_string(),
            status_check_rollup: vec![CheckItem {
                conclusion: Some("SUCCESS".to_string()),
                state: Option::None,
            }],
        };
        let info = convert_gh_data(&data);
        assert_eq!(info.number, 42);
        assert!(matches!(info.review_decision, ReviewDecision::Approved));
        assert!(matches!(info.checks, ChecksStatus::Pass));
    }

    #[test]
    fn test_convert_gh_data_empty_checks() {
        let data = GhPrData {
            number: 10,
            review_decision: "APPROVED".to_string(),
            status_check_rollup: vec![],
        };
        let info = convert_gh_data(&data);
        assert!(matches!(info.checks, ChecksStatus::None));
    }

    #[test]
    fn test_convert_gh_data_mixed_checks() {
        let data = GhPrData {
            number: 10,
            review_decision: "".to_string(),
            status_check_rollup: vec![
                CheckItem {
                    conclusion: Some("SUCCESS".to_string()),
                    state: Option::None,
                },
                CheckItem {
                    conclusion: Option::None,
                    state: Some("PENDING".to_string()),
                },
            ],
        };
        let info = convert_gh_data(&data);
        assert!(matches!(info.checks, ChecksStatus::Pending));
    }

    #[test]
    fn test_convert_gh_data_failure_check() {
        let data = GhPrData {
            number: 10,
            review_decision: "CHANGES_REQUESTED".to_string(),
            status_check_rollup: vec![
                CheckItem {
                    conclusion: Some("SUCCESS".to_string()),
                    state: Option::None,
                },
                CheckItem {
                    conclusion: Some("FAILURE".to_string()),
                    state: Option::None,
                },
            ],
        };
        let info = convert_gh_data(&data);
        assert!(matches!(
            info.review_decision,
            ReviewDecision::ChangesRequested
        ));
        assert!(matches!(info.checks, ChecksStatus::Fail));
    }
}
