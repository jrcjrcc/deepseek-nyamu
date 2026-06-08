//! # HTTP API 服务模式
//!
//! 本 crate 实现了 nyamu 框架的应用服务器，提供 HTTP API 和标准输入/输出
//! (Stdio) 两种传输模式，供外部客户端（如 Tauri 桌面应用、CLI 等）与核心
//! 运行时交互。主要组件包括：
//!
//! - **AppServerOptions**：服务启动配置，包括监听地址、配置文件路径、
//!   认证令牌、CORS 原点列表等。令牌在 Debug 输出中被自动脱敏。
//!
//! - **HTTP 路由 (axum)**：基于 axum 框架构建的 RESTful API：
//!   - `POST /thread`：线程管理（创建、启动、恢复、分叉、列表、归档等）
//!   - `POST /prompt`：提示词请求处理
//!   - `POST /tool`：工具调用，附带执行策略审批
//!   - `GET /jobs`：后台作业状态查询
//!   - `POST /mcp/startup`：MCP 服务器启动
//!   - `GET /healthz`：健康检查
//!   - 受保护路由通过 Bearer Token 中间件进行身份验证
//!   - CORS 默认允许 localhost 常见开发端口，支持自定义扩展
//!
//! - **Stdio 模式 (`run_stdio`)**：基于 JSON-RPC 2.0 的标准输入输出协议，
//!   支持完整的方法家族：`thread/*`、`app/*`（配置管理、模型列表、
//!   线程加载状态）、`prompt/*`，以及 `shutdown` 信号。
//!
//! - **认证机制**：支持显式令牌、环境变量 (`NYAMUWHALE_APP_SERVER_TOKEN`)、
//!   自动生成令牌三种方式。非回环地址绑定强制要求认证。
//!
//! - **配置管理**：通过 AppRequest 处理配置的读取（HTTP 下自动脱敏敏感值）、
//!   设置、删除和列表查询，并自动持久化到配置文件。

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, bail};
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use nyamu_agent::ModelRegistry;
use nyamu_config::{CliRuntimeOverrides, ConfigStore};
use nyamu_core::Runtime;
use nyamu_execpolicy::ExecPolicyEngine;
use nyamu_hooks::HookDispatcher;
use nyamu_mcp::McpManager;
use nyamu_protocol::{
    AppRequest, AppResponse, PromptRequest, PromptResponse, ThreadRequest, ThreadResponse,
};
use nyamu_tools::{ToolCall, ToolRegistry};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

const DEFAULT_CORS_ORIGINS: &[&str] = &[
    "http://localhost",
    "http://localhost:1420",
    "http://localhost:3000",
    "http://localhost:5173",
    "http://127.0.0.1",
    "http://127.0.0.1:1420",
    "tauri://localhost",
];

#[derive(Clone)]
pub struct AppServerOptions {
    pub listen: SocketAddr,
    pub config_path: Option<PathBuf>,
    pub auth_token: Option<String>,
    pub insecure_no_auth: bool,
    pub cors_origins: Vec<String>,
}

impl std::fmt::Debug for AppServerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppServerOptions")
            .field("listen", &self.listen)
            .field("config_path", &self.config_path)
            .field(
                "auth_token",
                &self.auth_token.as_ref().map(|_| "<redacted>"),
            )
            .field("insecure_no_auth", &self.insecure_no_auth)
            .field("cors_origins", &self.cors_origins)
            .finish()
    }
}

#[derive(Clone)]
struct AppState {
    config_path: Option<PathBuf>,
    config: Arc<RwLock<nyamu_config::ConfigToml>>,
    runtime: Arc<Mutex<Runtime>>,
    registry: ModelRegistry,
    auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCallRequest {
    call: ToolCall,
    #[serde(default)]
    cwd: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    jsonrpc: Option<String>,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug)]
struct JsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

