//! Configuration loading and data file path helpers.

use std::path::{Path, PathBuf};
use std::sync::RwLock;

use anyhow::{Context, Result};
use clap::Args;
use config as config_rs;
use config_rs::FileFormat;
use directories::ProjectDirs;
use once_cell::sync::OnceCell;
use serde::Deserialize;

/// Default relative directory where Taskter stores its data files.
pub const DIR: &str = ".taskter";
/// Default relative path for the Kanban board JSON file.
pub const BOARD_FILE: &str = ".taskter/board.json";
/// Default relative path for the OKRs JSON file.
pub const OKRS_FILE: &str = ".taskter/okrs.json";
/// Default relative path for the operation logs file.
pub const LOG_FILE: &str = ".taskter/logs.log";
/// Default relative path for the agents registry file.
pub const AGENTS_FILE: &str = ".taskter/agents.json";
/// Default relative path for the project description file.
pub const DESCRIPTION_FILE: &str = ".taskter/description.md";
/// Default relative path for the email configuration file.
pub const EMAIL_CONFIG_FILE: &str = ".taskter/email_config.json";
/// Default relative path for the running agents file.
pub const RUNNING_AGENTS_FILE: &str = ".taskter/running_agents.json";
/// Default relative path for the API responses debug log.
pub const RESPONSES_LOG_FILE: &str = ".taskter/api_responses.log";

/// Command-line overrides for configuration values. Higher precedence than env/file/defaults.
#[derive(Debug, Default, Clone, Args)]
pub struct ConfigOverrides {
    /// Explicit path to the configuration file.
    #[arg(long)]
    pub config_file: Option<PathBuf>,

    /// Override the data directory used for persistence.
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
    /// Override the board JSON file path.
    #[arg(long)]
    pub board_file: Option<PathBuf>,
    /// Override the OKRs JSON file path.
    #[arg(long)]
    pub okrs_file: Option<PathBuf>,
    /// Override the textual logs file path.
    #[arg(long)]
    pub log_file: Option<PathBuf>,
    /// Override the agents registry file path.
    #[arg(long)]
    pub agents_file: Option<PathBuf>,
    /// Override the project description file path.
    #[arg(long)]
    pub description_file: Option<PathBuf>,
    /// Override the email configuration file path.
    #[arg(long)]
    pub email_config_file: Option<PathBuf>,
    /// Override the running agents file path.
    #[arg(long)]
    pub running_agents_file: Option<PathBuf>,
    /// Override the API responses debug log path.
    #[arg(long)]
    pub responses_log_file: Option<PathBuf>,

    /// Override the OpenAI API key.
    #[arg(long)]
    pub openai_api_key: Option<String>,
    /// Override the OpenAI base URL used for default endpoints.
    #[arg(long)]
    pub openai_base_url: Option<String>,
    /// Override the OpenAI responses endpoint.
    #[arg(long)]
    pub openai_responses_endpoint: Option<String>,
    /// Override the OpenAI chat completions endpoint.
    #[arg(long)]
    pub openai_chat_endpoint: Option<String>,
    /// Override the OpenAI request style (responses/chat).
    #[arg(long)]
    pub openai_request_style: Option<String>,
    /// Override the OpenAI response format (JSON or type name).
    #[arg(long)]
    pub openai_response_format: Option<String>,

    /// Override the Gemini API key.
    #[arg(long)]
    pub gemini_api_key: Option<String>,

    /// Override the Ollama API key.
    #[arg(long)]
    pub ollama_api_key: Option<String>,
    /// Override the Ollama base URL.
    #[arg(long)]
    pub ollama_base_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ConfigState {
    overrides: ConfigOverrides,
    resolved: Option<ResolvedConfig>,
}

static CONFIG_STATE: OnceCell<RwLock<ConfigState>> = OnceCell::new();

fn state() -> &'static RwLock<ConfigState> {
    CONFIG_STATE.get_or_init(|| RwLock::new(ConfigState::default()))
}

fn env_flag(key: &str) -> bool {
    match std::env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            !(trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("0")
                || trimmed.eq_ignore_ascii_case("false")
                || trimmed.eq_ignore_ascii_case("off"))
        }
        Err(_) => false,
    }
}

fn host_config_disabled() -> bool {
    env_flag("TASKTER_DISABLE_HOST_CONFIG")
}

fn ensure_initialized() -> Result<()> {
    let mut guard = state().write().expect("Taskter config lock poisoned");
    if guard.resolved.is_some() {
        return Ok(());
    }
    let cfg = load_config(&guard.overrides)?;
    guard.resolved = Some(cfg);
    Ok(())
}

