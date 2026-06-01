//! CLI direct commands merged from `crates/cli`.
//!
//! These are commands that the old `codewhale` dispatcher handled directly
//! (without spawning the TUI binary): login, logout, auth, config, model,
//! thread, sandbox, app-server, mcp-server, metrics, update.

pub mod metrics;
pub mod update;

use std::io::{self, IsTerminal, Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand, ValueEnum};
use codewhale_config::{ConfigStore, ProviderKind};
use codewhale_mcp::{McpServerDefinition, run_stdio_server};
use codewhale_secrets::Secrets;
use codewhale_state::{StateStore, ThreadListFilters};

// ── Provider enum ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ProviderArg {
    Deepseek,
    NvidiaNim,
    Openai,
    Atlascloud,
    WanjieArk,
    Openrouter,
    Novita,
    Fireworks,
    Moonshot,
    Sglang,
    Vllm,
    Ollama,
}

impl From<ProviderArg> for ProviderKind {
    fn from(value: ProviderArg) -> Self {
        match value {
            ProviderArg::Deepseek => ProviderKind::Deepseek,
            ProviderArg::NvidiaNim => ProviderKind::NvidiaNim,
            ProviderArg::Openai => ProviderKind::Openai,
            ProviderArg::Atlascloud => ProviderKind::Atlascloud,
            ProviderArg::WanjieArk => ProviderKind::WanjieArk,
            ProviderArg::Openrouter => ProviderKind::Openrouter,
            ProviderArg::Novita => ProviderKind::Novita,
            ProviderArg::Fireworks => ProviderKind::Fireworks,
            ProviderArg::Moonshot => ProviderKind::Moonshot,
            ProviderArg::Sglang => ProviderKind::Sglang,
            ProviderArg::Vllm => ProviderKind::Vllm,
            ProviderArg::Ollama => ProviderKind::Ollama,
        }
    }
}

// ── Arg structs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AuthCommand {
    /// Show current provider and credential source state.
    Status,
    /// Save an API key to the shared user config file.
    Set {
        #[arg(long, value_enum)]
        provider: ProviderArg,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long = "api-key-stdin", default_value_t = false)]
        api_key_stdin: bool,
    },
    /// Report whether a provider has a key configured (never prints the value).
    Get {
        #[arg(long, value_enum)]
        provider: ProviderArg,
    },
    /// Delete a provider's key from config and secret-store storage.
    Clear {
        #[arg(long, value_enum)]
        provider: ProviderArg,
    },
    /// List all known providers with their auth state, without revealing keys.
    List,
    /// Advanced: migrate config-file keys into a platform credential store.
    #[command(hide = true)]
    Migrate {
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
    List,
    Path,
}

#[derive(Debug, Clone, Args)]
pub struct ModelArgs {
    #[command(subcommand)]
    pub command: ModelCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModelCommand {
    List {
        #[arg(long, value_enum)]
        provider: Option<ProviderArg>,
    },
    Resolve {
        model: Option<String>,
        #[arg(long, value_enum)]
        provider: Option<ProviderArg>,
    },
}

#[derive(Debug, Clone, Args)]
pub struct ThreadArgs {
    #[command(subcommand)]
    pub command: ThreadCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ThreadCommand {
    List {
        #[arg(long, default_value_t = false)]
        all: bool,
        #[arg(long)]
        limit: Option<usize>,
    },
    Read {
        thread_id: String,
    },
    Resume {
        thread_id: String,
    },
    Fork {
        thread_id: String,
    },
    Archive {
        thread_id: String,
    },
    Unarchive {
        thread_id: String,
    },
    SetName {
        thread_id: String,
        name: String,
    },
    ClearName {
        thread_id: String,
    },
}

#[derive(Debug, Clone, Args)]
pub struct AppServerArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, default_value_t = 8787)]
    pub port: u16,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long = "auth-token")]
    pub auth_token: Option<String>,
    #[arg(long, default_value_t = false)]
    pub insecure_no_auth: bool,
    #[arg(long = "cors-origin")]
    pub cors_origin: Vec<String>,
    #[arg(long, default_value_t = false)]
    pub stdio: bool,
}