#[derive(Debug)]
struct StdioDispatchResult {
    result: Value,
    should_exit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTransport {
    Http,
    Stdio,
}

#[derive(Debug, Deserialize)]
struct ConfigGetParams {
    key: String,
}

#[derive(Debug, Deserialize)]
struct ConfigSetParams {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct ThreadIdParams {
    thread_id: String,
}

#[derive(Debug, Deserialize)]
struct ThreadMessageParams {
    thread_id: String,
    input: String,
}

pub async fn run(options: AppServerOptions) -> Result<()> {
    let auth_token = resolve_auth_token(&options)?;
    let state = build_state(options.config_path.clone(), auth_token)?;
    let app = app_router(state, &options.cors_origins);

    let listener = tokio::net::TcpListener::bind(options.listen).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn app_router(state: AppState, cors_origins: &[String]) -> Router {
    let protected_routes = Router::new()
        .route("/thread", post(thread_handler))
        .route("/app", post(app_handler))
        .route("/prompt", post(prompt_handler))
        .route("/tool", post(tool_handler))
        .route("/jobs", get(jobs_handler))
        .route("/mcp/startup", post(mcp_startup_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_app_server_token,
        ));

    Router::new()
        .route("/healthz", get(healthz))
        .merge(protected_routes)
        .layer(cors_layer(cors_origins))
        .with_state(state)
}

pub async fn run_stdio(config_path: Option<PathBuf>) -> Result<()> {
    let state = build_state(config_path, None)?;
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = tokio::io::BufWriter::new(stdout);
    while let Some(line) = reader.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                let response = jsonrpc_error(
                    None,
                    JsonRpcError::parse_error(format!("invalid json: {err}")),
                );
                writer.write_all(response.to_string().as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                continue;
            }
        };

        if request
            .jsonrpc
            .as_deref()
            .is_some_and(|version| version != "2.0")
        {
            let response = jsonrpc_error(
                request.id,
                JsonRpcError::invalid_request("jsonrpc version must be 2.0"),
            );
            writer.write_all(response.to_string().as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
            continue;
        }

        let response = match dispatch_stdio_request(&state, &request.method, request.params).await {
            Ok(dispatch) => {
                let encoded = jsonrpc_result(request.id, dispatch.result);
                writer.write_all(encoded.to_string().as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                if dispatch.should_exit {
                    break;
                }
                continue;
            }
            Err(err) => jsonrpc_error(request.id, err),
        };

        writer.write_all(response.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

async fn healthz() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "protocol": "v2",
        "service": "deepwhale-app-server"
    }))
}

async fn thread_handler(
    State(state): State<AppState>,
    Json(req): Json<ThreadRequest>,
) -> Json<ThreadResponse> {
    let mut runtime = state.runtime.lock().await;
    match runtime.handle_thread(req).await {
        Ok(res) => Json(res),
        Err(err) => Json(ThreadResponse {
            thread_id: "error".to_string(),
            status: format!("error:{err}"),
            thread: None,
            threads: Vec::new(),
            model: None,
            model_provider: None,
            cwd: None,
            approval_policy: None,
            sandbox: None,
            events: Vec::new(),
            data: json!({}),
        }),
    }
}

async fn prompt_handler(
    State(state): State<AppState>,
    Json(req): Json<PromptRequest>,
) -> Json<PromptResponse> {
    let mut runtime = state.runtime.lock().await;
    let overrides = CliRuntimeOverrides::default();
    match runtime.handle_prompt(req, &overrides).await {
        Ok(res) => Json(res),
        Err(err) => Json(PromptResponse {
            output: err.to_string(),
            model: "unknown".to_string(),
            events: Vec::new(),
        }),
    }
}

async fn tool_handler(
    State(state): State<AppState>,
    Json(req): Json<ToolCallRequest>,
) -> Json<Value> {
    let runtime = state.runtime.lock().await;
    let cwd = req
        .cwd
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    match runtime
        .invoke_tool(
            req.call,
            nyamu_execpolicy::AskForApproval::OnRequest,
            &cwd,
        )
        .await
    {
        Ok(value) => Json(value),
        Err(err) => Json(json!({ "ok": false, "error": err.to_string() })),
    }
}

async fn jobs_handler(State(state): State<AppState>) -> Json<AppResponse> {
    let runtime = state.runtime.lock().await;
    Json(runtime.app_status())
}

async fn mcp_startup_handler(State(state): State<AppState>) -> Json<Value> {
    let runtime = state.runtime.lock().await;
    let summary = runtime.mcp_startup().await;
    Json(json!({
        "ok": true,
        "summary": summary
    }))
}

async fn app_handler(
    State(state): State<AppState>,
    Json(req): Json<AppRequest>,
) -> Json<AppResponse> {
    let response = process_app_request(&state, req, AppTransport::Http).await;
    Json(response)
}

async fn require_app_server_token(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    if let Some(ref expected) = state.auth_token {
        let provided = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        match provided {
            Some(token) if token == *expected => Ok(next.run(req).await),
            _ => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "invalid auth_token; provide via header: Authorization: Bearer <token>"
                })),
            )),
        }
    } else {
        Ok(next.run(req).await)
    }
}

