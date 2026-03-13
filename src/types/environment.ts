/**
 * Environment status returned by the backend check_environment command
 * 
 * This is the unified type definition used across the application.
 * Do not duplicate this interface in individual components.
 */
export interface EnvironmentStatus {
  /** Whether Node.js is installed */
  node_installed: boolean;
  /** Node.js version string (e.g., "v22.1.0") */
  node_version: string | null;
  /** Whether Node.js version meets requirement (>=22) */
  node_version_ok: boolean;
  /** Whether Git is installed */
  git_installed: boolean;
  /** Git version string (e.g., "2.43.0") */
  git_version: string | null;
  /** Whether OpenClaw is installed */
  openclaw_installed: boolean;
  /** OpenClaw version string (e.g., "2026.1.29") */
  openclaw_version: string | null;
  /** Whether gateway service is installed */
  gateway_service_installed: boolean;
  /** Whether config directory exists */
  config_dir_exists: boolean;
  /** Whether all requirements are met */
  ready: boolean;
  /** Operating system ("windows", "macos", or "linux") */
  os: string;
  /** Whether OpenClaw version is secure (>= 2026.1.29) */
  is_secure: boolean;
}