#[derive(Debug, Clone, Args)]
pub struct MetricsArgs {
    #[arg(long)]
    pub json: bool,
    #[arg(long, value_name = "DURATION")]
    pub since: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    #[arg(long)]
    pub beta: bool,
}

// ── Constants ──────────────────────────────────────────────────────────

const MCP_SERVER_DEFINITIONS_KEY: &str = "mcp.server_definitions";

/// Provider order used by `auth list` and `auth status` outputs.
const PROVIDER_LIST: [ProviderKind; 12] = [
    ProviderKind::Deepseek,
    ProviderKind::NvidiaNim,
    ProviderKind::Openai,
    ProviderKind::Atlascloud,
    ProviderKind::WanjieArk,
    ProviderKind::Openrouter,
    ProviderKind::Novita,
    ProviderKind::Fireworks,
    ProviderKind::Moonshot,
    ProviderKind::Sglang,
    ProviderKind::Vllm,
    ProviderKind::Ollama,
];

// ── Provider helpers ───────────────────────────────────────────────────

fn provider_slot(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Deepseek => "deepseek",
        ProviderKind::NvidiaNim => "nvidia-nim",
        ProviderKind::Openai => "openai",
        ProviderKind::Atlascloud => "atlascloud",
        ProviderKind::WanjieArk => "wanjie-ark",
        ProviderKind::Openrouter => "openrouter",
        ProviderKind::Novita => "novita",
        ProviderKind::Fireworks => "fireworks",
        ProviderKind::Moonshot => "moonshot",
        ProviderKind::Sglang => "sglang",
        ProviderKind::Vllm => "vllm",
        ProviderKind::Ollama => "ollama",
    }
}

fn provider_env_vars(provider: ProviderKind) -> &'static [&'static str] {
    match provider {
        ProviderKind::Deepseek => &["DEEPSEEK_API_KEY"],
        ProviderKind::Openrouter => &["OPENROUTER_API_KEY"],
        ProviderKind::Novita => &["NOVITA_API_KEY"],
        ProviderKind::NvidiaNim => &["NVIDIA_API_KEY", "NVIDIA_NIM_API_KEY", "DEEPSEEK_API_KEY"],
        ProviderKind::Fireworks => &["FIREWORKS_API_KEY"],
        ProviderKind::Moonshot => &["MOONSHOT_API_KEY", "KIMI_API_KEY"],
        ProviderKind::Sglang => &["SGLANG_API_KEY"],
        ProviderKind::Vllm => &["VLLM_API_KEY"],
        ProviderKind::Ollama => &["OLLAMA_API_KEY"],
        ProviderKind::Openai => &["OPENAI_API_KEY"],
        ProviderKind::Atlascloud => &["ATLASCLOUD_API_KEY"],
        ProviderKind::WanjieArk => &[
            "WANJIE_ARK_API_KEY",
            "WANJIE_API_KEY",
            "WANJIE_MAAS_API_KEY",
        ],
    }
}

fn provider_env_value(provider: ProviderKind) -> Option<(&'static str, String)> {
    provider_env_vars(provider).iter().find_map(|var| {
        std::env::var(var)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| (*var, value))
    })
}

fn provider_config_api_key(store: &ConfigStore, provider: ProviderKind) -> Option<&str> {
    let slot = store
        .config
        .providers
        .for_provider(provider)
        .api_key
        .as_deref();
    let root = (provider == ProviderKind::Deepseek)
        .then_some(store.config.api_key.as_deref())
        .flatten();
    slot.or(root).filter(|v| !v.trim().is_empty())
}

fn provider_config_set(store: &ConfigStore, provider: ProviderKind) -> bool {
    provider_config_api_key(store, provider).is_some()
}

