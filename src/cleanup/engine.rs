use crate::cleanup::targets;
use crate::core::*;
use crate::ui;
use colored::*;
use std::fs;
use walkdir::WalkDir;

// ── Public ───────────────────────────────────────────────────────────────────

pub fn run_cleanup() -> AppResult<()> {
    ui::print_header("🧹", "CLEANUP ENGINE", Color::Yellow);
    println!("  {}", "Scanning for junk files...".dimmed());
    println!();

    let all_targets = targets::discover_targets();

    if all_targets.is_empty() {
        println!(
            "  {} No junk files found! Your system is clean.",
            "✅".green()
        );
        return Ok(());
    }

    show_targets(&all_targets);
    show_totals(&all_targets);
    run_menu(&all_targets)?;

    Ok(())
}

// ── Display

fn show_targets(all_targets: &[CleanupTarget]) {
    let rows: Vec<Vec<String>> = all_targets
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let safe_icon = if t.is_safe { "✅" } else { "⚠️ " };
            let admin_icon = if t.requires_admin { "🔒" } else { "   " };
            vec![
                format!("{:>2}", i + 1),
                t.description.clone(),
                format!("{}", t.category),
                ui::fmt_size(t.size_bytes),
                format!("{}", t.file_count),
                format!("{} {}", safe_icon, admin_icon),
            ]
        })
        .collect();

    ui::print_table(
        &["#", "DESCRIPTION", "CATEGORY", "SIZE", "FILES", "SAFE"],
        &rows,
        &[4, 34, 16, 12, 8, 8],
    );
}

fn show_totals(all_targets: &[CleanupTarget]) {
    let total_size: u64 = all_targets.iter().map(|t| t.size_bytes).sum();
    let total_files: usize = all_targets.iter().map(|t| t.file_count).sum();
    let safe_size: u64 = all_targets
        .iter()
        .filter(|t| t.is_safe)
        .map(|t| t.size_bytes)
        .sum();

    println!("\n  {}", "━".repeat(60).dimmed());
    println!(
        "  {} {} in {} files",
        "Total found:".bold(),
        ui::colored_size(total_size),
        total_files
    );
    println!(
        "  {} {}",
        "Safe to delete:".bold(),
        ui::fmt_size(safe_size).green()
    );

    if total_size != safe_size {
        println!(
            "  {} 🔒 = requires Administrator privileges",
            "Note:".dimmed()
        );
    }
}

// ── Menu

fn run_menu(all_targets: &[CleanupTarget]) -> AppResult<()> {
    let safe_size: u64 = all_targets
        .iter()
        .filter(|t| t.is_safe)
        .map(|t| t.size_bytes)
        .sum();

    loop {
        print_menu_options();

        match ui::read_line().to_lowercase().as_str() {
            "0" | "q" => break,
            "a" | "all" => delete_all(all_targets, safe_size),
            "s" | "select" => delete_selected(all_targets),
            "d" | "dry" => dry_run(all_targets),
            _ => println!("  {}", "Unknown command.".yellow()),
        }
    }

    Ok(())
}

fn print_menu_options() {
    println!("\n  {}", "What would you like to do?".bold());
    println!("  {}  {}", "[A]".cyan(), "Delete ALL safe targets".white());
    println!(
        "  {}  {}",
        "[S]".cyan(),
        "Select specific targets to delete".white()
    );
    println!(
        "  {}  {}",
        "[D]".cyan(),
        "Dry run (preview what would be deleted)".white()
    );
    println!("  {}  {}", "[0]".cyan(), "Go back to main menu".white());
}

// ── Handlers

fn delete_all(all_targets: &[CleanupTarget], safe_size: u64) {
    let safe_targets: Vec<&CleanupTarget> = all_targets.iter().filter(|t| t.is_safe).collect();

    if ui::confirm(&format!(
        "Delete {} across {} targets?",
        ui::fmt_size(safe_size),
        safe_targets.len()
    )) {
        let results = run_cleanup_on(&safe_targets, false);
        show_summary(&results);
    }
}

