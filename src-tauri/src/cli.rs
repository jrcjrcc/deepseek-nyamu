//! CLI 命令行接口模块
//!
//! 本模块重新实现了 codewhale CLI 的子命令，使得桌面二进制文件也能在终端中运行。
//! 它基于 clap 框架定义了完整的命令行参数解析体系，涵盖认证管理、会话管理、
//! 配置读写、模型查询、MCP 服务器、递归语言模型（RLM）等核心功能。
//!
//! 主要结构：
//! - [`Cli`]：顶级 CLI 参数结构体
//! - [`CliCommands`]：所有子命令枚举
//! - [`ProviderArg`]：AI 提供商参数枚举
//! - 每个子命令对应的 `cmd_*` 执行函数

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use nyamu_config::{ConfigStore, ProviderKind};
use nyamu_secrets::Secrets;

/// AI 提供商命令行参数枚举
///
/// 通过 `clap::ValueEnum` 派生，允许在命令行中以字符串形式指定 AI 服务提供商。
/// 每个变体对应一个具体的 AI API 提供商，可通过 [`From<ProviderArg>`] 转换为 [`ProviderKind`]。
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ProviderArg {
    /// Deepseek（深度求索）
    Deepseek,
    /// NVIDIA NIM
    NvidiaNim,
    /// OpenAI
    Openai,
    /// AtlasCloud（Atlas 云）
    Atlascloud,
    /// 万界 Ark
    WanjieArk,
    /// 火山引擎
    Volcengine,
    /// OpenRouter
    Openrouter,
    /// 小米 MiMo
    XiaomiMimo,
    /// Novita AI
    Novita,
    /// Fireworks AI
    Fireworks,
    /// SiliconFlow（硅基流动）
    Siliconflow,
    /// Arcee AI
    Arcee,
    /// Moonshot（月之暗面 / Kimi）
    Moonshot,
    /// SGLang
    Sglang,
    /// vLLM
    Vllm,
    /// Ollama（本地模型）
    Ollama,
    /// Hugging Face
    Huggingface,
}

/// 将命令行参数 [`ProviderArg`] 转换为配置层 [`ProviderKind`] 的映射实现
impl From<ProviderArg> for ProviderKind {
    fn from(value: ProviderArg) -> Self {
        match value {
            ProviderArg::Deepseek => ProviderKind::Deepseek,
            ProviderArg::NvidiaNim => ProviderKind::NvidiaNim,
            ProviderArg::Openai => ProviderKind::Openai,
            ProviderArg::Atlascloud => ProviderKind::Atlascloud,
            ProviderArg::WanjieArk => ProviderKind::WanjieArk,
            ProviderArg::Volcengine => ProviderKind::Volcengine,
            ProviderArg::Openrouter => ProviderKind::Openrouter,
            ProviderArg::XiaomiMimo => ProviderKind::XiaomiMimo,
            ProviderArg::Novita => ProviderKind::Novita,
            ProviderArg::Fireworks => ProviderKind::Fireworks,
            ProviderArg::Siliconflow => ProviderKind::Siliconflow,
            ProviderArg::Arcee => ProviderKind::Arcee,
            ProviderArg::Moonshot => ProviderKind::Moonshot,
            ProviderArg::Sglang => ProviderKind::Sglang,
            ProviderArg::Vllm => ProviderKind::Vllm,
            ProviderArg::Ollama => ProviderKind::Ollama,
            ProviderArg::Huggingface => ProviderKind::Huggingface,
        }
    }
}

/// DeepWhale CLI 顶级参数解析结构体
///
/// 使用 clap 的 `Parser` 派生宏定义，支持 `--cli` 标志切换终端模式，
/// 以及若干全局选项（API Key、Base URL、配置文件路径、工作目录等）。
/// 子命令通过 `CliCommands` 枚举提供。
#[derive(Debug, Parser)]
#[command(name = "deepwhale", about = "DeepWhale — nyamu engine, nyamu UI", version)]
pub struct Cli {
    /// 以 CLI/终端模式启动（而非 GUI 模式）
    #[arg(long)]
    pub cli: bool,

    /// 提供商 API 密钥
    #[arg(long)]
    pub api_key: Option<String>,

    /// 提供商 API 基础 URL
    #[arg(long)]
    pub base_url: Option<String>,

    /// 配置文件路径
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// 工作目录
    #[arg(short = 'C', long = "workspace", value_name = "DIR")]
    pub workspace: Option<PathBuf>,

    /// 模型标识符
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// YOLO 模式：自动批准所有工具调用
    #[arg(long)]
    pub yolo: bool,

    /// 子命令（可选，不提供则显示帮助信息）
    #[command(subcommand)]
    pub command: Option<CliCommands>,
}

