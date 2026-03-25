//! Unit tests for AllSettings and unified settings API
//! 
//! Tests cover:
//! - 7.1 AllSettings serialization/deserialization
//! - 7.2 AllSettings From<Value> conversion
//! - 7.3 Empty file detection logic

#[cfg(test)]
mod tests {
    use crate::models::{
        AllSettings, BrowserSettings, CompactionSettings, GatewaySettings, MemorySettings,
        PdfSettings, SubagentDefaultsSettings, WebSettings, WorkspaceSettings,
    };
    use serde_json::json;

    // ============ 7.1 AllSettings Serialization/Deserialization Tests ============

    #[test]
    fn test_all_settings_serialization() {
        let settings = AllSettings {
            browser: BrowserSettings {
                enabled: true,
                color: Some("#ff0000".to_string()),
            },
            web: WebSettings {
                brave_api_key: Some("test-key".to_string()),
            },
            compaction: CompactionSettings {
                enabled: true,
                threshold: Some(8000),
                context_pruning: true,
                max_context_messages: Some(50),
            },
            workspace: WorkspaceSettings {
                workspace: Some("/home/user/ws".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
                time_format: Some("24h".to_string()),
                skip_bootstrap: true,
                bootstrap_max_chars: Some(10000),
            },
            gateway: GatewaySettings {
                port: 3000,
                log_level: "debug".to_string(),
            },
            subagent_defaults: SubagentDefaultsSettings {
                max_spawn_depth: Some(3),
                max_children_per_agent: Some(5),
                max_concurrent: Some(8),
                attachments_enabled: Some(true),
                attachments_max_total_bytes: Some(5242880),
            },
            tools_profile: "coding".to_string(),
            pdf: PdfSettings {
                max_pages: Some(10),
                max_bytes_mb: Some(5.0),
            },
            memory: MemorySettings {
                provider: Some("ollama".to_string()),
            },
            language: Some("zh".to_string()),
        };

        let json_str = serde_json::to_string(&settings).unwrap();
        assert!(json_str.contains("browser"));
        assert!(json_str.contains("language"));
    }

    #[test]
    fn test_all_settings_deserialization() {
        let json_val = json!({
            "browser": {"enabled": false, "color": "#00ff00"},
            "web": {"brave_api_key": "my-key"},
            "compaction": {"enabled": true, "threshold": 5000, "context_pruning": false, "max_context_messages": 30},
            "workspace": {"workspace": "/tmp", "timezone": "UTC", "time_format": "12h", "skip_bootstrap": false, "bootstrap_max_chars": 5000},
            "gateway": {"port": 8080, "log_level": "warn"},
            "subagent_defaults": {"max_spawn_depth": 2, "max_children_per_agent": 3, "max_concurrent": 4, "attachments_enabled": false, "attachments_max_total_bytes": 1000000},
            "tools_profile": "messaging",
            "pdf": {"max_pages": 20, "max_bytes_mb": 10.5},
            "memory": {"provider": null},
            "language": "en"
        });

        let settings: AllSettings = serde_json::from_value(json_val).unwrap();
        assert!(!settings.browser.enabled);
        assert_eq!(settings.browser.color, Some("#00ff00".to_string()));
        assert_eq!(settings.web.brave_api_key, Some("my-key".to_string()));
        assert!(settings.compaction.enabled);
        assert_eq!(settings.gateway.port, 8080);
        assert_eq!(settings.language, Some("en".to_string()));
    }

    #[test]
    fn test_all_settings_missing_fields_defaults() {
        let json_val = json!({
            "browser": {"enabled": true},
            "web": {},
            "compaction": {"enabled": false, "context_pruning": false},
            "workspace": {},
            "gateway": {"port": 3000, "log_level": "info"},
            "subagent_defaults": {},
            "tools_profile": "messaging",
            "pdf": {},
            "memory": {}
        });

        let settings: AllSettings = serde_json::from_value(json_val).unwrap();
        assert!(settings.browser.enabled);
        assert!(settings.browser.color.is_none());
        assert!(settings.web.brave_api_key.is_none());
        assert!(!settings.compaction.enabled);
        assert!(settings.workspace.timezone.is_some());
        assert!(settings.language.is_none());
    }

    #[test]
    fn test_language_field_option() {
        // With language
        let json_with = json!({
            "browser": {"enabled": true},
            "web": {},
            "compaction": {"enabled": false, "context_pruning": false},
            "workspace": {},
            "gateway": {"port": 3000, "log_level": "info"},
            "subagent_defaults": {},
            "tools_profile": "messaging",
            "pdf": {},
            "memory": {},
            "language": "zh"
        });
        let settings_with: AllSettings = serde_json::from_value(json_with).unwrap();
        assert_eq!(settings_with.language, Some("zh".to_string()));

        // Without language
        let json_without = json!({
            "browser": {"enabled": true},
            "web": {},
            "compaction": {"enabled": false, "context_pruning": false},
            "workspace": {},
            "gateway": {"port": 3000, "log_level": "info"},
            "subagent_defaults": {},
            "tools_profile": "messaging",
            "pdf": {},
            "memory": {}
        });
        let settings_without: AllSettings = serde_json::from_value(json_without).unwrap();
        assert!(settings_without.language.is_none());

        // With null language
        let json_null = json!({
            "browser": {"enabled": true},
            "web": {},
            "compaction": {"enabled": false, "context_pruning": false},
            "workspace": {},
            "gateway": {"port": 3000, "log_level": "info"},
            "subagent_defaults": {},
            "tools_profile": "messaging",
            "pdf": {},
            "memory": {},
            "language": null
        });
        let settings_null: AllSettings = serde_json::from_value(json_null).unwrap();
        assert!(settings_null.language.is_none());
    }

    // ============ 7.2 AllSettings From<Value> Conversion Tests ============

    #[test]
    fn test_from_value_complete() {
        let config = json!({
            "meta": {
                "gui": {
                    "browser": {
                        "enabled": true,
                        "color": "#123456"
                    }
                },
                "language": "en"
            },
            "web": {
                "braveApiKey": "test-key-123"
            },
            "agents": {
                "defaults": {
                    "compaction": {"threshold": 8000},
                    "contextPruning": {"maxMessages": 50},
                    "workspace": "/home/test",
                    "skipBootstrap": true,
                    "bootstrapMaxChars": 10000,
                    "subagents": {
                        "maxSpawnDepth": 3,
                        "maxChildrenPerAgent": 5,
                        "maxConcurrent": 8
                    }
                }
            },
            "manager": {
                "timezone": "America/New_York",
                "time_format": "12h",
                "log_level": "debug"
            },
            "gateway": {"port": 8080},
            "tools": {
                "profile": "coding",
                "sessions_spawn": {
                    "attachments": {
                        "enabled": true,
                        "maxTotalBytes": 10485760
                    }
                }
            },
            "pdfMaxPages": 15,
            "pdfMaxBytesMb": 8.5,
            "memorySearch": {"provider": "ollama"}
        });

        let settings = AllSettings::from(config);

        assert!(settings.browser.enabled);
        assert_eq!(settings.browser.color, Some("#123456".to_string()));
        assert_eq!(settings.web.brave_api_key, Some("test-key-123".to_string()));
        assert!(settings.compaction.enabled);
        assert_eq!(settings.compaction.threshold, Some(8000));
        assert!(settings.compaction.context_pruning);
        assert_eq!(settings.compaction.max_context_messages, Some(50));
        assert_eq!(settings.workspace.workspace, Some("/home/test".to_string()));
        assert_eq!(settings.workspace.timezone, Some("America/New_York".to_string()));
        assert_eq!(settings.workspace.time_format, Some("12h".to_string()));
        assert!(settings.workspace.skip_bootstrap);
        assert_eq!(settings.workspace.bootstrap_max_chars, Some(10000));
        assert_eq!(settings.gateway.port, 8080);
        assert_eq!(settings.gateway.log_level, "debug");
        assert_eq!(settings.subagent_defaults.max_spawn_depth, Some(3));
        assert_eq!(settings.subagent_defaults.max_children_per_agent, Some(5));
        assert_eq!(settings.subagent_defaults.max_concurrent, Some(8));
        assert_eq!(settings.subagent_defaults.attachments_enabled, Some(true));
        assert_eq!(settings.subagent_defaults.attachments_max_total_bytes, Some(10485760));
        assert_eq!(settings.tools_profile, "coding");
        assert_eq!(settings.pdf.max_pages, Some(15));
        assert_eq!(settings.pdf.max_bytes_mb, Some(8.5));
        assert_eq!(settings.memory.provider, Some("ollama".to_string()));
        assert_eq!(settings.language, Some("en".to_string()));
    }

    #[test]
    fn test_from_value_partial_defaults() {
        let config = json!({
            "meta": {"gui": {"browser": {}}},
            "gateway": {"port": 3000}
        });

        let settings = AllSettings::from(config);

        assert!(settings.browser.enabled);
        assert!(settings.browser.color.is_none());
        assert!(settings.web.brave_api_key.is_none());
        assert!(!settings.compaction.enabled);
        assert!(settings.workspace.timezone.is_some());
        assert_eq!(settings.gateway.port, 3000);
        assert_eq!(settings.gateway.log_level, "info");
        assert_eq!(settings.tools_profile, "messaging");
    }

    #[test]
    fn test_from_value_empty_object() {
        let config = json!({});

        let settings = AllSettings::from(config);

        assert!(settings.browser.enabled);
        assert!(settings.browser.color.is_none());
        assert!(!settings.compaction.enabled);
        assert_eq!(settings.gateway.port, 3000);
        assert_eq!(settings.gateway.log_level, "info");
        assert_eq!(settings.tools_profile, "messaging");
        assert!(settings.language.is_none());
    }

    // ============ 7.3 Empty File Detection Tests ============

    #[test]
    fn test_empty_content_detection() {
        let empty_cases = vec!["", "   ", "\t", "\n", "  \n\t  ", "null", "  null  "];
        for content in empty_cases {
            let trimmed = content.trim();
            let is_empty = trimmed.is_empty() || trimmed == "null";
            assert!(is_empty, "Expected '{}' to be detected as empty/null", content);
        }

        let valid_cases = vec!["{}", "{\"key\": \"value\"}", "[]", "true", "false", "123"];
        for content in valid_cases {
            let trimmed = content.trim();
            let is_empty = trimmed.is_empty() || trimmed == "null";
            assert!(!is_empty, "Expected '{}' to NOT be detected as empty/null", content);
        }
    }

    #[test]
    fn test_default_all_settings() {
        let settings = AllSettings::default();

        // Default trait behavior
        assert!(settings.browser.enabled); // Default is true via default_browser_enabled
        assert_eq!(settings.gateway.port, 3000);
        assert_eq!(settings.gateway.log_level, "info");
        assert_eq!(settings.tools_profile, "messaging");
        // Note: workspace.timezone is None in Default trait, but Some("Asia/Shanghai") in serde deserialization
        // This is because Default derive sets Option fields to None, but serde(default) uses custom function

        let json_str = serde_json::to_string(&settings).unwrap();
        assert!(json_str.contains("browser"));
        assert!(json_str.contains("gateway"));
    }
}