fn provider_keyring_api_key(secrets: &Secrets, provider: ProviderKind) -> Option<String> {
    secrets
        .get(provider_slot(provider))
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty())
}

fn provider_keyring_set(secrets: &Secrets, provider: ProviderKind) -> bool {
    provider_keyring_api_key(secrets, provider).is_some()
}

fn write_provider_api_key_to_config(
    store: &mut ConfigStore,
    provider: ProviderKind,
    api_key: &str,
) {
    store.config.provider = provider;
    store.config.auth_mode = Some("api_key".to_string());
    store.config.providers.for_provider_mut(provider).api_key = Some(api_key.to_string());
    if provider == ProviderKind::Deepseek {
        store.config.api_key = Some(api_key.to_string());
        if store.config.default_text_model.is_none() {
            store.config.default_text_model = Some(
                store
                    .config
                    .providers
                    .deepseek
                    .model
                    .clone()
                    .unwrap_or_else(|| "deepseek-v4-pro".to_string()),
            );
        }
    }
}

fn clear_provider_api_key_from_config(store: &mut ConfigStore, provider: ProviderKind) {
    store.config.providers.for_provider_mut(provider).api_key = None;
    if provider == ProviderKind::Deepseek {
        store.config.api_key = None;
    }
}

fn write_provider_api_key_to_keyring(secrets: &Secrets, provider: ProviderKind, api_key: &str) -> bool {
    secrets.set(provider_slot(provider), api_key).is_ok()
}

fn clear_provider_api_key_from_keyring(secrets: &Secrets, provider: ProviderKind) {
    let _ = secrets.delete(provider_slot(provider));
}

fn auth_status_lines(store: &ConfigStore, secrets: &Secrets) -> Vec<String> {
    let provider = store.config.provider;
    let config_key = provider_config_api_key(store, provider);
    let keyring_key = provider_keyring_api_key(secrets, provider);
    let env_key = provider_env_value(provider);

    let active_source = if config_key.is_some() {
        "config"
    } else if keyring_key.is_some() {
        "secret store"
    } else if env_key.is_some() {
        "env"
    } else {
        "missing"
    };
    let active_last4 = config_key
        .map(last4_label)
        .or_else(|| keyring_key.as_deref().map(last4_label))
        .or_else(|| env_key.as_ref().map(|(_, value)| last4_label(value)));
    let active_label = active_last4
        .map(|last4| format!("{active_source} (last4: {last4})"))
        .unwrap_or_else(|| active_source.to_string());

    let env_var_label = env_key
        .as_ref()
        .map(|(name, _)| (*name).to_string())
        .unwrap_or_else(|| provider_env_vars(provider).join("/"));
    let env_status = env_key
        .as_ref()
        .map(|(_, value)| format!("set, last4: {}", last4_label(value)))
        .unwrap_or_else(|| "unset".to_string());

    vec![
        format!("provider: {}", provider.as_str()),
        format!(
            "auth mode: {}",
            store.config.auth_mode.as_deref().unwrap_or("api_key")
        ),
        format!("active source: {active_label}"),
        "lookup order: config -> secret store -> env".to_string(),
        format!(
            "config file: {} ({})",
            store.path().display(),
            source_status(config_key, "missing")
        ),
        format!(
            "secret store: {} ({})",
            secrets.backend_name(),
            source_status(keyring_key.as_deref(), "missing")
        ),
        format!("env var: {env_var_label} ({env_status})"),
    ]
}

fn source_status(value: Option<&str>, missing_label: &str) -> String {
    value
        .map(|v| format!("set, last4: {}", last4_label(v)))
        .unwrap_or_else(|| missing_label.to_string())
}

fn last4_label(value: &str) -> String {
    let trimmed = value.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= 4 {
        return "<redacted>".to_string();
    }
    let last4: String = chars[chars.len() - 4..].iter().collect();
    format!("...{last4}")
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no " }
}