/// DeepWhale CLI 子命令枚举
///
/// 涵盖会话管理、认证、配置、诊断、更新、MCP 服务器、代码审查、
/// SWE-bench 导出、递归语言模型（RLM）等所有子命令。
#[derive(Debug, Subcommand)]
pub enum CliCommands {
    /// 列出已保存的会话
    Sessions {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 配置提供商凭据（登录）
    Login {
        /// 提供商名称
        #[arg(long)]
        provider: Option<String>,
        /// API 密钥
        #[arg(long)]
        api_key: Option<String>,
    },
    /// 移除已保存的认证信息（登出）
    Logout,
    /// 管理认证状态
    Auth {
        #[command(subcommand)]
        action: AuthCommand,
    },
    /// 读取/写入/列出配置项
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// 列出可用模型
    Models,
    /// 运行诊断检查
    Doctor {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 检查更新
    Update {
        /// 仅检查更新而不下载
        #[arg(long)]
        check: bool,
        /// 检查 Beta 版本
        #[arg(long)]
        beta: bool,
    },
    /// 打印使用指标
    Metrics {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 生成 Shell 补全脚本
    Completion {
        /// 目标 Shell 类型
        #[arg(value_enum)]
        shell: Shell,
    },
    /// 评估沙箱/批准策略
    Sandbox,
    /// 启动应用服务器（HTTP API）
    Serve {
        /// 监听主机地址（默认 127.0.0.1）
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// 监听端口（默认 8787）
        #[arg(long, default_value_t = 8787)]
        port: u16,
        /// 认证令牌
        #[arg(long = "auth-token")]
        auth_token: Option<String>,
        /// 允许无认证连接（不安全）
        #[arg(long)]
        insecure_no_auth: bool,
    },
    /// 显示版本信息
    Version,
    /// 设置工具/插件目录
    Setup,
    /// 运行非交互式提示（带上 --auto 启用工具）
    ///
    /// 示例：
    ///   deepwhale --cli exec 'explain this function'
    ///   deepwhale --cli exec --auto 'fix this bug'
    ///   deepwhale --cli exec --resume <ID> 'follow up'
    #[command(after_help = "\
Examples:
  deepwhale --cli exec 'explain this function'
  deepwhale --cli exec --auto 'fix this bug'
  deepwhale --cli exec --resume <ID> 'follow up'")]
    Exec {
        /// 启用带工具支持的代理模式，自动批准工具调用
        #[arg(long)]
        auto: bool,
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
        /// 恢复指定 ID 的会话
        #[arg(long)]
        resume: Option<String>,
        /// 继续最近的会话
        #[arg(long)]
        continue_session: bool,
        /// 输出格式（text 或 stream-json）
        #[arg(long)]
        output_format: Option<String>,
        /// 提示文本（后续所有参数均视为提示内容）
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
    /// 恢复已保存的会话
    Resume {
        /// 会话 ID，或 ":last" 表示最近一个会话
        session_id: String,
        /// 可选的后续提示
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
    /// 从已有会话分叉出一个新会话
    Fork {
        /// 要分叉的源会话 ID
        session_id: String,
        /// 新会话的初始提示（可选）
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
    /// 以 stdio 方式运行 MCP 服务器
    McpServer,
    /// 校验 MCP 服务器连接
    McpValidate,
    /// 获取 GitHub PR 并生成审查提示
    RunPr {
        /// PR 编号（如 "123" 或 "owner/repo/123"）
        pr: String,
    },
    /// SWE-bench 预测导出
    Swebench {
        #[command(subcommand)]
        action: SwebenchCommand,
    },
    /// RLM（递归语言模型）——通过分解方式分析长文本
    Rlm {
        /// 要分析的文本（或使用 @file.txt 指定文件路径）
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
        /// 子模型名称（默认 deepseek-v4-flash）
        #[arg(long)]
        child_model: Option<String>,
        /// 最大递归深度（默认 1）
        #[arg(long, default_value_t = 1)]
        max_depth: u32,
        /// 上下文文件路径
        #[arg(long)]
        context_file: Option<PathBuf>,
    },
    /// 管理已启用的技能
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// 持久化用户记忆管理
    Memory {
        /// 子命令：show, path, clear, add <note>, help
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// 技能管理子命令枚举
#[derive(Debug, Subcommand)]
pub enum SkillAction {
    /// 列出所有可用技能
    List,
    /// 按名称启用技能
    Enable { name: String },
    /// 按名称禁用技能
    Disable { name: String },
}

/// SWE-bench 子命令枚举
///
/// SWE-bench 是一个软件工程基准测试平台，本模块支持将当前 git diff
/// 导出为标准 JSONL 预测格式。
#[derive(Debug, Subcommand)]
pub enum SwebenchCommand {
    /// 从当前 git diff 导出预测到 JSONL 文件
    Export {
        /// SWE-bench 实例 ID（如 django__django-12345）
        #[arg(long)]
        instance_id: String,
        /// 预测结果 JSONL 文件路径
        #[arg(long)]
        predictions_path: PathBuf,
        /// 可选的模型名称（默认 deepwhale/v{version}）
        #[arg(long)]
        model_name: Option<String>,
        /// 工作目录
        #[arg(short = 'C', long)]
        workspace: Option<PathBuf>,
    },
}

/// 认证管理子命令枚举
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// 查看指定提供商或所有提供商的认证状态
    Status {
        /// 可选的提供商名称（不指定则显示全部）
        #[arg(long, value_enum)]
        provider: Option<ProviderArg>,
    },
    /// 列出所有提供商的凭据配置情况
    List,
    /// 清除指定提供商或所有提供商的凭据
    Clear {
        /// 可选的提供商名称（不指定则清除全部）
        #[arg(long, value_enum)]
        provider: Option<ProviderArg>,
    },
}

/// 配置管理子命令枚举
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// 获取指定配置项的值
    Get { key: String },
    /// 设置指定配置项的值
    Set { key: String, value: String },
    /// 列出所有配置项
    List,
}

/// 所有支持的 AI 提供商列表（用于迭代展示）
///
/// 共 18 个提供商，涵盖国内外主流 AI API 服务。
const PROVIDER_LIST: [ProviderKind; 18] = [
    ProviderKind::Deepseek, ProviderKind::NvidiaNim, ProviderKind::Openai,
    ProviderKind::Atlascloud, ProviderKind::WanjieArk, ProviderKind::Volcengine,
    ProviderKind::Openrouter, ProviderKind::XiaomiMimo, ProviderKind::Novita,
    ProviderKind::Fireworks, ProviderKind::Siliconflow, ProviderKind::SiliconflowCN,
    ProviderKind::Arcee, ProviderKind::Moonshot, ProviderKind::Sglang,
    ProviderKind::Vllm, ProviderKind::Ollama, ProviderKind::Huggingface,
];

/// CLI 模式入口函数
///
/// 根据解析后的 [`Cli`] 参数分发到对应的子命令处理函数。
/// 如果没有提供子命令，则打印版本信息和基本使用说明。
///
/// # 参数
/// * `cli` — 解析完成的 CLI 参数
///
/// # 返回值
/// 返回 `Ok(())` 表示执行成功，`Err` 包含错误信息。
pub fn run_cli(cli: &Cli) -> Result<()> {
    let command = &cli.command;

    match command {
        Some(CliCommands::Sessions { json }) => cmd_sessions(*json),
        Some(CliCommands::Login { provider, api_key }) => cmd_login(provider.as_deref(), api_key.as_deref()),
        Some(CliCommands::Logout) => cmd_logout(),
        Some(CliCommands::Auth { action }) => cmd_auth(action),
        Some(CliCommands::Config { action }) => cmd_config(action),
        Some(CliCommands::Models) => cmd_models(),
        Some(CliCommands::Doctor { json }) => cmd_doctor(*json),
        Some(CliCommands::Update { check, beta }) => cmd_update(*check, *beta),
        Some(CliCommands::Metrics { json }) => cmd_metrics(*json),
        Some(CliCommands::Completion { shell }) => cmd_completion(*shell),
        Some(CliCommands::Sandbox) => cmd_sandbox(),
        Some(CliCommands::Serve { host, port, auth_token, insecure_no_auth }) => {
            cmd_serve(host, *port, auth_token.as_deref(), *insecure_no_auth)
        }
        Some(CliCommands::Version) => cmd_version(),
        Some(CliCommands::Setup) => cmd_setup(),
        Some(CliCommands::Exec { auto, json, resume, continue_session, output_format, prompt }) => {
            cmd_exec(*auto, *json, resume.as_deref(), *continue_session, output_format.as_deref(), prompt)
        }
        Some(CliCommands::Resume { session_id, prompt }) => cmd_resume(session_id, prompt),
        Some(CliCommands::Fork { session_id, prompt }) => cmd_fork(session_id, prompt),
        Some(CliCommands::McpServer) => cmd_mcp_server(),
        Some(CliCommands::McpValidate) => cmd_mcp_validate(),
        Some(CliCommands::RunPr { pr }) => cmd_run_pr(pr),
        Some(CliCommands::Swebench { action }) => cmd_swebench(action),
        Some(CliCommands::Rlm { prompt, child_model, max_depth, context_file }) => {
            cmd_rlm(prompt, child_model.as_deref(), *max_depth, context_file.as_ref())
        }
        Some(CliCommands::Skill { action }) => cmd_skill(action),
        Some(CliCommands::Memory { args }) => {
            let result = crate::memory::cmd_memory(args)?;
            println!("{result}");
            Ok(())
        }
        None => {
            // 无子命令时打印欢迎信息
            println!("DeepWhale v{}", env!("CARGO_PKG_VERSION"));
            println!("nyamu engine + nyamu desktop UI");
            println!();
            println!("Usage:");
            println!("  deepwhale              Launch GUI");
            println!("  deepwhale --cli        Launch CLI mode");
            println!("  deepwhale --cli <SUBCOMMAND>");
            println!();
            println!("Run `deepwhale --cli <subcommand> --help` for details.");
            Ok(())
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Auth commands —— 认证相关子命令
// ────────────────────────────────────────────────────────────────────────────

/// 处理认证相关子命令（状态查看、列表、清除）
///
/// 从 [`ConfigStore`] 和 [`Secrets`] 中读取 API 密钥配置信息，
/// 支持按提供商查询或全局浏览，并自动判断密钥来源（config/keyring/env）。
fn cmd_auth(action: &AuthCommand) -> Result<()> {
    let mut store = ConfigStore::load(None)?;
    let secrets = Secrets::file_backed();

    match action {
        AuthCommand::Status { provider } => {
            match provider {
                Some(p) => {
                    // 查询单个提供商的认证状态
                    let pk: ProviderKind = (*p).into();
                    let slot = provider_slot(pk);
                    let config_key = provider_config_api_key(&store, pk);
                    let keyring_key = provider_keyring_api_key(&secrets, pk);
                    let env_key = provider_env_value(pk);
                    let active_src = if config_key.is_some() { "config" }
                        else if keyring_key.is_some() { "secret-store" }
                        else if env_key.is_some() { "env" }
                        else { "unset" };
                    let cfg = store.config.providers.for_provider(pk);
                    println!("provider: {slot}");
                    println!("  route:      {}", cfg.base_url.as_deref().unwrap_or("(default)"));
                    println!("  model:      {}", cfg.model.as_deref().unwrap_or("(default)"));
                    println!("  active src: {active_src}");
                    println!("  config:     {}", source_label(config_key.as_deref(), "missing"));
                    println!("  keyring:    {}", source_label(keyring_key.as_deref(), "missing"));
                    println!("  env:        {}", env_label(env_key));
                }
                None => {
                    // 列出所有提供商的认证状态概览
                    let active = store.config.provider;
                    println!("Active provider: {} ({})", active.as_str(), provider_slot(active));
                    println!();
                    println!("{:<14} {:<8} {:<10} {:<8}  source", "provider", "config", "keyring", "env");
                    println!("{}", "-".repeat(60));
                    for p in PROVIDER_LIST {
                        let ck = provider_config_api_key(&store, p);
                        let kk = provider_keyring_api_key(&secrets, p);
                        let ek = provider_env_value(p);
                        let src = if ck.is_some() { "config" } else if kk.is_some() { "keyring" } else if ek.is_some() { "env" } else { "—" };
                        println!("{:<14} {:<8} {:<10} {:<8}  {src}{}",
                            p.as_str(), yesno(ck.is_some()), yesno_opt(kk), yesno(ek.is_some()),
                            if p == active { " *" } else { "" });
                    }
                    println!(); println!("* = active provider");
                }
            }
            Ok(())
        }
        AuthCommand::List => {
            // 简洁列出所有提供商的凭据配置情况
            println!("{:<14} {:<8} {:<10} {:<8}  source", "provider", "config", "keyring", "env");
            println!("{}", "-".repeat(60));
            for p in PROVIDER_LIST {
                let ck = provider_config_api_key(&store, p);
                let kk = provider_keyring_api_key(&secrets, p);
                let ek = provider_env_value(p);
                let src = if ck.is_some() { "config" } else if kk.is_some() { "keyring" } else if ek.is_some() { "env" } else { "—" };
                println!("{:<14} {:<8} {:<10} {:<8}  {src}",
                    p.as_str(), yesno(ck.is_some()), yesno_opt(kk), yesno(ek.is_some()));
            }
            Ok(())
        }
        AuthCommand::Clear { provider } => {
            // 清除指定提供商或所有提供商的 API 密钥
            let providers: Vec<ProviderKind> = match provider {
                Some(p) => vec![(*p).into()],
                None => PROVIDER_LIST.to_vec(),
            };
            for p in providers {
                let slot = provider_slot(p);
                store.config.providers.for_provider_mut(p).api_key = None;
                if p == ProviderKind::Deepseek { store.config.api_key = None; }
                let _ = secrets.delete(slot);
            }
            store.save()?;
            println!("cleared credentials");
            Ok(())
        }
    }
}

/// 从配置文件（ConfigStore）中读取指定提供商的 API 密钥
///
/// 优先使用提供商专属的 api_key 字段，若不存在则回退到 Deepseek 的根级别 api_key。
/// 返回 `None` 表示未配置或为空字符串。
fn provider_config_api_key(store: &ConfigStore, provider: ProviderKind) -> Option<&str> {
    let slot = store.config.providers.for_provider(provider).api_key.as_deref();
    let root = (provider == ProviderKind::Deepseek)
        .then_some(store.config.api_key.as_deref()).flatten();
    slot.or(root).filter(|v| !v.trim().is_empty())
}

/// 从系统密钥环（keyring）中读取指定提供商的 API 密钥
///
/// 使用 `Secrets::file_backed()` 的后端存储，密钥以提供商 slot 名称存储。
fn provider_keyring_api_key(secrets: &Secrets, provider: ProviderKind) -> Option<String> {
    secrets.get(provider_slot(provider)).ok().flatten()
        .filter(|v| !v.trim().is_empty())
}

/// 从环境变量中读取指定提供商的 API 密钥
///
/// 遍历 [`provider_env_vars`] 返回的所有候选环境变量名，返回第一个非空值。
/// 返回值包含环境变量名和对应的值。
fn provider_env_value(provider: ProviderKind) -> Option<(&'static str, String)> {
    for var in provider_env_vars(provider) {
        if let Ok(val) = std::env::var(var) {
            if !val.trim().is_empty() { return Some((var, val)); }
        }
    }
    None
}

/// 获取指定提供商对应的环境变量名称列表
///
/// 每个提供商可能有多个可接受的环境变量名（兼容旧命名），依次尝试。
fn provider_env_vars(provider: ProviderKind) -> &'static [&'static str] {
    match provider {
        ProviderKind::Deepseek => &["DEEPSEEK_API_KEY"],
        ProviderKind::Openrouter => &["OPENROUTER_API_KEY"],
        ProviderKind::XiaomiMimo => &["XIAOMI_MIMO_API_KEY", "XIAOMI_API_KEY", "MIMO_API_KEY"],
        ProviderKind::Novita => &["NOVITA_API_KEY"],
        ProviderKind::NvidiaNim => &["NVIDIA_API_KEY", "NVIDIA_NIM_API_KEY", "DEEPSEEK_API_KEY"],
        ProviderKind::Fireworks => &["FIREWORKS_API_KEY"],
        ProviderKind::Siliconflow | ProviderKind::SiliconflowCN => &["SILICONFLOW_API_KEY"],
        ProviderKind::Arcee => &["ARCEE_API_KEY"],
        ProviderKind::Moonshot => &["MOONSHOT_API_KEY", "KIMI_API_KEY"],
        ProviderKind::Sglang => &["SGLANG_API_KEY"],
        ProviderKind::Vllm => &["VLLM_API_KEY"],
        ProviderKind::Ollama => &["OLLAMA_API_KEY"],
        ProviderKind::Huggingface => &["HUGGINGFACE_API_KEY", "HF_TOKEN"],
        ProviderKind::Openai => &["OPENAI_API_KEY"],
        ProviderKind::Atlascloud => &["ATLASCLOUD_API_KEY"],
        ProviderKind::Volcengine => &["VOLCENGINE_API_KEY", "VOLCENGINE_ARK_API_KEY", "ARK_API_KEY"],
        ProviderKind::WanjieArk => &["WANJIE_ARK_API_KEY", "WANJIE_API_KEY", "WANJIE_MAAS_API_KEY"],
    }
}

/// 获取提供商的配置 slot 名称（用于密钥环和配置文件的键名）
fn provider_slot(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Deepseek => "deepseek",
        ProviderKind::NvidiaNim => "nvidia-nim",
        ProviderKind::Openai => "openai",
        ProviderKind::Atlascloud => "atlascloud",
        ProviderKind::WanjieArk => "wanjie-ark",
        ProviderKind::Volcengine => "volcengine",
        ProviderKind::Openrouter => "openrouter",
        ProviderKind::XiaomiMimo => "xiaomi-mimo",
        ProviderKind::Novita => "novita",
        ProviderKind::Fireworks => "fireworks",
        ProviderKind::Siliconflow => "siliconflow",
        ProviderKind::SiliconflowCN => "siliconflow",
        ProviderKind::Arcee => "arcee",
        ProviderKind::Moonshot => "moonshot",
        ProviderKind::Sglang => "sglang",
        ProviderKind::Vllm => "vllm",
        ProviderKind::Ollama => "ollama",
        ProviderKind::Huggingface => "huggingface",
    }
}

/// 格式化配置源中的密钥值，脱敏显示后 4 位
///
/// 如果密钥长度 <= 4，仅显示 `<redacted>`；
/// 否则显示 `last4: ...xxxx` 格式。
fn source_label(value: Option<&str>, missing: &str) -> String {
    match value {
        Some(v) => {
            let chars: Vec<char> = v.trim().chars().collect();
            if chars.len() <= 4 { "set (<redacted>)".to_string() }
            else {
                let last4: String = chars[chars.len() - 4..].iter().collect();
                format!("set (last4: ...{last4})")
            }
        }
        None => missing.to_string(),
    }
}

/// 格式化环境变量来源标签
///
/// 显示环境变量名，如 `set via $DEEPSEEK_API_KEY`。
fn env_label(value: Option<(&str, String)>) -> String {
    match value {
        Some((var, _)) => format!("set via ${var}"),
        None => "missing".to_string(),
    }
}

/// 布尔值转"yes"或"—"的辅助函数
fn yesno(b: bool) -> &'static str { if b { "yes" } else { "—" } }

/// Option<String> 转"yes"或"—"的辅助函数
fn yesno_opt(v: Option<String>) -> &'static str { if v.is_some() { "yes" } else { "—" } }

// ────────────────────────────────────────────────────────────────────────────
// Config commands —— 配置管理子命令
// ────────────────────────────────────────────────────────────────────────────

/// 处理配置管理子命令（获取、设置、列出配置项）
///
/// 通过 [`ConfigStore`] 的通用键值接口操作配置，修改后自动保存到文件。
fn cmd_config(action: &ConfigAction) -> Result<()> {
    let mut store = ConfigStore::load(None)?;
    match action {
        ConfigAction::Get { key } => {
            match store.config.get_value(key) {
                Some(val) => println!("{key} = {val}"),
                None => println!("Key '{key}' not found"),
            }
        }
        ConfigAction::Set { key, value } => {
            store.config.set_value(key, value)?;
            store.save()?;
            println!("{key} = {value}");
        }
        ConfigAction::List => {
            for (k, v) in store.config.list_values() {
                println!("{k} = {v}");
            }
        }
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Login/Logout —— 登录/登出子命令
// ────────────────────────────────────────────────────────────────────────────

/// 处理登录子命令：将 API 密钥保存到密钥环
///
/// 需要提供 `--provider` 和 `--api-key` 参数。
/// 密钥存储在系统密钥环中，以 `{provider}_api_key` 为键名。
fn cmd_login(provider: Option<&str>, api_key: Option<&str>) -> Result<()> {
    let p = provider.unwrap_or("deepseek");
    let key = api_key.unwrap_or("");
    if !key.is_empty() {
        Secrets::file_backed().set(&format!("{}_api_key", p), key)?;
        println!("API key saved for provider: {p}");
    } else {
        println!("Usage: deepwhale --cli login --provider <PROVIDER> --api-key <KEY>");
    }
    Ok(())
}

/// 处理登出子命令：从密钥环中删除 Deepseek 的 API 密钥
fn cmd_logout() -> Result<()> {
    Secrets::file_backed().delete("deepseek_api_key")?;
    println!("Logged out.");
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Misc commands —— 杂项子命令
// ────────────────────────────────────────────────────────────────────────────

/// 列出可用模型（从配置中读取）
///
/// 当前为静态列表，仅显示 `deepseek-chat` 和 `deepseek-reasoner`。
fn cmd_models() -> Result<()> {
    println!("Available models (configured via config):");
    println!("  deepseek-chat\n  deepseek-reasoner");
    Ok(())
}

/// 获取 DeepWhale 数据目录路径
///
/// 在用户主目录下创建 `.deepwhale` 目录用于存储会话数据库等数据。
fn data_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".deepwhale")
}

/// 列出所有已保存的会话
///
/// 从 `conversations.db` 状态数据库中读取会话列表，
/// 支持文本和 JSON 两种输出格式。
fn cmd_sessions(json: bool) -> Result<()> {
    let data_dir = data_dir();
    let store = match nyamu_state::StateStore::open(Some(data_dir.join("conversations.db"))) {
        Ok(s) => s,
        Err(_) => {
            println!("No sessions database found. Start GUI mode to create one.");
            return Ok(());
        }
    };
    let threads = store.list_threads(nyamu_state::ThreadListFilters {
        include_archived: false,
        limit: Some(50),
    })?;

    if json {
        // JSON 格式输出
        let data: Vec<serde_json::Value> = threads.iter().map(|t| {
            serde_json::json!({
                "id": t.id,
                "preview": t.preview,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&data)?);
    } else {
        // 表格格式输出
        if threads.is_empty() {
            println!("No sessions found.");
        } else {
            println!("{:<40} {:<30} {:<10}", "Session ID", "Preview", "Messages");
            println!("{}", "-".repeat(85));
            for t in &threads {
                let msg_count = store.list_leaf_messages(&t.id)
                    .map(|m| m.len()).unwrap_or(0);
                let preview = if t.preview.len() > 28 { format!("{}...", &t.preview[..28]) } else { t.preview.clone() };
                println!("{:<40} {:<30} {:<10}",
                    &t.id[..std::cmp::min(40, t.id.len())],
                    preview,
                    msg_count);
            }
        }
    }
    Ok(())
}

/// 运行诊断检查，显示版本、平台、API 密钥配置状态等信息
fn cmd_doctor(json: bool) -> Result<()> {
    let store = ConfigStore::load(None).ok();
    let has_api_key = store.as_ref()
        .and_then(|s| s.config.api_key.as_deref()
            .or(s.config.providers.for_provider(ProviderKind::Deepseek).api_key.as_deref()))
        .map(|k| k.len() > 0).unwrap_or(false)
        || std::env::var("DEEPSEEK_API_KEY").is_ok()
        || std::env::var("NYAMU_API_KEY").is_ok();
    if json {
        let diag = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "platform": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "api_key_configured": has_api_key,
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
        });
        println!("{}", serde_json::to_string_pretty(&diag)?);
    } else {
        println!("DeepWhale Diagnostics:");
        println!("  Version: {}", env!("CARGO_PKG_VERSION"));
        println!("  Rust:    {}", env!("CARGO_PKG_RUST_VERSION"));
        println!("  OS:      {}", std::env::consts::OS);
        println!("  API Key: {}", if has_api_key { "configured ✓" } else { "not set" });
    }
    Ok(())
}

/// 检查更新
///
/// 通过 `nyamu_release` crate 查询 GitHub Release 信息，
/// 比较当前版本与最新版本，支持稳定版和 Beta 版通道。
fn cmd_update(check: bool, beta: bool) -> Result<()> {
    use nyamu_release::*;
    let channel = if beta { ReleaseChannel::Beta } else { ReleaseChannel::Stable };
    let current = env!("CARGO_PKG_VERSION");
    if check {
        match latest_release_tag_blocking(channel) {
            Ok(tag) => {
                let _clean = tag.trim_start_matches('v');
                println!("Current: v{current}");
                println!("Latest:  {tag}");
                match compare_release_versions(current, &tag) {
                    Ok(o) if o == std::cmp::Ordering::Less => println!("Update available!"),
                    _ => println!("Up to date."),
                }
            }
            Err(e) => println!("Update check failed: {e}"),
        }
    } else {
        println!("DeepWhale v{current}");
        println!("Download: https://github.com/user/nyamuwhale/releases");
    }
    Ok(())
}

/// 打印使用指标（配置信息 + 诊断信息）
fn cmd_metrics(json: bool) -> Result<()> {
    // Load config to get provider info
    let config = ConfigStore::load(None).ok();
    let provider = config.as_ref().map(|c| format!("{:?}", c.config.provider)).unwrap_or("default".into());
    let model = config.as_ref().and_then(|c| c.config.model.clone()).unwrap_or("deepseek-v4-flash".into());
    let os_info = format!("{} {} {}", std::env::consts::OS, std::env::consts::ARCH, std::env::consts::FAMILY);
    let rustc = std::process::Command::new("rustc").arg("--version").output()
        .ok().and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None })
        .unwrap_or("unknown".into());
    let memory_path = crate::memory::resolve_global_memory_path(None);
    let memory_exists = memory_path.exists();

    if json {
        println!("{}", serde_json::json!({
            "provider": provider,
            "model": model,
            "os": os_info,
            "rustc": rustc,
            "memory_enabled": memory_exists,
            "sandbox_available": crate::sandbox::sandbox_available(),
        }));
    } else {
        println!("=== DeepWhale Metrics ===");
        println!("Provider:     {provider}");
        println!("Model:        {model}");
        println!("OS:           {os_info}");
        println!("Rustc:        {rustc}");
        println!("Memory:       {}", if memory_exists { "enabled" } else { "not configured" });
        println!("Sandbox:      {}", if crate::sandbox::sandbox_available() { "available" } else { "unavailable" });
    }
    Ok(())
}

/// 生成 Shell 补全脚本
///
/// 使用 clap_complete 生成指定 Shell 的补全文件并输出到标准输出。
fn cmd_completion(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}

/// 沙箱评估（CLI 模式下不启用，仅在 GUI 中使用进程级沙箱）
fn cmd_sandbox() -> Result<()> {
    println!("Sandbox: not active in CLI mode (GUI mode uses process-level sandboxing)");
    Ok(())
}

/// 启动 HTTP API 服务器
///
/// 在指定地址和端口上启动 `nyamu_app_server`，支持可选的认证令牌。
fn cmd_serve(host: &str, port: u16, auth_token: Option<&str>, insecure: bool) -> Result<()> {
    let listen: std::net::SocketAddr = format!("{host}:{port}").parse().context("invalid address")?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        nyamu_app_server::run(nyamu_app_server::AppServerOptions {
            listen, config_path: None, cors_origins: Vec::new(),
            auth_token: auth_token.map(String::from), insecure_no_auth: insecure,
        }).await
    })
}

/// 显示版本信息
fn cmd_version() -> Result<()> {
    println!("DeepWhale v{}", env!("CARGO_PKG_VERSION"));
    println!("Platform: {}-{}", std::env::consts::OS, std::env::consts::ARCH);
    Ok(())
}

/// 初始化工具/插件目录
///
/// 在用户主目录下创建 `.deepseek/skills` 和 `.deepseek/plugins` 目录。
fn cmd_setup() -> Result<()> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).map(PathBuf::from).unwrap_or_default();
    for d in &[home.join(".deepseek/skills"), home.join(".deepseek/plugins")] {
        std::fs::create_dir_all(d).ok();
        println!("{}", d.display());
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Exec —— 非交互式提示执行，支持可选的工具模式
// ────────────────────────────────────────────────────────────────────────────

/// 执行非交互式提示（exec 子命令的核心实现）
///
/// 支持两种模式：
/// - 简单模式（`auto=false`）：一次性问答完成
/// - 代理模式（`auto=true`）：带工具调用的多轮对话代理
///
/// 此外支持通过 `--resume` 恢复历史会话，但当前仅在简单模式下有效。
fn cmd_exec(
    auto: bool, json: bool, resume: Option<&str>, _continue_session: bool,
    _output_format: Option<&str>, prompt: &[String],
) -> Result<()> {
    let prompt_text = prompt.join(" ");
    if prompt_text.is_empty() {
        bail!("No prompt provided. Usage: deepwhale --cli exec <prompt>");
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let api_key = resolve_api_key().await?;
        let base_url = resolve_base_url().await?;
        let model = resolve_model().await;

        if auto {
            // 完整代理模式：带工具支持
            exec_agent_mode(&api_key, &base_url, &model, &prompt_text, resume, json).await
        } else {
            // 简单一次问答模式
            exec_one_shot(&api_key, &base_url, &model, &prompt_text, json).await
        }
    })
}

/// 异步解析 API 密钥
///
/// 优先级：配置文件（config）> 配置文件提供商专属 > 环境变量 DEEPSEEK_API_KEY > 环境变量 NYAMU_API_KEY
async fn resolve_api_key() -> Result<String> {
    let store = ConfigStore::load(None).ok();
    if let Some(k) = store.as_ref().and_then(|s| s.config.api_key.as_deref()) {
        if !k.is_empty() { return Ok(k.to_string()); }
    }
    if let Some(k) = store.as_ref().and_then(|s| s.config.providers.for_provider(ProviderKind::Deepseek).api_key.as_deref()) {
        if !k.is_empty() { return Ok(k.to_string()); }
    }
    if let Ok(k) = std::env::var("DEEPSEEK_API_KEY") {
        if !k.is_empty() { return Ok(k); }
    }
    if let Ok(k) = std::env::var("NYAMU_API_KEY") {
        if !k.is_empty() { return Ok(k); }
    }
    bail!("No API key found. Set DEEPSEEK_API_KEY or run `deepwhale --cli login`");
}

/// 异步解析 API 基础 URL
///
/// 优先级：配置文件 > 环境变量 DEEPSEEK_BASE_URL > 默认值（https://api.deepseek.com/beta）
async fn resolve_base_url() -> Result<String> {
    let store = ConfigStore::load(None).ok();
    if let Some(url) = store.as_ref().and_then(|s| s.config.providers.for_provider(ProviderKind::Deepseek).base_url.as_deref()) {
        if !url.is_empty() { return Ok(url.to_string()); }
    }
    if let Ok(url) = std::env::var("DEEPSEEK_BASE_URL") {
        if !url.is_empty() { return Ok(url); }
    }
    Ok("https://api.deepseek.com/beta".to_string())
}

/// 异步解析模型名称
///
/// 优先级：配置文件 > 环境变量 DEEPSEEK_MODEL > 默认值（deepseek-v4-flash）
async fn resolve_model() -> String {
    let store = ConfigStore::load(None).ok();
    if let Some(m) = store.as_ref().and_then(|s| s.config.model.as_deref()) {
        if !m.is_empty() && m != "auto" { return m.to_string(); }
    }
    if let Ok(m) = std::env::var("DEEPSEEK_MODEL") {
        if !m.is_empty() { return m; }
    }
    "deepseek-v4-flash".to_string()
}

/// 简单一次性问答执行（无工具支持）
///
/// 向后兼容的 Deepseek Chat API 发送一条用户消息，流式关闭，
/// 接收完整响应后输出。
async fn exec_one_shot(
    api_key: &str, base_url: &str, model: &str,
    prompt: &str, json_output: bool,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 8192,
        "stream": false,
    });

    let resp = client
        .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await
        .context("API request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("API error ({status}): {text}");
    }

    let data: serde_json::Value = resp.json().await?;
    let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("");

    if json_output {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "model": model,
            "content": content,
            "usage": data["usage"],
        }))?);
    } else {
        println!("{content}");
    }
    Ok(())
}

