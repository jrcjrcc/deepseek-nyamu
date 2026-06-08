/**
 * App —— 应用根组件（增强版）
 *
 * 新增功能：
 * - 主题系统（light/dark/auto + 8调色板）
 * - OnboardingWizard 首次运行引导
 * - SettingsPanel 完整设置面板
 * - TodoPanel 置顶任务列表
 * - WorkspacePanel 文件浏览
 */
import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import { Composer } from "./components/Composer";
import { Transcript } from "./components/Transcript";
import { RightSidebar } from "./components/RightSidebar";
import { LeftSidebar } from "./components/LeftSidebar";
import { StatusBar } from "./components/StatusBar";
import { ModeBar } from "./components/ModeBar";
import { CommandPalette } from "./components/CommandPalette";
import { TodoPanel, type TodoItem } from "./components/TodoPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { OnboardingWizard } from "./components/OnboardingWizard";
import { Settings, PanelRight } from "lucide-react";
import type { Message, SessionInfo, ToolStartEvent, ToolEndEvent, TokenEvent, ReasoningEvent, ToolCall, NotificationEvent } from "./lib/bridge";
import type { Mode } from "./components/ModeBar";
import * as bridge from "./lib/bridge";
import { loadTheme, applyTheme } from "./lib/theme";
import { buildCommandDispatcher } from "./lib/commandRunner";

