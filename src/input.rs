use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StatusInput {
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub transcript_path: Option<String>,
    pub model: Option<Model>,
    pub workspace: Option<Workspace>,
    pub version: Option<String>,
    pub cost: Option<Cost>,
    pub context_window: Option<ContextWindow>,
    pub exceeds_200k_tokens: Option<bool>,
    pub rate_limits: Option<RateLimits>,
    pub vim: Option<Vim>,
    pub agent: Option<Agent>,
    pub worktree: Option<Worktree>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
    pub added_dirs: Option<Vec<String>>,
    pub git_worktree: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Cost {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_api_duration_ms: Option<u64>,
    pub total_lines_added: Option<u64>,
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindow {
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub context_window_size: Option<u64>,
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Debug, Deserialize)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Vim {
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Agent {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Worktree {
    pub name: Option<String>,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub original_cwd: Option<String>,
    pub original_branch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_object() {
        let input: StatusInput = serde_json::from_str("{}").unwrap();
        assert!(input.model.is_none());
        assert!(input.cwd.is_none());
        assert!(input.workspace.is_none());
        assert!(input.cost.is_none());
        assert!(input.context_window.is_none());
    }

    #[test]
    fn test_full_json() {
        let json = r#"{
            "cwd": "/Users/test/src/project",
            "session_id": "abc123",
            "model": {
                "id": "claude-opus-4-6",
                "display_name": "Opus"
            },
            "workspace": {
                "current_dir": "/Users/test/src/project",
                "project_dir": "/Users/test/src/project"
            },
            "cost": {
                "total_cost_usd": 1.23,
                "total_duration_ms": 45000,
                "total_api_duration_ms": 30000,
                "total_lines_added": 100,
                "total_lines_removed": 50
            },
            "context_window": {
                "total_input_tokens": 230000,
                "total_output_tokens": 5000,
                "context_window_size": 1000000,
                "used_percentage": 23.0,
                "remaining_percentage": 77.0,
                "current_usage": {
                    "input_tokens": 8500,
                    "output_tokens": 1200,
                    "cache_creation_input_tokens": 5000,
                    "cache_read_input_tokens": 3000
                }
            },
            "exceeds_200k_tokens": false,
            "rate_limits": {
                "five_hour": { "used_percentage": 10.0, "resets_at": 1700000000 },
                "seven_day": { "used_percentage": 5.0, "resets_at": 1700500000 }
            },
            "vim": { "mode": "normal" },
            "agent": { "name": "main" },
            "worktree": {
                "name": "feature-branch",
                "path": "/tmp/worktrees/feature",
                "branch": "feature-branch",
                "original_cwd": "/Users/test/src/project",
                "original_branch": "main"
            }
        }"#;

        let input: StatusInput = serde_json::from_str(json).unwrap();
        assert_eq!(
            input.model.as_ref().unwrap().display_name.as_deref(),
            Some("Opus")
        );
        assert_eq!(input.cost.as_ref().unwrap().total_duration_ms, Some(45000));
        assert_eq!(
            input
                .context_window
                .as_ref()
                .unwrap()
                .current_usage
                .as_ref()
                .unwrap()
                .input_tokens,
            Some(8500)
        );
        assert_eq!(input.exceeds_200k_tokens, Some(false));
        assert_eq!(
            input.worktree.as_ref().unwrap().branch.as_deref(),
            Some("feature-branch")
        );
    }

    #[test]
    fn test_partial_json() {
        let json = r#"{
            "cwd": "/Users/test",
            "model": { "id": "claude-opus-4-6", "display_name": "Opus" }
        }"#;

        let input: StatusInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.cwd.as_deref(), Some("/Users/test"));
        assert_eq!(
            input.model.as_ref().unwrap().display_name.as_deref(),
            Some("Opus")
        );
        assert!(input.workspace.is_none());
        assert!(input.cost.is_none());
        assert!(input.context_window.is_none());
        assert!(input.worktree.is_none());
    }
}
