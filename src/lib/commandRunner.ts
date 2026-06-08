/**
 * commandRunner.ts —— 命令执行引擎
 *
 * 将 commands.ts 中的命令分派到三种处理方式：
 * - frontend：直接在前端执行（setState、调用 App 回调等）
 * - bridge：通过 Tauri IPC 调用 Rust 后端
 * - engine：作为普通用户消息发送给 AI 处理
 */

import * as bridge from "./bridge";
import { findCommandById } from "./commands";

export interface CommandContext {
  // 消息操作
  messages: any[];
  setMessages: (msgs: any[] | ((prev: any[]) => any[])) => void;
  onSubmit: (content: string) => void;

  // 模式
  mode: string;
  setMode: (mode: string | ((prev: string) => string)) => void;

  // UI 面板
  setShowSettings: (v: boolean | ((prev: boolean) => boolean)) => void;
  setShowHistory: (v: boolean | ((prev: boolean) => boolean)) => void;
  setShowCommandPalette: (v: boolean | ((prev: boolean) => boolean)) => void;

  // 会话
  handleNewSession: () => void;
  sessions: any[];
  activeSession: string | null;
  handleSelectSession: (id: string) => void;

  // 主题
  themeState?: { mode: string; style: string };
  setThemeState?: (state: { mode: string; style: string }) => void;

  // Composer 编辑
  setInput?: (text: string) => void;

  // 通知
  notify?: (msg: string) => void;
}

/**
 * 构建命令调度器
 * 返回 (id: string, args: string) => Promise<void> 函数
 */
