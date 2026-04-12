mod format;
mod git;
mod input;

use std::io::Read;

use input::StatusInput;

fn main() {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();
    let input: StatusInput = serde_json::from_str(&buf).unwrap_or_else(|e| {
        eprintln!("statusline: {e}");
        std::process::exit(1);
    });
    println!("{}", format::format_line1(&input));

    let repo_path = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(input.cwd.as_deref());
    if let Some(path) = repo_path
        && let Some(git_info) = git::get_git_info(path) {
            println!("{}", format::format_line2(&git_info));
        }
}