fn keyring_status_short(state: Option<bool>) -> &'static str {
    match state {
        Some(true) => "yes",
        Some(false) => "no ",
        None => "n/a",
    }
}

// ── Auth helpers for the List command ──────────────────────────────────

fn provider_auth_state<'a>(store: &'a ConfigStore, secrets: &'a Secrets, provider: ProviderKind) -> ProviderAuthState<'a> {
    ProviderAuthState {
        name: provider.as_str(),
        config: provider_config_set(store, provider),
        secret_store: provider_keyring_set(secrets, provider),
        env: provider_env_set(provider),
        active: store.config.provider == provider,
    }
}

struct ProviderAuthState<'a> {
    name: &'a str,
    config: bool,
    secret_store: bool,
    env: bool,
    active: bool,
}

fn provider_env_set(provider: ProviderKind) -> bool {
    provider_env_value(provider).is_some()
}

// ── MCP server helpers ─────────────────────────────────────────────────

fn load_mcp_server_definitions(store: &ConfigStore) -> Vec<McpServerDefinition> {
    let Some(raw) = store.config.get_value(MCP_SERVER_DEFINITIONS_KEY) else {
        return Vec::new();
    };
    match parse_mcp_server_definitions(&raw) {
        Ok(definitions) => definitions,
        Err(err) => {
            eprintln!(
                "warning: failed to parse persisted MCP server definitions ({MCP_SERVER_DEFINITIONS_KEY}): {err}"
            );
            Vec::new()
        }
    }
}

fn parse_mcp_server_definitions(raw: &str) -> Result<Vec<McpServerDefinition>> {
    if let Ok(parsed) = serde_json::from_str::<Vec<McpServerDefinition>>(raw) {
        return Ok(parsed);
    }
    let unwrapped: String = serde_json::from_str(raw)
        .with_context(|| format!("invalid JSON payload at key {MCP_SERVER_DEFINITIONS_KEY}"))?;
    serde_json::from_str::<Vec<McpServerDefinition>>(&unwrapped).with_context(|| {
        format!("invalid MCP server definition list in key {MCP_SERVER_DEFINITIONS_KEY}")
    })
}

fn persist_mcp_server_definitions(
    store: &mut ConfigStore,
    definitions: &[McpServerDefinition],
) -> Result<()> {
    let encoded = serde_json::to_string(definitions).context("failed to encode MCP server definitions")?;
    store.config.set_value(MCP_SERVER_DEFINITIONS_KEY, &encoded)?;
    store.save()
}

// ── STDIN helpers ──────────────────────────────────────────────────────

fn read_api_key_from_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("failed to read api key from stdin")?;
    let key = input.trim().to_string();
    if key.is_empty() {
        bail!("empty API key provided");
    }
    Ok(key)
}

fn prompt_api_key(slot: &str) -> Result<String> {
    eprint!("Enter API key for {slot}: ");
    io::stderr().flush().ok();
    if !io::stdin().is_terminal() {
        return read_api_key_from_stdin();
    }
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("failed to read API key from stdin")?;
    let key = buf.trim().to_string();
    if key.is_empty() {
        bail!("empty API key provided");
    }
    Ok(key)
}

// ── Login ──────────────────────────────────────────────────────────────

pub fn run_login_command(store: &mut ConfigStore, provider_override: Option<ProviderArg>, api_key: Option<String>) -> Result<()> {
    let secrets = Secrets::auto_detect();
    let provider: ProviderKind = provider_override.map(Into::into).unwrap_or(ProviderKind::Deepseek);
    store.config.provider = provider;

    let api_key = match api_key {
        Some(v) => v,
        None => read_api_key_from_stdin()?,
    };
    write_provider_api_key_to_config(store, provider, &api_key);
    let keyring_saved = write_provider_api_key_to_keyring(&secrets, provider, &api_key);
    store.save()?;
    let destination = if keyring_saved {
        format!("{} and {}", store.path().display(), secrets.backend_name())
    } else {
        store.path().display().to_string()
    };
    if provider == ProviderKind::Deepseek {
        println!("logged in using API key mode (deepseek); saved key to {destination}");
    } else {
        println!(
            "logged in using API key mode ({}); saved key to {destination}",
            provider.as_str(),
        );
    }
    Ok(())
}

