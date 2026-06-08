/**
 * RightSidebar —— 右侧边栏（工具/任务/代理/上下文面板）
 *
 * 包含四个标签页：
 * - Work：工具调用状态面板（运行中/已完成/失败的实时工具列表）
 * - Tasks：任务管理（预留）
 * - Agents：子代理管理（预留）
 * - Context：上下文/技能管理（复用了 PromptManager）
 *
 * Work 面板根据 isWaiting 和 activeTools 实时展示工具执行状态。
 */
import { useState, useEffect, useCallback } from "react";
import { X, Workflow, CheckSquare, Users, FileText, Loader, FolderTree, ClipboardList, Copy, Code, FileCode } from "lucide-react";
import type { ToolCall } from "../lib/bridge";
import * as bridge from "../lib/bridge";
import { ContextMenu, type MenuItem } from "./ContextMenu";
import { PromptManager } from "./PromptManager";
import { WorkspacePanel } from "./WorkspacePanel";
import { PlanPanel } from "./PlanPanel";

type Tab = "work" | "tasks" | "agents" | "workspace" | "plan" | "context";

interface RightSidebarProps {
  onClose: () => void;
  activeTools?: ToolCall[];
  isWaiting?: boolean;
}

export function RightSidebar({ onClose, activeTools = [], isWaiting = false }: RightSidebarProps) {
  const [tab, setTab] = useState<Tab>("work");

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "work", label: "Work", icon: <Workflow size={14} /> },
    { id: "tasks", label: "Tasks", icon: <CheckSquare size={14} /> },
    { id: "agents", label: "Agents", icon: <Users size={14} /> },
    { id: "workspace", label: "Workspace", icon: <FolderTree size={14} /> },
    { id: "plan", label: "Plan", icon: <ClipboardList size={14} /> },
    { id: "context", label: "Context", icon: <FileText size={14} /> },
  ];

  return (
    <div className="right-sidebar">
      <div className="sidebar-tabs">
        {tabs.map((t) => (
          <button
            key={t.id}
            className={`sidebar-tab ${tab === t.id ? "active" : ""}`}
            onClick={() => setTab(t.id)}
            title={t.label}
          >
            {t.icon}
          </button>
        ))}
        <button className="sidebar-tab close" onClick={onClose} title="Close sidebar">
          <X size={14} />
        </button>
      </div>
      <div className="sidebar-content">
        {tab === "work" && <WorkPanel activeTools={activeTools} isWaiting={isWaiting} />}
        {tab === "tasks" && <TasksPanel />}
        {tab === "agents" && <AgentsPanel />}
        {tab === "workspace" && <WorkspacePanel />}
        {tab === "plan" && <PlanPanel />}
        {tab === "context" && <ContextPanel />}
      </div>
    </div>
  );
}

