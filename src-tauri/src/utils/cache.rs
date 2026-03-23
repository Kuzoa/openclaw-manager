use crate::models::{DetectionResult, DetectionStep};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use super::platform;
use super::shell;

// TTL constants for cache entries (in seconds)
const TTL_STABLE_PATHS: u64 = 604800; // 7 days
const TTL_OPENCLAW_PATH: u64 = 86400; // 24 hours
const CACHE_FILE_VERSION: u32 = 1;

/// Single cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheEntry {
    pub value: String,
    pub cached_at: String, // ISO 8601 timestamp
    pub ttl_seconds: u64,
}

/// Persistent cache file structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentCacheFile {
    pub version: u32,
    pub cache: HashMap<String, CacheEntry>,
}

/// Global environment cache instance
/// Uses `once_cell::sync::Lazy` for thread-safe lazy initialization
pub static ENVIRONMENT_CACHE: Lazy<EnvironmentCache> = Lazy::new(EnvironmentCache::new);

/// One-time initialization guard for cache directory
static CACHE_DIR_SET: OnceLock<()> = OnceLock::new();

/// A cached value that distinguishes between "not yet initialized" and "initialized to None"
///
/// - `None` = not yet initialized (lazy load pending)
/// - `Some(None)` = initialized, but value is None (e.g., OpenClaw not installed)
/// - `Some(Some(value))` = initialized with a value
type CachedValue<T> = RwLock<Option<Option<T>>>;

/// Environment cache for storing expensive-to-compute environment data
///
/// This cache reduces repeated shell command executions by storing:
/// - npm global prefix (from `npm config get prefix`)
/// - OpenClaw installation path
/// - OpenClaw version
/// - Node.js version
/// - Git version
/// - Security status (whether OpenClaw version >= 2026.1.29)
///
/// # Thread Safety
///
/// Uses `RwLock` for thread-safe access. Multiple readers can access
/// concurrently, while writers (initialization, invalidation) block.
///
/// # Cache Invalidation
///
/// Call `invalidate()` to clear all cached values. This should be done after:
/// - Installing OpenClaw
/// - Updating OpenClaw
/// - Uninstalling OpenClaw
/// - User manually refreshes environment from UI
pub struct EnvironmentCache {
    /// npm global prefix from `npm config get prefix`
    pub(crate) npm_prefix: CachedValue<String>,
    /// OpenClaw executable path
    pub(crate) openclaw_path: CachedValue<String>,
    /// OpenClaw version string
    pub(crate) openclaw_version: CachedValue<String>,
    /// Node.js version string
    pub(crate) node_version: CachedValue<String>,
    /// Git version string
    pub(crate) git_version: CachedValue<String>,
    /// Whether OpenClaw version is secure (>= 2026.1.29)
    pub(crate) is_secure: CachedValue<bool>,
    /// Detection steps for OpenClaw path detection
    pub(crate) detection_steps: CachedValue<Vec<DetectionStep>>,
    /// Whether Gateway Service is installed (cached to avoid slow openclaw gateway status)
    pub(crate) gateway_installed: CachedValue<bool>,
    /// Cache directory path for persistent storage
    cache_dir: RwLock<Option<PathBuf>>,
    /// Cache hit flags for detection_steps generation
    /// key: "npm_prefix" or "openclaw_path"
    /// value: true means cache hit from file, false means miss or invalid
    cache_hit_flags: RwLock<HashMap<String, bool>>,
}