/// Initialise configuration with the provided CLI overrides.
pub fn init(overrides: &ConfigOverrides) -> Result<()> {
    let mut guard = state().write().expect("Taskter config lock poisoned");
    guard.overrides = overrides.clone();
    guard.resolved = Some(load_config(&guard.overrides)?);
    Ok(())
}

/// Force a reload of the configuration using the last provided overrides.
pub fn force_reload() -> Result<()> {
    let mut guard = state().write().expect("Taskter config lock poisoned");
    let overrides = guard.overrides.clone();
    guard.resolved = Some(load_config(&overrides)?);
    Ok(())
}

fn with_config<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&ResolvedConfig) -> T,
{
    ensure_initialized()?;
    let guard = state().read().expect("Taskter config lock poisoned");
    let cfg = guard
        .resolved
        .as_ref()
        .context("configuration not initialised")?;
    Ok(f(cfg))
}

/// Path to the Taskter data directory.
pub fn dir() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.data_dir.clone())
}

/// Path to the Kanban board JSON file.
pub fn board_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.board.clone())
}

/// Path to the OKRs JSON file.
pub fn okrs_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.okrs.clone())
}

/// Path to the execution logs file.
pub fn log_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.log.clone())
}

/// Path to the agents registry JSON file.
pub fn agents_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.agents.clone())
}

/// Path to the project description Markdown file.
pub fn description_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.description.clone())
}

/// Path to the email configuration JSON file.
pub fn email_config_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.email_config.clone())
}

/// Path tracking currently running agents.
pub fn running_agents_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.running_agents.clone())
}

/// Path to the debug API responses log.
pub fn responses_log_path() -> Result<PathBuf> {
    with_config(|cfg| cfg.paths.responses_log.clone())
}

/// Resolved OpenAI provider settings.
pub fn openai() -> Result<OpenAiResolved> {
    with_config(|cfg| cfg.providers.openai.clone())
}

/// Resolved Gemini provider settings.
pub fn gemini() -> Result<GeminiResolved> {
    with_config(|cfg| cfg.providers.gemini.clone())
}

/// Resolved Ollama provider settings.
pub fn ollama() -> Result<OllamaResolved> {
    with_config(|cfg| cfg.providers.ollama.clone())
}

/// Return the API key configured for the given provider identifier.
pub fn provider_api_key(provider: &str) -> Result<Option<String>> {
    with_config(|cfg| cfg.providers.api_key_for(provider))
}

/// Resolved configuration shared across the application.
#[derive(Debug, Clone)]
struct ResolvedConfig {
    paths: ResolvedPaths,
    providers: ResolvedProviders,
}

#[derive(Debug, Clone)]
struct ResolvedPaths {
    data_dir: PathBuf,
    board: PathBuf,
    okrs: PathBuf,
    log: PathBuf,
    agents: PathBuf,
    description: PathBuf,
    email_config: PathBuf,
    running_agents: PathBuf,
    responses_log: PathBuf,
}

#[derive(Debug, Clone)]
struct ResolvedProviders {
    openai: OpenAiResolved,
    gemini: GeminiResolved,
    ollama: OllamaResolved,
}

