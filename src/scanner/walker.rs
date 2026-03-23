use crate::core::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use walkdir::WalkDir;

pub fn dir_size(path: &Path) -> (u64, usize) {
    let total_bytes = AtomicU64::new(0);
    let file_count = AtomicUsize::new(0);

    // Use walkdir with error handling for permission issues

    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) // Skip permission errors
        .for_each(|entry| {
            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    total_bytes.fetch_add(meta.len(), Ordering::Relaxed);
                    file_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

    (
        total_bytes.load(Ordering::Relaxed),
        file_count.load(Ordering::Relaxed),
    )
}
/// Scan top-level children of a directory and return their sizes
/// Uses parallel iteration for speed on folders with many subdirectories
pub fn scan_children(path: &Path, max_results: usize) -> Vec<SizedEntry> {
    // List immediate children
    let children: Vec<PathBuf> = match std::fs::read_dir(path) {
        Ok(entries) => entries.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(_) => return Vec::new(),
    };

    // Calculate sizes in parallel using rayon
    let mut results: Vec<SizedEntry> = children
        .par_iter()
        .map(|child| {
            let is_dir = child.is_dir();
            let (size, count) = if is_dir {
                dir_size(child)
            } else {
                let size = child.metadata().map(|m| m.len()).unwrap_or(0);
                (size, 1)
            };

            SizedEntry {
                path: child.clone(),
                size_bytes: size,
                file_count: count,
                is_dir,
            }
        })
        .collect();

    // Sort by size descending
    results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    results.truncate(max_results);
    results
}

/// Find the N largest individual files under a path
///
///

pub fn find_largest_files(path: &Path, max_results: usize) -> Vec<SizedEntry> {
    let mut results: Vec<SizedEntry> = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.into_path();
            let size = std::fs::metadata(&path).map(|m| m.len()).ok()?;
            Some(SizedEntry {
                path,
                size_bytes: size,
                file_count: 1,
                is_dir: false,
            })
        })
        .collect();

    // Sort once at the end — much faster than sorting on every insertion
    results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    results.truncate(max_results);
    results
}

// Find the smallest N files under a path
pub fn find_smallest_files(path: &Path, max_results: usize) -> Vec<SizedEntry> {
    let mut results: Vec<SizedEntry> = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.into_path();
            let size = std::fs::metadata(&path).map(|m| m.len()).ok()?;
            Some(SizedEntry {
                path,
                size_bytes: size,
                file_count: 1,
                is_dir: false,
            })
        })
        .collect();

    // Sort once at the end — much faster than sorting on every insertion
    results.sort_by(|a, b| a.size_bytes.cmp(&b.size_bytes));
    results.truncate(max_results);
    results
}

// Get file type breakdown — aggregates by extension
pub fn file_type_breakdown(path: &Path, max_results: usize) -> Vec<FileTypeStats> {
    let mut map: HashMap<String, (u64, usize)> = HashMap::new();

    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .for_each(|entry| {
            if let Ok(meta) = entry.metadata() {
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("(no ext)")
                    .to_lowercase();

                let slot = map.entry(ext).or_insert((0, 0));
                slot.0 += meta.len();
                slot.1 += 1;
            }
        });

    let total: u64 = map.values().map(|(s, _)| s).sum();

    let mut results: Vec<FileTypeStats> = map
        .into_iter()
        .map(|(ext, (bytes, count))| FileTypeStats {
            extension: ext,
            total_bytes: bytes,
            file_count: count,
            percentage: if total > 0 {
                (bytes as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    results.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    results.truncate(max_results);
    results
}

/// Get disk/drive information using sysinfo
pub fn get_drives() -> Vec<DriveInfo> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    disks
        .iter()
        .map(|d| {
            let total = d.total_space();
            let available = d.available_space();
            let used = total.saturating_sub(available);
            let usage = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            DriveInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                free_bytes: available,
                usage_percent: usage,
                fs_type: d.file_system().to_string_lossy().to_string(),
            }
        })
        .collect()
}