impl EnvironmentCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            npm_prefix: RwLock::new(None),
            openclaw_path: RwLock::new(None),
            openclaw_version: RwLock::new(None),
            node_version: RwLock::new(None),
            git_version: RwLock::new(None),
            is_secure: RwLock::new(None),
            detection_steps: RwLock::new(None),
            gateway_installed: RwLock::new(None),
            cache_dir: RwLock::new(None),
            cache_hit_flags: RwLock::new(HashMap::new()),
        }
    }

    /// Get npm global prefix with lazy initialization
    ///
    /// Executes `npm config get prefix` on first call and caches the result.
    /// Uses double-checked locking to prevent race conditions.
    /// Now checks file cache first before executing npm command.
    pub fn get_npm_prefix(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.npm_prefix.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.npm_prefix.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // Try to load from file cache first
        if let Some(cache_data) = self.load_cache_from_file() {
            if let Some(cached_path) = self.get_cached_path(&cache_data, "npm_prefix") {
                info!("[Cache] npm_prefix: using cached value");
                // Mark as cache hit
                {
                    let mut flags = self.cache_hit_flags.write().unwrap();
                    flags.insert("npm_prefix".to_string(), true);
                }
                *guard = Some(Some(cached_path.clone()));
                return Some(cached_path);
            }
        }

        // Cache miss - execute npm command
        info!("[Cache] npm_prefix: cache miss, executing npm config get prefix");
        let result = self.fetch_npm_prefix_internal();

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());

        // IMPORTANT: Drop the write lock before calling save_paths_to_file()
        // to avoid deadlock (save_paths_to_file() needs read lock on npm_prefix)
        drop(guard);

        // Save to file cache
        self.save_paths_to_file();

        result
    }

    /// Internal function to fetch npm prefix
    fn fetch_npm_prefix_internal(&self) -> Option<String> {
        let result = if platform::is_windows() {
            shell::run_cmd_output("npm config get prefix")
        } else {
            shell::run_bash_output("npm config get prefix 2>/dev/null")
        };

        match result {
            Ok(prefix) => {
                let prefix = prefix.trim();
                if !prefix.is_empty() {
                    info!("[Cache] npm_prefix: '{}'", prefix);
                    Some(prefix.to_string())
                } else {
                    warn!("[Cache] npm_prefix: empty result");
                    None
                }
            }
            Err(e) => {
                warn!("[Cache] npm_prefix: failed to get npm prefix: {}", e);
                None
            }
        }
    }

    /// Get OpenClaw executable path with lazy initialization
    ///
    /// Uses cached npm_prefix if available, falls back to the existing
    /// Phase 2/3 detection logic in shell::get_openclaw_path_internal.
    /// Uses double-checked locking to prevent race conditions.
    /// Now checks file cache first before executing detection.
    pub fn get_openclaw_path(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.openclaw_path.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // IMPORTANT: Ensure npm_prefix is initialized BEFORE acquiring openclaw_path write lock
        // to avoid deadlock (detect_openclaw_path_with_steps_internal reads npm_prefix)
        // This also helps populate the cache for faster detection
        let _ = self.get_npm_prefix();

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.openclaw_path.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // Try to load from file cache first
        if let Some(cache_data) = self.load_cache_from_file() {
            if let Some(cached_path) = self.get_cached_path(&cache_data, "openclaw_path") {
                info!("[Cache] openclaw_path: using cached value");
                // Mark as cache hit
                {
                    let mut flags = self.cache_hit_flags.write().unwrap();
                    flags.insert("openclaw_path".to_string(), true);
                }
                *guard = Some(Some(cached_path.clone()));
                // Set simplified detection steps for cache hit
                {
                    let mut steps_guard = self.detection_steps.write().unwrap();
                    *steps_guard = Some(Some(vec![DetectionStep {
                        phase: "Cache: Using cached path".to_string(),
                        action: "Loading from cache".to_string(),
                        target: cached_path.clone(),
                        result: DetectionResult::Found,
                        message: Some("Path loaded from cache".to_string()),
                    }]));
                }
                return Some(cached_path);
            }
        }

        // Cache miss - detect path with steps
        info!("[Cache] openclaw_path: cache miss, detecting...");
        let (result, steps) = self.detect_openclaw_path_with_steps_internal();

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());

        // Also cache the detection steps
        {
            let mut steps_guard = self.detection_steps.write().unwrap();
            *steps_guard = Some(Some(steps));
        }

        // IMPORTANT: Drop the write lock before calling save_paths_to_file()
        // to avoid deadlock (save_paths_to_file() needs read lock on openclaw_path)
        drop(guard);

        // Save to file cache
        self.save_paths_to_file();

        result
    }

    /// Get detection steps for OpenClaw path detection
    /// 
    /// Returns the cached detection steps, or detects them if not yet cached.
    /// This provides detailed information about how OpenClaw was found (or not found).
    pub fn get_detection_steps(&self) -> Vec<DetectionStep> {
        // First check with read lock (fast path)
        {
            let guard = self.detection_steps.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone().unwrap_or_default();
            }
        }

        // Ensure path detection runs (which also populates steps)
        self.get_openclaw_path();

        // Return the now-cached steps
        let guard = self.detection_steps.read().unwrap();
        guard.as_ref().unwrap().clone().unwrap_or_default()
    }

    /// Get Gateway Service installed status with lazy initialization and caching
    ///
    /// This is cached to avoid the slow `openclaw gateway status` command (~7s).
    /// The result is persisted to the cache file with 24-hour TTL.
    pub fn get_gateway_installed(&self) -> Option<bool> {
        // First check with read lock (fast path)
        {
            let guard = self.gateway_installed.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.gateway_installed.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // Try to load from file cache first
        if let Some(cache_data) = self.load_cache_from_file() {
            if let Some(entry) = cache_data.cache.get("gateway_installed") {
                if self.is_entry_valid(entry) {
                    let value = entry.value == "true";
                    info!("[Cache] Cache hit for 'gateway_installed': {}", entry.value);
                    *guard = Some(Some(value));
                    return Some(value);
                }
            }
        }

        // Cache miss - need to detect
        info!("[Cache] gateway_installed: cache miss, detecting...");
        
        // Execute the slow command
        let result = match shell::run_openclaw(&["gateway", "status"]) {
            Ok(output) => {
                let lower = output.to_lowercase();
                if lower.contains("not installed") || lower.contains("not found") {
                    false
                } else {
                    true
                }
            }
            Err(e) => {
                let lower = e.to_lowercase();
                if lower.contains("not installed") || lower.contains("not found") {
                    false
                } else {
                    debug!("[Cache] Gateway status check failed: {}", e);
                    false
                }
            }
        };
        
        info!("[Cache] gateway_installed: {}", result);

        // Store in memory cache
        *guard = Some(Some(result));
        
        // Drop the write lock before saving to file
        drop(guard);

        // Save to file cache
        self.save_gateway_installed_to_file(result);

        Some(result)
    }

    /// Save gateway_installed to cache file
    fn save_gateway_installed_to_file(&self, value: bool) {
        let mut cache_data = EnvironmentCacheFile {
            version: CACHE_FILE_VERSION,
            cache: HashMap::new(),
        };

        // Load existing cache to preserve other entries
        if let Some(existing) = self.load_cache_from_file() {
            cache_data = existing;
        }

        let now = chrono::Utc::now().to_rfc3339();
        cache_data.cache.insert(
            "gateway_installed".to_string(),
            CacheEntry {
                value: value.to_string(),
                cached_at: now,
                ttl_seconds: TTL_OPENCLAW_PATH, // 24 hours
            },
        );

        self.save_cache_to_file(&cache_data);
    }

    /// Internal function to detect OpenClaw path with detailed steps
    /// Returns (path, detection_steps)
    fn detect_openclaw_path_with_steps_internal(&self) -> (Option<String>, Vec<DetectionStep>) {
        let mut steps: Vec<DetectionStep> = Vec::new();

        // Phase 1: Use cached npm prefix
        // IMPORTANT: We read the npm_prefix directly from the cache without calling get_npm_prefix()
        // to avoid potential deadlock (get_npm_prefix() may call save_paths_to_file() which needs openclaw_path read lock)
        let npm_prefix_result = {
            let guard = self.npm_prefix.read().unwrap();
            guard.as_ref().and_then(|v| v.clone())
        };
        
        match &npm_prefix_result {
            Some(prefix) => {
                let openclaw_path = if platform::is_windows() {
                    format!("{}\\openclaw.cmd", prefix)
                } else {
                    format!("{}/bin/openclaw", prefix)
                };

                info!("[Cache] Phase 1: Checking npm prefix path: {}", openclaw_path);
                
                if std::path::Path::new(&openclaw_path).exists() {
                    info!("[Cache] Found openclaw via npm prefix: {}", openclaw_path);
                    steps.push(DetectionStep {
                        phase: "Phase 1: npm global prefix".to_string(),
                        action: "Checking npm prefix".to_string(),
                        target: openclaw_path.clone(),
                        result: DetectionResult::Found,
                        message: None,
                    });
                    return (Some(openclaw_path), steps);
                } else {
                    steps.push(DetectionStep {
                        phase: "Phase 1: npm global prefix".to_string(),
                        action: "Checking npm prefix".to_string(),
                        target: openclaw_path,
                        result: DetectionResult::NotFound,
                        message: None,
                    });
                }
            }
            None => {
                // npm prefix check failed
                steps.push(DetectionStep {
                    phase: "Phase 1: npm global prefix".to_string(),
                    action: "Checking npm prefix".to_string(),
                    target: "npm config get prefix".to_string(),
                    result: DetectionResult::Error,
                    message: Some("Failed to get npm global prefix".to_string()),
                });
            }
        }

        // Phase 2: Check hardcoded paths
        info!("[Cache] Phase 2: Checking hardcoded paths...");
        let possible_paths = if platform::is_windows() {
            get_windows_openclaw_paths()
        } else {
            get_unix_openclaw_paths()
        };

        if !possible_paths.is_empty() {
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    info!("[Cache] Found openclaw at {}", path);
                    steps.push(DetectionStep {
                        phase: "Phase 2: Hardcoded paths".to_string(),
                        action: "Checking path".to_string(),
                        target: path.clone(),
                        result: DetectionResult::Found,
                        message: None,
                    });
                    return (Some(path), steps);
                } else {
                    steps.push(DetectionStep {
                        phase: "Phase 2: Hardcoded paths".to_string(),
                        action: "Checking path".to_string(),
                        target: path,
                        result: DetectionResult::NotFound,
                        message: None,
                    });
                }
            }
        }

        // Phase 3: Check PATH
        info!("[Cache] Phase 3: Checking PATH...");
        if shell::command_exists("openclaw") {
            steps.push(DetectionStep {
                phase: "Phase 3: PATH environment".to_string(),
                action: "Checking PATH".to_string(),
                target: "openclaw".to_string(),
                result: DetectionResult::Found,
                message: None,
            });
            return (Some("openclaw".to_string()), steps);
        } else {
            steps.push(DetectionStep {
                phase: "Phase 3: PATH environment".to_string(),
                action: "Checking PATH".to_string(),
                target: "openclaw".to_string(),
                result: DetectionResult::NotFound,
                message: None,
            });
        }

        // Last resort: search via user shell (Unix only)
        if !platform::is_windows() {
            if let Ok(path) = shell::run_bash_output("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; which openclaw 2>/dev/null") {
                if !path.is_empty() && std::path::Path::new(&path).exists() {
                    info!("[Cache] Found openclaw via user shell: {}", path);
                    // This is still part of Phase 3 - found via shell
                    steps.push(DetectionStep {
                        phase: "Phase 3: PATH environment".to_string(),
                        action: "Checking user shell".to_string(),
                        target: path.clone(),
                        result: DetectionResult::Found,
                        message: None,
                    });
                    return (Some(path), steps);
                }
            }
        }

        // Ensure at least one step exists if nothing found
        if steps.is_empty() {
            steps.push(DetectionStep {
                phase: "System".to_string(),
                action: "Environment check".to_string(),
                target: "openclaw".to_string(),
                result: DetectionResult::NotFound,
                message: Some("No detection phases found any openclaw installation".to_string()),
            });
        }

        (None, steps)
    }

    /// Get OpenClaw version with lazy initialization
    ///
    /// First checks if OpenClaw is installed (via get_openclaw_path).
    /// If not installed, returns None immediately without executing any command.
    /// Uses double-checked locking to prevent race conditions.
    /// Implements retry logic: if version detection fails, invalidates cache and retries once.
    pub fn get_openclaw_version(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.openclaw_version.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // IMPORTANT: Get openclaw_path BEFORE acquiring openclaw_version write lock
        // to avoid deadlock (get_openclaw_path() needs its own locks)
        let openclaw_path = self.get_openclaw_path();

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.openclaw_version.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // First check if OpenClaw is installed (uses cached path)
        if openclaw_path.is_none() {
            info!("[Cache] openclaw_version: OpenClaw not installed, skipping version check");
            // Cache the None result to avoid repeated checks
            *guard = Some(None);
            return None;
        }

        // OpenClaw is installed, execute version command
        info!("[Cache] openclaw_version: cache miss, executing openclaw --version");
        let result = shell::run_openclaw(&["--version"])
            .ok()
            .map(|v| v.trim().to_string());

        // If version detection failed, try invalidating cache and retrying once
        if result.is_none() {
            warn!("[Cache] openclaw_version: detection failed, invalidating cache and retrying");
            
            // IMPORTANT: Drop the write lock before calling invalidate_entry and get_openclaw_path
            drop(guard);
            
            self.invalidate_entry("openclaw_path");

            // Re-get path (may trigger full detection)
            let new_path = self.get_openclaw_path();
            
            // Re-acquire write lock
            let mut guard = self.openclaw_version.write().unwrap();
            
            if new_path.is_some() && new_path != openclaw_path {
                // Path changed, retry version detection
                info!("[Cache] openclaw_version: path changed, retrying version detection");
                let retry_result = shell::run_openclaw(&["--version"])
                    .ok()
                    .map(|v| v.trim().to_string());

                // Write to cache (wrap in Some to mark as initialized)
                *guard = Some(retry_result.clone());

                if let Some(ref v) = retry_result {
                    info!("[Cache] openclaw_version: '{}' (after retry)", v);
                } else {
                    warn!("[Cache] openclaw_version: still failed after retry");
                }

                return retry_result;
            }
            
            // Write the failed result
            *guard = Some(None);
            return None;
        }

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());

        if let Some(ref v) = result {
            info!("[Cache] openclaw_version: '{}'", v);
        } else {
            warn!("[Cache] openclaw_version: OpenClaw installed but version check failed");
        }

        result
    }

    /// Get Node.js version with lazy initialization
    /// Uses double-checked locking to prevent race conditions.
    pub fn get_node_version(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.node_version.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.node_version.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // Cache miss - detect version
        info!("[Cache] node_version: cache miss, detecting...");
        let result = self.detect_node_version_internal();

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());

        if let Some(ref v) = result {
            info!("[Cache] node_version: '{}'", v);
        }

        result
    }

    /// Internal function to detect Node.js version
    fn detect_node_version_internal(&self) -> Option<String> {
        if platform::is_windows() {
            // Windows: First try direct call
            if let Ok(v) = shell::run_cmd_output("node --version") {
                let version = v.trim().to_string();
                if !version.is_empty() && version.starts_with('v') {
                    return Some(version);
                }
            }

            // Check common installation paths
            let possible_paths = get_windows_node_paths();
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    let cmd = format!("\"{}\" --version", path);
                    if let Ok(output) = shell::run_cmd_output(&cmd) {
                        let version = output.trim().to_string();
                        if !version.is_empty() && version.starts_with('v') {
                            return Some(version);
                        }
                    }
                }
            }

            None
        } else {
            // Unix: First try direct call
            if let Ok(v) = shell::run_command_output("node", &["--version"]) {
                return Some(v.trim().to_string());
            }

            // Check common paths
            let possible_paths = get_unix_node_paths();
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    if let Ok(output) = shell::run_command_output(&path, &["--version"]) {
                        return Some(output.trim().to_string());
                    }
                }
            }

            // Try user shell
            if let Ok(output) = shell::run_bash_output("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; node --version 2>/dev/null") {
                if !output.is_empty() && output.starts_with('v') {
                    return Some(output.trim().to_string());
                }
            }

            None
        }
    }

    /// Get Git version with lazy initialization
    /// Uses double-checked locking to prevent race conditions.
    pub fn get_git_version(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.git_version.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.git_version.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // Cache miss - detect version
        info!("[Cache] git_version: cache miss, detecting...");
        let result = self.detect_git_version_internal();

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());

        if let Some(ref v) = result {
            info!("[Cache] git_version: '{}'", v);
        }

        result
    }

    /// Internal function to detect Git version
    fn detect_git_version_internal(&self) -> Option<String> {
        if platform::is_windows() {
            if let Ok(v) = shell::run_cmd_output("git --version") {
                let version = v.trim().to_string();
                if !version.is_empty() && version.contains("git version") {
                    let ver = version.replace("git version ", "");
                    let ver = ver.split('.').take(3).collect::<Vec<_>>().join(".");
                    return Some(ver);
                }
            }
            None
        } else {
            if let Ok(v) = shell::run_command_output("git", &["--version"]) {
                let version = v.trim().to_string();
                if !version.is_empty() && version.contains("git version") {
                    let ver = version.replace("git version ", "");
                    return Some(ver.trim().to_string());
                }
            }
            None
        }
    }

    /// Check if OpenClaw version is secure (>= 2026.1.29) with lazy initialization
    /// Uses double-checked locking to prevent race conditions.
    pub fn get_is_secure(&self) -> Option<bool> {
        // First check with read lock (fast path)
        {
            let guard = self.is_secure.read().unwrap();
            if guard.is_some() {
                return *guard.as_ref().unwrap();
            }
        }

        // IMPORTANT: Get openclaw_version BEFORE acquiring is_secure write lock
        // to avoid deadlock (get_openclaw_version() needs its own locks)
        info!("[Cache] is_secure: cache miss, checking version...");
        let version = self.get_openclaw_version();

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.is_secure.write().unwrap();
        if guard.is_some() {
            return *guard.as_ref().unwrap();
        }

        // Cache miss - check version
        let result = version.as_ref().map(|v| {
            // Basic string comparison assuming YYYY.M.D format
            let is_secure = v.as_str() >= "2026.1.29";
            info!("[Cache] is_secure: {} (version: {})", is_secure, v);
            is_secure
        });

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result);
        result
    }

    /// Invalidate all cached values
    ///
    /// Should be called after:
    /// - Installing OpenClaw
    /// - Updating OpenClaw
    /// - Uninstalling OpenClaw
    /// - User manually refreshes environment from UI
    ///
    /// Note: Users may manually install/uninstall OpenClaw via terminal.
    /// In such cases, the UI "Refresh" button should trigger this invalidation.
    pub fn invalidate(&self) {
        info!("[Cache] Invalidating all cached environment data...");

        // Clear each cache field (set to None = not initialized)
        {
            let mut guard = self.npm_prefix.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.openclaw_path.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.openclaw_version.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.node_version.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.git_version.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.is_secure.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.detection_steps.write().unwrap();
            *guard = None;
        }
        {
            let mut guard = self.gateway_installed.write().unwrap();
            *guard = None;
        }
        // Clear cache hit flags
        {
            let mut guard = self.cache_hit_flags.write().unwrap();
            guard.clear();
        }

        // Delete cache file if it exists
        if let Some(cache_file) = self.get_cache_file_path() {
            if cache_file.exists() {
                if let Err(e) = std::fs::remove_file(&cache_file) {
                    warn!("[Cache] Failed to delete cache file: {}", e);
                } else {
                    info!("[Cache] Cache file deleted: {:?}", cache_file);
                }
            }
        }

        info!("[Cache] Cache invalidation complete");
    }

    /// Invalidate a specific cache entry
    ///
    /// This is used when a specific path becomes invalid (e.g., version detection fails)
    /// but other cached paths are still valid.
    pub fn invalidate_entry(&self, key: &str) {
        info!("[Cache] Invalidating cache entry: {}", key);

        // Clear specific memory cache field
        match key {
            "npm_prefix" => {
                let mut guard = self.npm_prefix.write().unwrap();
                *guard = None;
            }
            "openclaw_path" => {
                let mut guard = self.openclaw_path.write().unwrap();
                *guard = None;
            }
            _ => {
                warn!("[Cache] Unknown cache key: {}", key);
                return;
            }
        }

        // Clear cache hit flag
        {
            let mut flags = self.cache_hit_flags.write().unwrap();
            flags.remove(key);
        }

        // Remove entry from file cache
        if let Some(mut cache_data) = self.load_cache_from_file() {
            if cache_data.cache.remove(key).is_some() {
                info!("[Cache] Removed '{}' from file cache", key);
                self.save_cache_to_file(&cache_data);
            }
        }

        info!("[Cache] Cache entry '{}' invalidated", key);
    }

    /// Set cache directory path (can only be called once)
    ///
    /// This should be called during application setup to initialize
    /// the persistent cache storage location.
    pub fn set_cache_dir(&self, path: PathBuf) {
        if CACHE_DIR_SET.set(()).is_ok() {
            let mut guard = self.cache_dir.write().unwrap();
            *guard = Some(path);
            info!("[Cache] Cache directory set: {:?}", *guard);
        } else {
            warn!("[Cache] Cache directory already set, ignoring duplicate call");
        }
    }

    /// Set cache directory for testing purposes (bypasses OnceLock)
    #[cfg(test)]
    pub fn set_cache_dir_for_test(&self, path: PathBuf) {
        let mut guard = self.cache_dir.write().unwrap();
        *guard = Some(path);
    }

    /// Get cache file path (returns None if cache_dir not set)
    fn get_cache_file_path(&self) -> Option<PathBuf> {
        let guard = self.cache_dir.read().unwrap();
        guard.as_ref().map(|dir| dir.join("environment.json"))
    }

    /// Ensure cache directory exists
    ///
    /// Returns true if directory exists or was created successfully,
    /// false if creation failed (will use memory-only mode)
    fn ensure_cache_dir(&self) -> bool {
        let guard = self.cache_dir.read().unwrap();
        if let Some(dir) = guard.as_ref() {
            if dir.exists() {
                return true;
            }
            if let Err(e) = std::fs::create_dir_all(dir) {
                warn!(
                    "[Cache] Failed to create cache directory: {}, using memory-only mode",
                    e
                );
                return false;
            }
            return true;
        }
        false
    }

    /// Save cache to file using atomic write (temp file + rename)
    ///
    /// This function writes to a temporary file first, then renames it
    /// to ensure atomic operation on most platforms.
    pub fn save_cache_to_file(&self, cache_data: &EnvironmentCacheFile) {
        if !self.ensure_cache_dir() {
            debug!("[Cache] Cache directory not available, skipping file save");
            return;
        }

        let cache_file = match self.get_cache_file_path() {
            Some(path) => path,
            None => {
                debug!("[Cache] No cache file path, skipping file save");
                return;
            }
        };

        let temp_file = cache_file.with_extension("json.tmp");

        // Write to temp file
        let json_str = match serde_json::to_string_pretty(cache_data) {
            Ok(s) => s,
            Err(e) => {
                warn!("[Cache] Failed to serialize cache: {}", e);
                return;
            }
        };

        if let Err(e) = std::fs::write(&temp_file, json_str) {
            warn!("[Cache] Failed to write temp cache file: {}", e);
            return;
        }

        // Rename temp file to final file (atomic on most platforms)
        if let Err(e) = std::fs::rename(&temp_file, &cache_file) {
            warn!("[Cache] Failed to rename cache file: {}", e);
            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);
        } else {
            info!("[Cache] Cache saved to: {:?}", cache_file);
        }
    }

    /// Load cache from file
    ///
    /// Returns None if:
    /// - Cache file doesn't exist (first run)
    /// - JSON parsing fails (corrupted cache)
    /// - Version mismatch (old format)
    pub fn load_cache_from_file(&self) -> Option<EnvironmentCacheFile> {
        let cache_file = self.get_cache_file_path()?;

        if !cache_file.exists() {
            debug!("[Cache] Cache file does not exist: {:?}", cache_file);
            return None;
        }

        let content = match std::fs::read_to_string(&cache_file) {
            Ok(c) => c,
            Err(e) => {
                warn!("[Cache] Failed to read cache file: {}", e);
                // Delete corrupted file
                let _ = std::fs::remove_file(&cache_file);
                return None;
            }
        };

        let cache_data: EnvironmentCacheFile = match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(e) => {
                warn!("[Cache] Failed to parse cache file: {}", e);
                // Delete corrupted file
                let _ = std::fs::remove_file(&cache_file);
                return None;
            }
        };

        // Check version
        if cache_data.version != CACHE_FILE_VERSION {
            warn!(
                "[Cache] Unsupported cache version: {}, ignoring cache",
                cache_data.version
            );
            // Delete old version file
            let _ = std::fs::remove_file(&cache_file);
            return None;
        }

        info!("[Cache] Cache loaded from: {:?}", cache_file);
        Some(cache_data)
    }

    /// Check if a cache entry is valid (TTL not expired)
    pub(crate) fn is_entry_valid(&self, entry: &CacheEntry) -> bool {
        // Parse cached_at timestamp
        let cached_at = match chrono::DateTime::parse_from_rfc3339(&entry.cached_at) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(e) => {
                warn!("[Cache] Failed to parse cached_at timestamp: {}", e);
                return false;
            }
        };

        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(cached_at);

        // Check if elapsed time exceeds TTL
        elapsed.num_seconds() < entry.ttl_seconds as i64
    }

    /// Check if a path exists in the filesystem
    pub(crate) fn is_path_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    /// Get cached path with validation (TTL + path existence)
    ///
    /// Returns None if:
    /// - Entry not in cache
    /// - TTL expired
    /// - Path doesn't exist in filesystem
    pub(crate) fn get_cached_path(&self, cache_data: &EnvironmentCacheFile, key: &str) -> Option<String> {
        let entry = cache_data.cache.get(key)?;

        // Check TTL
        if !self.is_entry_valid(entry) {
            info!("[Cache] Cache entry '{}' TTL expired", key);
            return None;
        }

        // Check path existence
        if !self.is_path_exists(&entry.value) {
            info!(
                "[Cache] Cached path '{}' no longer exists: {}",
                key, entry.value
            );
            return None;
        }

        info!("[Cache] Cache hit for '{}': {}", key, entry.value);
        Some(entry.value.clone())
    }

    /// Save current paths to cache file
    ///
    /// This should be called after successfully detecting paths.
    fn save_paths_to_file(&self) {
        let mut cache_data = EnvironmentCacheFile {
            version: CACHE_FILE_VERSION,
            cache: HashMap::new(),
        };

        // Load existing cache to preserve other entries
        if let Some(existing) = self.load_cache_from_file() {
            cache_data = existing;
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Add npm_prefix if available
        {
            let guard = self.npm_prefix.read().unwrap();
            if let Some(Some(value)) = guard.as_ref() {
                cache_data.cache.insert(
                    "npm_prefix".to_string(),
                    CacheEntry {
                        value: value.clone(),
                        cached_at: now.clone(),
                        ttl_seconds: TTL_STABLE_PATHS,
                    },
                );
            }
        }

        // Add openclaw_path if available
        {
            let guard = self.openclaw_path.read().unwrap();
            if let Some(Some(value)) = guard.as_ref() {
                cache_data.cache.insert(
                    "openclaw_path".to_string(),
                    CacheEntry {
                        value: value.clone(),
                        cached_at: now.clone(),
                        ttl_seconds: TTL_OPENCLAW_PATH,
                    },
                );
            }
        }

        self.save_cache_to_file(&cache_data);
    }
}

