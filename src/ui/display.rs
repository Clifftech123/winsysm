// src/ui/display.rs
// Handles all visual output: banner, progress bars, tables, headers, colors

use crate::core::*;
use colored::*;
use humansize::{format_size, BINARY};

/// The main app banner shown at startup
pub fn print_banner() {
    let banner = r#"
    ██╗    ██╗██╗███╗   ██╗███████╗██╗   ██╗███████╗███╗   ███╗
    ██║    ██║██║████╗  ██║██╔════╝╚██╗ ██╔╝██╔════╝████╗ ████║
    ██║ █╗ ██║██║██╔██╗ ██║███████╗ ╚████╔╝ ███████╗██╔████╔██║
    ██║███╗██║██║██║╚██╗██║╚════██║  ╚██╔╝  ╚════██║██║╚██╔╝██║
    ╚███╔███╔╝██║██║ ╚████║███████║   ██║   ███████║██║ ╚═╝ ██║
     ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚══════╝   ╚═╝   ╚══════╝╚═╝     ╚═╝"#;

    println!("{}", banner.cyan().bold());
    println!(
        "{}",
        "    Windows System Manager v1.0 — Scan · Monitor · Clean · Protect"
            .white()
            .dimmed()
    );
    println!();
}

/// Show the main menu
pub fn print_menu() {
    let separator = "━".repeat(52);
    println!("\n{}", separator.dimmed());
    println!("  {}  {}", "[1]".cyan().bold(), "Scan Disk Usage".white());
    println!(
        "  {}  {}",
        "[2]".magenta().bold(),
        "Monitor Processes".white()
    );
    println!(
        "  {}  {}",
        "[3]".yellow().bold(),
        "Clean Up Junk Files".white()
    );
    println!("  {}  {}", "[4]".red().bold(), "Security Scan".white());
    println!(
        "  {}  {}",
        "[5]".green().bold(),
        "System Health Report".white()
    );
    println!("  {}  {}", "[6]".white().bold(), "Settings".dimmed());
    println!("  {}  {}", "[0]".white().bold(), "Exit".dimmed());
    println!("{}", separator.dimmed());
}

/// Print a section header
/// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
/// ┃  🔍 DISK SCANNER                  ┃
/// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
pub fn print_header(icon: &str, title: &str, color: Color) {
    let width: usize = 54;
    let inner = format!("  {} {}", icon, title);
    let padding = width.saturating_sub(inner.len());

    println!();
    println!("{}", format!("┏{}┓", "━".repeat(width)).color(color));
    println!(
        "{}",
        format!("┃{}{}┃", inner, " ".repeat(padding)).color(color)
    );
    println!("{}", format!("┗{}┛", "━".repeat(width)).color(color));
    println!();
}

/// Create a visual progress bar
/// progress_bar(75.0, 30) → "██████████████████████░░░░░░░░"
pub fn progress_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Colorized progress bar based on percentage thresholds
pub fn colored_bar(percent: f64, width: usize) -> ColoredString {
    let bar = progress_bar(percent, width);
    if percent >= 90.0 {
        bar.red().bold()
    } else if percent >= 70.0 {
        bar.yellow()
    } else {
        bar.green()
    }
}

/// Format bytes into human-readable size
pub fn fmt_size(bytes: u64) -> String {
    format_size(bytes, BINARY)
}

/// Format bytes with color (red if large, green if small)
pub fn colored_size(bytes: u64) -> ColoredString {
    let s = fmt_size(bytes);
    if bytes > 10 * 1024 * 1024 * 1024 {
        // > 10 GB
        s.red().bold()
    } else if bytes > 1024 * 1024 * 1024 {
        // > 1 GB
        s.yellow()
    } else {
        s.green()
    }
}

/// Print a formatted table with aligned columns
///
/// Example:
/// ```
/// print_table(
///     &["NAME", "SIZE", "FILES"],
///     &[vec!["Windows".into(), "18 GB".into(), "24000".into()]],
///     &[20, 12, 10],
/// );
/// ```
pub fn print_table(headers: &[&str], rows: &[Vec<String>], widths: &[usize]) {
    // Header
    let header_line: String = headers
        .iter()
        .zip(widths.iter())
        .map(|(h, w)| format!("{:<width$}", h, width = w))
        .collect::<Vec<_>>()
        .join("  ");
    println!("  {}", header_line.bold().dimmed());

    // Separator
    let sep: String = widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("──");
    println!("  {}", sep.dimmed());

    // Rows
    for row in rows {
        let line: String = row
            .iter()
            .zip(widths.iter())
            .map(|(cell, w)| format!("{:<width$}", cell, width = w))
            .collect::<Vec<_>>()
            .join("  ");
        println!("  {}", line);
    }
}

/// Print a key-value pair nicely
pub fn print_kv(key: &str, value: &str) {
    println!("  {}: {}", key.dimmed(), value.white());
}

/// Print drive info with visual bar
pub fn print_drive_info(drive: &DriveInfo) {
    let bar = colored_bar(drive.usage_percent, 30);
    println!(
        "  {} {} {}",
        format!("Drive {}:", drive.mount_point).cyan().bold(),
        drive.name.white(),
        format!("({})", drive.fs_type).dimmed()
    );
    println!(
        "  Total: {}  Used: {}  Free: {}",
        fmt_size(drive.total_bytes).white(),
        colored_size(drive.used_bytes),
        fmt_size(drive.free_bytes).green()
    );
    println!("  {} {:.1}%", bar, drive.usage_percent);
    println!();
}

/// Print threat with color-coded severity
pub fn print_threat(threat: &Threat, index: usize) {
    let level_str = match threat.level {
        ThreatLevel::Critical => format!("🔴 {}", threat.level).red().bold(),
        ThreatLevel::High => format!("🟠 {}", threat.level).red(),
        ThreatLevel::Medium => format!("🟡 {}", threat.level).yellow(),
        ThreatLevel::Low => format!("🟢 {}", threat.level).green(),
        ThreatLevel::Info => format!("🔵 {}", threat.level).blue(),
    };

    println!(
        "  {:>2}. {} [{}]",
        index + 1,
        level_str,
        threat.source.to_string().dimmed()
    );
    println!("      {}", threat.description.white());
    println!("      Recommendation: {}", threat.recommendation.dimmed());
    println!();
}

/// Print health score as a big visual card
pub fn print_health_score(report: &HealthReport) {
    let (score_color, rating_emoji) = match report.score {
        90..=100 => (Color::Green, "🟢"),
        70..=89 => (Color::Green, "🟡"),
        50..=69 => (Color::Yellow, "🟠"),
        30..=49 => (Color::Red, "🔴"),
        _ => (Color::Red, "💀"),
    };

    let box_width = 44;
    let score_text = format!("{}/100", report.score);
    let rating_text = format!("{} {}", rating_emoji, report.rating);

    println!();
    println!(
        "{}",
        format!("╔{}╗", "═".repeat(box_width)).color(score_color)
    );
    println!(
        "{}",
        format!("║{:^width$}║", "SYSTEM HEALTH SCORE", width = box_width).color(score_color)
    );
    println!(
        "{}",
        format!("║{:^width$}║", score_text, width = box_width)
            .color(score_color)
            .bold()
    );
    println!(
        "{}",
        format!("║{:^width$}║", rating_text, width = box_width).color(score_color)
    );
    println!(
        "{}",
        format!("╚{}╝", "═".repeat(box_width)).color(score_color)
    );
    println!();
}

/// Pause and wait for Enter
pub fn pause() {
    println!("\n  {}", "Press Enter to return to menu...".dimmed());
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}
