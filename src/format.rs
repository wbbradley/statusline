use std::path::Path;

use crate::{git::GitInfo, input::StatusInput, plan::plan_md_line_count, pr::PrInfo};

const BLUE: &str = "\x1b[38;2;131;165;152m";
const GREY_BLUE: &str = "\x1b[38;2;124;142;158m";
const AQUA: &str = "\x1b[38;2;142;192;124m";
const YELLOW: &str = "\x1b[38;2;250;189;47m";
const GREEN: &str = "\x1b[38;2;184;187;38m";
const ORANGE: &str = "\x1b[38;2;254;128;25m";
const RED: &str = "\x1b[38;2;251;73;52m";
const BORDER: &str = "\x1b[38;2;61;65;63m";
const BG: &str = "\x1b[48;2;30;30;30m";
const RESET: &str = "\x1b[0m";
const FG_RESET: &str = "\x1b[39m";

fn colored(color: &str, text: &str) -> String {
    format!("{color}{text}{FG_RESET}")
}

fn sep(n: usize) -> String {
    let bars = "─".repeat(n);
    format!("{BORDER}{bars}{FG_RESET}")
}

pub fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            width += 1;
        }
    }
    width
}

pub fn frame_lines(lines: &[&str]) -> Vec<String> {
    let max_width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
    let last = lines.len().saturating_sub(1);
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let (l, r) = match (i, i == last) {
                (0, true) => ("╭", "╮"),
                (0, false) => ("╭", "╮"),
                (_, true) => ("╰", "╯"),
                _ => ("│", "│"),
            };
            let pad = "─".repeat(max_width - visible_width(line));
            format!("{BG}{BORDER}{l}─{FG_RESET}{line}{BORDER}{pad}─{r}{RESET}")
        })
        .collect()
}

fn abbreviate_tokens(n: u64) -> String {
    if n < 1_000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{}k", n / 1_000)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

fn tilde_contract(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if path.starts_with(&home) => format!("~{}", &path[home.len()..]),
        _ => path.to_string(),
    }
}

fn context_tokens(input: &StatusInput) -> Option<u64> {
    let cu = input.context_window.as_ref()?.current_usage.as_ref()?;
    let sum = cu.input_tokens.unwrap_or(0)
        + cu.cache_creation_input_tokens.unwrap_or(0)
        + cu.cache_read_input_tokens.unwrap_or(0);
    Some(sum)
}

fn join_aligned(left: &str, right: &str, min_width: Option<usize>) -> String {
    if left.is_empty() && right.is_empty() {
        return String::new();
    }
    if right.is_empty() {
        return left.to_string();
    }
    if left.is_empty() {
        return right.to_string();
    }
    let min_sep = 2;
    if let Some(w) = min_width {
        let pad_width = w.saturating_sub(visible_width(left) + visible_width(right));
        let pad_width = pad_width.max(min_sep);
        format!("{left}{}{right}", sep(pad_width))
    } else {
        format!("{left}{}{right}", sep(min_sep))
    }
}

fn hostname() -> Option<String> {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .filter(|s| !s.is_empty())
}

fn os_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        std::env::consts::OS
    }
}

pub fn format_line1(input: &StatusInput, is_worktree: bool, min_width: Option<usize>) -> String {
    format_line1_with_env(
        input,
        is_worktree,
        min_width,
        hostname().as_deref(),
        os_name(),
    )
}

