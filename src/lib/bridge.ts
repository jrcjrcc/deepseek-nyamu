/**
 * IPC 桥接层 —— 前端与 Tauri Rust 后端的通信枢纽
 *
 * 职责：
 * 1. 封装所有 @tauri-apps/api 的 invoke() 调用（命令式通信）
 * 2. 封装所有 listen() 事件监听（流式推送：token、推理、工具调用状态）
 * 3. 定义前后端共享的数据类型（Message、ToolCall、SessionInfo 等）
 *
 * 设计原则：前端组件只依赖本模块，不直接引用 Tauri API，
 * 使得后续迁移到其他桌面框架（如 Electron）时只需替换本文件。
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/* ─── 数据类型定义 ─────────────────────────────────────────────── */

/** 会话元信息 */
export interface SessionInfo {
  id: string;               // 会话唯一标识（UUID）
  title: string;            // 会话标题（用户可自定义）
  created_at: string;       // 创建时间（RFC3339 格式）
  message_count: number;    // 消息条数
}

/** 工具调用卡片（用于在聊天中展示工具执行状态） */
export interface ToolCall {
  name: string;             // 工具名称（如 read_file, exec_shell）
  input: string;            // 输入参数（JSON 字符串）
  output?: string;          // 执行结果
  status: string;           // 状态: "running" | "completed" | "failed"
}

/** 聊天消息（包含用户消息和助手回复） */
export interface Message {
  role: string;                 // "user" | "assistant" | "tool_call"
  content: string;              // 消息文本内容
  timestamp: string;            // 时间戳
  tool_calls?: ToolCall[];      // 关联的工具调用（assistant/tool_call 消息）
  reasoning?: string;           // 推理链/思维过程（assistant 消息）
}

/* ─── 命令式 IPC（请求-响应模式） ─────────────────────────────── */

/** 发送消息到 AI 并获取回复 */
export async function submitMessage(
  sessionId: string,
  content: string,
  mode?: string,
): Promise<string> {
  return invoke<string>("submit_message", {
    sessionId,
    content,
    mode: mode || null,
  });
}

/** 直接执行 Shell 命令（对应前端的 ! 前缀语法） */
export async function execShellDirect(
  command: string,
  cwd?: string,
): Promise<string> {
  return invoke<string>("exec_shell_direct", { command, cwd: cwd || null });
}

/** 获取所有会话列表 */
export async function getSessions(): Promise<SessionInfo[]> {
  return invoke<SessionInfo[]>("get_sessions");
}

/** 获取指定会话的消息历史 */
export async function getConversation(
  sessionId: string,
): Promise<Message[]> {
  return invoke<Message[]>("get_conversation", {
    sessionId,
  });
}

/** 创建新会话 */
export async function newSession(
  title: string,
): Promise<SessionInfo> {
  return invoke<SessionInfo>("new_session", { title });
}

/** 读取当前配置（JSON 格式） */
export async function getConfig(): Promise<string> {
  return invoke<string>("get_config");
}

/** 更新配置项（以 JSON 键值对形式传入） */
export async function updateConfig(
  configJson: string,
): Promise<void> {
  return invoke<void>("update_config", { configJson });
}

/* ─── 事件类型定义（流式推送） ───────────────────────────────── */

/** 工具调用开始事件（后端开始执行某个工具） */
export interface ToolStartEvent {
  id: string;
  name: string;
  arguments: string;
}

/** 工具调用结束事件（后端执行完毕，带回结果） */
export interface ToolEndEvent {
  id: string;
  name: string;
  output: string;
  success: boolean;
}

/** 监听工具调用开始事件 */
export function onToolStart(
  callback: (event: ToolStartEvent) => void,
): Promise<UnlistenFn> {
  return listen<ToolStartEvent>("tool:start", (e) => callback(e.payload));
}

/** 监听工具调用结束事件 */
export function onToolEnd(
  callback: (event: ToolEndEvent) => void,
): Promise<UnlistenFn> {
  return listen<ToolEndEvent>("tool:end", (e) => callback(e.payload));
}

/** Token 流事件（AI 逐 Token 生成回复） */
export interface TokenEvent {
  token: string;  // 本次推送的单个 Token
  full: string;   // 到目前为止的完整回复文本
}

/** 监听 AI 生成 Token 流 */
export function onToken(
  callback: (event: TokenEvent) => void,
): Promise<UnlistenFn> {
  return listen<TokenEvent>("token", (e) => callback(e.payload));
}

/** 推理 Token 流事件（AI 的思考过程，逐 Token 推送） */
export interface ReasoningEvent {
  token: string;
  full: string;
}

/** 监听推理/思考过程流 */
export function onReasoning(
  callback: (event: ReasoningEvent) => void,
): Promise<UnlistenFn> {
  return listen<ReasoningEvent>("reasoning", (e) => callback(e.payload));
}

/** 监听推理结束信号（此时 full 包含完整推理内容） */
export function onReasoningDone(
  callback: (full: string) => void,
): Promise<UnlistenFn> {
  return listen<{full: string}>("reasoning:done", (e) => callback(e.payload.full));
}

/* ─── 用量/费用事件 ──────────────────────────────────────────── */

export interface UsageEvent {
  input_tokens: number;
  output_tokens: number;
  cache_hit_tokens: number;
  cache_miss_tokens: number;
  cost_dollars: number;
}

/** 监听 Token 用量和费用事件 */
export function onUsage(
  callback: (event: UsageEvent) => void,
): Promise<UnlistenFn> {
  return listen<UsageEvent>("usage", (e) => callback(e.payload));
}

