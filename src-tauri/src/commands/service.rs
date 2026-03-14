use crate::models::ServiceStatus;
use crate::utils::shell;
use log::{debug, error, info, warn};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tauri::command;

// Track if service stop was intentional (manual stop) vs unexpected (crash/restart command)
static INTENTIONAL_STOP: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows CREATE_NO_WINDOW flag to hide console window
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const SERVICE_PORT: u16 = 18789;

/// Check gateway health via direct HTTP request (fast, no Node.js process spawn)
/// Returns true if gateway is healthy and responding
fn check_gateway_health_http(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/health", port);
    match ureq::get(&url).timeout(Duration::from_secs(1)).call() {
        Ok(response) => {
            debug!(
                "[Service] Health check HTTP response: {}",
                response.status()
            );
            response.status() == 200
        }
        Err(e) => {
            debug!("[Service] Health check HTTP failed: {}", e);
            false
        }
    }
}

/// Check if a service is listening on the port, return PID
/// Simple and direct: port in use = service running
fn check_port_listening(port: u16) -> Option<u32> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .ok()?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .and_then(|line| line.trim().parse::<u32>().ok())
        } else {
            None
        }
    }

    #[cfg(windows)]
    {
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    if let Some(pid_str) = line.split_whitespace().last() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            return Some(pid);
                        }
                    }
                }
            }
        }
        None
    }
}

