use crate::core::*;
use crate::scanner::walker;
use std::env;
use std::path::PathBuf;

// ── Environment Paths ────────────────────────────────────────────────────────

struct EnvPaths {
    local_appdata: String,
    appdata: String,
}

impl EnvPaths {
    fn load() -> Self {
        let username = env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
        let user_profile =
            env::var("USERPROFILE").unwrap_or_else(|_| format!("C:\\Users\\{}", username));
        let local_appdata = env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| format!("{}\\AppData\\Local", user_profile));
        let appdata =
            env::var("APPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Roaming", user_profile));

        Self {
            local_appdata,
            appdata,
        }
    }
}

// ── Public API

/// Build the full list of cleanup targets for this system
pub fn discover_targets() -> Vec<CleanupTarget> {
    let paths = EnvPaths::load();
    let mut targets = Vec::new();

    add_temp_targets(&mut targets, &paths);
    add_browser_targets(&mut targets, &paths);
    add_windows_update_targets(&mut targets);
    add_thumbnail_targets(&mut targets, &paths);
    add_crash_dump_targets(&mut targets, &paths);
    add_installer_targets(&mut targets);
    add_recycle_bin_targets(&mut targets);

    targets
}

/// Quick estimate of total reclaimable space (used by health module)
pub fn estimate_reclaimable() -> u64 {
    discover_targets()
        .iter()
        .filter(|t| t.is_safe)
        .map(|t| t.size_bytes)
        .sum()
}

// ── Private Category Helpers ─────────────────────────────────────────────────

fn add_temp_targets(targets: &mut Vec<CleanupTarget>, paths: &EnvPaths) {
    add_target(
        targets,
        "win-temp",
        PathBuf::from("C:\\Windows\\Temp"),
        "Windows system temporary files",
        CleanupCategory::TempFiles,
        true,
        true,
    );

    add_target(
        targets,
        "user-temp",
        PathBuf::from(format!("{}\\Temp", paths.local_appdata)),
        "User temporary files",
        CleanupCategory::TempFiles,
        true,
        false,
    );

    // Prefetch belongs here — Windows rebuilds these automatically after reboot
    add_target(
        targets,
        "prefetch",
        PathBuf::from("C:\\Windows\\Prefetch"),
        "Windows Prefetch files",
        CleanupCategory::TempFiles,
        true,
        true,
    );
}

fn add_browser_targets(targets: &mut Vec<CleanupTarget>, paths: &EnvPaths) {
    add_target(
        targets,
        "chrome-cache",
        PathBuf::from(format!(
            "{}\\Google\\Chrome\\User Data\\Default\\Cache",
            paths.local_appdata
        )),
        "Google Chrome browser cache",
        CleanupCategory::BrowserCache,
        true,
        false,
    );

    add_target(
        targets,
        "chrome-code-cache",
        PathBuf::from(format!(
            "{}\\Google\\Chrome\\User Data\\Default\\Code Cache",
            paths.local_appdata
        )),
        "Google Chrome code cache",
        CleanupCategory::BrowserCache,
        true,
        false,
    );

    add_target(
        targets,
        "edge-cache",
        PathBuf::from(format!(
            "{}\\Microsoft\\Edge\\User Data\\Default\\Cache",
            paths.local_appdata
        )),
        "Microsoft Edge browser cache",
        CleanupCategory::BrowserCache,
        true,
        false,
    );

    // Firefox profiles live under APPDATA (Roaming), not LOCALAPPDATA
    add_target(
        targets,
        "firefox-cache",
        PathBuf::from(format!("{}\\Mozilla\\Firefox\\Profiles", paths.appdata)),
        "Mozilla Firefox cache profiles",
        CleanupCategory::BrowserCache,
        true,
        false,
    );
}

fn add_windows_update_targets(targets: &mut Vec<CleanupTarget>) {
    add_target(
        targets,
        "win-update-dl",
        PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download"),
        "Windows Update downloaded files",
        CleanupCategory::WindowsUpdate,
        true,
        true,
    );
}

fn add_thumbnail_targets(targets: &mut Vec<CleanupTarget>, paths: &EnvPaths) {
    add_target(
        targets,
        "thumbnails",
        PathBuf::from(format!(
            "{}\\Microsoft\\Windows\\Explorer",
            paths.local_appdata
        )),
        "Windows Explorer thumbnail cache — regenerated automatically",
        CleanupCategory::Thumbnails,
        true,
        false,
    );
}

fn add_crash_dump_targets(targets: &mut Vec<CleanupTarget>, paths: &EnvPaths) {
    add_target(
        targets,
        "crash-dumps",
        PathBuf::from(format!("{}\\CrashDumps", paths.local_appdata)),
        "Application crash dump files",
        CleanupCategory::CrashDumps,
        true,
        false,
    );

    add_target(
        targets,
        "win-minidumps",
        PathBuf::from("C:\\Windows\\Minidump"),
        "Windows minidump files",
        CleanupCategory::CrashDumps,
        true,
        true,
    );
}

fn add_installer_targets(targets: &mut Vec<CleanupTarget>) {
    // Mild risk: deleting this may break uninstall/repair for some older software
    add_target(
        targets,
        "installer-cache",
        PathBuf::from("C:\\Windows\\Installer\\$PatchCache$"),
        "Windows Installer patch cache — may affect uninstall/repair of some programs",
        CleanupCategory::InstallerCache,
        false, // marked not safe to prevent accidental auto-clean
        true,
    );
}

fn add_recycle_bin_targets(targets: &mut Vec<CleanupTarget>) {
    // Files here are permanently deleted — user must explicitly confirm
    add_target(
        targets,
        "recycle-bin",
        PathBuf::from("C:\\$Recycle.Bin"),
        "Recycle Bin contents — permanently deleted, cannot be recovered",
        CleanupCategory::RecycleBin,
        false, // marked not safe to force a confirmation prompt
        true,
    );
}

// ── Core Helper ──────────────────────────────────────────────────────────────

/// Only adds a target if its path actually exists on this machine
fn add_target(
    targets: &mut Vec<CleanupTarget>,
    id: &str,
    path: PathBuf,
    description: &str,
    category: CleanupCategory,
    is_safe: bool,
    requires_admin: bool,
) {
    if path.exists() {
        let (size, count) = walker::dir_size(&path);
        targets.push(CleanupTarget {
            id: id.to_string(),
            path,
            description: description.to_string(),
            category,
            size_bytes: size,
            file_count: count,
            is_safe,
            requires_admin,
        });
    }
}
