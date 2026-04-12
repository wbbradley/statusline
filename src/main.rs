mod format;
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
}
