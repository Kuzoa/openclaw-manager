use std::fs;
use tempfile::TempDir;

/// Test file utilities
mod file_utils {
    use super::*;

    #[test]
    fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json").to_string_lossy().to_string();

        openclaw_manager::utils::file::write_file(&file_path, r#"{"key": "value"}"#).unwrap();

        let content = openclaw_manager::utils::file::read_file(&file_path).unwrap();
        assert_eq!(content, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.txt").to_string_lossy().to_string();
        let non_existing_file = temp_dir.path().join("not_exists.txt").to_string_lossy().to_string();

        openclaw_manager::utils::file::write_file(&existing_file, "content").unwrap();

        assert!(openclaw_manager::utils::file::file_exists(&existing_file));
        assert!(!openclaw_manager::utils::file::file_exists(&non_existing_file));
    }

    #[test]
    fn test_write_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path()
            .join("nested")
            .join("dir")
            .join("file.txt")
            .to_string_lossy()
            .to_string();

        openclaw_manager::utils::file::write_file(&nested_path, "content").unwrap();

        assert!(openclaw_manager::utils::file::file_exists(&nested_path));
    }

    #[test]
    fn test_append_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("append.txt").to_string_lossy().to_string();

        openclaw_manager::utils::file::write_file(&file_path, "line1").unwrap();
        openclaw_manager::utils::file::append_file(&file_path, "line2").unwrap();

        let content = openclaw_manager::utils::file::read_file(&file_path).unwrap();
        assert!(content.contains("line1"));
        assert!(content.contains("line2"));
    }
}

/// Test env file utilities
mod env_utils {
    use super::*;

    #[test]
    fn test_set_and_read_env_value() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env").to_string_lossy().to_string();

        openclaw_manager::utils::file::set_env_value(&env_file, "API_KEY", "test123").unwrap();

        let value = openclaw_manager::utils::file::read_env_value(&env_file, "API_KEY");
        assert_eq!(value, Some("test123".to_string()));
    }

    #[test]
    fn test_update_env_value() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env").to_string_lossy().to_string();

        openclaw_manager::utils::file::set_env_value(&env_file, "KEY", "value1").unwrap();
        openclaw_manager::utils::file::set_env_value(&env_file, "KEY", "value2").unwrap();

        let value = openclaw_manager::utils::file::read_env_value(&env_file, "KEY");
        assert_eq!(value, Some("value2".to_string()));
    }

    #[test]
    fn test_remove_env_value() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env").to_string_lossy().to_string();

        openclaw_manager::utils::file::set_env_value(&env_file, "KEY1", "value1").unwrap();
        openclaw_manager::utils::file::set_env_value(&env_file, "KEY2", "value2").unwrap();
        openclaw_manager::utils::file::remove_env_value(&env_file, "KEY1").unwrap();

        assert!(openclaw_manager::utils::file::read_env_value(&env_file, "KEY1").is_none());
        assert_eq!(
            openclaw_manager::utils::file::read_env_value(&env_file, "KEY2"),
            Some("value2".to_string())
        );
    }
}

/// Test log sanitizer
mod log_sanitizer_tests {
    #[test]
    fn test_sanitize_bearer_token() {
        let input = r#"Authorization: Bearer sk-test123456789"#;
        let sanitized = openclaw_manager::utils::log_sanitizer::sanitize(input);
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("sk-test123456789"));
    }

    #[test]
    fn test_sanitize_api_key() {
        let input = r#"api_key=sk-abcdefghijklmnop123456789"#;
        let sanitized = openclaw_manager::utils::log_sanitizer::sanitize(input);
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_no_redaction_for_safe_content() {
        let input = r#"{"message": "Hello, World!"}"#;
        let sanitized = openclaw_manager::utils::log_sanitizer::sanitize(input);
        assert_eq!(input, sanitized);
    }
}
