use crate::input::StatusInput;

const BLUE: &str = "\x1b[38;2;131;165;152m";
const AQUA: &str = "\x1b[38;2;142;192;124m";
const YELLOW: &str = "\x1b[38;2;250;189;47m";
const GRAY: &str = "\x1b[38;2;168;153;132m";
const RESET: &str = "\x1b[0m";

fn colored(color: &str, text: &str) -> String {
    format!("{color}{text}{RESET}")
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

fn format_duration(ms: u64) -> String {
    let total_secs = ms / 1000;
    if total_secs < 60 {
        format!("{total_secs}s")
    } else if total_secs < 3600 {
        format!("{}m", total_secs / 60)
    } else {
        format!("{}h{}m", total_secs / 3600, (total_secs % 3600) / 60)
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

    if let Some(name) = input.model.as_ref().and_then(|m| m.display_name.as_deref()) {
        segments.push(colored(BLUE, &format!("[{name}]")));
    }

    if let Some(dir) = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
    {
        segments.push(colored(AQUA, &tilde_contract(dir)));
    }

    if let Some(ctx) = context_tokens(input) {
        segments.push(colored(YELLOW, &format!("ctx: {}", abbreviate_tokens(ctx))));
    }

    if let Some(total) = input
        .context_window
        .as_ref()
        .and_then(|cw| cw.total_input_tokens)
    {
        segments.push(colored(
            YELLOW,
            &format!("total: {}", abbreviate_tokens(total)),
        ));
    }

    if let Some(dur) = input.cost.as_ref().and_then(|c| c.total_duration_ms) {
        segments.push(colored(GRAY, &format_duration(dur)));
    }

    segments.join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{ContextWindow, Cost, CurrentUsage, Model, Workspace};

    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
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
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(5_000), "5s");
        assert_eq!(format_duration(59_999), "59s");
        assert_eq!(format_duration(60_000), "1m");
        assert_eq!(format_duration(90_000), "1m");
        assert_eq!(format_duration(720_000), "12m");
        assert_eq!(format_duration(3_600_000), "1h0m");
        assert_eq!(format_duration(4_980_000), "1h23m");
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
        assert_eq!(
            line,
            "[Opus]  /tmp/test-project  ctx: 145k  total: 230k  12m"
        );
    }

    #[test]
    fn test_format_line1_empty() {
        let input = StatusInput::default();
        assert_eq!(format_line1(&input), "");
    }
}