fn format_line1_with_env(
    input: &StatusInput,
    is_worktree: bool,
    min_width: Option<usize>,
    host: Option<&str>,
    os: &str,
) -> String {
    let workspace_dir = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref());

    let wt = input.worktree.as_ref();
    let wt_name_input = input
        .workspace
        .as_ref()
        .and_then(|w| w.git_worktree.as_deref())
        .or_else(|| wt.and_then(|w| w.name.as_deref()));
    let original_cwd = wt.and_then(|w| w.original_cwd.as_deref());
    let original_branch = wt.and_then(|w| w.original_branch.as_deref());

    // Trigger: git2 detection OR any input worktree metadata.
    let in_worktree = is_worktree || wt_name_input.is_some() || wt.is_some();

    // Directory: replace with original_cwd when known, else show current_dir as-is.
    let display_dir = if in_worktree {
        original_cwd.or(workspace_dir)
    } else {
        workspace_dir
    };
    let dir_part = display_dir.map(|dir| colored(AQUA, &tilde_contract(dir)));

    // Worktree name: git_worktree / worktree.name, else basename of the worktree dir.
    let wt_name = if in_worktree {
        wt_name_input.map(str::to_string).or_else(|| {
            workspace_dir
                .and_then(|d| Path::new(d).file_name())
                .map(|f| f.to_string_lossy().into_owned())
        })
    } else {
        None
    };

    let worktree_segment = wt_name.map(|name| {
        let mut body = format!("🌿 {name}");
        if let Some(branch) = original_branch {
            body.push_str(&format!(" ←{branch}"));
        }
        colored(GREY_BLUE, &body)
    });

    let os_host = match host {
        Some(h) => colored(GREY_BLUE, &format!("{} {h}", os.to_lowercase())),
        None => colored(GREY_BLUE, os),
    };

    let plan_segment = workspace_dir.map(|dir| {
        let body = match plan_md_line_count(Path::new(dir)) {
            Some(n) => format!("🗒 {n}"),
            None => "🗒 -".to_string(),
        };
        colored(GREY_BLUE, &body)
    });

    let dir_with_wt = match (dir_part, worktree_segment) {
        (Some(d), Some(w)) => Some(format!("{d} {w}")),
        (Some(d), None) => Some(d),
        (None, Some(w)) => Some(w),
        (None, None) => None,
    };

    let mut left = match dir_with_wt {
        Some(dir) => format!("{dir} {os_host}"),
        None => os_host,
    };
    if let Some(seg) = plan_segment {
        left = format!("{left}{}{seg}", sep(2));
    }

    let model_segment = input
        .model
        .as_ref()
        .and_then(|m| m.id.as_deref())
        .map(|id| format!("[{id}]"));
    if let Some(seg) = model_segment {
        left = format!("{seg}{}{left}", sep(2));
    }

    let right = context_tokens(input)
        .map(|ctx| colored(ORANGE, &abbreviate_tokens(ctx)))
        .unwrap_or_default();

    join_aligned(&left, &right, min_width)
}

pub fn format_line2(git: &GitInfo, pr: Option<&PrInfo>, min_width: Option<usize>) -> String {
    let mut left_segments: Vec<String> = Vec::new();

    let branch_segment = match &git.sha {
        Some(sha) => format!(
            "{} {}",
            colored(GREEN, &format!("⎇ {}", git.branch)),
            colored(GREY_BLUE, sha)
        ),
        None => colored(GREEN, &format!("⎇ {}", git.branch)),
    };
    left_segments.push(branch_segment);

    let mut counts = Vec::new();
    if git.staged > 0 {
        counts.push(colored(GREEN, &format!("+{}", git.staged)));
    }
    if git.modified > 0 {
        counts.push(colored(YELLOW, &format!("~{}", git.modified)));
    }
    if !counts.is_empty() {
        left_segments.push(counts.join(&sep(1)));
    }

    if let Some(pr) = pr {
        left_segments.push(format_pr_segment(pr));
    }

    let left = left_segments.join(&sep(2));

    let right = if git.has_upstream {
        colored(ORANGE, &format!("↑{}↓{}", git.ahead, git.behind))
    } else {
        String::new()
    };

    join_aligned(&left, &right, min_width)
}

