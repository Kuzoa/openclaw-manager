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
            version: Some("1.0.0".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"running\":true"));
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }

    #[test]
    fn test_service_status_deserialization() {
        let json = r#"{"running":false,"pid":null,"version":null}"#;
        let status: ServiceStatus = serde_json::from_str(json).unwrap();

        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.version.is_none());
    }

    #[test]
    fn test_service_status_with_pid() {
        let json = r#"{"running":true,"pid":5678,"version":"2.0.0"}"#;
        let status: ServiceStatus = serde_json::from_str(json).unwrap();

        assert!(status.running);
        assert_eq!(status.pid, Some(5678));
        assert_eq!(status.version, Some("2.0.0".to_string()));
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