pub fn run_logout_command(store: &mut ConfigStore) -> Result<()> {
    let secrets = Secrets::auto_detect();
    let active_provider = store.config.provider;
    store.config.api_key = None;
    for provider in &PROVIDER_LIST {
        clear_provider_api_key_from_config(store, *provider);
    }
    clear_provider_api_key_from_keyring(&secrets, active_provider);
    store.config.auth_mode = None;
    store.save()?;
    println!("logged out");
    Ok(())
}

// ── Auth ───────────────────────────────────────────────────────────────

pub fn run_auth_command(store: &mut ConfigStore, command: AuthCommand) -> Result<()> {
    let secrets = Secrets::auto_detect();
    match command {
        AuthCommand::Status => {
            for line in auth_status_lines(store, &secrets) {
                println!("{line}");
            }
            Ok(())
        }
        AuthCommand::Set { provider, api_key, api_key_stdin } => {
            let provider: ProviderKind = provider.into();
            let slot = provider_slot(provider);
            if provider == ProviderKind::Ollama && api_key.is_none() && !api_key_stdin {
                store.config.provider = provider;
                let provider_cfg = store.config.providers.for_provider_mut(provider);
                if provider_cfg.base_url.is_none() {
                    provider_cfg.base_url = Some("http://localhost:11434/v1".to_string());
                }
                store.save()?;
                println!("configured {slot} provider in {} (API key optional)", store.path().display());
                return Ok(());
            }
            let api_key = match (api_key, api_key_stdin) {
                (Some(v), _) => v,
                (None, true) => read_api_key_from_stdin()?,
                (None, false) => prompt_api_key(slot)?,
            };
            write_provider_api_key_to_config(store, provider, &api_key);
            let keyring_saved = write_provider_api_key_to_keyring(&secrets, provider, &api_key);
            store.save()?;
            if keyring_saved {
                println!("saved API key for {slot} to {} and {}", store.path().display(), secrets.backend_name());
            } else {
                println!("saved API key for {slot} to {}", store.path().display());
            }
            Ok(())
        }
        AuthCommand::Get { provider } => {
            let provider: ProviderKind = provider.into();
            let slot = provider_slot(provider);
            let in_file = provider_config_set(store, provider);
            let in_keyring = !in_file && provider_keyring_set(&secrets, provider);
            let in_env = provider_env_set(provider);
            let source = if in_file { Some("config-file") }
                else if in_keyring { Some("secret-store") }
                else if in_env { Some("env") }
                else { None };
            match source {
                Some(source) => println!("{slot}: set (source: {source})"),
                None => println!("{slot}: not set"),
            }
            Ok(())
        }
        AuthCommand::Clear { provider } => {
            let provider: ProviderKind = provider.into();
            let slot = provider_slot(provider);
            clear_provider_api_key_from_config(store, provider);
            clear_provider_api_key_from_keyring(&secrets, provider);
            store.save()?;
            println!("cleared API key for {slot} from config and secret store");
            Ok(())
        }
        AuthCommand::List => {
            println!("{:<13} {:<6} {:<6} {:<6} active", "provider", "config", "store", "env");
            for provider in &PROVIDER_LIST {
                let state = provider_auth_state(store, &secrets, *provider);
                println!(
                    "{:<13} {:<6} {:<6} {:<6} {}",
                    state.name,
                    yes_no(state.config),
                    yes_no(state.secret_store),
                    yes_no(state.env),
                    if state.active { "yes" } else { "no " },
                );
            }
            Ok(())
        }
        AuthCommand::Migrate { dry_run } => {
            let mut migrated = 0usize;
            let mut skipped = 0usize;
            for provider in &PROVIDER_LIST {
                let slot = provider_slot(*provider);
                let config_value = provider_config_api_key(store, *provider);
                let keyring_value = provider_keyring_api_key(&secrets, *provider);
                match (config_value, keyring_value.clone()) {
                    (Some(key), None) => {
                        if dry_run {
                            println!("would migrate {slot} (key present in config, not in secret store)");
                        } else {
                            // store says set -> the config file has the key
                            write_provider_api_key_to_keyring(&secrets, *provider, key);
                            println!("migrated {slot}");
                        }
                        migrated += 1;
                    }
                    (None, Some(ref kv)) => {
                        if dry_run {
                            println!("would restore {slot} (key in secret store, config slot empty)");
                        } else {
                            write_provider_api_key_to_config(store, *provider, kv);
                            println!("restored {slot}");
                        }
                        migrated += 1;
                    }
                    _ => {
                        skipped += 1;
                    }
                }
            }
            if dry_run {
                println!("\nWould migrate/restore {migrated} provider(s).");
            } else {
                println!("\nMigrated/restored {migrated} provider(s); {skipped} unchanged.");
                store.save()?;
            }
            Ok(())
        }
    }
}