fn build_state(config_path: Option<PathBuf>, auth_token: Option<String>) -> Result<AppState> {
    let store = ConfigStore::load(config_path.clone())?;
    let config = store.config;
    let data_dir = dirs::data_dir()
        .map(|d| d.join("deepwhale"))
        .unwrap_or_else(|| PathBuf::from(".deepwhale"));
    std::fs::create_dir_all(&data_dir).ok();

    let state = nyamu_state::StateStore::open(Some(data_dir.join("conversations.db")))
        .unwrap_or_else(|_| nyamu_state::StateStore::open(None).expect("in-memory fallback"));

    let tool_registry = Arc::new(ToolRegistry::default());
    let exec_policy = ExecPolicyEngine::new(Vec::new(), Vec::new());
    let hooks = HookDispatcher::default();
    let mcp_manager = Arc::new(McpManager::new());
    let registry = ModelRegistry::new(Vec::new());

    let runtime = Runtime::new(config.clone(), registry.clone(), state, tool_registry, mcp_manager, exec_policy, hooks);

    Ok(AppState {
        config_path,
        config: Arc::new(RwLock::new(config)),
        runtime: Arc::new(Mutex::new(runtime)),
        registry,
        auth_token,
    })
}

fn resolve_auth_token(options: &AppServerOptions) -> Result<Option<String>> {
    if let Some(token) = &options.auth_token {
        let trimmed = token.trim().to_string();
        if trimmed.is_empty() {
            bail!("auth_token cannot be empty");
        }
        return Ok(Some(trimmed));
    }

    if let Some(token) = app_server_token_from_env() {
        return Ok(Some(token));
    }

    if options.insecure_no_auth {
        let is_loopback = options.listen.ip().is_loopback();
        if !is_loopback {
            bail!(
                "refusing unauthenticated app-server bind on {}: \
                 provide --auth-token, set NYAMUWHALE_APP_SERVER_TOKEN, \
                 or use --insecure-no-auth on a loopback address",
                options.listen
            );
        }
        return Ok(None);
    }

    let token = format!("dwapp_{}", Uuid::new_v4());
    eprintln!(
        "app-server auth token (set via --auth-token or NYAMUWHALE_APP_SERVER_TOKEN): {token}"
    );
    Ok(Some(token))
}

fn app_server_token_from_env() -> Option<String> {
    std::env::var("NYAMUWHALE_APP_SERVER_TOKEN")
        .ok()
        .or_else(|| std::env::var("DEEPWHALE_APP_SERVER_TOKEN").ok())
}

fn cors_layer(cors_origins: &[String]) -> CorsLayer {
    let mut origins: Vec<HeaderValue> = DEFAULT_CORS_ORIGINS
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    for origin in cors_origins {
        let trimmed = origin.trim();
        if !trimmed.is_empty() {
            if let Ok(hv) = HeaderValue::from_str(trimmed) {
                origins.push(hv);
            }
        }
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .max_age(std::time::Duration::from_secs(86400))
}

fn parse_params<T: DeserializeOwned>(params: Value) -> std::result::Result<T, JsonRpcError> {
    serde_json::from_value(params).map_err(|err| {
        JsonRpcError::invalid_params(format!("failed to parse params: {err}"))
    })
}

fn params_or_object(params: Value) -> Value {
    if params.is_null() {
        json!({})
    } else {
        params
    }
}

fn jsonrpc_result(id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn jsonrpc_error(id: Option<Value>, err: JsonRpcError) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": err.code,
            "message": err.message,
            "data": err.data,
        }
    })
}