pub fn format_pr_segment(pr: &PrInfo) -> String {
    use crate::pr::{ChecksStatus, ReviewDecision};

    let mut parts: Vec<String> = Vec::new();
    parts.push(colored(BLUE, &format!("PR #{}", pr.number)));

    match &pr.review_decision {
        ReviewDecision::Approved => parts.push(colored(GREEN, "✓ approved")),
        ReviewDecision::ChangesRequested => parts.push(colored(RED, "✗ changes requested")),
        ReviewDecision::ReviewRequired => parts.push(colored(YELLOW, "? review needed")),
        ReviewDecision::None => {}
    }

    match &pr.checks {
        ChecksStatus::Pass => parts.push(colored(GREEN, "● checks pass")),
        ChecksStatus::Fail => parts.push(colored(RED, "✗ checks fail")),
        ChecksStatus::Pending => parts.push(colored(YELLOW, "○ checks pending")),
        ChecksStatus::None => {}
    }

    parts.join(&sep(2))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        git::GitInfo,
        input::{ContextWindow, Cost, CurrentUsage, Model, Workspace, Worktree},
        pr::{ChecksStatus, PrInfo, ReviewDecision},
    };

    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn test_abbreviate_tokens() {
        assert_eq!(abbreviate_tokens(0), "0");
        assert_eq!(abbreviate_tokens(500), "500");
        assert_eq!(abbreviate_tokens(1_000), "1k");
        assert_eq!(abbreviate_tokens(1_500), "1k");
        assert_eq!(abbreviate_tokens(145_000), "145k");
        assert_eq!(abbreviate_tokens(1_000_000), "1.0M");
        assert_eq!(abbreviate_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_tilde_contract() {
        let home = std::env::var("HOME").unwrap();
        assert_eq!(
            tilde_contract(&format!("{home}/src/project")),
            "~/src/project"
        );
        assert_eq!(tilde_contract("/tmp/other"), "/tmp/other");
    }

    #[test]
    fn test_context_tokens() {
        let input = StatusInput {
            context_window: Some(ContextWindow {
                current_usage: Some(CurrentUsage {
                    input_tokens: Some(8500),
                    output_tokens: Some(1200),
                    cache_creation_input_tokens: Some(130_000),
                    cache_read_input_tokens: Some(6_500),
                }),
                total_input_tokens: None,
                total_output_tokens: None,
                context_window_size: None,
                used_percentage: None,
                remaining_percentage: None,
            }),
            ..Default::default()
        };
        assert_eq!(context_tokens(&input), Some(145_000));

        let empty = StatusInput::default();
        assert_eq!(context_tokens(&empty), None);
    }

    #[test]
    fn test_format_line1_full() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_string_lossy().into_owned();
        std::fs::write(tmp.path().join("PLAN.md"), "a\nb\nc\n").unwrap();
        let input = StatusInput {
            model: Some(Model {
                id: Some("claude-opus-4-6".to_string()),
                display_name: Some("Opus".to_string()),
            }),
            workspace: Some(Workspace {
                current_dir: Some(dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: None,
            }),
            context_window: Some(ContextWindow {
                total_input_tokens: Some(230_000),
                current_usage: Some(CurrentUsage {
                    input_tokens: Some(8500),
                    output_tokens: None,
                    cache_creation_input_tokens: Some(130_000),
                    cache_read_input_tokens: Some(6_500),
                }),
                total_output_tokens: None,
                context_window_size: None,
                used_percentage: None,
                remaining_percentage: None,
            }),
            cost: Some(Cost {
                total_duration_ms: Some(720_000),
                total_cost_usd: None,
                total_api_duration_ms: None,
                total_lines_added: None,
                total_lines_removed: None,
            }),
            ..Default::default()
        };

        let line = strip_ansi(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(
            line,
            format!("[claude-opus-4-6]──{dir} macos myhost──🗒 3──145k")
        );
    }

    #[test]
    fn test_format_line1_no_model() {
        let input = StatusInput::default();
        let line = strip_ansi(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "Linux",
        ));
        assert!(
            !line.starts_with('['),
            "line 1 should not lead with a bracketed model id when model is absent: {line:?}"
        );
    }

    #[test]
    fn test_format_line1_no_workspace() {
        let input = StatusInput::default();
        let line = strip_ansi(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "Linux",
        ));
        assert_eq!(line, "linux myhost");

        let line_no_host = strip_ansi(&format_line1_with_env(&input, false, None, None, "macOS"));
        assert_eq!(line_no_host, "macOS");
    }

    #[test]
    fn test_format_line1_right_aligned() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_string_lossy().into_owned();
        // No PLAN.md → segment is "🗒 -".
        let input = StatusInput {
            workspace: Some(Workspace {
                current_dir: Some(dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: None,
            }),
            context_window: Some(ContextWindow {
                current_usage: Some(CurrentUsage {
                    input_tokens: Some(8500),
                    output_tokens: None,
                    cache_creation_input_tokens: Some(130_000),
                    cache_read_input_tokens: Some(6_500),
                }),
                total_input_tokens: None,
                total_output_tokens: None,
                context_window_size: None,
                used_percentage: None,
                remaining_percentage: None,
            }),
            ..Default::default()
        };

        // Natural: "<dir> macos myhost" + "──🗒 -" + "──145k"
        let natural = strip_ansi(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(natural, format!("{dir} macos myhost──🗒 -──145k"));

        let natural_width = visible_width(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "macOS",
        ));

        // With min_width wider than natural: token count pushed right
        let wide = strip_ansi(&format_line1_with_env(
            &input,
            false,
            Some(natural_width + 9),
            Some("myhost"),
            "macOS",
        ));
        assert!(wide.starts_with(&dir));
        assert!(wide.ends_with("145k"));
        assert_eq!(
            visible_width(&format_line1_with_env(
                &input,
                false,
                Some(natural_width + 9),
                Some("myhost"),
                "macOS"
            )),
            natural_width + 9
        );

        // With min_width narrower than natural: falls back to min separator
        let narrow = strip_ansi(&format_line1_with_env(
            &input,
            false,
            Some(10),
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(narrow, format!("{dir} macos myhost──🗒 -──145k"));
    }

    #[test]
    fn test_format_line1_with_plan_md() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_string_lossy().into_owned();
        std::fs::write(tmp.path().join("PLAN.md"), "one\ntwo\nthree\n").unwrap();
        let input = StatusInput {
            workspace: Some(Workspace {
                current_dir: Some(dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: None,
            }),
            ..Default::default()
        };
        let line = strip_ansi(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(line, format!("{dir} macos myhost──🗒 3"));
    }

    #[test]
    fn test_format_line1_missing_plan_md() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_string_lossy().into_owned();
        let input = StatusInput {
            workspace: Some(Workspace {
                current_dir: Some(dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: None,
            }),
            context_window: Some(ContextWindow {
                current_usage: Some(CurrentUsage {
                    input_tokens: Some(8500),
                    output_tokens: None,
                    cache_creation_input_tokens: Some(130_000),
                    cache_read_input_tokens: Some(6_500),
                }),
                total_input_tokens: None,
                total_output_tokens: None,
                context_window_size: None,
                used_percentage: None,
                remaining_percentage: None,
            }),
            ..Default::default()
        };
        let line = strip_ansi(&format_line1_with_env(
            &input,
            false,
            None,
            Some("myhost"),
            "macOS",
        ));
        let placeholder = "🗒 -";
        assert!(
            line.contains(placeholder),
            "line missing placeholder: {line}"
        );
        let plan_idx = line.find(placeholder).unwrap();
        let host_idx = line.find("myhost").unwrap();
        let tokens_idx = line.find("145k").unwrap();
        assert!(host_idx < plan_idx);
        assert!(plan_idx < tokens_idx);
    }

    #[test]
    fn test_format_line1_worktree_with_original() {
        let wt_tmp = TempDir::new().unwrap();
        let wt_dir = wt_tmp.path().to_string_lossy().into_owned();
        let orig_tmp = TempDir::new().unwrap();
        let orig_dir = orig_tmp.path().to_string_lossy().into_owned();
        let input = StatusInput {
            workspace: Some(Workspace {
                current_dir: Some(wt_dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: Some("devin-29610".to_string()),
            }),
            worktree: Some(Worktree {
                name: Some("devin-29610".to_string()),
                path: Some(wt_dir.clone()),
                branch: Some("devin-29610".to_string()),
                original_cwd: Some(orig_dir.clone()),
                original_branch: Some("main".to_string()),
            }),
            ..Default::default()
        };
        let line = strip_ansi(&format_line1_with_env(
            &input,
            true,
            None,
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(
            line,
            format!("{orig_dir} 🌿 devin-29610 ←main macos myhost──🗒 -")
        );
    }

    #[test]
    fn test_format_line1_worktree_no_parent_branch() {
        let wt_tmp = TempDir::new().unwrap();
        let wt_dir = wt_tmp.path().to_string_lossy().into_owned();
        let orig_tmp = TempDir::new().unwrap();
        let orig_dir = orig_tmp.path().to_string_lossy().into_owned();
        let input = StatusInput {
            workspace: Some(Workspace {
                current_dir: Some(wt_dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: Some("devin-29610".to_string()),
            }),
            worktree: Some(Worktree {
                name: Some("devin-29610".to_string()),
                path: Some(wt_dir.clone()),
                branch: Some("devin-29610".to_string()),
                original_cwd: Some(orig_dir.clone()),
                original_branch: None,
            }),
            ..Default::default()
        };
        let line = strip_ansi(&format_line1_with_env(
            &input,
            true,
            None,
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(line, format!("{orig_dir} 🌿 devin-29610 macos myhost──🗒 -"));
    }

    #[test]
    fn test_format_line1_worktree_detected_only() {
        let wt_tmp = TempDir::new().unwrap();
        let wt_dir = wt_tmp.path().to_string_lossy().into_owned();
        let basename = wt_tmp
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let input = StatusInput {
            workspace: Some(Workspace {
                current_dir: Some(wt_dir.clone()),
                project_dir: None,
                added_dirs: None,
                git_worktree: None,
            }),
            ..Default::default()
        };
        let line = strip_ansi(&format_line1_with_env(
            &input,
            true,
            None,
            Some("myhost"),
            "macOS",
        ));
        assert_eq!(line, format!("{wt_dir} 🌿 {basename} macos myhost──🗒 -"));
    }

    #[test]
    fn test_format_line2_full() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: None,
            staged: 3,
            modified: 2,
            ahead: 1,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };
        assert_eq!(
            strip_ansi(&format_line2(&git, None, None)),
            "⎇ main──+3─~2──↑1↓0"
        );
    }

    #[test]
    fn test_format_line2_clean() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: None,
            staged: 0,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None, None)), "⎇ main──↑0↓0");
    }

    #[test]
    fn test_format_line2_no_upstream() {
        let git = GitInfo {
            branch: "feature".to_string(),
            sha: None,
            staged: 0,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: false,
            origin_url: None,
            is_worktree: false,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None, None)), "⎇ feature");
    }

    #[test]
    fn test_format_line2_staged_only() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: None,
            staged: 5,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };
        assert_eq!(
            strip_ansi(&format_line2(&git, None, None)),
            "⎇ main──+5──↑0↓0"
        );
    }

    #[test]
    fn test_format_line2_modified_only() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: None,
            staged: 0,
            modified: 3,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };
        assert_eq!(
            strip_ansi(&format_line2(&git, None, None)),
            "⎇ main──~3──↑0↓0"
        );
    }

    #[test]
    fn test_format_line2_with_sha() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: Some("9769e18a".to_string()),
            staged: 0,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };
        assert_eq!(
            strip_ansi(&format_line2(&git, None, None)),
            "⎇ main 9769e18a──↑0↓0"
        );
    }

    #[test]
    fn test_format_pr_segment_approved_pass() {
        let pr = PrInfo {
            number: 42,
            review_decision: ReviewDecision::Approved,
            checks: ChecksStatus::Pass,
        };
        assert_eq!(
            strip_ansi(&format_pr_segment(&pr)),
            "PR #42──✓ approved──● checks pass"
        );
    }

    #[test]
    fn test_format_pr_segment_changes_requested_fail() {
        let pr = PrInfo {
            number: 42,
            review_decision: ReviewDecision::ChangesRequested,
            checks: ChecksStatus::Fail,
        };
        assert_eq!(
            strip_ansi(&format_pr_segment(&pr)),
            "PR #42──✗ changes requested──✗ checks fail"
        );
    }

    #[test]
    fn test_format_pr_segment_no_review_pending() {
        let pr = PrInfo {
            number: 42,
            review_decision: ReviewDecision::None,
            checks: ChecksStatus::Pending,
        };
        assert_eq!(
            strip_ansi(&format_pr_segment(&pr)),
            "PR #42──○ checks pending"
        );
    }

    #[test]
    fn test_format_pr_segment_none_none() {
        let pr = PrInfo {
            number: 42,
            review_decision: ReviewDecision::None,
            checks: ChecksStatus::None,
        };
        assert_eq!(strip_ansi(&format_pr_segment(&pr)), "PR #42");
    }

    #[test]
    fn test_format_line2_with_pr() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: None,
            staged: 1,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };
        let pr = PrInfo {
            number: 7,
            review_decision: ReviewDecision::Approved,
            checks: ChecksStatus::Pass,
        };
        let line = strip_ansi(&format_line2(&git, Some(&pr), None));
        assert_eq!(line, "⎇ main──+1──PR #7──✓ approved──● checks pass──↑0↓0");
    }

    #[test]
    fn test_format_line2_right_aligned() {
        let git = GitInfo {
            branch: "main".to_string(),
            sha: None,
            staged: 0,
            modified: 0,
            ahead: 1,
            behind: 0,
            has_upstream: true,
            origin_url: None,
            is_worktree: false,
        };

        // Natural: "⎇ main" (6) + "──" (2) + "↑1↓0" (4) = 12
        let natural = strip_ansi(&format_line2(&git, None, None));
        assert_eq!(natural, "⎇ main──↑1↓0");

        // With min_width wider: upstream pushed right
        let wide = strip_ansi(&format_line2(&git, None, Some(20)));
        assert!(wide.starts_with("⎇ main"));
        assert!(wide.ends_with("↑1↓0"));
        assert_eq!(visible_width(&format_line2(&git, None, Some(20))), 20);

        // With min_width narrower: falls back to min separator
        let narrow = strip_ansi(&format_line2(&git, None, Some(5)));
        assert_eq!(narrow, "⎇ main──↑1↓0");
    }

    #[test]
    fn test_frame_lines_aligned() {
        let short = "short";
        let long = "a longer line here";
        let framed: Vec<String> = frame_lines(&[short, long])
            .iter()
            .map(|l| strip_ansi(l))
            .collect();
        assert_eq!(framed[0], "╭─short──────────────╮");
        assert_eq!(framed[1], "╰─a longer line here─╯");
        // Visual widths should match (byte lengths differ due to multi-byte ─ in padding).
        assert_eq!(visible_width(&framed[0]), visible_width(&framed[1]));
    }

    #[test]
    fn test_frame_lines_single() {
        let framed: Vec<String> = frame_lines(&["hello"])
            .iter()
            .map(|l| strip_ansi(l))
            .collect();
        assert_eq!(framed[0], "╭─hello─╮");
    }

    #[test]
    fn test_visible_width() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width(&colored(BLUE, "hello")), 5);
        assert_eq!(visible_width(""), 0);
    }
}