// ── Config ─────────────────────────────────────────────────────────────

pub fn run_config_command(store: &mut ConfigStore, command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Get { key } => {
            if let Some(value) = store.config.get_display_value(&key) {
                println!("{value}");
                Ok(())
            } else {
                bail!("unknown config key: {key}");
            }
        }
        ConfigCommand::Set { key, value } => {
            store.config.set_value(&key, &value)?;
            store.save()?;
            println!("set {key} to {value}");
            Ok(())
        }
        ConfigCommand::Unset { key } => {
            store.config.unset_value(&key)?;
            store.save()?;
            println!("unset {key}");
            Ok(())
        }
        ConfigCommand::List => {
            let raw = toml::to_string_pretty(&store.config)
                .context("failed to serialize config")?;
            println!("{raw}");
            Ok(())
        }
        ConfigCommand::Path => {
            println!("{}", store.path().display());
            Ok(())
        }
    }
}

// ── Model ──────────────────────────────────────────────────────────────

pub fn run_model_command(command: ModelCommand) -> Result<()> {
    use codewhale_agent::ModelRegistry;
    let registry = ModelRegistry::default();
    match command {
        ModelCommand::List { provider } => {
            let filter = provider.map(ProviderKind::from);
            for model in registry.list().into_iter().filter(|m| match filter {
                Some(p) => m.provider == p,
                None => true,
            }) {
                println!("{} ({})", model.id, model.provider.as_str());
            }
            Ok(())
        }
        ModelCommand::Resolve { model, provider } => {
            let model_name = model.as_deref();
            let resolved = registry.resolve(model_name, provider.map(ProviderKind::from));
            println!(
                "{} ({})",
                resolved.resolved.id, resolved.resolved.provider.as_str()
            );
            Ok(())
        }
    }
}

// ── Thread ─────────────────────────────────────────────────────────────