impl JsonRpcError {
    fn parse_error(message: impl Into<String>) -> Self {
        Self { code: -32700, message: message.into(), data: None }
    }
    fn invalid_request(message: impl Into<String>) -> Self {
        Self { code: -32600, message: message.into(), data: None }
    }
    fn method_not_found(method: impl Into<String>) -> Self {
        Self { code: -32601, message: format!("method not found: {}", method.into()), data: None }
    }
    fn invalid_params(message: impl Into<String>) -> Self {
        Self { code: -32602, message: message.into(), data: None }
    }
    fn internal(message: impl Into<String>) -> Self {
        Self { code: -32603, message: message.into(), data: None }
    }
}

async fn handle_thread_request(
    state: &AppState,
    req: ThreadRequest,
) -> std::result::Result<ThreadResponse, JsonRpcError> {
    let mut runtime = state.runtime.lock().await;
    runtime
        .handle_thread(req)
        .await
        .map_err(|err| JsonRpcError::internal(err.to_string()))
}

async fn handle_prompt_request(
    state: &AppState,
    req: PromptRequest,
) -> std::result::Result<PromptResponse, JsonRpcError> {
    let mut runtime = state.runtime.lock().await;
    runtime
        .handle_prompt(req, &CliRuntimeOverrides::default())
        .await
        .map_err(|err| JsonRpcError::internal(err.to_string()))
}