function WorkPanel({ activeTools, isWaiting }: { activeTools: ToolCall[]; isWaiting: boolean }) {
  const recentTools = activeTools.slice(-10);
  const runningCount = recentTools.filter((t) => t.status === "running").length;
  const completedCount = recentTools.filter((t) => t.status === "completed").length;
  const failedCount = recentTools.filter((t) => t.status === "failed").length;
  const [contextMenu, setContextMenu] = useState<{
    x: number; y: number; tool: ToolCall;
  } | null>(null);

  const handleContextMenu = useCallback((e: React.MouseEvent, tool: ToolCall) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, tool });
  }, []);

  useEffect(() => {
    if (!contextMenu) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setContextMenu(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [contextMenu]);

  const buildToolMenuItems = useCallback((tool: ToolCall): MenuItem[] => {
    const items: MenuItem[] = [];

    items.push({
      id: "copy-name",
      label: "Copy tool name",
      icon: <FileCode size={14} />,
      action: () => bridge.writeClipboard(tool.name),
    });

    if (tool.input) {
      items.push({
        id: "copy-input",
        label: "Copy input",
        icon: <Code size={14} />,
        action: () => bridge.writeClipboard(tool.input),
      });
    }

    if (tool.output) {
      items.push({
        id: "copy-output",
        label: "Copy output",
        icon: <Copy size={14} />,
        action: () => bridge.writeClipboard(tool.output!),
      });
    }

    return items;
  }, []);

  return (
    <div className="panel-content">
      <h3>Work Status</h3>

      {/* Status indicator */}
      <div className="status-cards">
        <div className="status-card">
          <span className={`status-dot ${isWaiting ? "working" : "ready"}`} />
          <span>{isWaiting ? "Working..." : "Idle"}</span>
          {runningCount > 0 && <span className="badge running">{runningCount}</span>}
          {completedCount > 0 && <span className="badge completed">{completedCount}</span>}
          {failedCount > 0 && <span className="badge failed">{failedCount}</span>}
        </div>
      </div>

      {/* Running tools */}
      {runningCount > 0 && (
        <>
          <h3 style={{ marginTop: 12 }}>Running</h3>
          <div className="tool-list">
            {recentTools.filter(t => t.status === "running").map((t, i) => (
              <div key={i} className="tool-item running" onContextMenu={(e) => handleContextMenu(e, t)}>
                <Loader size={12} className="spin" />
                <span className="tool-name">{t.name}</span>
              </div>
            ))}
          </div>
        </>
      )}

      {/* Recent activity */}
      {recentTools.length > 0 && (
        <>
          <h3 style={{ marginTop: 12 }}>Recent</h3>
          <div className="tool-list">
            {recentTools.filter(t => t.status !== "running").reverse().map((t, i) => (
              <div key={i} className={`tool-item ${t.status}`} onContextMenu={(e) => handleContextMenu(e, t)}>
                <span className="tool-status-icon">
                  {t.status === "completed" ? "✓" : "✗"}
                </span>
                <span className="tool-name">{t.name}</span>
              </div>
            ))}
          </div>
        </>
      )}

      {recentTools.length === 0 && (
        <p className="panel-hint">Tool calls will appear here.</p>
      )}

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={buildToolMenuItems(contextMenu.tool)}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}

function TasksPanel() {
  const [usage, setUsage] = useState<bridge.SessionUsage | null>(null);

  useEffect(() => {
    bridge.getSessionUsage().then(setUsage).catch(() => {});
    const interval = setInterval(() => {
      bridge.getSessionUsage().then(setUsage).catch(() => {});
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="panel-content">
      <h3>Session Usage</h3>
      {usage ? (
        <div className="context-stats">
          <div className="stat-row"><span>Input tokens</span><code>{usage.input_tokens.toLocaleString()}</code></div>
          <div className="stat-row"><span>Output tokens</span><code>{usage.output_tokens.toLocaleString()}</code></div>
          <div className="stat-row"><span>Total cost</span><code>${usage.total_cost.toFixed(4)}</code></div>
          <div className="stat-row"><span>Cache hit rate</span><code>{(usage.cache_hit_rate * 100).toFixed(0)}%</code></div>
        </div>
      ) : (
        <p className="panel-hint">Loading usage data...</p>
      )}
    </div>
  );
}

function AgentsPanel() {
  const [agents, setAgents] = useState<any[]>([]);

  useEffect(() => {
    bridge.getSubagents().then(setAgents).catch(() => {});
    const interval = setInterval(() => {
      bridge.getSubagents().then(setAgents).catch(() => {});
    }, 3000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="panel-content">
      <h3>Sub-agents</h3>
      {agents.length === 0 ? (
        <p className="panel-hint">No active sub-agents.</p>
      ) : (
        <div className="tool-list">
          {agents.map((a) => (
            <div key={a.id} className={`tool-item ${a.status === "running" ? "running" : a.status === "done" ? "completed" : "failed"}`}>
              {a.status === "running" ? (
                <Loader size={12} className="spin" />
              ) : a.status === "done" ? (
                <span className="tool-status-icon">✓</span>
              ) : (
                <span className="tool-status-icon">✗</span>
              )}
              <span className="tool-name" title={a.prompt?.slice(0, 60)}>
                {a.id?.slice(0, 8)}... {a.status}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ContextPanel() {
  return <PromptManager />;
}