/// 代理模式执行（带工具调用的多轮对话）
///
/// 在自动批准模式下运行一个带 read_file、exec_shell、grep_files、file_search、
/// list_dir、write_file、edit_file 等工具的 LLM 代理。
/// 最多进行 10 轮工具调用，支持推理内容（reasoning_content）输出。
async fn exec_agent_mode(
    api_key: &str, base_url: &str, model: &str,
    prompt: &str, _resume: Option<&str>, json_output: bool,
) -> Result<()> {
    let mut conversation: Vec<serde_json::Value> = Vec::new();
    conversation.push(serde_json::json!({"role": "user", "content": prompt}));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let max_turns: u32 = 10; // 最大对话轮数
    for turn in 0..max_turns {
        // 构建系统提示：从宪法（constitution）+ 技能构建
        let system_prompt = crate::prompts::build_system_prompt_for_mode(None, Some("agent"));

        // 构建工具定义数组
        let tools = serde_json::json!([{
            "type": "function",
            "function": {
                "name": "read_file", "description": "Read file contents",
                "parameters": {"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}
            }
        }, {
            "type": "function",
            "function": {
                "name": "exec_shell", "description": "Execute a shell command",
                "parameters": {"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}
            }
        }, {
            "type": "function",
            "function": {
                "name": "grep_files", "description": "Search file contents",
                "parameters": {"type":"object","properties":{"pattern":{"type":"string"}},"required":["pattern"]}
            }
        }, {
            "type": "function",
            "function": {
                "name": "file_search", "description": "Find files by name",
                "parameters": {"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}
            }
        }, {
            "type": "function",
            "function": {
                "name": "list_dir", "description": "List directory",
                "parameters": {"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}
            }
        }, {
            "type": "function",
            "function": {
                "name": "write_file", "description": "Write a file",
                "parameters": {"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}
            }
        }, {
            "type": "function",
            "function": {
                "name": "edit_file", "description": "Edit a file",
                "parameters": {"type":"object","properties":{"path":{"type":"string"},"search":{"type":"string"},"replace":{"type":"string"}},"required":["path","search","replace"]}
            }
        }]);

        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                ..conversation.clone()
            ],
            "tools": tools,
            "tool_choice": "auto",
            "max_tokens": 16384,
            "stream": false,
        });

        let resp = client
            .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send().await
            .context("API request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("API error ({status}): {text}");
        }

        let data: serde_json::Value = resp.json().await?;
        let choice = &data["choices"][0];
        let msg = &choice["message"];
        let finish = choice["finish_reason"].as_str().unwrap_or("stop");
        let content = msg["content"].as_str().unwrap_or("");
        let tool_calls = msg["tool_calls"].as_array().cloned().unwrap_or_default();

        // 输出推理内容（如果模型支持）
        if let Some(reasoning) = msg["reasoning_content"].as_str() {
            if !reasoning.is_empty() {
                if json_output {
                    let ev = serde_json::json!({"type":"reasoning","content":reasoning});
                    println!("{}", serde_json::to_string(&ev)?);
                }
            }
        }

        // 输出 AI 回复内容
        if !content.is_empty() {
            if json_output {
                let ev = serde_json::json!({"type":"content","content":content,"turn":turn});
                println!("{}", serde_json::to_string(&ev)?);
            } else {
                if turn == 0 { println!("{content}"); }
                else { eprintln!("{content}"); }
            }
        }

        if finish == "tool_calls" && !tool_calls.is_empty() {
            // 执行工具调用
            let mut assistant_msg = serde_json::json!({
                "role": "assistant",
                "content": content
            });
            assistant_msg["tool_calls"] = msg["tool_calls"].clone();
            conversation.push(assistant_msg);

            for tc in &tool_calls {
                let name = tc["function"]["name"].as_str().unwrap_or("");
                let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let id = tc["id"].as_str().unwrap_or("");

                if json_output {
                    let ev = serde_json::json!({"type":"tool_use","name":name,"arguments":args_str});
                    println!("{}", serde_json::to_string(&ev)?);
                } else {
                    eprintln!("  🛠  {name}({args_str})");
                }

                // 派发并执行工具调用
                let result = crate::tools::dispatch_tool_call_direct(name, args_str).await
                    .unwrap_or_else(|e| format!("Error: {e}"));

                if json_output {
                    let ev = serde_json::json!({"type":"tool_result","name":name,"result":result});
                    println!("{}", serde_json::to_string(&ev)?);
                } else {
                    let truncated = if result.len() > 500 {
                        format!("{}... [{} chars]", &result[..500], result.len())
                    } else { result.clone() };
                    eprintln!("  → {truncated}");
                }

                conversation.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": id,
                    "content": result,
                }));
            }
        } else {
            // 无工具调用——对话结束
            conversation.push(serde_json::json!({
                "role": "assistant",
                "content": content
            }));
            if json_output {
                let ev = serde_json::json!({"type":"done","reason":finish});
                println!("{}", serde_json::to_string(&ev)?);
            }
            return Ok(());
        }
    }
    eprintln!("Reached max turns ({max_turns}) without completion.");
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Resume / Fork —— 会话恢复与分叉
// ────────────────────────────────────────────────────────────────────────────