/** 当前会话的聚合用量统计 */
export interface SessionUsage {
  input_tokens: number;
  output_tokens: number;
  total_cost: number;
  cache_hit_rate: number;   // 缓存命中率（0.0 ~ 1.0）
}

/** 获取当前会话的聚合用量 */
export async function getSessionUsage(): Promise<SessionUsage> {
  return invoke<SessionUsage>("get_session_usage");
}

/** 重命名会话 */
export async function renameSession(sessionId: string, title: string): Promise<void> {
  return invoke<void>("rename_session", { sessionId, title });
}

/** 删除会话 */
export async function deleteSession(sessionId: string): Promise<void> {
  return invoke<void>("delete_session", { sessionId });
}

/** 读取记忆文件内容 */
export async function getMemory(): Promise<string> {
  return invoke<string>("get_memory");
}

/** 获取所有子代理状态列表 */
export async function getSubagents(): Promise<any[]> {
  return invoke<any[]>("get_subagents");
}

/* ─── 模型/Effort 切换 ────────────────────────────────── */

export interface ModelInfo {
  name: string;
  active: boolean;
}

/** 获取可用模型列表 */
export async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("list_models");
}

/** 切换当前模型 */
export async function setModel(modelName: string): Promise<void> {
  return invoke<void>("set_model", { modelName });
}

/** 设置推理力度 */
export async function setEffort(effort: string): Promise<void> {
  return invoke<void>("set_effort", { effort });
}

/* ─── 文件浏览 ────────────────────────────────────────── */

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  children?: FileEntry[];
}

/** 递归扫描目录结构 */
export async function listDirectoryTree(path: string): Promise<{ path: string; entries: FileEntry[] }> {
  return invoke<any>("list_directory_tree", { path });
}

/** 读取文件内容 */
export async function readFileContent(path: string): Promise<string> {
  return invoke<string>("read_file_content", { path });
}

/** 获取当前会话的文件变更快照列表 */
export async function getSessionChanges(): Promise<any[]> {
  return invoke<any[]>("get_session_changes");
}

/* ─── 设置管理 ────────────────────────────────────────── */

/** 检查是否需要首次运行引导 */
export async function needsOnboarding(): Promise<boolean> {
  return invoke<boolean>("needs_onboarding");
}

/** 设置 API Key */
export async function connectKey(apiKey: string): Promise<void> {
  return invoke<void>("connect_key", { apiKey: apiKey });
}

/* ─── Slash 命令后端接口 ──────────────────────────────── */

/** 获取系统提示词 */
export async function getSystemPrompt(): Promise<string> {
  return invoke<string>("get_system_prompt");
}

/** 清除当前会话上下文 (compact) */
export async function purgeContext(): Promise<void> {
  return invoke<void>("purge_context");
}

/** 检查提供商余额 */
export async function getBalance(): Promise<string> {
  return invoke<string>("get_balance");
}

/** 获取缓存命中统计 */
export async function getCacheStats(): Promise<any> {
  return invoke<any>("get_cache_stats");
}

/** 导出会话到 Markdown */
export async function exportSession(sessionId: string, path: string | null): Promise<string> {
  return invoke<string>("export_session", { sessionId, path });
}

/** 保存会话到文件 */
export async function saveSessionFile(path: string): Promise<void> {
  return invoke<void>("save_session_file", { path });
}

/** 从文件加载会话 */
export async function loadSessionFile(path: string): Promise<void> {
  return invoke<void>("load_session_file", { path });
}

/** 切换 LSP 诊断 */
export async function toggleLsp(enabled: boolean): Promise<void> {
  return invoke<void>("toggle_lsp", { enabled });
}

/** 获取工作区文件变更 diff */
export async function getWorkspaceDiff(): Promise<string> {
  return invoke<string>("get_workspace_diff");
}

/** 获取工作区信息 */
export async function getWorkspaceInfo(): Promise<any> {
  return invoke<any>("get_workspace_info");
}

/** 列出可用技能 */
export async function listSkills(): Promise<any[]> {
  return invoke<any[]>("list_skills");
}

/* ─── 计划管理 ──────────────────────────────────────────── */

/** plan:updated 事件 payload */
export interface PlanUpdatedEvent {
  plan: any;
}

/** 获取所有会话计划 */
export async function getPlans(): Promise<any[]> {
  return invoke<any[]>("get_session_plans");
}

/** 获取最新计划 */
export async function getLatestPlan(): Promise<any> {
  return invoke<any>("get_latest_plan");
}

/** 监听计划更新事件 */
export function onPlanUpdated(
  callback: (event: PlanUpdatedEvent) => void,
): Promise<UnlistenFn> {
  return listen<PlanUpdatedEvent>("plan:updated", (e) => callback(e.payload));
}

/* ─── 通知事件 ───────────────────────────────────────────── */

export interface NotificationEvent {
  title: string;
  body: string;
}

/** 监听桌面通知事件（后端长时间工具完成时触发） */
export function onNotification(
  callback: (event: NotificationEvent) => void,
): Promise<UnlistenFn> {
  return listen<NotificationEvent>("notification", (e) => callback(e.payload));
}

/* ─── 剪贴板 ─────────────────────────────────────────────── */

/** 写入文本到系统剪贴板（优先使用 Web API，回退到 Tauri 插件） */
export async function writeClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // Fallback: use Tauri clipboard-manager plugin via invoke
    try {
      await invoke("plugin:clipboard-manager|write_text", { text, label: "" });
    } catch {
      // Last resort: try exec command
    }
  }
}
