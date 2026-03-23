// src/scanner/analyzer.rs
// Interactive disk analysis command — drives the scanner UI

use crate::core::*;
use crate::scanner::walker;
use crate::ui;
use colored::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Main entry point for disk scanning
pub fn run() -> AppResult<()> {
    ui::print_header("🔍", "DISK SCANNER", Color::Cyan);

    // Show all drives
    let drives = walker::get_drives();
    if drives.is_empty() {
        println!("  {}", "No drives found!".red());
        return Ok(());
    }

    println!("  {}", "Detected drives:".bold());
    println!();
    for drive in &drives {
        ui::print_drive_info(drive);
    }

    // Pick drive to scan (default C:\)
    let scan_path = if drives.len() == 1 {
        PathBuf::from(&drives[0].mount_point)
    } else {
        let input = ui::prompt("Enter drive letter to scan (e.g. C)");
        if input.is_empty() {
            PathBuf::from("C:\\")
        } else {
            PathBuf::from(format!("{}:\\", input.trim_end_matches(':')))
        }
    };

    // Start drill-down loop
    drill_down(&scan_path)?;

    Ok(())
}

/// Drill-down scan — shows folder contents and lets user go deeper
fn drill_down(start_path: &Path) -> AppResult<()> {
    let mut current_path = start_path.to_path_buf();

    loop {
        println!(
            "\n  {} {}",
            "Scanning:".dimmed(),
            current_path.display().to_string().cyan().bold()
        );

        let timer = Instant::now();

        // Scan children
        let children = walker::scan_children(&current_path, 25);
        let elapsed = timer.elapsed().as_secs_f64();
        let total_parent: u64 = children.iter().map(|c| c.size_bytes).sum();

        if children.is_empty() {
            println!(
                "  {}",
                "No accessible items found in this directory.".yellow()
            );
            println!("  {}", "(Permission denied or empty directory)".dimmed());
        } else {
            // Build table rows
            let rows: Vec<Vec<String>> = children
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let name = entry
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| entry.path.display().to_string());

                    let icon = if entry.is_dir { "📁" } else { "📄" };
                    let pct = if total_parent > 0 {
                        (entry.size_bytes as f64 / total_parent as f64) * 100.0
                    } else {
                        0.0
                    };

                    vec![
                        format!("{:>2}", i + 1),
                        format!("{} {}", icon, name),
                        ui::fmt_size(entry.size_bytes),
                        format!("{}", entry.file_count),
                        format!("{} {:.0}%", ui::progress_bar(pct, 15), pct),
                    ]
                })
                .collect();

            ui::print_table(
                &["#", "NAME", "SIZE", "FILES", "SHARE"],
                &rows,
                &[4, 30, 12, 8, 22],
            );

            // Summary
            let total: u64 = children.iter().map(|c| c.size_bytes).sum();
            println!(
                "\n  {} {} in {} items (scanned in {:.1}s)",
                "Total:".bold(),
                ui::colored_size(total),
                children.len(),
                elapsed
            );
        }

        // Navigation menu
        println!("\n  {}", "Options:".bold());
        println!(
            "  {}  {}",
            "[N]".cyan(),
            "Enter number to drill into a folder".white()
        );
        println!("  {}  {}", "[T]".cyan(), "Show file type breakdown".white());
        println!("  {}  {}", "[F]".cyan(), "Find largest files here".white());
        println!("  {}  {}", "[B]".cyan(), "Go back / up one level".white());
        println!("  {}  {}", "[0]".cyan(), "Return to main menu".white());

        let input = ui::read_line().to_lowercase();

        match input.as_str() {
            "0" | "q" | "quit" => break,
            "b" | "back" | ".." => {
                if let Some(parent) = current_path.parent() {
                    current_path = parent.to_path_buf();
                } else {
                    println!("  {}", "Already at root!".yellow());
                }
            }
            "t" | "types" => {
                show_file_types(&current_path);
            }
            "f" | "files" => {
                show_largest_files(&current_path);
            }
            other => {
                if let Ok(num) = other.parse::<usize>() {
                    if num >= 1 && num <= children.len() {
                        let selected = &children[num - 1];
                        if selected.is_dir {
                            current_path = selected.path.clone();
                        } else {
                            println!(
                                "  {} {} ({})",
                                "File:".bold(),
                                selected.path.display(),
                                ui::fmt_size(selected.size_bytes)
                            );
                        }
                    } else {
                        println!("  {}", "Invalid number.".red());
                    }
                } else {
                    println!(
                        "  {}",
                        "Unknown command. Try a number, T, F, B, or 0.".yellow()
                    );
                }
            }
        }
    }

    Ok(())
}

/// Show file type breakdown for current directory
fn show_file_types(path: &Path) {
    println!(
        "\n  {} {}",
        "Analyzing file types in".dimmed(),
        path.display().to_string().cyan()
    );

    let timer = Instant::now();
    let types = walker::file_type_breakdown(path, 20);
    let elapsed = timer.elapsed().as_secs_f64();

    let rows: Vec<Vec<String>> = types
        .iter()
        .map(|t| {
            vec![
                format!(".{}", t.extension),
                ui::fmt_size(t.total_bytes),
                format!("{}", t.file_count),
                format!(
                    "{} {:.1}%",
                    ui::progress_bar(t.percentage, 15),
                    t.percentage
                ),
            ]
        })
        .collect();

    println!();
    ui::print_table(
        &["EXT", "TOTAL SIZE", "COUNT", "SHARE"],
        &rows,
        &[14, 14, 10, 22],
    );
    println!("  {} {:.1}s", "Analyzed in".dimmed(), elapsed);
}

/// Show largest individual files
fn show_largest_files(path: &Path) {
    println!(
        "\n  {} {}",
        "Finding largest files in".dimmed(),
        path.display().to_string().cyan()
    );

    let timer = Instant::now();
    let files = walker::find_largest_files(path, 20);
    let elapsed = timer.elapsed().as_secs_f64();

    let rows: Vec<Vec<String>> = files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = f
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let dir = f
                .path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default();

            vec![
                format!("{:>2}", i + 1),
                name,
                ui::fmt_size(f.size_bytes),
                dir,
            ]
        })
        .collect();

    println!();
    ui::print_table(
        &["#", "FILE NAME", "SIZE", "LOCATION"],
        &rows,
        &[4, 28, 12, 36],
    );
    println!("  {} {:.1}s", "Found in".dimmed(), elapsed);
}