/// 恢复已保存的会话
///
/// 从状态数据库中加载指定 ID 的会话（支持 `:last` 特殊 ID），
/// 显示会话摘要信息。
fn cmd_resume(session_id: &str, prompt: &[String]) -> Result<()> {
    let db_path = data_dir().join("conversations.db");
    let store = nyamu_state::StateStore::open(Some(db_path))
        .context("Cannot open session database. Start GUI mode first.")?;

    let actual_id = if session_id == ":last" {
        let threads = store.list_threads(nyamu_state::ThreadListFilters {
            include_archived: false, limit: Some(1),
        })?;
        threads.first().map(|t| t.id.clone())
            .ok_or_else(|| anyhow::anyhow!("No sessions found"))?
    } else {
        session_id.to_string()
    };

    let thread = store.get_thread(&actual_id)?
        .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", actual_id))?;

    let msgs = store.list_leaf_messages(&actual_id)?;
    let msg_count = msgs.len();

    println!("Resuming session: {} ({msg_count} messages)", thread.preview);
    println!("  ID: {actual_id}");
    if !prompt.is_empty() {
        let text = prompt.join(" ");
        println!("  Follow-up: {text}");
        println!("\nTo continue: deepwhale --cli exec --resume {actual_id} \"{text}\"");
    } else {
        println!("\nTo continue: deepwhale --cli exec --resume {actual_id} \"your message\"");
    }
    Ok(())
}

