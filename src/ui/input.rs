use colored::*;
use std::io::{self, Write};

// Read a trimmed line from stdin
pub fn read_line() -> String {
    print!("{}", ">".cyan().bold());
    io::stdout().flush().unwrap_or(());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    input.trim().to_string()
}

// Read input with a custom prompt
pub fn prompt(message: &str) -> String {
    print!("  {} {} ", message.white(), "›".cyan().bold());
    io::stdout().flush().unwrap_or(());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    input.trim().to_string()
}

pub fn confirm(message: &str) -> bool {
    let input = prompt(&format!("{} [Y/N]", message));
    matches!(input.to_ascii_lowercase().as_str(), "y" | "yes")
}

/// Ask user to pick from a numbered list, returns 0-based index or None
pub fn pick_options(options: &[&str]) -> Option<usize> {
    for (i, opt) in options.iter().enumerate() {
        println!("  {}  {}", format!("[{}]", i + 1).cyan(), opt.white());
    }
    let input = prompt("Select an option");
    input.parse::<usize>().ok().and_then(|n| {
        if n > 0 && n <= options.len() {
            Some(n - 1)
        } else {
            None
        }
    })
}

/// Ask user to pick multiple options (comma-separated)
pub fn pick_multiple(options: &[&str]) -> Vec<usize> {
    for (i, opt) in options.iter().enumerate() {
        println!("  {}  {}", format!("[{}]", i + 1).cyan(), opt.white());
    }
    println!(
        "  {}",
        "Enter numbers separated by commas (e.g. 1,3,5) or 'a' for all:".dimmed()
    );
    let input = read_line();
    let lower = input.to_lowercase();

    if lower == "a" || lower == "all" {
        return (0..options.len()).collect();
    }

    input
        .split(',')
        .filter_map(|s| s.trim().parse::<usize>().ok())
        .filter(|&n| n >= 1 && n <= options.len())
        .map(|n| n - 1)
        .collect()
}

// Read a number from the user
pub fn read_number(prompt_msg: &str) -> Option<u32> {
    let input = prompt(prompt_msg);
    input.parse().ok()
}

/// Clear the terminal screen
pub fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap_or(());
}

pub fn clear_line() {
    print!("\x1B[2K\r");
    io::stdout().flush().unwrap_or(());
}

pub fn delete_line() {
    print!("\x1B[2K\r");
    io::stdout().flush().unwrap_or(());
}
