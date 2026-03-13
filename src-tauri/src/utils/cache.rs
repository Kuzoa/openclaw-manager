use log::{debug, info, warn};
use std::sync::RwLock;

use super::platform;
use super::shell;

/// Global environment cache instance
/// Uses `once_cell::sync::Lazy` for thread-safe lazy initialization
pub static ENVIRONMENT_CACHE: EnvironmentCache = EnvironmentCache::new();

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
}

impl EnvironmentCache {
    /// Create a new empty cache
    pub const fn new() -> Self {
        Self {
            npm_prefix: RwLock::new(None),
            openclaw_path: RwLock::new(None),
            openclaw_version: RwLock::new(None),
            node_version: RwLock::new(None),
            git_version: RwLock::new(None),
            is_secure: RwLock::new(None),
        }
    }

    /// Get npm global prefix with lazy initialization
    ///
    /// Executes `npm config get prefix` on first call and caches the result.
    /// Uses double-checked locking to prevent race conditions.
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

        // Cache miss - execute npm command
        info!("[Cache] npm_prefix: cache miss, executing npm config get prefix");
        let result = self.fetch_npm_prefix_internal();

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());
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
    pub fn get_openclaw_path(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.openclaw_path.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.openclaw_path.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // Cache miss - detect path
        info!("[Cache] openclaw_path: cache miss, detecting...");
        let result = self.detect_openclaw_path_internal();

        // Write to cache (wrap in Some to mark as initialized)
        *guard = Some(result.clone());
        result
    }

    /// Internal function to detect OpenClaw path
    /// Replicates the logic from shell::get_openclaw_path but uses cached npm_prefix
    fn detect_openclaw_path_internal(&self) -> Option<String> {
        // Phase 1: Use cached npm prefix
        if let Some(prefix) = self.get_npm_prefix() {
            let openclaw_path = if platform::is_windows() {
                format!("{}\\openclaw.cmd", prefix)
            } else {
                format!("{}/bin/openclaw", prefix)
            };

            debug!("[Cache] Checking npm prefix path: {}", openclaw_path);
            if std::path::Path::new(&openclaw_path).exists() {
                info!("[Cache] Found openclaw via npm prefix: {}", openclaw_path);
                return Some(openclaw_path);
            }
        }

        // Phase 2: Check hardcoded paths
        info!("[Cache] Phase 2: Checking hardcoded paths...");

        if platform::is_windows() {
            let possible_paths = get_windows_openclaw_paths();
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    info!("[Cache] Found openclaw at {}", path);
                    return Some(path);
                }
            }
        } else {
            let possible_paths = get_unix_openclaw_paths();
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    info!("[Cache] Found openclaw at {}", path);
                    return Some(path);
                }
            }
        }

        // Phase 3: Check PATH
        if shell::command_exists("openclaw") {
            return Some("openclaw".to_string());
        }

        // Last resort: search via user shell (Unix only)
        if !platform::is_windows() {
            if let Ok(path) = shell::run_bash_output("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; which openclaw 2>/dev/null") {
                if !path.is_empty() && std::path::Path::new(&path).exists() {
                    info!("[Cache] Found openclaw via user shell: {}", path);
                    return Some(path);
                }
            }
        }

        None
    }

    /// Get OpenClaw version with lazy initialization
    ///
    /// First checks if OpenClaw is installed (via get_openclaw_path).
    /// If not installed, returns None immediately without executing any command.
    /// Uses double-checked locking to prevent race conditions.
    pub fn get_openclaw_version(&self) -> Option<String> {
        // First check with read lock (fast path)
        {
            let guard = self.openclaw_version.read().unwrap();
            if guard.is_some() {
                return guard.as_ref().unwrap().clone();
            }
        }

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.openclaw_version.write().unwrap();
        if guard.is_some() {
            return guard.as_ref().unwrap().clone();
        }

        // First check if OpenClaw is installed (uses cached path)
        let openclaw_path = self.get_openclaw_path();
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

        // Acquire write lock and check again (prevents race condition)
        let mut guard = self.is_secure.write().unwrap();
        if guard.is_some() {
            return *guard.as_ref().unwrap();
        }

        // Cache miss - check version
        info!("[Cache] is_secure: cache miss, checking version...");
        let version = self.get_openclaw_version();
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

        info!("[Cache] Cache invalidation complete");
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