/// Get possible OpenClaw installation paths on Unix systems
fn get_unix_openclaw_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // npm global installation paths
    paths.push("/usr/local/bin/openclaw".to_string());
    paths.push("/opt/homebrew/bin/openclaw".to_string()); // Homebrew on Apple Silicon
    paths.push("/usr/bin/openclaw".to_string());

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // npm global installation to user directory
        paths.push(format!("{}/.npm-global/bin/openclaw", home_str));

        // nvm versions
        for version in [
            "v22.0.0", "v22.1.0", "v22.2.0", "v22.11.0", "v22.12.0", "v23.0.0",
        ] {
            paths.push(format!(
                "{}/.nvm/versions/node/{}/bin/openclaw",
                home_str, version
            ));
        }

        // Check nvm alias default
        let nvm_default = format!("{}/.nvm/alias/default", home_str);
        if let Ok(version) = std::fs::read_to_string(&nvm_default) {
            let version = version.trim();
            if !version.is_empty() {
                paths.insert(
                    0,
                    format!("{}/.nvm/versions/node/v{}/bin/openclaw", home_str, version),
                );
            }
        }

        // fnm
        paths.push(format!("{}/.fnm/aliases/default/bin/openclaw", home_str));

        // volta
        paths.push(format!("{}/.volta/bin/openclaw", home_str));

        // pnpm
        paths.push(format!("{}/.pnpm/bin/openclaw", home_str));
        paths.push(format!("{}/Library/pnpm/openclaw", home_str));

        // asdf
        paths.push(format!("{}/.asdf/shims/openclaw", home_str));

        // mise
        paths.push(format!("{}/.local/share/mise/shims/openclaw", home_str));

        // yarn
        paths.push(format!("{}/.yarn/bin/openclaw", home_str));
        paths.push(format!(
            "{}/.config/yarn/global/node_modules/.bin/openclaw",
            home_str
        ));
    }

    paths
}

