use regex::Regex;
use std::sync::OnceLock;

/// Sanitizes sensitive information from log messages.
///
/// Redacts:
/// - API Keys (OpenAI, Anthropic, Google, generic patterns)
/// - Bearer tokens
/// - Private keys
/// - Generic secrets/tokens
/// - Sensitive URL parameters
pub fn sanitize(message: &str) -> String {
    let mut sanitized = message.to_string();

    // Compile regex patterns once
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            // OpenAI API Keys (sk-...)
            (Regex::new(r"sk-[a-zA-Z0-9]{32,}").unwrap(), "sk-***[REDACTED]***"),
            // Anthropic API Keys (sk-ant-...)
            (Regex::new(r"sk-ant-[a-zA-Z0-9\-_]{20,}").unwrap(), "sk-ant-***[REDACTED]***"),
            // Google API Keys (AIza...)
            (Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap(), "AIza***[REDACTED]***"),
            // Hugging Face Tokens (hf_...)
            (Regex::new(r"hf_[a-zA-Z0-9]{30,}").unwrap(), "hf_***[REDACTED]***"),
            // Bearer Tokens
            (Regex::new(r"(Bearer\s+)[a-zA-Z0-9\-_=\.]{20,}").unwrap(), "$1***[REDACTED]***"),
            // Telegram Bot Tokens (123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11)
            (Regex::new(r"[0-9]{8,10}:[a-zA-Z0-9_-]{35}").unwrap(), "***[TELEGRAM TOKEN REDACTED]***"),
            // Slack Tokens (xoxb-...)
            (Regex::new(r"xox[baprs]-[0-9a-zA-Z]{10,48}").unwrap(), "xox-***[REDACTED]***"),
            // Private Keys (BEGIN PRIVATE KEY...)
            (Regex::new(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----").unwrap(), "[PRIVATE KEY BLOCK REDACTED]"),
            // Generic "key", "token", "secret" assignments in JSON or query params
            // Matches: "api_key": "..." or apiKey=...
            (Regex::new(r#"(?i)(api_?key|access_?token|secret|password|private_?key)["']?\s*[:=]\s*["']?([a-zA-Z0-9\-_=]+)["']?"#).unwrap(), "$1=***[REDACTED]***"),
             // Sensitive URL parameters
            (Regex::new(r"(?i)(key|token|sig|signature)=([a-zA-Z0-9\-_%]+)").unwrap(), "$1=***[REDACTED]***"),
        ]
    });

    for (regex, replacement) in patterns {
        sanitized = regex.replace_all(&sanitized, *replacement).to_string();
    }

    sanitized
}
