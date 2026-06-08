/**
 * HistoryPanel —— 会话历史面板（带右键菜单）
 *
 * 功能：
 * 1. 按日期分组会话（Today / Yesterday / 日期）
 * 2. 搜索会话（按标题和 ID 过滤）
 * 3. 重命名会话（点击铅笔图标 / 右键菜单 Rename）
 * 4. 删除会话（二次确认：点击垃圾桶 / 右键菜单 Delete）
 * 5. 新建会话
 * 6. 右键菜单：Export / Fork / Copy ID
 */
import { useState, useMemo, useEffect, useCallback } from "react";
import { Search, Pencil, Trash2, Check, X, Plus, Copy, ExternalLink, GitBranch } from "lucide-react";
import type { SessionInfo } from "../lib/bridge";
import * as bridge from "../lib/bridge";
import { ContextMenu, type MenuItem } from "./ContextMenu";

interface HistoryPanelProps {
  sessions: SessionInfo[];
  activeSession: string | null;
  onSelect: (id: string) => void;
  onNew: () => void;
  onClose: () => void;
  onRename?: (id: string, title: string) => void;
  onDelete?: (id: string) => void;
}

export function HistoryPanel({
  sessions, activeSession, onSelect, onNew, onClose, onRename, onDelete,
}: HistoryPanelProps) {
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [confirming, setConfirming] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{
    x: number; y: number; session: SessionInfo;
  } | null>(null);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return sessions;
    return sessions.filter((s) =>
      [s.title, s.id].some((part) => part.toLowerCase().includes(q))
    );
  }, [query, sessions]);

  // Group by day
  const groups = useMemo(() => {
    const now = new Date();
    const today = now.toDateString();
    const yesterday = new Date(now.getTime() - 86400000).toDateString();
    const map = new Map<string, SessionInfo[]>();
    for (const s of filtered) {
      const d = new Date(s.created_at).toDateString();
      let label = d === today ? "Today" : d === yesterday ? "Yesterday" : d;
      const arr = map.get(label) || [];
      arr.push(s);
      map.set(label, arr);
    }
    return Array.from(map.entries());
  }, [filtered]);

  const startRename = (s: SessionInfo) => {
    setConfirming(null);
    setContextMenu(null);
    setEditing(s.id);
    setDraft(s.title);
  };
  const commitRename = (id: string) => {
    if (onRename) onRename(id, draft.trim());
    setEditing(null);
  };
  const handleDelete = (id: string) => {
    if (onDelete) onDelete(id);
    setConfirming(null);
  };

  // Close context menu on Escape (ContextMenu handles outside-click itself)
  const handleContextMenu = useCallback((e: React.MouseEvent, session: SessionInfo) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, session });
  }, []);

  useEffect(() => {
    if (!contextMenu) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setContextMenu(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [contextMenu]);

  const buildSessionMenuItems = (s: SessionInfo): MenuItem[] => {
    return [
      {
        id: "rename",
        label: "Rename",
        icon: <Pencil size={14} />,
        action: () => startRename(s),
      },
      {
        id: "delete",
        label: "Delete",
        icon: <Trash2 size={14} />,
        danger: true,
        action: () => {
          if (onDelete) {
            setConfirming(s.id);
            // Auto-confirm if already confirming
            if (confirming === s.id) handleDelete(s.id);
          }
        },
      },
      {
        id: "separator-1",
        separator: true,
        label: "",
      },
      {
        id: "fork",
        label: "Fork session",
        icon: <GitBranch size={14} />,
        action: async () => {
          try {
            const session = await bridge.newSession("Fork: " + s.title);
            onSelect(session.id);
          } catch {}
        },
      },
      {
        id: "export",
        label: "Export session",
        icon: <ExternalLink size={14} />,
        action: async () => {
          try {
            const result = await bridge.exportSession(s.id, null);
            // Save to clipboard as fallback
            await bridge.writeClipboard(result);
          } catch {}
        },
      },
      {
        id: "separator-2",
        separator: true,
        label: "",
      },
      {
        id: "copy-id",
        label: "Copy session ID",
        icon: <Copy size={14} />,
        action: () => bridge.writeClipboard(s.id),
      },
    ];
  };

  return (
    <div className="history-panel">
      <header className="drawer__head">
        <div className="drawer__title">History</div>
        <div className="drawer__actions">
          <button className="chip" onClick={onNew} title="New session">
            <Plus size={14} />
          </button>
          <button className="chip" onClick={onClose} title="Close">✕</button>
        </div>
      </header>

      <div className="history-body">
        {sessions.length > 0 && (
          <label className="history-search">
            <Search size={13} />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search sessions..."
            />
          </label>
        )}

        {sessions.length === 0 ? (
          <div className="mem-empty">No sessions yet. Start a new chat!</div>
        ) : groups.length === 0 ? (
          <div className="mem-empty">No results</div>
        ) : (
          groups.map(([label, items]) => (
            <section className="mem-section" key={label}>
              <div className="mem-section__title">{label}</div>
              {items.map((s) => {
                const isActive = s.id === activeSession;
                const isEditing = editing === s.id;
                const isConfirming = confirming === s.id;

                return (
                  <div
                    key={s.id}
                    className={`hist-item${isActive ? " hist-item--active" : ""}`}
                    onContextMenu={(e) => handleContextMenu(e, s)}
                  >
                    {isEditing ? (
                      <input
                        className="hist-item__rename"
                        autoFocus
                        value={draft}
                        onChange={(e) => setDraft(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") commitRename(s.id);
                          if (e.key === "Escape") setEditing(null);
                        }}
                        onBlur={() => commitRename(s.id)}
                      />
                    ) : (
                      <button className="hist-item__main" onClick={() => onSelect(s.id)}>
                        <div className="hist-item__title">{s.title}</div>
                        <div className="hist-item__meta">
                          <span>{s.message_count} msg</span>
                          {isActive && <span className="hist-item__badge">active</span>}
                        </div>
                      </button>
                    )}

                    {!isEditing && (
                      <div className="hist-item__actions">
                        {isConfirming ? (
                          <>
                            <button className="hist-act hist-act--danger" onClick={() => handleDelete(s.id)}>
                              <Check size={13} />
                            </button>
                            <button className="hist-act" onClick={() => setConfirming(null)}>
                              <X size={13} />
                            </button>
                          </>
                        ) : (
                          <>
                            {onRename && (
                              <button className="hist-act" onClick={() => startRename(s)}>
                                <Pencil size={13} />
                              </button>
                            )}
                            {onDelete && (
                              <button className="hist-act hist-act--danger" onClick={() => setConfirming(s.id)}>
                                <Trash2 size={13} />
                              </button>
                            )}
                          </>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </section>
          ))
        )}
      </div>

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={buildSessionMenuItems(contextMenu.session)}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}