/// Get possible OpenClaw installation paths on Windows
fn get_windows_openclaw_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // nvm4w
    paths.push("C:\\nvm4w\\nodejs\\openclaw.cmd".to_string());

    // npm global in user directory
    if let Some(home) = dirs::home_dir() {
        let npm_path = format!("{}\\AppData\\Roaming\\npm\\openclaw.cmd", home.display());
        paths.push(npm_path);
    }

    // Program Files
    paths.push("C:\\Program Files\\nodejs\\openclaw.cmd".to_string());

    paths
}

/// Get possible Node.js paths on Unix systems
fn get_unix_node_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // Homebrew
    paths.push("/opt/homebrew/bin/node".to_string());
    paths.push("/usr/local/bin/node".to_string());
    paths.push("/usr/bin/node".to_string());

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // nvm versions
        paths.push(format!("{}/.nvm/versions/node/v22.0.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.1.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.2.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.11.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.12.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v23.0.0/bin/node", home_str));

        // nvm alias default
        let nvm_default = format!("{}/.nvm/alias/default", home_str);
        if let Ok(version) = std::fs::read_to_string(&nvm_default) {
            let version = version.trim();
            if !version.is_empty() {
                paths.insert(
                    0,
                    format!("{}/.nvm/versions/node/v{}/bin/node", home_str, version),
                );
            }
        }

        // fnm
        paths.push(format!("{}/.fnm/aliases/default/bin/node", home_str));

        // volta
        paths.push(format!("{}/.volta/bin/node", home_str));

        // asdf
        paths.push(format!("{}/.asdf/shims/node", home_str));

        // mise
        paths.push(format!("{}/.local/share/mise/shims/node", home_str));
    }

    paths
}

