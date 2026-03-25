use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenClaw complete configuration - corresponds to openclaw.json structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClawConfig {
    /// Agent configuration
    #[serde(default)]
    pub agents: AgentsConfig,
    /// Model configuration
    #[serde(default)]
    pub models: ModelsConfig,
    /// Gateway configuration
    #[serde(default)]
    pub gateway: GatewayConfig,
    /// Channel configuration
    #[serde(default)]
    pub channels: HashMap<String, serde_json::Value>,
    /// Plugin configuration
    #[serde(default)]
    pub plugins: PluginsConfig,
    /// MCP configuration
    #[serde(default)]
    pub mcp: HashMap<String, MCPConfig>,
    /// Metadata
    #[serde(default)]
    pub meta: MetaConfig,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    /// Default configuration
    #[serde(default)]
    pub defaults: AgentDefaults,
}

/// Agent default configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentDefaults {
    /// Model configuration
    #[serde(default)]
    pub model: AgentModelConfig,
    /// Available model list (provider/model -> {})
    #[serde(default)]
    pub models: HashMap<String, serde_json::Value>,
    /// Compression configuration
    #[serde(default)]
    pub compaction: Option<serde_json::Value>,
    /// Context pruning
    #[serde(rename = "contextPruning", default)]
    pub context_pruning: Option<serde_json::Value>,
    /// Heartbeat configuration
    #[serde(default)]
    pub heartbeat: Option<serde_json::Value>,
    /// Maximum concurrency
    #[serde(rename = "maxConcurrent", default)]
    pub max_concurrent: Option<u32>,
    /// Sub-agent configuration
    #[serde(default)]
    pub subagents: Option<serde_json::Value>,
}

/// Agent model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentModelConfig {
    /// Primary model (format: provider/model-id)
    #[serde(default)]
    pub primary: Option<String>,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    /// Provider configuration mapping
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API URL
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    /// API Key
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    /// Model list
    #[serde(default)]
    pub models: Vec<ModelConfig>,
}

/// Model configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// API type (anthropic-messages / openai-completions)
    #[serde(default)]
    pub api: Option<String>,
    /// Supported input types
    #[serde(default)]
    pub input: Vec<String>,
    /// Context window size
    #[serde(rename = "contextWindow", default)]
    pub context_window: Option<u32>,
    /// Maximum output tokens
    #[serde(rename = "maxTokens", default)]
    pub max_tokens: Option<u32>,
    /// Whether reasoning mode is supported
    #[serde(default)]
    pub reasoning: Option<bool>,
    /// Cost configuration
    #[serde(default)]
    pub cost: Option<ModelCostConfig>,
}

/// Model cost configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCostConfig {
    #[serde(default)]
    pub input: f64,
    #[serde(default)]
    pub output: f64,
    #[serde(rename = "cacheRead", default)]
    pub cache_read: f64,
    #[serde(rename = "cacheWrite", default)]
    pub cache_write: f64,
}

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    /// Mode: local or cloud
    #[serde(default)]
    pub mode: Option<String>,
    /// Authentication configuration
    #[serde(default)]
    pub auth: Option<GatewayAuthConfig>,
}

/// Gateway authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayAuthConfig {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub entries: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub installs: HashMap<String, serde_json::Value>,
}

/// MCP configuration (supports both stdio and HTTP modes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// Command to run (for stdio servers)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    /// Arguments (for stdio servers)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// URL (for HTTP/remote MCP servers)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// Whether enabled
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,
}

fn default_mcp_enabled() -> bool {
    true
}

/// Metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaConfig {
    #[serde(rename = "lastTouchedAt", default)]
    pub last_touched_at: Option<String>,
    #[serde(rename = "lastTouchedVersion", default)]
    pub last_touched_version: Option<String>,
}

// ============ Data structures for frontend display ============

/// Official Provider preset (for frontend display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialProvider {
    /// Provider ID (used in configuration)
    pub id: String,
    /// Display name
    pub name: String,
    /// Icon (emoji)
    pub icon: String,
    /// Official API URL
    pub default_base_url: Option<String>,
    /// API type
    pub api_type: String,
    /// Recommended model list
    pub suggested_models: Vec<SuggestedModel>,
    /// Whether API Key is required
    pub requires_api_key: bool,
    /// Default API Key
    pub default_api_key: Option<String>,
    /// Documentation URL
    pub docs_url: Option<String>,
}

