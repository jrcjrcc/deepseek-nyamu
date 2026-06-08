/**
 * LeftSidebar —— 左侧边栏（多标签页）
 *
 * 包含四个标签页：
 * - History：会话历史列表
 * - Memory：用户记忆管理（预留）
 * - Skills：技能/工具管理
 * - Settings：显示当前配置信息
 */
import { useState, useEffect } from "react";
import { MessageSquare, Brain, Puzzle, Settings, X } from "lucide-react";
import { HistoryPanel } from "./HistoryPanel";
import { PromptManager } from "./PromptManager";
import type { SessionInfo } from "../lib/bridge";
import * as bridge from "../lib/bridge";

type Tab = "history" | "memory" | "skills" | "settings";

interface LeftSidebarProps {
  sessions: SessionInfo[];
  activeSession: string | null;
  onSelect: (id: string) => void;
  onNew: () => void;
  onRename?: (id: string, title: string) => void;
  onDelete?: (id: string) => void;
  onClose: () => void;
}

export function LeftSidebar({ sessions, activeSession, onSelect, onNew, onRename, onDelete, onClose }: LeftSidebarProps) {
  const [tab, setTab] = useState<Tab>("history");

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "history", label: "History", icon: <MessageSquare size={14} /> },
    { id: "memory", label: "Memory", icon: <Brain size={14} /> },
    { id: "skills", label: "Skills", icon: <Puzzle size={14} /> },
    { id: "settings", label: "Settings", icon: <Settings size={14} /> },
  ];

  return (
    <div className="left-sidebar">
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
        <button className="sidebar-tab close" onClick={onClose} title="Close">
          <X size={14} />
        </button>
      </div>
      <div className="sidebar-content">
        {tab === "history" && (
          <HistoryPanel
            sessions={sessions}
            activeSession={activeSession}
            onSelect={onSelect}
            onNew={onNew}
            onRename={onRename}
            onDelete={onDelete}
            onClose={() => {}}
          />
        )}
        {tab === "memory" && <MemoryPanel />}
        {tab === "skills" && <PromptManager />}
        {tab === "settings" && (
          <div className="panel-content">
            <h3>Settings</h3>
            <div className="context-stats">
              <div className="stat-row"><span>Theme</span><code>Pastel Dream</code></div>
              <div className="stat-row"><span>Model</span><code>deepseek-v4-flash</code></div>
              <div className="stat-row"><span>Sandbox</span><code>enforce</code></div>
            </div>
            <p className="panel-hint">Press <strong>⌘,</strong> to open the full settings panel.</p>
          </div>
        )}
      </div>
    </div>
  );
}

function MemoryPanel() {
  const [memory, setMemory] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    bridge.getMemory().then((content) => {
      setMemory(content || "(empty — no memory file found)");
    }).catch(() => {
      setMemory("Error reading memory file.");
    }).finally(() => setLoading(false));
  }, []);

  return (
    <div className="panel-content">
      <div className="drawer__head">
        <div className="drawer__title">Memory</div>
      </div>
      <div className="mem-body">
        {loading ? (
          <p className="panel-hint">Loading...</p>
        ) : (
          <pre className="mem-content">{memory}</pre>
        )}
      </div>
    </div>
  );
}