/// Get possible Node.js paths on Windows systems
fn get_windows_node_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // Standard installation
    paths.push("C:\\Program Files\\nodejs\\node.exe".to_string());
    paths.push("C:\\Program Files (x86)\\nodejs\\node.exe".to_string());

    // nvm4w
    paths.push("C:\\nvm4w\\nodejs\\node.exe".to_string());

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // nvm for Windows
        paths.push(format!(
            "{}\\AppData\\Roaming\\nvm\\current\\node.exe",
            home_str
        ));

        // fnm
        paths.push(format!(
            "{}\\AppData\\Roaming\\fnm\\aliases\\default\\node.exe",
            home_str
        ));
        paths.push(format!(
            "{}\\AppData\\Local\\fnm\\aliases\\default\\node.exe",
            home_str
        ));
        paths.push(format!("{}\\.fnm\\aliases\\default\\node.exe", home_str));

        // volta
        paths.push(format!(
            "{}\\AppData\\Local\\Volta\\bin\\node.exe",
            home_str
        ));

        // scoop
        paths.push(format!(
            "{}\\scoop\\apps\\nodejs\\current\\node.exe",
            home_str
        ));
        paths.push(format!(
            "{}\\scoop\\apps\\nodejs-lts\\current\\node.exe",
            home_str
        ));
    }

    // chocolatey
    paths.push("C:\\ProgramData\\chocolatey\\lib\\nodejs\\tools\\node.exe".to_string());

    // From environment variables
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files_x86));
    }

    // nvm-windows symlink
    if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
        paths.insert(0, format!("{}\\node.exe", nvm_symlink));
    }

    paths
}

#[cfg(test)]
mod cache_tests;
