mod format;
mod git;
mod input;
mod pr;

use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use input::StatusInput;

fn log_input(buf: &str) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(buf) else {
        return;
    };
    let Ok(line) = serde_json::to_string(&value) else {
        return;
    };
    let path = std::path::Path::new(&home).join("statusline.log");
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{line}");
}

fn main() {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .unwrap_or_else(|e| {
            eprintln!("statusline: {e}");
            std::process::exit(1);
        });
    log_input(&buf);
    let input: StatusInput = serde_json::from_str(&buf).unwrap_or_else(|e| {
        eprintln!("statusline: {e}");
        std::process::exit(1);
    });
    let repo_path = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(input.cwd.as_deref());
    let git_info = repo_path.and_then(git::get_git_info);
    let pr_info = git_info.as_ref().and_then(|g| {
        g.origin_url
            .as_deref()
            .and_then(|url| pr::get_pr_info(url, &g.branch))
    });

    // First pass: natural widths (no right-alignment).
    let line1_natural = format::format_line1(&input, None);
    let line2_natural = git_info
        .as_ref()
        .map(|g| format::format_line2(g, pr_info.as_ref(), None));
    let max_width = [
        (!line1_natural.is_empty()).then(|| format::visible_width(&line1_natural)),
        line2_natural.as_deref().map(format::visible_width),
    ]
    .into_iter()
    .flatten()
    .max();

    // Second pass: right-align both lines to the shared max width.
    let line1 = format::format_line1(&input, max_width);
    let line2 = git_info
        .as_ref()
        .map(|g| format::format_line2(g, pr_info.as_ref(), max_width));

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