/// Find ALL PIDs using a given port (not just the first one)
fn find_all_port_pids(port: u16) -> Vec<u32> {
    let mut pids = Vec::new();

    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
        {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if let Ok(pid) = line.trim().parse::<u32>() {
                        if pid > 0 && !pids.contains(&pid) {
                            pids.push(pid);
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        let mut cmd = Command::new("netstat");
        cmd.args(["-ano"]);
        cmd.creation_flags(CREATE_NO_WINDOW);

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains(&format!(":{}", port)) {
                        if let Some(pid_str) = line.split_whitespace().last() {
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                if pid > 0 && !pids.contains(&pid) {
                                    pids.push(pid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pids
}

/// Get service status
/// Uses HTTP health check to verify the gateway is actually responding,
/// not just that the port is busy (which could be svchost.exe or another process).
///
/// Optimization: Check port first, skip health check if port is not listening.
/// Uses direct HTTP request instead of spawning Node.js process (much faster).
#[command]
pub async fn get_service_status() -> Result<ServiceStatus, String> {
    use std::time::Instant;
    let total_start = Instant::now();

    // Fast check: Is port listening? (usually < 50ms)
    let port_start = Instant::now();
    let pid = check_port_listening(SERVICE_PORT);
    let port_duration = port_start.elapsed();
    let total_duration = total_start.elapsed();

    // If port is not listening, gateway is definitely not running
    // Skip the health check entirely
    let pid_val = match pid {
        Some(p) => p,
        None => {
            // Log when port is not listening (for debugging)
            info!(
                "[Service] get_service_status: port={}ms, health=skipped, total={}ms, running=false (port not listening)",
                port_duration.as_millis(),
                total_duration.as_millis()
            );
            return Ok(ServiceStatus {
                running: false,
                pid: None,
                port: SERVICE_PORT,
                uptime_seconds: None,
                memory_mb: None,
                cpu_percent: None,
            });
        }
    };

    // Port is listening, now verify it's actually openclaw gateway via HTTP
    // Direct HTTP request is much faster than spawning Node.js process
    let health_start = Instant::now();
    let health_ok = check_gateway_health_http(SERVICE_PORT);
    let health_duration = health_start.elapsed();
    let total_duration = total_start.elapsed();

    // Log for monitoring (always log since we did health check)
    info!(
        "[Service] get_service_status: port={}ms, health={}ms (http), total={}ms, running={}",
        port_duration.as_millis(),
        health_duration.as_millis(),
        total_duration.as_millis(),
        health_ok
    );

    Ok(ServiceStatus {
        running: health_ok,
        pid: if health_ok { Some(pid_val) } else { None },
        port: SERVICE_PORT,
        uptime_seconds: None,
        memory_mb: None,
        cpu_percent: None,
    })
}

/// Start service
#[command]
pub async fn start_service() -> Result<String, String> {
    info!("[Service] Starting service...");

    // Check if already running via health check
    let health_ok = shell::run_openclaw(&["gateway", "health", "--timeout", "2000"]).is_ok();
    if health_ok {
        info!("[Service] Service is already running (health check passed)");
        return Err("Service is already running".to_string());
    }

    // Check if openclaw command exists
    let openclaw_path = shell::get_openclaw_path();
    if openclaw_path.is_none() {
        info!("[Service] openclaw command not found");
        return Err(
            "openclaw command not found, please install it via npm install -g openclaw".to_string(),
        );
    }
    info!("[Service] openclaw path: {:?}", openclaw_path);

    // Clear any processes squatting on the port (e.g. svchost.exe)
    let squatter_pids = find_all_port_pids(SERVICE_PORT);
    if !squatter_pids.is_empty() {
        info!(
            "[Service] Found {} process(es) on port {}, killing...",
            squatter_pids.len(),
            SERVICE_PORT
        );
        for pid in &squatter_pids {
            #[cfg(windows)]
            {
                let mut cmd = Command::new("taskkill");
                cmd.args(["/F", "/PID", &pid.to_string()]);
                cmd.creation_flags(CREATE_NO_WINDOW);
                let _ = cmd.output();
            }
            #[cfg(unix)]
            {
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
        }
        // Wait for port to free up
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    // Start gateway in background
    info!("[Service] Starting gateway in background...");
    shell::spawn_openclaw_gateway().map_err(|e| format!("Failed to start service: {}", e))?;

    // Phase 1: Wait for port to become active (fast check, 1s intervals, max 15s)
    info!(
        "[Service] Waiting for port {} to start listening...",
        SERVICE_PORT
    );
    let mut port_up = false;
    for i in 1..=15 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if check_port_listening(SERVICE_PORT).is_some() {
            info!("[Service] Port {} is now active ({}s)", SERVICE_PORT, i);
            port_up = true;
            break;
        }
    }
    if !port_up {
        return Err("Service start timeout: port not listening after 15s".to_string());
    }

    // Phase 2: Verify gateway is healthy (one attempt with generous timeout)
    info!("[Service] Verifying gateway health...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let health_ok = shell::run_openclaw(&["gateway", "health", "--timeout", "5000"]).is_ok();
    let pid = check_port_listening(SERVICE_PORT);

    if health_ok {
        info!("[Service] Gateway is healthy!");
    } else {
        warn!("[Service] Gateway health check failed, port is active but gateway may still be initializing");
    }

    // Reset stop flag
    INTENTIONAL_STOP.store(false, Ordering::Relaxed);

    // Spawn supervisor thread
    thread::spawn(|| {
        info!("[Service Supervisor] Thread started");
        loop {
            thread::sleep(Duration::from_secs(10));

            // If stop was intentional, exit supervisor
            if INTENTIONAL_STOP.load(Ordering::Relaxed) {
                info!("[Service Supervisor] Intentional stop detected, exiting thread");
                break;
            }

            // Check if service is running via health check
            if shell::run_openclaw(&["gateway", "health", "--timeout", "3000"]).is_err() {
                warn!("[Service Supervisor] Gateway health check failed! Restarting...");

                // Double check flag just in case
                if INTENTIONAL_STOP.load(Ordering::Relaxed) {
                    break;
                }

                if let Err(e) = shell::spawn_openclaw_gateway() {
                    error!("[Service Supervisor] Failed to restart service: {}", e);
                } else {
                    info!("[Service Supervisor] Restart command sent");
                    // Wait for it to come up so we don't spam restarts
                    thread::sleep(Duration::from_secs(15));
                }
            }
        }
    });

    if let Some(pid) = check_port_listening(SERVICE_PORT) {
        Ok(format!("Service started, PID: {}", pid))
    } else {
        Ok("Service started (pid unknown)".to_string())
    }
}

/// Stop service
/// Stop service
#[command]
pub async fn stop_service() -> Result<String, String> {
    info!("[Service] Stopping service...");

    // Set flag so supervisor knows this is intentional
    INTENTIONAL_STOP.store(true, Ordering::Relaxed);

    // 1. Try graceful stop
    let _ = shell::run_openclaw(&["gateway", "stop"]);

    // Wait a bit
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let status = get_service_status().await?;
        if !status.running {
            info!("[Service] Successfully stopped (graceful)");
            return Ok("Service stopped".to_string());
        }
    }

    // 2. Try force stop via CLI
    info!("[Service] Graceful stop failed, trying CLI force stop...");
    let _ = shell::run_openclaw(&["gateway", "stop", "--force"]);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let status = get_service_status().await?;
    if !status.running {
        info!("[Service] Successfully stopped (CLI force)");
        return Ok("Service stopped".to_string());
    }

    // 3. Last resort: Kill process by PID
    if let Some(pid) = status.pid {
        info!("[Service] CLI force stop failed, killing PID {}...", pid);

        #[cfg(windows)]
        {
            let mut cmd = Command::new("taskkill");
            cmd.args(["/F", "/PID", &pid.to_string()]);
            cmd.creation_flags(CREATE_NO_WINDOW);
            if let Ok(output) = cmd.output() {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("[Service] Failed to taskkill PID {}: {}", pid, stderr);
                }
            }
        }

        #[cfg(unix)]
        {
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
        }

        std::thread::sleep(std::time::Duration::from_millis(1000));

        let final_status = get_service_status().await?;
        if !final_status.running {
            info!("[Service] Successfully killed process");
            return Ok("Service stopped (killed)".to_string());
        }
    }

    Err("Failed to stop service after all attempts".to_string())
}

/// Restart service
#[command]
pub async fn restart_service() -> Result<String, String> {
    info!("[Service] Restarting service...");

    // Step 1: Stop the service if it's running
    match stop_service().await {
        Ok(_) => {
            info!("[Service] Service stopped successfully");
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }
        Err(e) => {
            info!(
                "[Service] Failed to stop service: {}, trying to continue anyway...",
                e
            );
        }
    }

    // Step 2: Clear any remaining processes on the port
    let squatter_pids = find_all_port_pids(SERVICE_PORT);
    if !squatter_pids.is_empty() {
        info!(
            "[Service] Clearing {} process(es) still on port {}...",
            squatter_pids.len(),
            SERVICE_PORT
        );
        for pid in &squatter_pids {
            #[cfg(windows)]
            {
                let mut cmd = Command::new("taskkill");
                cmd.args(["/F", "/PID", &pid.to_string()]);
                cmd.creation_flags(CREATE_NO_WINDOW);
                let _ = cmd.output();
            }
            #[cfg(unix)]
            {
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    // Step 3: Start the gateway
    info!("[Service] Starting gateway in background...");
    shell::spawn_openclaw_gateway().map_err(|e| format!("Failed to start service: {}", e))?;

    // Step 4: Wait for port to become active (max 15s)
    info!(
        "[Service] Waiting for port {} to start listening...",
        SERVICE_PORT
    );
    for i in 1..=15 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if check_port_listening(SERVICE_PORT).is_some() {
            info!("[Service] Port {} is now active ({}s)", SERVICE_PORT, i);
            // Give gateway a moment to fully initialize
            std::thread::sleep(std::time::Duration::from_secs(2));
            if let Some(pid) = check_port_listening(SERVICE_PORT) {
                info!("[Service] Successfully restarted, PID: {}", pid);
                return Ok(format!("Service restarted, PID: {}", pid));
            }
            return Ok("Service restarted".to_string());
        }
    }

    info!("[Service] Restart timeout, port still not listening");
    Err("Service restart timeout (15s), please check openclaw logs".to_string())
}

/// Get logs
#[command]
pub async fn get_logs(lines: Option<u32>) -> Result<Vec<String>, String> {
    let n = lines.unwrap_or(100);

    match shell::run_openclaw(&["logs", "--limit", &n.to_string()]) {
        Ok(output) => Ok(output.lines().map(|s| s.to_string()).collect()),
        Err(e) => Err(format!("Failed to read logs: {}", e)),
    }
}

/// Kill ALL processes using port 18789
#[command]
pub async fn kill_all_port_processes() -> Result<String, String> {
    info!(
        "[Service] Kill All: Finding all processes on port {}...",
        SERVICE_PORT
    );

    let pids = find_all_port_pids(SERVICE_PORT);

    if pids.is_empty() {
        info!(
            "[Service] Kill All: No processes found on port {}",
            SERVICE_PORT
        );
        return Ok("No processes found on port 18789".to_string());
    }

    info!(
        "[Service] Kill All: Found {} process(es): {:?}",
        pids.len(),
        pids
    );

    let mut killed = 0u32;
    let mut failed = 0u32;

    for pid in &pids {
        info!("[Service] Kill All: Killing PID {}...", pid);

        #[cfg(windows)]
        {
            let mut cmd = Command::new("taskkill");
            cmd.args(["/F", "/PID", &pid.to_string()]);
            cmd.creation_flags(CREATE_NO_WINDOW);

            match cmd.output() {
                Ok(output) if output.status.success() => {
                    info!("[Service] Kill All: Successfully killed PID {}", pid);
                    killed += 1;
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!(
                        "[Service] Kill All: Failed to kill PID {}: {}",
                        pid,
                        stderr.trim()
                    );
                    failed += 1;
                }
                Err(e) => {
                    warn!("[Service] Kill All: Error killing PID {}: {}", pid, e);
                    failed += 1;
                }
            }
        }

        #[cfg(unix)]
        {
            match Command::new("kill").args(["-9", &pid.to_string()]).output() {
                Ok(output) if output.status.success() => {
                    info!("[Service] Kill All: Successfully killed PID {}", pid);
                    killed += 1;
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!(
                        "[Service] Kill All: Failed to kill PID {}: {}",
                        pid,
                        stderr.trim()
                    );
                    failed += 1;
                }
                Err(e) => {
                    warn!("[Service] Kill All: Error killing PID {}: {}", pid, e);
                    failed += 1;
                }
            }
        }
    }

    let msg = if failed == 0 {
        format!("Killed {} process(es) on port 18789", killed)
    } else {
        format!(
            "Killed {}, failed to kill {} process(es) on port 18789",
            killed, failed
        )
    };

    info!("[Service] Kill All: {}", msg);
    Ok(msg)
}