export default function App() {
  /* ─── 核心状态 ────────────────────────────────────────────────── */
  const [messages, setMessages] = useState<Message[]>([]);
  const [isWaiting, setIsWaiting] = useState(false);
  const [showHistory, setShowHistory] = useState(true);
  const [showRightSidebar, setShowRightSidebar] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [currentReasoning, setCurrentReasoning] = useState("");
  const [activeTools, setActiveTools] = useState<ToolCall[]>([]);
  const [mode, setMode] = useState<Mode>("agent");
  const [todoItems, setTodoItems] = useState<TodoItem[]>([]);
  const pendingToolsRef = useRef<Map<string, Message>>(new Map());

  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [activeSession, setActiveSession] = useState<string | null>(null);

  /* ─── 初始化：主题 + 引导检测 ─────────────────────────── */
  useEffect(() => {
    // Apply saved theme immediately
    const saved = loadTheme();
    applyTheme(saved);

    // Check if onboarding is needed
    bridge.needsOnboarding().then((needed) => {
      setShowOnboarding(needed);
    }).catch(() => {});
  }, []);

  /* ─── 主题状态（供 commandRunner 使用） ─────────────── */
  const [themeState, setThemeState] = useState<{ mode: string; style: string }>(() => loadTheme());

  /* ─── 键盘快捷键 ──────────────────────────────────────── */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.startsWith("Mac");
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      if (modKey && e.key === "k") {
        e.preventDefault();
        setShowCommandPalette((p) => !p);
        return;
      }
      if (modKey && e.key === "r") {
        e.preventDefault();
        if (sessions.length > 0 && activeSession !== sessions[0].id) {
          setActiveSession(sessions[0].id);
          bridge.getConversation(sessions[0].id).then((msgs) => {
            setMessages(msgs.map((m: any) => ({
              role: m.role, content: m.content, timestamp: m.timestamp,
              tool_calls: m.tool_calls, reasoning: m.reasoning,
            })));
          }).catch(() => {});
        }
        return;
      }
      if (e.key === "F1") {
        e.preventDefault();
        setShowCommandPalette((p) => !p);
        return;
      }
      if (e.key === "Escape") {
        if (showCommandPalette) { setShowCommandPalette(false); return; }
        if (showSettings) { setShowSettings(false); return; }
      }
      if (modKey && e.key === "n") {
        e.preventDefault();
        bridge.newSession("New Chat").then((session) => {
          setActiveSession(session.id);
          setMessages([]);
          setSessions((prev) => [...prev, session]);
        }).catch(() => {});
        return;
      }
      if (modKey && e.key === ",") {
        e.preventDefault();
        setShowSettings((p) => !p);
        return;
      }
      if (modKey && e.shiftKey && e.key === "H") {
        e.preventDefault();
        setShowHistory((p) => !p);
        return;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showCommandPalette, showSettings, sessions, activeSession]);

  /* ─── 事件监听 ────────────────────────────────────────── */
  useEffect(() => {
    bridge.getSessions().then((s) => {
      setSessions(s);
      if (s.length > 0) {
        setActiveSession(s[0].id);
        bridge.getConversation(s[0].id).then((msgs) => {
          setMessages(msgs.map((m: any) => ({
            role: m.role, content: m.content, timestamp: m.timestamp,
            tool_calls: m.tool_calls, reasoning: m.reasoning,
          })));
        }).catch(() => {});
      }
    }).catch(() => {});

    const unlisteners: (() => void)[] = [];

    bridge.onToolStart((event: ToolStartEvent) => {
      const toolMsg: Message = {
        role: "tool_call",
        content: `**Tool: ${event.name}**`,
        timestamp: new Date().toISOString(),
        tool_calls: [{ name: event.name, input: event.arguments, output: undefined, status: "running" }],
      };
      pendingToolsRef.current.set(event.id, toolMsg);
      setMessages((prev) => [...prev, toolMsg]);
      setActiveTools((prev) => [...prev, { name: event.name, input: event.arguments, status: "running" }]);
    }).then((u) => unlisteners.push(u));

    bridge.onToolEnd((event: ToolEndEvent) => {
      setMessages((prev) => {
        const updated = [...prev];
        for (let i = updated.length - 1; i >= 0; i--) {
          const tc = updated[i].tool_calls?.[0];
          if (tc && tc.name === event.name && tc.status === "running") {
            tc.status = event.success ? "completed" : "failed";
            tc.output = event.output;
            updated[i] = { ...updated[i], content: `**Tool: ${event.name}** ${event.success ? '✓' : '✗'}` };
            break;
          }
        }
        return updated;
      });
      pendingToolsRef.current.delete(event.id);
      setActiveTools((prev) => prev.map((t) =>
        t.name === event.name && t.status === "running"
          ? { ...t, status: event.success ? "completed" : "failed", output: event.output }
          : t
      ));
    }).then((u) => unlisteners.push(u));

    bridge.onReasoning((event: ReasoningEvent) => {
      setCurrentReasoning(event.full);
    }).then((u) => unlisteners.push(u));

    bridge.onReasoningDone((full: string) => {
      setMessages((prev) => {
        if (prev.length === 0) {
          return [{ role: "assistant", content: "", timestamp: new Date().toISOString(), reasoning: full }];
        }
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last.role === "assistant") {
          updated[updated.length - 1] = { ...last, reasoning: full };
        } else {
          updated.push({ role: "assistant", content: "", timestamp: new Date().toISOString(), reasoning: full });
        }
        return updated;
      });
      setCurrentReasoning("");
    }).then((u) => unlisteners.push(u));

    bridge.onToken((event: TokenEvent) => {
      setMessages((prev) => {
        if (prev.length === 0) {
          return [{ role: "assistant", content: event.full, timestamp: new Date().toISOString() }];
        }
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last.role === "assistant" && !last.tool_calls) {
          updated[updated.length - 1] = { ...last, content: event.full };
        } else {
          updated.push({ role: "assistant", content: event.full, timestamp: new Date().toISOString() });
        }
        return updated;
      });
    }).then((u) => unlisteners.push(u));

    bridge.onNotification((event: NotificationEvent) => {
      if ("Notification" in window && Notification.permission === "granted") {
        new Notification(event.title, { body: event.body });
      }
    }).then((u) => unlisteners.push(u));

    return () => { unlisteners.forEach((u) => u()); };
  }, []);

  /* ─── 消息提交 ────────────────────────────────────────── */
  const handleSubmit = useCallback(async (content: string) => {
    if (!content.trim() || isWaiting) return;

    const trimmed = content.trim();
    if (trimmed.startsWith("! ")) {
      // ... shell command handling
      const command = trimmed.slice(2).trim();
      if (!command) return;
      setIsWaiting(true);
      setMessages((prev) => [...prev, {
        role: "user", content: `\`! ${command}\``, timestamp: new Date().toISOString(),
      }]);
      const shellMsg: Message = {
        role: "tool_call", content: `Running: \`${command}\``,
        timestamp: new Date().toISOString(),
        tool_calls: [{ name: "shell", input: command, output: undefined, status: "running" }],
      };
      setMessages((prev) => [...prev, shellMsg]);
      try {
        const output = await bridge.execShellDirect(command);
        setMessages((prev) => {
          const updated = [...prev];
          for (let i = updated.length - 1; i >= 0; i--) {
            const tc = updated[i].tool_calls?.[0];
            if (tc && tc.name === "shell" && tc.status === "running") {
              tc.status = "completed"; tc.output = output;
              const truncated = output.length > 2000 ? output.slice(0, 2000) + "\n... [truncated]" : output;
              updated[i] = { ...updated[i], content: `\`\`\`\n${truncated}\n\`\`\`` };
              break;
            }
          }
          return updated;
        });
      } catch (e: any) {
        setMessages((prev) => {
          const updated = [...prev];
          for (let i = updated.length - 1; i >= 0; i--) {
            const tc = updated[i].tool_calls?.[0];
            if (tc && tc.name === "shell" && tc.status === "running") {
              tc.status = "failed"; tc.output = String(e);
              updated[i] = { ...updated[i], content: `**Error:** ${e}` };
              break;
            }
          }
          return updated;
        });
      } finally { setIsWaiting(false); }
      return;
    }

    setIsWaiting(true);
    setMessages((prev) => [...prev, {
      role: "user", content, timestamp: new Date().toISOString(),
    }]);
    try {
      const sessionId = activeSession || "default";
      await bridge.submitMessage(sessionId, content, mode);
    } catch (e: any) {
      setMessages((prev) => [...prev, {
        role: "assistant", content: `Error: ${e}`, timestamp: new Date().toISOString(),
      }]);
    } finally { setIsWaiting(false); }
  }, [isWaiting, activeSession, mode]);

  /* ─── 会话管理 ────────────────────────────────────────── */
  const handleSelectSession = useCallback(async (id: string) => {
    setActiveSession(id);
    bridge.getConversation(id).then((msgs) => {
      setMessages(msgs.map((m: any) => ({
        role: m.role, content: m.content, timestamp: m.timestamp,
        tool_calls: m.tool_calls, reasoning: m.reasoning,
      })));
    }).catch(() => setMessages([]));
  }, []);

  const handleNewSession = useCallback(async () => {
    const session = await bridge.newSession("New Chat");
    setActiveSession(session.id);
    setMessages([]);
    setSessions((prev) => [...prev, session]);
  }, []);

  const handleRenameSession = useCallback(async (id: string, title: string) => {
    await bridge.renameSession(id, title);
    setSessions((prev) => prev.map((s) => s.id === id ? { ...s, title } : s));
  }, []);

  const handleDeleteSession = useCallback(async (id: string) => {
    await bridge.deleteSession(id);
    setSessions((prev) => prev.filter((s) => s.id !== id));
    if (activeSession === id) { setMessages([]); setActiveSession(null); }
  }, [activeSession]);

  const handleDeleteMessage = useCallback((index: number) => {
    setMessages((prev) => prev.filter((_, i) => i !== index));
  }, []);

  /* ─── 命令调度器 ────────────────────────────────────── */
  // Use a ref to avoid ordering dependency with handleSubmit
  const handleSubmitRef = useRef(handleSubmit);
  handleSubmitRef.current = handleSubmit;

  const executeCommand = useMemo(() => buildCommandDispatcher({
    messages,
    setMessages,
    onSubmit: (content: string) => handleSubmitRef.current(content),
    mode,
    setMode: (m: any) => setMode(m),
    setShowSettings,
    setShowHistory,
    setShowCommandPalette,
    handleNewSession: () => handleNewSession(),
    sessions,
    activeSession,
    handleSelectSession,
    themeState,
    setThemeState: (state) => setThemeState(state as any),
    setInput: (text) => {
      const fn = (window as any).__composerSetInput;
      if (fn) fn(text);
    },
    notify: (msg: string) => {
      setMessages((prev: Message[]) => [...prev, {
        role: "assistant",
        content: `*${msg}*`,
        timestamp: new Date().toISOString(),
      }]);
    },
  }), [messages, mode, sessions, activeSession, themeState]);

  /* ─── 命令面板动作 ────────────────────────────────────── */
  const handlePaletteAction = useCallback(async (action: string) => {
    setShowCommandPalette(false);
    // Try as a command ID from the registry first
    try {
      const result = await executeCommand(action, "");
      if (result !== undefined) return;
    } catch { /* fall through to legacy */ }
    // Legacy actions
    switch (action) {
      case "new-session": await handleNewSession(); break;
      case "toggle-history": setShowHistory((p) => !p); break;
      case "toggle-sidebar": setShowRightSidebar((p) => !p); break;
      case "cycle-mode": setMode((p) => p === "agent" ? "plan" : p === "plan" ? "yolo" : "agent"); break;
      case "clear-chat": setMessages([]); break;
      case "open-settings": setShowSettings(true); break;
    }
  }, [handleNewSession, executeCommand]);

  /* ─── UI 布局 ─────────────────────────────────────────── */
  return (
    <>
      {/* Onboarding overlay */}
      {showOnboarding && (
        <OnboardingWizard onComplete={() => setShowOnboarding(false)} />
      )}

      {/* Settings modal */}
      {showSettings && (
        <SettingsPanel onClose={() => setShowSettings(false)} />
      )}

      <div className="app-container" onContextMenu={(e) => e.preventDefault()}>
        {/* Left sidebar */}
        {showHistory && (
          <LeftSidebar
            sessions={sessions}
            activeSession={activeSession}
            onSelect={handleSelectSession}
            onNew={handleNewSession}
            onRename={handleRenameSession}
            onDelete={handleDeleteSession}
            onClose={() => setShowHistory(false)}
          />
        )}

        <div className="main-panel">
          {/* Chat area */}
          <div className="chat-area">
            {messages.length === 0 ? (
              <div className="welcome-screen">
                <h1>nyamuWhale</h1>
                <p>nyamu 引擎 · 35+ 工具 · 3 种模式</p>
                <div className="welcome-hints">
                  <div className="hint-card"><strong>⌘K</strong> 命令面板</div>
                  <div className="hint-card"><strong>⌘,</strong> 设置</div>
                  <div className="hint-card"><strong>! command</strong> 执行 Shell</div>
                  <div className="hint-card"><strong>/</strong> 快速命令</div>
                  <div className="hint-card"><strong>⌘R</strong> 恢复会话</div>
                  <div className="hint-card"><strong>⌘N</strong> 新会话</div>
                </div>
              </div>
            ) : (
              <Transcript
                messages={messages}
                liveReasoning={currentReasoning}
                isWaiting={isWaiting}
                onDeleteMessage={handleDeleteMessage}
              />
            )}
          </div>

          {/* Todo panel (pinned above composer) */}
          <TodoPanel
            items={todoItems}
            onClose={() => setTodoItems([])}
          />

          {/* Mode bar + Composer */}
          <ModeBar
            mode={mode}
            onCycleMode={() => setMode((p) => p === "agent" ? "plan" : p === "plan" ? "yolo" : "agent")}
          />
          <Composer
            onSubmit={handleSubmit}
            isWaiting={isWaiting}
            executeCommand={executeCommand}
            onToggleHistory={() => setShowHistory((p) => !p)}
            onToggleRightSidebar={() => setShowRightSidebar((p) => !p)}
          />

          {/* Status bar */}
          <StatusBar
            sessionCount={sessions.length}
            isWaiting={isWaiting}
            modelName="deepseek-v4-flash"
            reasoningEffort="max"
          />
        </div>

        {/* Right sidebar */}
        {showRightSidebar && (
          <RightSidebar
            onClose={() => setShowRightSidebar(false)}
            activeTools={activeTools}
            isWaiting={isWaiting}
          />
        )}

        {/* Command palette */}
        {showCommandPalette && (
          <CommandPalette
            onAction={handlePaletteAction}
            onClose={() => setShowCommandPalette(false)}
          />
        )}
      </div>
    </>
  );
}
