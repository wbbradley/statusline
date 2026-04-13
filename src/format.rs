use crate::{git::GitInfo, input::StatusInput, pr::PrInfo};

const BLUE: &str = "\x1b[38;2;131;165;152m";
const AQUA: &str = "\x1b[38;2;142;192;124m";
const YELLOW: &str = "\x1b[38;2;250;189;47m";
const GREEN: &str = "\x1b[38;2;184;187;38m";
const ORANGE: &str = "\x1b[38;2;254;128;25m";
const RED: &str = "\x1b[38;2;251;73;52m";
const BORDER: &str = "\x1b[38;2;91;95;93m";
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

fn visible_width(s: &str) -> usize {
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

pub fn format_line1(input: &StatusInput) -> String {
    let mut segments: Vec<String> = Vec::new();

    if let Some(dir) = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
    {
        segments.push(colored(AQUA, &tilde_contract(dir)));
    }

    if let Some(ctx) = context_tokens(input) {
        segments.push(colored(ORANGE, &abbreviate_tokens(ctx)));
    }

    segments.join(&sep(2))
}

pub fn format_line2(git: &GitInfo, pr: Option<&PrInfo>) -> String {
    let mut segments: Vec<String> = Vec::new();

    segments.push(colored(GREEN, &format!("⎇ {}", git.branch)));

    let mut counts = Vec::new();
    if git.staged > 0 {
        counts.push(colored(GREEN, &format!("+{}", git.staged)));
    }
    if git.modified > 0 {
        counts.push(colored(YELLOW, &format!("~{}", git.modified)));
    }
    if !counts.is_empty() {
        segments.push(counts.join(&sep(1)));
    }

    if git.has_upstream {
        segments.push(colored(ORANGE, &format!("↑{}↓{}", git.ahead, git.behind)));
    }

    if let Some(pr) = pr {
        segments.push(format_pr_segment(pr));
    }

    segments.join(&sep(2))
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
    use super::*;
    use crate::{
        git::GitInfo,
        input::{ContextWindow, Cost, CurrentUsage, Model, Workspace},
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
        let input = StatusInput {
            model: Some(Model {
                id: None,
                display_name: Some("Opus".to_string()),
            }),
            workspace: Some(Workspace {
                current_dir: Some("/tmp/test-project".to_string()),
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

        let line = strip_ansi(&format_line1(&input));
        assert_eq!(line, "/tmp/test-project──145k");
    }

    #[test]
    fn test_format_line1_empty() {
        let input = StatusInput::default();
        assert_eq!(format_line1(&input), "");
    }

    #[test]
    fn test_format_line2_full() {
        let git = GitInfo {
            branch: "main".to_string(),
            staged: 3,
            modified: 2,
            ahead: 1,
            behind: 0,
            has_upstream: true,
            origin_url: None,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None)), "⎇ main──+3─~2──↑1↓0");
    }

    #[test]
    fn test_format_line2_clean() {
        let git = GitInfo {
            branch: "main".to_string(),
            staged: 0,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None)), "⎇ main──↑0↓0");
    }

    #[test]
    fn test_format_line2_no_upstream() {
        let git = GitInfo {
            branch: "feature".to_string(),
            staged: 0,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: false,
            origin_url: None,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None)), "⎇ feature");
    }

    #[test]
    fn test_format_line2_staged_only() {
        let git = GitInfo {
            branch: "main".to_string(),
            staged: 5,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None)), "⎇ main──+5──↑0↓0");
    }

    #[test]
    fn test_format_line2_modified_only() {
        let git = GitInfo {
            branch: "main".to_string(),
            staged: 0,
            modified: 3,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
        };
        assert_eq!(strip_ansi(&format_line2(&git, None)), "⎇ main──~3──↑0↓0");
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
            staged: 1,
            modified: 0,
            ahead: 0,
            behind: 0,
            has_upstream: true,
            origin_url: None,
        };
        let pr = PrInfo {
            number: 7,
            review_decision: ReviewDecision::Approved,
            checks: ChecksStatus::Pass,
        };
        let line = strip_ansi(&format_line2(&git, Some(&pr)));
        assert_eq!(line, "⎇ main──+1──↑0↓0──PR #7──✓ approved──● checks pass");
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