impl ResolvedProviders {
    fn api_key_for(&self, provider: &str) -> Option<String> {
        match provider {
            "openai" => self.openai.api_key.clone(),
            "gemini" => self.gemini.api_key.clone(),
            "ollama" => self.ollama.api_key.clone(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiResolved {
    pub api_key: Option<String>,
    pub base_url: String,
    pub responses_endpoint: String,
    pub chat_endpoint: String,
    pub request_style: Option<String>,
    pub response_format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeminiResolved {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OllamaResolved {
    pub api_key: Option<String>,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct RawConfig {
    paths: PathsSection,
    providers: ProvidersSection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct PathsSection {
    data_dir: PathBuf,
    board_file: Option<PathBuf>,
    okrs_file: Option<PathBuf>,
    log_file: Option<PathBuf>,
    agents_file: Option<PathBuf>,
    description_file: Option<PathBuf>,
    email_config_file: Option<PathBuf>,
    running_agents_file: Option<PathBuf>,
    responses_log_file: Option<PathBuf>,
}

impl Default for PathsSection {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(DIR),
            board_file: None,
            okrs_file: None,
            log_file: None,
            agents_file: None,
            description_file: None,
            email_config_file: None,
            running_agents_file: None,
            responses_log_file: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct ProvidersSection {
    openai: OpenAiSection,
    gemini: GeminiSection,
    ollama: OllamaSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct OpenAiSection {
    api_key: Option<String>,
    base_url: Option<String>,
    responses_endpoint: Option<String>,
    chat_endpoint: Option<String>,
    request_style: Option<String>,
    response_format: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct GeminiSection {
    api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct OllamaSection {
    api_key: Option<String>,
    base_url: Option<String>,
}

fn load_config(overrides: &ConfigOverrides) -> Result<ResolvedConfig> {
    let disable_host_config = host_config_disabled();
    if !disable_host_config {
        dotenvy::dotenv().ok();
    }

    let mut builder = config_rs::Config::builder();

    if let Some(path) = overrides.config_file.as_ref() {
        builder = builder.add_source(
            config_rs::File::from(path.as_path())
                .format(FileFormat::Toml)
                .required(true),
        );
    } else if !disable_host_config {
        if let Some(project_dirs) = default_config_path() {
            builder = builder.add_source(
                config_rs::File::from(project_dirs)
                    .format(FileFormat::Toml)
                    .required(false),
            );
        }
    }

    builder = builder.add_source(config_rs::Environment::with_prefix("TASKTER").separator("__"));

    let raw: RawConfig = builder
        .build()
        .context("failed to build Taskter configuration sources")?
        .try_deserialize()
        .context("failed to deserialize Taskter configuration")?;

    let mut merged = raw;
    apply_legacy_environment(&mut merged);
    apply_cli_overrides(&mut merged, overrides);

    resolve(merged)
}

fn default_config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "taskter").map(|dirs| dirs.config_dir().join("config.toml"))
}

fn apply_legacy_environment(raw: &mut RawConfig) {
    if host_config_disabled() {
        return;
    }
    if raw.providers.openai.api_key.is_none() {
        if let Ok(val) = std::env::var("OPENAI_API_KEY") {
            if !val.trim().is_empty() {
                raw.providers.openai.api_key = Some(val);
            }
        }
    }
    if raw.providers.openai.base_url.is_none() {
        if let Ok(val) = std::env::var("OPENAI_BASE_URL") {
            if !val.trim().is_empty() {
                raw.providers.openai.base_url = Some(val);
            }
        }
    }
    if raw.providers.openai.responses_endpoint.is_none() {
        if let Ok(val) = std::env::var("OPENAI_RESPONSES_ENDPOINT") {
            if !val.trim().is_empty() {
                raw.providers.openai.responses_endpoint = Some(val);
            }
        }
    }
    if raw.providers.openai.chat_endpoint.is_none() {
        if let Ok(val) = std::env::var("OPENAI_CHAT_ENDPOINT") {
            if !val.trim().is_empty() {
                raw.providers.openai.chat_endpoint = Some(val);
            }
        }
    }
    if raw.providers.openai.request_style.is_none() {
        if let Ok(val) = std::env::var("OPENAI_REQUEST_STYLE") {
            if !val.trim().is_empty() {
                raw.providers.openai.request_style = Some(val);
            }
        }
    }
    if raw.providers.openai.response_format.is_none() {
        if let Ok(val) = std::env::var("OPENAI_RESPONSE_FORMAT") {
            if !val.trim().is_empty() {
                raw.providers.openai.response_format = Some(val);
            }
        }
    }
    if raw.providers.gemini.api_key.is_none() {
        if let Ok(val) = std::env::var("GEMINI_API_KEY") {
            if !val.trim().is_empty() {
                raw.providers.gemini.api_key = Some(val);
            }
        }
    }
    if raw.providers.ollama.api_key.is_none() {
        if let Ok(val) = std::env::var("OLLAMA_API_KEY") {
            if !val.trim().is_empty() {
                raw.providers.ollama.api_key = Some(val);
            }
        }
    }
    if raw.providers.ollama.base_url.is_none() {
        if let Ok(val) = std::env::var("OLLAMA_BASE_URL") {
            if !val.trim().is_empty() {
                raw.providers.ollama.base_url = Some(val);
            }
        }
    }
}

fn apply_cli_overrides(raw: &mut RawConfig, overrides: &ConfigOverrides) {
    if let Some(dir) = overrides.data_dir.as_ref() {
        raw.paths.data_dir = dir.clone();
    }
    if let Some(path) = overrides.board_file.as_ref() {
        raw.paths.board_file = Some(path.clone());
    }
    if let Some(path) = overrides.okrs_file.as_ref() {
        raw.paths.okrs_file = Some(path.clone());
    }
    if let Some(path) = overrides.log_file.as_ref() {
        raw.paths.log_file = Some(path.clone());
    }
    if let Some(path) = overrides.agents_file.as_ref() {
        raw.paths.agents_file = Some(path.clone());
    }
    if let Some(path) = overrides.description_file.as_ref() {
        raw.paths.description_file = Some(path.clone());
    }
    if let Some(path) = overrides.email_config_file.as_ref() {
        raw.paths.email_config_file = Some(path.clone());
    }
    if let Some(path) = overrides.running_agents_file.as_ref() {
        raw.paths.running_agents_file = Some(path.clone());
    }
    if let Some(path) = overrides.responses_log_file.as_ref() {
        raw.paths.responses_log_file = Some(path.clone());
    }

    if let Some(value) = overrides.openai_api_key.as_ref() {
        raw.providers.openai.api_key = Some(value.clone());
    }
    if let Some(value) = overrides.openai_base_url.as_ref() {
        raw.providers.openai.base_url = Some(value.clone());
    }
    if let Some(value) = overrides.openai_responses_endpoint.as_ref() {
        raw.providers.openai.responses_endpoint = Some(value.clone());
    }
    if let Some(value) = overrides.openai_chat_endpoint.as_ref() {
        raw.providers.openai.chat_endpoint = Some(value.clone());
    }
    if let Some(value) = overrides.openai_request_style.as_ref() {
        raw.providers.openai.request_style = Some(value.clone());
    }
    if let Some(value) = overrides.openai_response_format.as_ref() {
        raw.providers.openai.response_format = Some(value.clone());
    }

    if let Some(value) = overrides.gemini_api_key.as_ref() {
        raw.providers.gemini.api_key = Some(value.clone());
    }

    if let Some(value) = overrides.ollama_api_key.as_ref() {
        raw.providers.ollama.api_key = Some(value.clone());
    }
    if let Some(value) = overrides.ollama_base_url.as_ref() {
        raw.providers.ollama.base_url = Some(value.clone());
    }
}

fn resolve(raw: RawConfig) -> Result<ResolvedConfig> {
    let paths = resolve_paths(raw.paths);
    let providers = resolve_providers(raw.providers)?;
    Ok(ResolvedConfig { paths, providers })
}

fn resolve_paths(paths: PathsSection) -> ResolvedPaths {
    let data_dir = if paths.data_dir.as_os_str().is_empty() {
        PathBuf::from(DIR)
    } else {
        paths.data_dir
    };

    let board = resolve_path(&data_dir, paths.board_file, "board.json");
    let okrs = resolve_path(&data_dir, paths.okrs_file, "okrs.json");
    let log = resolve_path(&data_dir, paths.log_file, "logs.log");
    let agents = resolve_path(&data_dir, paths.agents_file, "agents.json");
    let description = resolve_path(&data_dir, paths.description_file, "description.md");
    let email_config = resolve_path(&data_dir, paths.email_config_file, "email_config.json");
    let running_agents = resolve_path(&data_dir, paths.running_agents_file, "running_agents.json");
    let responses_log = resolve_path(&data_dir, paths.responses_log_file, "api_responses.log");

    ResolvedPaths {
        data_dir,
        board,
        okrs,
        log,
        agents,
        description,
        email_config,
        running_agents,
        responses_log,
    }
}

fn resolve_path(data_dir: &Path, explicit: Option<PathBuf>, default_name: &str) -> PathBuf {
    if let Some(path) = explicit {
        path
    } else {
        data_dir.join(default_name)
    }
}

fn resolve_providers(providers: ProvidersSection) -> Result<ResolvedProviders> {
    let openai = resolve_openai(providers.openai)?;
    let gemini = resolve_gemini(providers.gemini);
    let ollama = resolve_ollama(providers.ollama);

    Ok(ResolvedProviders {
        openai,
        gemini,
        ollama,
    })
}

fn resolve_openai(section: OpenAiSection) -> Result<OpenAiResolved> {
    let base_url = clean_string(section.base_url)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "https://api.openai.com".to_string());
    let normalized_base = base_url.trim_end_matches('/').to_string();
    let responses_endpoint = clean_string(section.responses_endpoint)
        .unwrap_or_else(|| format!("{normalized_base}/v1/responses"));
    let chat_endpoint = clean_string(section.chat_endpoint)
        .unwrap_or_else(|| format!("{normalized_base}/v1/chat/completions"));

    let response_format = clean_string(section.response_format);
    if let Some(ref raw) = response_format {
        if raw.trim_start().starts_with('{') {
            serde_json::from_str::<serde_json::Value>(raw)
                .context("OPENAI response_format override is not valid JSON")?;
        }
    }

    Ok(OpenAiResolved {
        api_key: clean_string(section.api_key),
        base_url: normalized_base,
        responses_endpoint,
        chat_endpoint,
        request_style: clean_string(section.request_style),
        response_format,
    })
}

fn resolve_gemini(section: GeminiSection) -> GeminiResolved {
    GeminiResolved {
        api_key: clean_string(section.api_key),
    }
}

fn resolve_ollama(section: OllamaSection) -> OllamaResolved {
    let base_url = clean_string(section.base_url)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    OllamaResolved {
        api_key: clean_string(section.api_key),
        base_url: base_url.trim_end_matches('/').to_string(),
    }
}

fn clean_string(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
