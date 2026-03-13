#[cfg(test)]
mod tests {
    use super::super::log_sanitizer::sanitize;

    #[test]
    fn test_redact_openai_key() {
        let log = "Using API key sk-1234567890abcdef1234567890abcdef1234567890abcdef for request";
        let sanitized = sanitize(log);
        assert!(sanitized.contains("sk-***[REDACTED]***"));
        assert!(!sanitized.contains("1234567890abcdef"));
    }

    #[test]
    fn test_redact_bearer_token() {
        let log = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ";
        let sanitized = sanitize(log);
        assert!(sanitized.contains("Bearer ***[REDACTED]***"));
        assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiIs"));
    }

    #[test]
    fn test_redact_json_field() {
        let log = r#"{"api_key": "secret_value_123", "user": "test"}"#;
        let sanitized = sanitize(log);
        assert!(sanitized.contains("api_key=***[REDACTED]***"));
        assert!(!sanitized.contains("secret_value_123"));
    }

    #[test]
    fn test_redact_url_params() {
        let log = "GET /api/v1/resource?token=sensitive_token_value&id=123";
        let sanitized = sanitize(log);
        assert!(sanitized.contains("token=***[REDACTED]***"));
        assert!(sanitized.contains("id=123"));
        assert!(!sanitized.contains("sensitive_token_value"));
    }

    #[test]
    fn test_no_false_positives() {
        let log = "User clicked button with id=key_button";
        let sanitized = sanitize(log);
        assert_eq!(log, sanitized);
    }
}
