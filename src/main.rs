mod format;
mod git;
mod input;
mod pr;

use std::io::Read;

use input::StatusInput;

fn main() {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .unwrap_or_else(|e| {
            eprintln!("statusline: {e}");
            std::process::exit(1);
        });
    let input: StatusInput = serde_json::from_str(&buf).unwrap_or_else(|e| {
        eprintln!("statusline: {e}");
        std::process::exit(1);
    });
    let line1 = format::format_line1(&input);

    let repo_path = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(input.cwd.as_deref());
    let line2 = repo_path.and_then(git::get_git_info).map(|git_info| {
        let pr_info = git_info
            .origin_url
            .as_deref()
            .and_then(|url| pr::get_pr_info(url, &git_info.branch));
        format::format_line2(&git_info, pr_info.as_ref())
    });

    let mut lines: Vec<&str> = Vec::new();
    if !line1.is_empty() {
        lines.push(&line1);
    }
    if let Some(ref l2) = line2 {
        lines.push(l2);
    }
    for framed in format::frame_lines(&lines) {
        println!("{framed}");
    }
}