export function buildCommandDispatcher(ctx: CommandContext) {
  return async (id: string, args: string): Promise<string | void> => {
    const cmd = findCommandById(id);
    if (!cmd) {
      // 命令未找到，当作普通消息发送
      ctx.onSubmit(`/${id} ${args}`.trim());
      return;
    }

    switch (cmd.handler) {
      /* ─── Frontend 直接处理 ──────────────────────────── */
      case "frontend": {
        switch (cmd.id) {
          case "help":
            // 触发 CommandPalette 或显示帮助
            ctx.setShowCommandPalette(true);
            break;

          case "clear":
            ctx.setMessages([]);
            break;

          case "exit":
            try {
              const { getCurrentWindow } = await import("@tauri-apps/api/window");
              await getCurrentWindow().close();
            } catch {
              ctx.notify?.("Exit is only available in the desktop app");
            }
            break;

          case "new":
            ctx.handleNewSession();
            break;

          case "fork": {
            // 分支：用最后一条消息创建新会话
            const lastMsg = ctx.messages[ctx.messages.length - 1];
            ctx.handleNewSession();
            break;
          }

          case "sessions":
            ctx.setShowHistory(true);
            break;

          case "mode": {
            const valid = ["agent", "plan", "yolo"];
            const target = valid.includes(args.trim()) ? args.trim() : "";
            if (target) {
              ctx.setMode(target);
            } else {
              ctx.setMode((prev: string) =>
                prev === "agent" ? "plan" : prev === "plan" ? "yolo" : "agent"
              );
            }
            break;
          }

          case "theme":
            if (ctx.setThemeState && ctx.themeState) {
              const { loadTheme, saveTheme, applyTheme } = await import("./theme");
              if (args.trim()) {
                const styles = ["pastel", "graphite", "ember", "aurora", "midnight", "sandstone", "porcelain", "glacier"];
                const found = styles.find(s => s.startsWith(args.trim().toLowerCase()));
                if (found) {
                  const next = { ...ctx.themeState, style: found } as any;
                  ctx.setThemeState(next);
                  saveTheme(next);
                  applyTheme(next);
                }
              } else {
                ctx.setShowSettings(true);
              }
            }
            break;

          case "verbose": {
            const on = args.trim().toLowerCase();
            if (on === "on" || on === "off") {
              localStorage.setItem("nyamuwhale-verbose", on);
            }
            break;
          }

          case "settings":
            ctx.setShowSettings(true);
            break;

          case "subagents": {
            // 显示状态：通过 notify 返回给用户
            try {
              const agents = await bridge.getSubagents();
              const count = agents.length;
              ctx.notify?.(`Sub-agents: ${count} active`);
            } catch {
              ctx.notify?.("Unable to fetch sub-agents");
            }
            break;
          }

          case "workspace":
            ctx.notify?.("Current workspace: " + await getWorkspacePath());
            break;

          case "links":
            ctx.notify?.("Links: https://platform.deepseek.com/");
            break;

          case "home":
            // 显示会话列表
            ctx.setShowHistory(true);
            break;

          case "logout":
            try {
              await bridge.connectKey("");
              ctx.notify?.("API key cleared. Restart to reconfigure.");
            } catch { /* ignore */ }
            break;

          case "status":
            ctx.notify?.(`Session: ${ctx.activeSession || "none"} | Mode: ${ctx.mode} | Sessions: ${ctx.sessions.length}`);
            break;

          case "undo": {
            // 移除最后两条消息（user + assistant 对）
            ctx.setMessages((prev: any[]) => {
              if (prev.length < 2) return [];
              return prev.slice(0, -2);
            });
            break;
          }

          case "retry": {
            // 找到最后一条用户消息重新发送
            const lastUserIdx = findLastIndex(ctx.messages, (m: any) => m.role === "user");
            if (lastUserIdx >= 0) {
              const lastUserMsg = ctx.messages[lastUserIdx].content;
              ctx.setMessages((prev: any[]) => prev.slice(0, lastUserIdx));
              // 延迟发送以清除 assistant 响应
              setTimeout(() => ctx.onSubmit(lastUserMsg), 50);
            }
            break;
          }

          case "edit": {
            // 将最后一条用户消息填入 composer
            if (ctx.setInput) {
              const lastUser = [...ctx.messages].reverse().find((m: any) => m.role === "user");
              if (lastUser) {
                ctx.setInput(lastUser.content);
                // 移除最后的消息对
                ctx.setMessages((prev: any[]) => {
                  const lastUserIdx = findLastIndex(prev, (m: any) => m.role === "user");
                  if (lastUserIdx >= 0) return prev.slice(0, lastUserIdx);
                  return prev;
                });
              }
            }
            break;
          }

          case "translate": {
            const current = localStorage.getItem("nyamuwhale-translate");
            const next = current === "on" ? "off" : "on";
            localStorage.setItem("nyamuwhale-translate", next);
            ctx.notify?.(`Translate: ${next}`);
            break;
          }

          case "statusline":
            ctx.notify?.("Status line items are configured in settings.");
            ctx.setShowSettings(true);
            break;

          case "context":
            ctx.notify?.(`Messages: ${ctx.messages.length} | Mode: ${ctx.mode}`);
            break;

          case "feedback":
            window.open("https://github.com/anthropics/claude-code/issues", "_blank");
            break;

          default:
            // 未知的 frontend 命令，当作消息发送
            ctx.onSubmit(`/${cmd.id} ${args}`.trim());
        }
        break;
      }

      /* ─── Bridge IPC ────────────────────────────────── */
      case "bridge": {
        switch (cmd.id) {
          case "model": {
            const name = args.trim();
            if (name) {
              try { await bridge.setModel(name); } catch (e: any) {
                ctx.notify?.(`Failed to switch model: ${e}`);
              }
            } else {
              const models = await bridge.listModels();
              const active = models.find(m => m.active);
              ctx.notify?.(`Current model: ${active?.name || "unknown"}`);
            }
            break;
          }

          case "effort": {
            const level = args.trim();
            if (["off", "low", "medium", "high", "max"].includes(level)) {
              try { await bridge.setEffort(level); } catch (e: any) {
                ctx.notify?.(`Failed to set effort: ${e}`);
              }
            } else {
              ctx.notify?.("Usage: /effort <off|low|medium|high|max>");
            }
            break;
          }

          case "memory": {
            try {
              const content = await bridge.getMemory();
              ctx.notify?.(content || "(empty memory)");
            } catch {
              ctx.notify?.("Unable to read memory.");
            }
            break;
          }

          case "config": {
            try {
              const cfg = await bridge.getConfig();
              ctx.notify?.(cfg.slice(0, 500));
            } catch { ctx.notify?.("Unable to read config."); }
            break;
          }

          case "tokens":
          case "cost": {
            try {
              const usage = await bridge.getSessionUsage();
              ctx.notify?.(`Input: ${usage.input_tokens} | Output: ${usage.output_tokens} | Cost: $${usage.total_cost.toFixed(4)} | Cache: ${(usage.cache_hit_rate * 100).toFixed(0)}%`);
            } catch { ctx.notify?.("Unable to fetch usage."); }
            break;
          }

          case "rename": {
            const title = args.trim();
            if (title && ctx.activeSession) {
              try { await bridge.renameSession(ctx.activeSession, title); } catch { ctx.notify?.("Failed to rename."); }
            } else {
              ctx.notify?.("Usage: /rename <new title>");
            }
            break;
          }

          case "save": {
            if (args.trim()) {
              try { await bridge.saveSessionFile(args.trim()); } catch { ctx.notify?.("Failed to save."); }
            } else {
              ctx.notify?.("Usage: /save <path>");
            }
            break;
          }

          case "compact": {
            try { await bridge.purgeContext(); ctx.notify?.("Context compacted."); } catch { ctx.notify?.("Compact failed."); }
            break;
          }

          case "export": {
            try {
              const result = await bridge.exportSession(ctx.activeSession || "", args.trim() || null);
              ctx.notify?.(result);
            } catch { ctx.notify?.("Export failed."); }
            break;
          }

          case "load":
            ctx.notify?.("Use /load <path> to load a session file.");
            break;

          case "diff": {
            try {
              const diff = await bridge.getWorkspaceDiff();
              ctx.notify?.(diff || "(no changes)");
            } catch { ctx.notify?.("Unable to get diff."); }
            break;
          }

          case "system": {
            try {
              const prompt = await bridge.getSystemPrompt();
              ctx.notify?.(prompt.slice(0, 500) || "(empty system prompt)");
            } catch { ctx.notify?.("Unable to get system prompt."); }
            break;
          }

          case "balance": {
            try {
              const bal = await bridge.getBalance();
              ctx.notify?.(bal);
            } catch { ctx.notify?.("Unable to check balance."); }
            break;
          }

          case "cache": {
            try {
              const stats = await bridge.getCacheStats();
              const s = typeof stats === "object" ? JSON.stringify(stats, null, 2) : String(stats);
              ctx.notify?.(s.slice(0, 500));
            } catch { ctx.notify?.("Unable to get cache stats."); }
            break;
          }

          case "lsp": {
            const on = args.trim().toLowerCase();
            if (on === "on" || on === "off") {
              try { await bridge.toggleLsp(on === "on"); } catch { ctx.notify?.("LSP toggle failed."); }
            } else {
              ctx.notify?.("Usage: /lsp [on|off]");
            }
            break;
          }

          case "skills": {
            try {
              const skills = await bridge.listSkills();
              const names = skills.map((s: any) => s.name).join(", ");
              ctx.notify?.(`Skills: ${names || "(none)"}`);
            } catch { ctx.notify?.("Unable to list skills."); }
            break;
          }

          case "change":
            ctx.notify?.("nyamuWhale v0.8.53");
            break;

          default:
            ctx.onSubmit(`/${cmd.id} ${args}`.trim());
        }
        break;
      }

      /* ─── Engine (发送给 AI) ────────────────────────── */
      case "engine": {
        ctx.onSubmit(`/${cmd.id} ${args}`.trim());
        break;
      }
    }
  };
}

/** 辅助：从右向左查找符合条件的索引 */
function findLastIndex<T>(arr: T[], predicate: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (predicate(arr[i])) return i;
  }
  return -1;
}

/** 获取 workspace 路径 */
async function getWorkspacePath(): Promise<string> {
  try {
    const info = await bridge.getWorkspaceInfo();
    return info.path || "(not set)";
  } catch {
    return "(unknown)";
  }
}