/// Recommended model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedModel {
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Context window
    pub context_window: Option<u32>,
    /// Maximum output
    pub max_tokens: Option<u32>,
    /// Whether recommended
    pub recommended: bool,
}

/// Configured Provider (read from configuration file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredProvider {
    /// Provider name (key in configuration)
    pub name: String,
    /// API URL
    pub base_url: String,
    /// API Key (masked for display)
    pub api_key_masked: Option<String>,
    /// Whether API Key exists
    pub has_api_key: bool,
    /// Configured model list
    pub models: Vec<ConfiguredModel>,
}

/// Configured model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguredModel {
    /// Full model ID (provider/model-id)
    pub full_id: String,
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// API type
    pub api_type: Option<String>,
    /// Context window
    pub context_window: Option<u32>,
    /// Maximum output
    pub max_tokens: Option<u32>,
    /// Whether it is the primary model
    pub is_primary: bool,
}

/// AI configuration overview (returned to frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigOverview {
    /// Primary model
    pub primary_model: Option<String>,
    /// Configured provider list
    pub configured_providers: Vec<ConfiguredProvider>,
    /// Available model list
    pub available_models: Vec<String>,
}

// ============ Legacy data structures for compatibility ============

/// AI Provider option (for frontend display) - legacy compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderOption {
    /// Provider ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Icon (emoji)
    pub icon: String,
    /// Official API URL
    pub default_base_url: Option<String>,
    /// Recommended model list
    pub models: Vec<AIModelOption>,
    /// Whether API Key is required
    pub requires_api_key: bool,
}

/// AI model option - legacy compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelOption {
    /// Model ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Whether recommended
    pub recommended: bool,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Channel ID
    pub id: String,
    /// Channel type
    pub channel_type: String,
    /// Whether enabled
    pub enabled: bool,
    /// Configuration details
    pub config: HashMap<String, serde_json::Value>,
}

/// Environment variable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub key: String,
    pub value: String,
}

// ============ AllSettings - Unified settings for Settings page ============

/// Browser configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    #[serde(default = "default_browser_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub color: Option<String>,
}

fn default_browser_enabled() -> bool {
    true
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            enabled: default_browser_enabled(),
            color: None,
        }
    }
}

/// Web search configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebSettings {
    #[serde(default)]
    pub brave_api_key: Option<String>,
}

/// Compaction configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactionSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub threshold: Option<u32>,
    #[serde(default)]
    pub context_pruning: bool,
    #[serde(default)]
    pub max_context_messages: Option<u32>,
}

/// Workspace configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default = "default_timezone")]
    pub timezone: Option<String>,
    #[serde(default)]
    pub time_format: Option<String>,
    #[serde(default)]
    pub skip_bootstrap: bool,
    #[serde(default)]
    pub bootstrap_max_chars: Option<u32>,
}

fn default_timezone() -> Option<String> {
    Some("Asia/Shanghai".to_string())
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            workspace: None,
            timezone: default_timezone(),
            time_format: None,
            skip_bootstrap: false,
            bootstrap_max_chars: None,
        }
    }
}

/// Gateway configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySettings {
    #[serde(default = "default_gateway_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_gateway_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for GatewaySettings {
    fn default() -> Self {
        Self {
            port: default_gateway_port(),
            log_level: default_log_level(),
        }
    }
}

/// Subagent defaults for Settings page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubagentDefaultsSettings {
    #[serde(default)]
    pub max_spawn_depth: Option<u32>,
    #[serde(default)]
    pub max_children_per_agent: Option<u32>,
    #[serde(default)]
    pub max_concurrent: Option<u32>,
    #[serde(default)]
    pub attachments_enabled: Option<bool>,
    #[serde(default)]
    pub attachments_max_total_bytes: Option<u64>,
}

/// PDF configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfSettings {
    #[serde(default)]
    pub max_pages: Option<u64>,
    #[serde(default)]
    pub max_bytes_mb: Option<f64>,
}

/// Memory configuration for Settings page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySettings {
    #[serde(default)]
    pub provider: Option<String>,
}

/// Unified settings for Settings page - combines all configuration sections
/// This struct is used for atomic read/write operations to avoid race conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllSettings {
    pub browser: BrowserSettings,
    pub web: WebSettings,
    pub compaction: CompactionSettings,
    pub workspace: WorkspaceSettings,
    pub gateway: GatewaySettings,
    pub subagent_defaults: SubagentDefaultsSettings,
    #[serde(default = "default_tools_profile")]
    pub tools_profile: String,
    pub pdf: PdfSettings,
    pub memory: MemorySettings,
    #[serde(default)]
    pub language: Option<String>,
}

