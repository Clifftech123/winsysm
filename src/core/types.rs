use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// DISK TYPES
/// Information about a single disk/drive
#[derive(Debug, Clone, Serialize)]
pub struct DriveInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f64,
    pub fs_type: String,
}

/// A folder or file with its measured size
#[derive(Debug, Clone, Serialize)]
pub struct SizedEntry {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: usize,
    pub is_dir: bool,
}

/// Breakdown of disk usage by file extension
#[derive(Debug, Clone, Serialize)]
pub struct FileTypeStats {
    pub extension: String,
    pub total_bytes: u64,
    pub file_count: usize,
    pub percentage: f64,
}

/// Complete result of a disk scan
#[derive(Debug, Clone, Serialize)]
pub struct DiskScanResult {
    pub drive: DriveInfo,
    pub top_folders: Vec<SizedEntry>,
    pub top_files: Vec<SizedEntry>,
    pub file_types: Vec<FileTypeStats>,
    pub scan_duration_secs: f64,
}

// PROCESS TYPES
/// A single running process
#[derive(Debug, Clone, Serialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub exe_path: Option<PathBuf>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub parent_pid: Option<u32>,
    pub user: Option<String>,
}

/// How to sort the process list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessSortBy {
    Cpu,
    Memory,
    Name,
    Pid,
}

/// Complete result of a process scan
#[derive(Debug, Clone, Serialize)]
pub struct ProcessScanResult {
    pub processes: Vec<ProcessEntry>,
    pub total_count: usize,
    pub total_cpu: f32,
    pub total_memory_bytes: u64,
    pub system_memory_total: u64,
    pub system_memory_used: u64,
    pub cpu_core_count: usize,
}

// CLEANUP TYPES

/// A location that can be cleaned up
#[derive(Debug, Clone, Serialize)]
pub struct CleanupTarget {
    pub id: String,
    pub path: PathBuf,
    pub description: String,
    pub category: CleanupCategory,
    pub size_bytes: u64,
    pub file_count: usize,
    pub is_safe: bool,
    pub requires_admin: bool,
}

/// Categories of junk files
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum CleanupCategory {
    TempFiles,
    BrowserCache,
    WindowsUpdate,
    LogFiles,
    RecycleBin,
    Thumbnails,
    CrashDumps,
    InstallerCache,
}

impl std::fmt::Display for CleanupCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TempFiles => write!(f, "Temp Files"),
            Self::BrowserCache => write!(f, "Browser Cache"),
            Self::WindowsUpdate => write!(f, "Windows Update"),
            Self::LogFiles => write!(f, "Log Files"),
            Self::RecycleBin => write!(f, "Recycle Bin"),
            Self::Thumbnails => write!(f, "Thumbnails"),
            Self::CrashDumps => write!(f, "Crash Dumps"),
            Self::InstallerCache => write!(f, "Installer Cache"),
        }
    }
}

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize)]
pub struct CleanupResult {
    pub target_id: String,
    pub freed_bytes: u64,
    pub files_deleted: usize,
    pub files_failed: usize,
    pub errors: Vec<String>,
}

// SECURITY TYPES

/// Threat severity levels
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
    Info = 0,
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// What kind of threat was detected
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ThreatSource {
    ProcessMimic,       // Name mimics system process
    SuspiciousPath,     // Running from Temp/Downloads
    RandomName,         // Random-looking filename
    HighCpuUnknown,     // High CPU from unknown process
    SuspiciousStartup,  // Startup entry from bad location
    ExecutableInBadDir, // .exe in temp/appdata/programdata
    NetworkActivity,    // Unknown process with network connections
    MultipleInstances,  // Too many copies of a system process
}

impl std::fmt::Display for ThreatSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessMimic => write!(f, "Process Name Mimicry"),
            Self::SuspiciousPath => write!(f, "Suspicious Exe Location"),
            Self::RandomName => write!(f, "Random Process Name"),
            Self::HighCpuUnknown => write!(f, "High CPU Unknown Process"),
            Self::SuspiciousStartup => write!(f, "Suspicious Startup Entry"),
            Self::ExecutableInBadDir => write!(f, "Executable in Bad Directory"),
            Self::NetworkActivity => write!(f, "Suspicious Network Activity"),
            Self::MultipleInstances => write!(f, "Multiple Instances"),
        }
    }
}

/// A single detected threat
#[derive(Debug, Clone, Serialize)]
pub struct Threat {
    pub level: ThreatLevel,
    pub source: ThreatSource,
    pub process_name: Option<String>,
    pub pid: Option<u32>,
    pub path: Option<PathBuf>,
    pub description: String,
    pub recommendation: String,
}

/// Complete security scan result
#[derive(Debug, Clone, Serialize)]
pub struct SecurityScanResult {
    pub threats: Vec<Threat>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub scan_duration_secs: f64,
}

// HEALTH TYPES

/// Health status for a single check
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum HealthStatus {
    Good,
    Warning,
    Critical,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Good => write!(f, "✅ GOOD"),
            Self::Warning => write!(f, "⚠️  WARNING"),
            Self::Critical => write!(f, "🔴 CRITICAL"),
        }
    }
}

/// A single health check result
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub category: String,
    pub status: HealthStatus,
    pub score_impact: i32,
    pub message: String,
    pub detail: Option<String>,
}

/// Overall system health report
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub score: i32,
    pub rating: String,
    pub checks: Vec<HealthCheck>,
    pub recommendations: Vec<String>,
    pub top_cpu_processes: Vec<ProcessEntry>,
    pub top_memory_processes: Vec<ProcessEntry>,
    pub generated_at: String,
}

// CONFIG TYPE

/// User-configurable settings (saved to config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub scan_depth: usize,
    pub top_n_results: usize,
    pub cleanup_dry_run_default: bool,
    pub log_file_max_age_days: u32,
    pub security_check_network: bool,
    pub health_export_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            scan_depth: 1,
            top_n_results: 20,
            cleanup_dry_run_default: true,
            log_file_max_age_days: 30,
            security_check_network: true,
            health_export_path: None,
        }
    }
}