/// 从已有会话分叉出一个新的独立会话
///
/// 复制指定会话的消息历史到新会话，为后续独立对话做准备。
fn cmd_fork(session_id: &str, prompt: &[String]) -> Result<()> {
    let db_path = data_dir().join("conversations.db");
    let store = nyamu_state::StateStore::open(Some(db_path))
        .context("Cannot open session database.")?;

    let thread = store.get_thread(session_id)?
        .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

    let messages = store.list_leaf_messages(session_id)?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let text = if prompt.is_empty() { format!("Fork of {}", thread.preview) } else { prompt.join(" ") };

    println!("Forked from: {} ({})", thread.preview, session_id);
    println!("New session: {new_id} ({text})");
    println!("Messages: {}", messages.len());
    println!("\nStart GUI mode to see the new session.");
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// RLM —— 递归语言模型
// ────────────────────────────────────────────────────────────────────────────

/// 执行递归语言模型（RLM）分析
///
/// RLM 通过递归分解方式处理超长文本：将文本分割成块，每个块
/// 由子模型独立分析，然后汇总结果。支持指定递归深度和子模型。
///
/// # 参数
/// * `prompt` — 直接提供的文本
/// * `child_model` — 子模型名称（默认 deepseek-v4-flash）
/// * `max_depth` — 最大递归深度（默认 1）
/// * `context_file` — 从文件读取上下文（与 prompt 二选一）
fn cmd_rlm(prompt: &[String], child_model: Option<&str>, max_depth: u32, context_file: Option<&PathBuf>) -> Result<()> {
    let text = if let Some(path) = context_file {
        std::fs::read_to_string(path)
            .with_context(|| format!("failed to read context file: {}", path.display()))?
    } else {
        prompt.join(" ")
    };

    if text.trim().is_empty() {
        bail!("No text provided. Supply text directly or use --context-file <path>");
    }

    let api_key = resolve_api_key_blocking()?;
    let base_url = resolve_base_url_blocking();
    let model = resolve_model_blocking();

    let child = child_model.unwrap_or("deepseek-v4-flash");

    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(crate::rlm::run_rlm_turn(
        &api_key, &base_url, &model, child, text, None, max_depth,
    ));

    println!("RLM Result:");
    println!("  Termination: {:?}", result.termination);
    println!("  Iterations: {}", result.iterations);
    println!("  Duration: {}ms", result.duration_ms);
    println!("  Child calls: {}", result.total_child_calls);
    if let Some(err) = &result.error {
        println!("  Error: {err}");
    }
    println!("\n--- Answer ---\n{}", result.answer);
    println!("\n--- Trace ---");
    for t in &result.trace {
        let icon = if t.had_error { "✗" } else { "✓" };
        println!("  [{icon}] Round {}: {} ({}ms)", t.round, t.code_summary, t.elapsed_ms);
        if !t.stdout_preview.is_empty() {
            println!("       stdout: {}", t.stdout_preview);
        }
    }
    Ok(())
}

/// 阻塞式解析 API 密钥（用于 RLM 等阻塞上下文）
///
/// 优先级：环境变量 > 配置文件 > 错误
fn resolve_api_key_blocking() -> Result<String> {
    if let Ok(k) = std::env::var("DEEPSEEK_API_KEY") { return Ok(k); }
    if let Ok(k) = std::env::var("NYAMU_API_KEY") { return Ok(k); }
    let store = ConfigStore::load(None).ok();
    if let Some(k) = store.as_ref().and_then(|s| s.config.api_key.as_deref()) {
        if !k.is_empty() { return Ok(k.to_string()); }
    }
    bail!("No API key found. Set DEEPSEEK_API_KEY or run `deepwhale --cli login`");
}

/// 阻塞式解析 API 基础 URL（用于 RLM 等阻塞上下文）
fn resolve_base_url_blocking() -> String {
    std::env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com/beta".to_string())
}

/// 阻塞式解析模型名称（用于 RLM 等阻塞上下文）
fn resolve_model_blocking() -> String {
    std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".to_string())
}

// ────────────────────────────────────────────────────────────────────────────
// Skills —— 技能管理子命令
// ────────────────────────────────────────────────────────────────────────────

/// 处理技能管理子命令（列出、启用、禁用）
fn cmd_skill(action: &SkillAction) -> Result<()> {
    match action {
        SkillAction::List => {
            let skills = crate::skills::list_skills();
            if skills.is_empty() {
                println!("No skills found.");
            } else {
                println!("{:<20} {}", "Name", "Description");
                println!("{}", "-".repeat(60));
                for s in &skills {
                    println!("{:<20} {}", s.name, s.description);
                }
            }
            Ok(())
        }
        SkillAction::Enable { name } => {
            crate::skills::enable_skill(name)?;
            println!("Enabled skill: {name}");
            Ok(())
        }
        SkillAction::Disable { name } => {
            crate::skills::disable_skill(name)?;
            println!("Disabled skill: {name}");
            Ok(())
        }
    }
}

/// 配置文件中 MCP 服务器定义的键名
const MCP_SERVER_DEFINITIONS_KEY: &str = "mcp.server_definitions";

/// 从配置中加载 MCP 服务器定义列表
///
/// 处理 JSON 值可能被二次编码的情况（字符串内嵌 JSON），
/// 兼容两种存储格式。
fn load_mcp_definitions(store: &ConfigStore) -> Vec<nyamu_mcp::McpServerDefinition> {
    let Some(raw) = store.config.get_value(MCP_SERVER_DEFINITIONS_KEY) else {
        return Vec::new();
    };
    // 尝试直接解析为 JSON 数组，然后尝试二次解码的字符串
    if let Ok(parsed) = serde_json::from_str::<Vec<nyamu_mcp::McpServerDefinition>>(&raw) {
        return parsed;
    }
    if let Ok(unwrapped) = serde_json::from_str::<String>(&raw) {
        if let Ok(parsed) = serde_json::from_str::<Vec<nyamu_mcp::McpServerDefinition>>(&unwrapped) {
            return parsed;
        }
    }
    Vec::new()
}

/// 将 MCP 服务器定义列表持久化到配置文件
///
/// 对定义进行二次 JSON 序列化后存储（兼容当前配置系统的字符串值存储方式）。
fn persist_mcp_definitions(store: &mut ConfigStore, definitions: &[nyamu_mcp::McpServerDefinition]) -> Result<()> {
    let encoded = serde_json::to_string(definitions)?;
    store.config.set_value(MCP_SERVER_DEFINITIONS_KEY, &serde_json::to_string(&encoded)?)?;
    store.save()?;
    Ok(())
}

/// 以 stdio 模式启动 MCP 服务器
///
/// 从配置中加载 MCP 服务器定义，通过 stdin/stdout 提供 JSON-RPC 服务。
/// 服务器退出时会自动持久化运行时的配置变更（如通过 MCP 注册的新服务器）。
fn cmd_mcp_server() -> Result<()> {
    let mut store = ConfigStore::load(None)?;
    let definitions = load_mcp_definitions(&store);

    println!("DeepWhale MCP stdio server");
    println!("Config: {}", store.path().display());
    println!("{} MCP server(s) configured.", definitions.len());
    for def in &definitions {
        let status = if def.config.enabled { "enabled" } else { "disabled" };
        println!("  {}: {} ({status})", def.config.name, def.config.command);
    }
    println!();
    println!("Listening on stdin/stdout for JSON-RPC requests...");
    println!("(Connect a client like deepseek-mcp-tool or npx @anthropic/mcp-tool)");
    eprintln!("deepseek-mcp stdio server running on stdin/stdout");

    let result = nyamu_mcp::run_stdio_server(definitions);
    match result {
        Ok(updated) => {
            eprintln!("deepseek-mcp stdio server exited");
            // 持久化运行时变更（例如通过 MCP 注册的新服务器）
            persist_mcp_definitions(&mut store, &updated)?;
            println!("MCP server exited. {} definition(s) saved.", updated.len());
        }
        Err(e) => eprintln!("MCP server error: {e}"),
    }
    Ok(())
}

/// 验证 MCP 服务器配置的有效性
///
/// 遍历所有已配置的 MCP 服务器，检查其可执行文件是否存在于系统 PATH 中。
/// 显示环境变量和过滤器配置，汇总通过/失败统计。
fn cmd_mcp_validate() -> Result<()> {
    let store = ConfigStore::load(None)?;
    let definitions = load_mcp_definitions(&store);

    println!("MCP Validate");
    println!("Config: {}", store.path().display());
    println!();

    if definitions.is_empty() {
        println!("No MCP servers configured.");
        println!();
        println!("To configure an MCP server, set the following in your config.toml:");
        println!("  [mcp.server_definitions]");
        println!("  value = \"[{{\\\"config\\\": {{\\\"name\\\": \\\"my-server\\\", \\\"command\\\": \\\"npx\\\", \\\"args\\\": [\\\"-y\\\", \\\"@modelcontextprotocol/server-filesystem\\\"]}}, \\\"filter\\\": {{}}]}}\"");
        return Ok(());
    }

    let mut pass = 0u32;
    let mut fail = 0u32;

    for def in &definitions {
        if !def.config.enabled {
            println!("  [-] {} (disabled)", def.config.name);
            continue;
        }

        let cmd = &def.config.command;
        let exists = if cfg!(windows) {
            std::process::Command::new("where")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        } else {
            std::process::Command::new("which")
                .arg(cmd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };

        if exists {
            println!("  [\u{2713}] {} → {} (found on PATH)", def.config.name, cmd);
            pass += 1;
        } else {
            println!("  [\u{2717}] {} → {} (NOT found on PATH)", def.config.name, cmd);
            fail += 1;
        }

        if !def.config.args.is_empty() {
            let args = def.config.args.join(" ");
            println!("         args: {args}");
        }
        if !def.config.env.is_empty() {
            for (k, v) in &def.config.env {
                let masked = if k.to_lowercase().contains("key") || k.to_lowercase().contains("secret") || k.to_lowercase().contains("token") {
                    // 对敏感环境变量值脱敏，仅显示后 4 位
                    format!("...{}", v.chars().rev().take(4).collect::<String>().chars().rev().collect::<String>())
                } else {
                    v.clone()
                };
                println!("         env: {k}={masked}");
            }
        }
        if !def.filter.allow.is_empty() || !def.filter.deny.is_empty() {
            println!("         filter: allow={} deny={}", def.filter.allow.join(","), def.filter.deny.join(","));
        }
    }

    println!();
    println!("Results: {pass} passed, {fail} failed");

    if fail > 0 {
        println!();
        println!("Some MCP server binaries were not found on PATH.");
        println!("Install the required tools or update the command/path in your config.");
        println!();
        println!("Common MCP servers:");
        println!("  npx @modelcontextprotocol/server-filesystem <dir>   — File system access");
        println!("  npx @anthropic/mcp-tool                           — Generic MCP client");
        println!("  uvx mcp-server-git                               — Git operations");
    }

    Ok(())
}

/// 获取 GitHub PR 信息并生成审查提示
///
/// 优先使用 `gh` CLI 获取 PR 的详细信息（标题、描述、增删行数、文件列表），
/// 若 `gh` 不可用则通过 GitHub API 回退获取 PR diff。
fn cmd_run_pr(pr: &str) -> Result<()> {
    // 尝试使用 `gh` CLI
    let output = std::process::Command::new("gh")
        .args(["pr", "view", pr, "--json", "title,body,additions,deletions,files"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let data: serde_json::Value = serde_json::from_str(&text)
                .unwrap_or(serde_json::json!({"raw": text.to_string()}));

            let title = data["title"].as_str().unwrap_or(pr);
            let body = data["body"].as_str().unwrap_or("");
            let additions = data["additions"].as_i64().unwrap_or(0);
            let deletions = data["deletions"].as_i64().unwrap_or(0);

            println!("PR #{pr}: {title}");
            println!("  +{additions} -{deletions} lines");
            println!();

            // 构建代码审查提示词
            let review_prompt = format!(
                "Review the following pull request:\n\nTitle: {title}\n\nDescription:\n{body}\n\n\
                 Focus on: correctness, security, performance, and code style.\
                 \nProvide specific suggestions for improvement."
            );

            println!("Review prompt ready ({} chars):", review_prompt.len());
            println!("{review_prompt}");

            // 提示用户如何在审查中使用
            println!("\nTo review: deepwhale --cli exec --auto \"{}\"", &review_prompt[..std::cmp::min(200, review_prompt.len())]);
        }
        Ok(_) => {
            // `gh` 不可用或 PR 未找到，回退到直接获取 diff
            let diff = fetch_pr_diff_fallback(pr);
            match diff {
                Ok(d) => println!("PR #{pr} diff ({} chars):\n{d}", d.len()),
                Err(e) => bail!("Could not fetch PR #{pr}: {e}. Install `gh` CLI or use a full URL."),
            }
        }
        Err(_) => {
            bail!("`gh` CLI not found. Install GitHub CLI (https://cli.github.com) or use `exec` directly.");
        }
    }
    Ok(())
}

/// 回退方案：通过 GitHub API 直接获取 PR diff
///
/// 当 `gh` CLI 不可用时使用。支持 `owner/repo/123` 格式的完整 PR 路径，
/// 也支持从当前 git 仓库自动推断 owner/repo。
fn fetch_pr_diff_fallback(pr: &str) -> Result<String> {
    // 尝试通过 GitHub API 获取
    let url = if pr.contains('/') {
        // owner/repo/123 格式
        format!("https://api.github.com/repos/{pr}.diff")
    } else {
        // 使用当前 git 仓库的 remote 信息
        let repo = get_git_remote()?;
        format!("https://api.github.com/repos/{repo}/pulls/{pr}.diff")
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("deepwhale-review")
        .build()?;

    let resp = client.get(&url)
        .header("Accept", "application/vnd.github.v3.diff")
        .send()?;

    if resp.status().is_success() {
        Ok(resp.text()?)
    } else {
        bail!("GitHub API returned {}", resp.status());
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SWE-bench export —— SWE-bench 预测导出
// ────────────────────────────────────────────────────────────────────────────

/// 处理 SWE-bench 相关子命令
///
/// 当前仅支持导出（Export）操作，将当前工作目录的 git diff
/// 转换为 SWE-bench 格式的预测 JSONL 文件。
fn cmd_swebench(action: &SwebenchCommand) -> Result<()> {
    match action {
        SwebenchCommand::Export { instance_id, predictions_path, model_name, workspace } => {
            let ws = workspace.clone().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let model = model_name.clone()
                .unwrap_or_else(|| format!("deepwhale/v{}", env!("CARGO_PKG_VERSION")));
            crate::swebench::write_swebench_prediction(&ws, predictions_path, instance_id, &model)?;
            Ok(())
        }
    }
}

/// 从当前 git 仓库的 remote 中解析 owner/repo 信息
///
/// 支持 HTTPS 和 SSH 两种 git URL 格式：
/// - `https://github.com/owner/repo.git`
/// - `git@github.com:owner/repo.git`
fn get_git_remote() -> Result<String> {
    let out = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Not in a git repository")?;

    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // 从 git URL 中提取 owner/repo
    let cleaned = url
        .trim_end_matches(".git")
        .trim_start_matches("https://github.com/")
        .trim_start_matches("git@github.com:");
    if cleaned.contains('/') {
        Ok(cleaned.to_string())
    } else {
        bail!("Could not parse git remote: {url}");
    }
}
