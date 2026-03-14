/// Service status tests
/// Note: Process management tests are marked with #[ignore] as they require
/// actual system processes and may not work in all environments.

use openclaw_manager::models::ServiceStatus;

mod service_status {
    use super::*;

    #[test]
    fn test_service_status_serialization() {
        let status = ServiceStatus {
            running: true,
            pid: Some(1234),
            port: 18789,
            uptime_seconds: Some(3600),
            memory_mb: Some(128.5),
            cpu_percent: Some(2.5),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"running\":true"));
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"port\":18789"));
    }

    #[test]
    fn test_service_status_roundtrip() {
        let original = ServiceStatus {
            running: true,
            pid: Some(5678),
            port: 18789,
            uptime_seconds: Some(100),
            memory_mb: Some(64.0),
            cpu_percent: Some(1.5),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ServiceStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.running, original.running);
        assert_eq!(parsed.pid, original.pid);
        assert_eq!(parsed.port, original.port);
        assert_eq!(parsed.uptime_seconds, original.uptime_seconds);
    }

    #[test]
    fn test_service_status_default() {
        let status = ServiceStatus::default();
        
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert_eq!(status.port, 18789);
    }

    #[test]
    fn test_service_status_stopped() {
        let status = ServiceStatus {
            running: false,
            pid: None,
            port: 18789,
            uptime_seconds: None,
            memory_mb: None,
            cpu_percent: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        let parsed: ServiceStatus = serde_json::from_str(&json).unwrap();

        assert!(!parsed.running);
        assert!(parsed.pid.is_none());
    }
}

/// Platform detection tests
mod platform_tests {
    #[test]
    fn test_get_os() {
        let os = openclaw_manager::utils::platform::get_os();
        // Should return a valid OS string
        assert!(["windows", "macos", "linux"].contains(&os.as_str()));
    }

    #[test]
    fn test_get_arch() {
        let arch = openclaw_manager::utils::platform::get_arch();
        // Should return a valid architecture string
        assert!(["x86_64", "aarch64", "arm"].contains(&arch.as_str()));
    }

    #[test]
    fn test_is_windows_or_macos() {
        // At least one of these should be true (but not both)
        let is_win = openclaw_manager::utils::platform::is_windows();
        let is_mac = openclaw_manager::utils::platform::is_macos();

        // They are mutually exclusive
        assert!(!(is_win && is_mac));
    }

    #[test]
    fn test_config_dir_format() {
        let config_dir = openclaw_manager::utils::platform::get_config_dir();
        
        // Should contain .openclaw
        assert!(config_dir.contains(".openclaw"));
    }
}

/// Port checking tests (may not work in all environments)
mod port_tests {
    use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_port_not_in_use() {
        // Try to bind to a random port to get an available port
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        // Now the port should be free
        // This is a basic sanity check that port binding works
        let result = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
        assert!(result.is_ok(), "Port should be available after closing listener");
    }
}