fn default_tools_profile() -> String {
    "messaging".to_string()
}

impl Default for AllSettings {
    fn default() -> Self {
        Self {
            browser: BrowserSettings::default(),
            web: WebSettings::default(),
            compaction: CompactionSettings::default(),
            workspace: WorkspaceSettings::default(),
            gateway: GatewaySettings::default(),
            subagent_defaults: SubagentDefaultsSettings::default(),
            tools_profile: default_tools_profile(),
            pdf: PdfSettings::default(),
            memory: MemorySettings::default(),
            language: None,
        }
    }
}

impl From<serde_json::Value> for AllSettings {
    fn from(config: serde_json::Value) -> Self {
        // Browser settings
        let browser = BrowserSettings {
            enabled: config
                .pointer("/meta/gui/browser/enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            color: config
                .pointer("/meta/gui/browser/color")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Web settings
        let web = WebSettings {
            brave_api_key: config
                .pointer("/web/braveApiKey")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Compaction settings
        let compaction_val = config.pointer("/agents/defaults/compaction");
        let pruning_val = config.pointer("/agents/defaults/contextPruning");

        let compaction = CompactionSettings {
            enabled: compaction_val
                .map(|v| v.as_bool().unwrap_or(true))
                .unwrap_or(false),
            threshold: compaction_val
                .and_then(|v| v.get("threshold"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            // contextPruning can be: true/false, or an object like {"maxMessages": 50}
            context_pruning: pruning_val
                .map(|v| {
                    if v.is_boolean() {
                        v.as_bool().unwrap_or(false)
                    } else if v.is_object() {
                        true // Object means enabled with settings
                    } else {
                        false
                    }
                })
                .unwrap_or(false),
            max_context_messages: pruning_val
                .and_then(|v| v.get("maxMessages"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        };

        // Workspace settings
        let workspace = WorkspaceSettings {
            workspace: config
                .pointer("/agents/defaults/workspace")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timezone: config
                .pointer("/manager/timezone")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| Some("Asia/Shanghai".to_string())),
            time_format: config
                .pointer("/manager/time_format")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            skip_bootstrap: config
                .pointer("/agents/defaults/skipBootstrap")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            bootstrap_max_chars: config
                .pointer("/agents/defaults/bootstrapMaxChars")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        };

        // Gateway settings
        let gateway = GatewaySettings {
            port: config
                .pointer("/gateway/port")
                .and_then(|v| v.as_u64())
                .map(|v| v as u16)
                .unwrap_or(3000),
            log_level: config
                .pointer("/manager/log_level")
                .and_then(|v| v.as_str())
                .or_else(|| config.pointer("/gateway/logLevel").and_then(|v| v.as_str()))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "info".to_string()),
        };

        // Subagent defaults
        let subagent_defaults = SubagentDefaultsSettings {
            max_spawn_depth: config
                .pointer("/agents/defaults/subagents/maxSpawnDepth")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            max_children_per_agent: config
                .pointer("/agents/defaults/subagents/maxChildrenPerAgent")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            max_concurrent: config
                .pointer("/agents/defaults/subagents/maxConcurrent")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            attachments_enabled: config
                .pointer("/tools/sessions_spawn/attachments/enabled")
                .and_then(|v| v.as_bool()),
            attachments_max_total_bytes: config
                .pointer("/tools/sessions_spawn/attachments/maxTotalBytes")
                .and_then(|v| v.as_u64()),
        };

        // Tools profile
        let tools_profile = config
            .pointer("/tools/profile")
            .and_then(|v| v.as_str())
            .unwrap_or("messaging")
            .to_string();

        // PDF settings
        let pdf = PdfSettings {
            max_pages: config.get("pdfMaxPages").and_then(|v| v.as_u64()),
            max_bytes_mb: config.get("pdfMaxBytesMb").and_then(|v| v.as_f64()),
        };

        // Memory settings
        let memory = MemorySettings {
            provider: config
                .pointer("/memorySearch/provider")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Language setting
        let language = config
            .pointer("/meta/language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        AllSettings {
            browser,
            web,
            compaction,
            workspace,
            gateway,
            subagent_defaults,
            tools_profile,
            pdf,
            memory,
            language,
        }
    }
}