pub fn run_thread_command(command: ThreadCommand) -> Result<()> {
    let state = StateStore::open(None)?;
    match command {
        ThreadCommand::List { all, limit } => {
            let threads = state.list_threads(ThreadListFilters {
                include_archived: all,
                limit,
            })?;
            for thread in threads {
                println!(
                    "{} | {} | {} | {}",
                    thread.id,
                    thread.name.clone().unwrap_or_else(|| "(unnamed)".to_string()),
                    thread.model_provider,
                    thread.cwd.display()
                );
            }
            Ok(())
        }
        ThreadCommand::Read { thread_id } => {
            let thread = state.get_thread(&thread_id)?;
            println!("{}", serde_json::to_string_pretty(&thread)?);
            Ok(())
        }
        ThreadCommand::Archive { thread_id } => {
            state.mark_archived(&thread_id)?;
            println!("archived {thread_id}");
            Ok(())
        }
        ThreadCommand::Unarchive { thread_id } => {
            state.mark_unarchived(&thread_id)?;
            println!("unarchived {thread_id}");
            Ok(())
        }
        ThreadCommand::SetName { thread_id, name } => {
            let mut thread = state
                .get_thread(&thread_id)?
                .with_context(|| format!("thread not found: {thread_id}"))?;
            thread.name = Some(name);
            thread.updated_at = chrono::Utc::now().timestamp();
            state.upsert_thread(&thread)?;
            println!("renamed {thread_id}");
            Ok(())
        }
        ThreadCommand::ClearName { thread_id } => {
            let mut thread = state
                .get_thread(&thread_id)?
                .with_context(|| format!("thread not found: {thread_id}"))?;
            thread.name = None;
            thread.updated_at = chrono::Utc::now().timestamp();
            state.upsert_thread(&thread)?;
            println!("cleared name for {thread_id}");
            Ok(())
        }
        ThreadCommand::Resume { .. } | ThreadCommand::Fork { .. } => {
            // These should be handled by the interactive TUI, not here
            bail!("Use `codewhale resume` or `codewhale fork` directly (not `codewhale thread resume/fork`)");
        }
    }
}

// ── Sandbox ────────────────────────────────────────────────────────────

pub fn run_sandbox_check_command(command_str: String) -> Result<()> {
    use codewhale_execpolicy::{AskForApproval, ExecPolicyContext, ExecPolicyEngine};
    let engine = ExecPolicyEngine::new(Vec::new(), vec!["rm -rf".to_string()]);
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let decision = engine.check(ExecPolicyContext {
        command: &command_str,
        cwd: &cwd.display().to_string(),
        ask_for_approval: AskForApproval::OnRequest,
        sandbox_mode: Some("workspace-write"),
    })?;
    println!("{}", serde_json::to_string_pretty(&decision)?);
    Ok(())
}

// ── App Server ─────────────────────────────────────────────────────────

pub fn run_app_server_command(args: AppServerArgs) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to create tokio runtime")?;
    if args.stdio {
        return runtime.block_on(codewhale_app_server::run_stdio(args.config));
    }
    let listen: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .with_context(|| format!("invalid app-server listen address {}:{}", args.host, args.port))?;
    runtime.block_on(codewhale_app_server::run(codewhale_app_server::AppServerOptions {
        listen,
        config_path: args.config,
        auth_token: args.auth_token.or_else(app_server_token_from_env),
        insecure_no_auth: args.insecure_no_auth,
        cors_origins: args.cors_origin,
    }))
}

fn app_server_token_from_env() -> Option<String> {
    std::env::var("CODEWHALE_APP_SERVER_TOKEN")
        .ok()
        .or_else(|| std::env::var("DEEPSEEK_APP_SERVER_TOKEN").ok())
}

// ── MCP Server ─────────────────────────────────────────────────────────

pub fn run_mcp_server_command(store: &mut ConfigStore) -> Result<()> {
    let persisted = load_mcp_server_definitions(store);
    let updated = run_stdio_server(persisted)?;
    persist_mcp_server_definitions(store, &updated)
}

// ── Metrics ────────────────────────────────────────────────────────────

pub fn run_metrics_command(args: MetricsArgs) -> Result<()> {
    let since = match args.since.as_deref() {
        Some(s) => {
            Some(metrics::parse_since(s).with_context(|| format!("invalid --since value: {s:?}"))?)
        }
        None => None,
    };
    metrics::run(metrics::MetricsArgs {
        json: args.json,
        since,
    })
}

// ── Update ─────────────────────────────────────────────────────────────

pub fn run_update_command(args: UpdateArgs) -> Result<()> {
    update::run_update(args.beta)
}
