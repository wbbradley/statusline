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
    if !line1.is_empty() {
        println!("{line1}");
    }

    let repo_path = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(input.cwd.as_deref());
    if let Some(path) = repo_path
        && let Some(git_info) = git::get_git_info(path)
    {
        let pr_info = git_info
            .origin_url
            .as_deref()
            .and_then(|url| pr::get_pr_info(url, &git_info.branch));
        println!("{}", format::format_line2(&git_info, pr_info.as_ref()));
    }
}