async fn dispatch_stdio_request(
    state: &AppState,
    method: &str,
    params: Value,
) -> std::result::Result<StdioDispatchResult, JsonRpcError> {
    let outcome = match method {
        "healthz" | "app/healthz" => StdioDispatchResult {
            result: json!({
                "status": "ok",
                "service": "deepwhale-app-server",
                "transport": "stdio"
            }),
            should_exit: false,
        },
        "capabilities" => StdioDispatchResult {
            result: json!({
                "transport": "stdio",
                "families": ["thread/*", "app/*", "prompt/*"],
                "methods": [
                    "healthz",
                    "thread/capabilities",
                    "thread/request",
                    "thread/create",
                    "thread/start",
                    "thread/resume",
                    "thread/fork",
                    "thread/list",
                    "thread/read",
                    "thread/set_name",
                    "thread/archive",
                    "thread/unarchive",
                    "thread/message",
                    "app/capabilities",
                    "app/request",
                    "app/config/get",
                    "app/config/set",
                    "app/config/unset",
                    "app/config/list",
                    "app/models",
                    "app/thread_loaded_list",
                    "prompt/capabilities",
                    "prompt/request",
                    "prompt/run",
                    "shutdown"
                ]
            }),
            should_exit: false,
        },
        "thread/capabilities" => StdioDispatchResult {
            result: json!({
                "methods": [
                    "thread/request",
                    "thread/create",
                    "thread/start",
                    "thread/resume",
                    "thread/fork",
                    "thread/list",
                    "thread/read",
                    "thread/set_name",
                    "thread/archive",
                    "thread/unarchive",
                    "thread/message"
                ]
            }),
            should_exit: false,
        },
        "thread/request" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/create" | "thread/start" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/resume" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/fork" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/list" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/read" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/set_name" => {
            let request: ThreadRequest = parse_params(params)?;
            let response = handle_thread_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/archive" => {
            let parsed: ThreadIdParams = parse_params(params_or_object(params))?;
            let response = handle_thread_request(
                state,
                ThreadRequest::Archive {
                    thread_id: parsed.thread_id,
                },
            )
            .await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/unarchive" => {
            let parsed: ThreadIdParams = parse_params(params_or_object(params))?;
            let response = handle_thread_request(
                state,
                ThreadRequest::Unarchive {
                    thread_id: parsed.thread_id,
                },
            )
            .await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "thread/message" => {
            let parsed: ThreadMessageParams = parse_params(params_or_object(params))?;
            let response = handle_thread_request(
                state,
                ThreadRequest::Message {
                    thread_id: parsed.thread_id,
                    input: parsed.input,
                },
            )
            .await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/capabilities" => {
            let response =
                process_app_request(state, AppRequest::Capabilities, AppTransport::Stdio).await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/request" => {
            let request: AppRequest = parse_params(params)?;
            let response = process_app_request(state, request, AppTransport::Stdio).await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/config/get" => {
            let parsed: ConfigGetParams = parse_params(params_or_object(params))?;
            let response = process_app_request(
                state,
                AppRequest::ConfigGet { key: parsed.key },
                AppTransport::Stdio,
            )
            .await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/config/set" => {
            let parsed: ConfigSetParams = parse_params(params_or_object(params))?;
            let response = process_app_request(
                state,
                AppRequest::ConfigSet {
                    key: parsed.key,
                    value: parsed.value,
                },
                AppTransport::Stdio,
            )
            .await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/config/unset" => {
            let parsed: ConfigGetParams = parse_params(params_or_object(params))?;
            let response = process_app_request(
                state,
                AppRequest::ConfigUnset { key: parsed.key },
                AppTransport::Stdio,
            )
            .await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/config/list" => {
            let response =
                process_app_request(state, AppRequest::ConfigList, AppTransport::Stdio).await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/models" => {
            let response =
                process_app_request(state, AppRequest::Models, AppTransport::Stdio).await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "app/thread_loaded_list" | "app/thread-loaded-list" => {
            let response =
                process_app_request(state, AppRequest::ThreadLoadedList, AppTransport::Stdio).await;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "prompt/capabilities" => StdioDispatchResult {
            result: json!({
                "methods": ["prompt/request", "prompt/run"]
            }),
            should_exit: false,
        },
        "prompt/request" | "prompt/run" => {
            let request: PromptRequest = parse_params(params)?;
            let response = handle_prompt_request(state, request).await?;
            StdioDispatchResult {
                result: serde_json::to_value(response)
                    .map_err(|err| JsonRpcError::internal(err.to_string()))?,
                should_exit: false,
            }
        }
        "shutdown" => StdioDispatchResult {
            result: json!({"ok": true, "status": "stopped"}),
            should_exit: true,
        },
        _ => return Err(JsonRpcError::method_not_found(method)),
    };
    Ok(outcome)
}

async fn process_app_request(
    state: &AppState,
    req: AppRequest,
    transport: AppTransport,
) -> AppResponse {
    match req {
        AppRequest::Capabilities => AppResponse {
            ok: true,
            data: json!({
                "routes": ["/thread", "/app", "/prompt", "/tool", "/jobs", "/mcp/startup"],
                "config": ["get", "set", "unset", "list"],
                "events": ["response_start", "response_delta", "response_end", "tool_call_start", "tool_call_result", "mcp_startup_update", "mcp_startup_complete"],
                "transport": "stdio+http",
                "config_path": state.config_path.as_ref().map(|p| p.display().to_string()),
            }),
            events: Vec::new(),
        },
        AppRequest::ConfigGet { key } => {
            let cfg = state.config.read().await;
            let value = match transport {
                AppTransport::Http => cfg.get_display_value(&key),
                AppTransport::Stdio => cfg.get_value(&key),
            };
            AppResponse {
                ok: true,
                data: json!({ "key": key, "value": value }),
                events: Vec::new(),
            }
        }
        AppRequest::ConfigSet { key, value } => {
            let mut cfg = state.config.write().await;
            let result = cfg.set_value(&key, &value);
            let ok = result.is_ok();
            let message = result.err().map(|e| e.to_string());
            let snapshot = cfg.clone();
            drop(cfg);
            let _ = persist_config(state, snapshot).await;
            AppResponse {
                ok,
                data: json!({ "key": key, "value": value, "error": message }),
                events: Vec::new(),
            }
        }
        AppRequest::ConfigUnset { key } => {
            let mut cfg = state.config.write().await;
            let result = cfg.unset_value(&key);
            let ok = result.is_ok();
            let message = result.err().map(|e| e.to_string());
            let snapshot = cfg.clone();
            drop(cfg);
            let _ = persist_config(state, snapshot).await;
            AppResponse {
                ok,
                data: json!({ "key": key, "error": message }),
                events: Vec::new(),
            }
        }
        AppRequest::ConfigList => {
            let cfg = state.config.read().await;
            AppResponse {
                ok: true,
                data: json!({ "values": cfg.list_values() }),
                events: Vec::new(),
            }
        }
        AppRequest::Models => AppResponse {
            ok: true,
            data: json!({ "models": state.registry.list() }),
            events: Vec::new(),
        },
        AppRequest::ThreadLoadedList => {
            let mut runtime = state.runtime.lock().await;
            let response = runtime
                .handle_thread(nyamu_protocol::ThreadRequest::List(
                    nyamu_protocol::ThreadListParams {
                        include_archived: false,
                        limit: Some(50),
                    },
                ))
                .await;
            match response {
                Ok(thread_resp) => AppResponse {
                    ok: true,
                    data: json!({ "threads": thread_resp.threads }),
                    events: thread_resp.events,
                },
                Err(err) => AppResponse {
                    ok: false,
                    data: json!({ "error": err.to_string() }),
                    events: Vec::new(),
                },
            }
        }
    }
}

async fn persist_config(state: &AppState, config: nyamu_config::ConfigToml) -> Result<()> {
    if state.config_path.is_none() {
        return Ok(());
    }
    let mut store = ConfigStore::load(state.config_path.clone())?;
    store.config = config;
    store.save()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use nyamu_protocol::AppRequest;
    use std::fs;
    use tower::ServiceExt;

    fn app_with_config(auth_token: Option<&str>) -> (Router, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config_path = tmp.path().join("config.toml");
        fs::write(&config_path, "api_key = \"sk-deepseek-secret\"\n").expect("write config");
        let state = build_state(
            Some(config_path),
            auth_token.map(std::string::ToString::to_string),
        )
        .expect("state");
        (app_router(state, &[]), tmp)
    }

    async fn response_body_json(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        serde_json::from_slice(&bytes).expect("json response")
    }

    #[tokio::test]
    async fn http_app_routes_require_bearer_token_when_auth_enabled() {
        let (app, _tmp) = app_with_config(Some("test-token"));
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/app")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&AppRequest::ConfigGet {
                            key: "api_key".to_string(),
                        })
                        .expect("request json"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn http_config_get_redacts_sensitive_values_after_auth() {
        let (app, _tmp) = app_with_config(Some("test-token"));
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/app")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&AppRequest::ConfigGet {
                            key: "api_key".to_string(),
                        })
                        .expect("request json"),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_json(response).await;
        assert_eq!(body["data"]["value"], "sk-d***cret");
    }

    #[tokio::test]
    async fn cors_does_not_allow_arbitrary_origins() {
        let (app, _tmp) = app_with_config(Some("test-token"));
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/healthz")
                    .header(header::ORIGIN, "https://attacker.example")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }

    #[test]
    fn non_loopback_bind_without_auth_fails_fast() {
        let options = AppServerOptions {
            listen: "0.0.0.0:8787".parse().expect("socket addr"),
            config_path: None,
            auth_token: None,
            insecure_no_auth: true,
            cors_origins: Vec::new(),
        };

        let err = resolve_auth_token(&options).expect_err("non-loopback unauth should fail");
        assert!(
            err.to_string()
                .contains("refusing unauthenticated app-server bind")
        );
    }

    #[tokio::test]
    async fn stdio_transport_keeps_raw_config_get_for_legacy_clients() {
        let state = build_state(None, None).expect("state");
        {
            let mut cfg = state.config.write().await;
            cfg.api_key = Some("sk-deepseek-secret".to_string());
        }

        let response = process_app_request(
            &state,
            AppRequest::ConfigGet {
                key: "api_key".to_string(),
            },
            AppTransport::Stdio,
        )
        .await;

        assert_eq!(response.data["value"], "sk-deepseek-secret");
    }

    // ── resolve_auth_token ─────────────────────────────────────────────

    #[test]
    fn auth_token_empty_string_fails() {
        let options = AppServerOptions {
            listen: "127.0.0.1:0".parse().expect("addr"),
            config_path: None,
            auth_token: Some("  ".to_string()),
            insecure_no_auth: false,
            cors_origins: Vec::new(),
        };
        let err = resolve_auth_token(&options).expect_err("empty token should fail");
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn auth_token_generated_when_none_provided() {
        let options = AppServerOptions {
            listen: "127.0.0.1:0".parse().expect("addr"),
            config_path: None,
            auth_token: None,
            insecure_no_auth: false,
            cors_origins: Vec::new(),
        };
        let token = resolve_auth_token(&options).unwrap();
        assert!(token.is_some());
        assert!(token.unwrap().starts_with("dwapp_"));
    }

    #[test]
    fn auth_token_explicit_is_preserved() {
        let options = AppServerOptions {
            listen: "127.0.0.1:0".parse().expect("addr"),
            config_path: None,
            auth_token: Some("my-secret".to_string()),
            insecure_no_auth: false,
            cors_origins: Vec::new(),
        };
        let token = resolve_auth_token(&options).unwrap();
        assert_eq!(token.as_deref(), Some("my-secret"));
    }

    #[test]
    fn insecure_no_auth_on_loopback_returns_none() {
        let options = AppServerOptions {
            listen: "127.0.0.1:0".parse().expect("addr"),
            config_path: None,
            auth_token: None,
            insecure_no_auth: true,
            cors_origins: Vec::new(),
        };
        let token = resolve_auth_token(&options).unwrap();
        assert!(token.is_none());
    }

    // ── cors_layer ─────────────────────────────────────────────────────

    #[test]
    fn cors_layer_includes_default_origins() {
        let layer = cors_layer(&[]);
        let _ = layer;
    }

    #[test]
    fn cors_layer_adds_extra_origins() {
        let extras = vec!["https://example.com".to_string()];
        let layer = cors_layer(&extras);
        let _ = layer;
    }

    #[test]
    fn cors_layer_skips_empty_origins() {
        let extras = vec!["".to_string(), "  ".to_string()];
        let layer = cors_layer(&extras);
        let _ = layer;
    }

    // ── JsonRpc helpers ────────────────────────────────────────────────

    #[test]
    fn params_or_object_returns_object_for_null() {
        let result = params_or_object(Value::Null);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn params_or_object_passthrough_for_non_null() {
        let input = json!({"key": "value"});
        let result = params_or_object(input.clone());
        assert_eq!(result, input);
    }

    #[test]
    fn jsonrpc_result_format() {
        let result = jsonrpc_result(Some(json!(1)), json!({"ok": true}));
        assert_eq!(result["jsonrpc"], "2.0");
        assert_eq!(result["id"], 1);
        assert_eq!(result["result"]["ok"], true);
    }

    #[test]
    fn jsonrpc_result_null_id() {
        let result = jsonrpc_result(None, json!(null));
        assert_eq!(result["id"], Value::Null);
    }

    #[test]
    fn jsonrpc_error_format() {
        let err = jsonrpc_error(Some(json!(2)), JsonRpcError::internal("oops"));
        assert_eq!(err["jsonrpc"], "2.0");
        assert_eq!(err["id"], 2);
        assert_eq!(err["error"]["code"], -32603);
        assert_eq!(err["error"]["message"], "oops");
    }

    #[test]
    fn jsonrpc_error_codes() {
        assert_eq!(JsonRpcError::parse_error("").code, -32700);
        assert_eq!(JsonRpcError::invalid_request("").code, -32600);
        assert_eq!(JsonRpcError::method_not_found("x").code, -32601);
        assert_eq!(JsonRpcError::invalid_params("").code, -32602);
        assert_eq!(JsonRpcError::internal("").code, -32603);
    }

    // ── AppServerOptions ───────────────────────────────────────────────

    #[test]
    fn app_server_options_debug_does_not_leak_token() {
        let options = AppServerOptions {
            listen: "127.0.0.1:8080".parse().expect("addr"),
            config_path: None,
            auth_token: Some("secret-token".to_string()),
            insecure_no_auth: false,
            cors_origins: vec!["https://example.com".to_string()],
        };
        let debug = format!("{options:?}");
        assert!(!debug.contains("secret-token"));
        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("8080"));
    }

    // ── Default CORS origins ──────────────────────────────────────────

    #[test]
    fn default_cors_origins_include_common_dev_ports() {
        assert!(DEFAULT_CORS_ORIGINS.contains(&"http://localhost:3000"));
        assert!(DEFAULT_CORS_ORIGINS.contains(&"http://localhost:5173"));
        assert!(DEFAULT_CORS_ORIGINS.contains(&"tauri://localhost"));
    }
}