fn delete_selected(all_targets: &[CleanupTarget]) {
    let labels: Vec<&str> = all_targets.iter().map(|t| t.description.as_str()).collect();
    let indices = ui::pick_multiple(&labels);
    let selected: Vec<&CleanupTarget> = indices.iter().map(|&i| &all_targets[i]).collect();
    let sel_size: u64 = selected.iter().map(|t| t.size_bytes).sum();

    if !selected.is_empty()
        && ui::confirm(&format!(
            "Delete {} from {} targets?",
            ui::fmt_size(sel_size),
            selected.len()
        ))
    {
        let results = run_cleanup_on(&selected, false);
        show_summary(&results);
    }
}

fn dry_run(all_targets: &[CleanupTarget]) {
    println!(
        "\n  {}",
        "═══ DRY RUN (nothing will be deleted) ═══".yellow().bold()
    );
    let safe_targets: Vec<&CleanupTarget> = all_targets.iter().filter(|t| t.is_safe).collect();
    let _ = run_cleanup_on(&safe_targets, true);
    println!("  {}", "═══ End of dry run ═══".yellow().bold());
}

// ── Execution

fn run_cleanup_on(targets: &[&CleanupTarget], dry_run: bool) -> Vec<CleanupResult> {
    targets.iter().map(|t| clean_one(t, dry_run)).collect()
}

fn clean_one(target: &CleanupTarget, dry_run: bool) -> CleanupResult {
    let prefix = if dry_run {
        "[DRY RUN]".yellow().to_string()
    } else {
        "Cleaning".white().to_string()
    };
    println!(
        "\n  {} {} ({})",
        prefix,
        target.description.cyan(),
        ui::fmt_size(target.size_bytes)
    );

    let mut freed = 0u64;
    let mut deleted = 0usize;
    let mut failed = 0usize;
    let mut errors = Vec::new();

    let entries: Vec<_> = WalkDir::new(&target.path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    let total = entries.len();

    for (i, entry) in entries.iter().enumerate() {
        if i % 100 == 0 || i == total - 1 {
            let pct = if total > 0 {
                ((i + 1) as f64 / total as f64) * 100.0
            } else {
                100.0
            };
            print!(
                "\r  {} {}/{} files ({:.0}%)",
                ui::progress_bar(pct, 20),
                i + 1,
                total,
                pct
            );
        }

        if let Ok(meta) = entry.metadata() {
            let size = meta.len();
            if dry_run {
                freed += size;
                deleted += 1;
            } else {
                match fs::remove_file(entry.path()) {
                    Ok(()) => {
                        freed += size;
                        deleted += 1;
                    }
                    Err(e) => {
                        failed += 1;
                        if errors.len() < 5 {
                            errors.push(format!("{}: {}", entry.path().display(), e));
                        }
                    }
                }
            }
        }
    }

    println!();

    if !dry_run {
        remove_empty_dirs(&target.path);
    }

    if dry_run {
        println!(
            "  {} Would free {} ({} files)",
            "→".cyan(),
            ui::fmt_size(freed).green(),
            deleted
        );
    } else {
        println!(
            "  {} Freed {} ({} deleted, {} skipped)",
            "✅".green(),
            ui::fmt_size(freed).green(),
            deleted,
            failed
        );
    }

    CleanupResult {
        target_id: target.id.clone(),
        freed_bytes: freed,
        files_deleted: deleted,
        files_failed: failed,
        errors,
    }
}

// ── Summary

fn show_summary(results: &[CleanupResult]) {
    let total_freed: u64 = results.iter().map(|r| r.freed_bytes).sum();
    let total_deleted: usize = results.iter().map(|r| r.files_deleted).sum();
    let total_failed: usize = results.iter().map(|r| r.files_failed).sum();

    println!("\n  {}", "━".repeat(50).dimmed());
    println!(
        "  {} {} freed | {} deleted | {} skipped",
        "DONE:".green().bold(),
        ui::fmt_size(total_freed).green().bold(),
        total_deleted,
        total_failed
    );

    let all_errors: Vec<&String> = results.iter().flat_map(|r| r.errors.iter()).collect();
    if !all_errors.is_empty() {
        println!("\n  {} (first 5):", "Errors".yellow());
        for err in all_errors.iter().take(5) {
            println!("    {}", err.dimmed());
        }
    }
}

// ─ Helpers

fn remove_empty_dirs(path: &std::path::Path) {
    let dirs: Vec<_> = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.into_path())
        .collect();

    for dir in dirs.iter().rev() {
        if dir == path {
            continue;
        }
        let _ = fs::remove_dir(dir);
    }
}
