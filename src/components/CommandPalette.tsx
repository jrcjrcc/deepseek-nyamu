/**
 * CommandPalette —— 命令面板（⌘K）- 增强版
 *
 * 类似 VS Code 的命令面板，通过 ⌘K / F1 触发。
 * 从 centralized commands.ts 加载所有命令。
 * 支持：
 * - 分类过滤（输入 "c:" 或 "config:" 过滤特定类别）
 * - 模糊匹配（别名也可匹配）
 * - 键盘导航
 * - 快捷键提示
 */
import { useState, useEffect, useRef, useMemo, KeyboardEvent } from "react";
import {
  getAllCommands, findCommands, getCategoryLabel,
  CATEGORY_ORDER, CATEGORY_LABELS, type CommandDef, type CommandCategory,
} from "../lib/commands";

interface CommandPaletteProps {
  onAction: (action: string) => void;
  onClose: () => void;
}

/** 类别到前缀的映射 */
const CATEGORY_PREFIXES: Record<string, CommandCategory> = {};
for (const [cat, label] of Object.entries(CATEGORY_LABELS)) {
  CATEGORY_PREFIXES[cat] = cat as CommandCategory;
  CATEGORY_PREFIXES[label] = cat as CommandCategory;
}

export function CommandPalette({ onAction, onClose }: CommandPaletteProps) {
  const [filter, setFilter] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // 检测分类前缀过滤 (c:core 或 config:)
  const { categoryFilter, searchQuery } = useMemo(() => {
    const parts = filter.split(":");
    if (parts.length >= 2) {
      const prefix = parts[0].trim().toLowerCase();
      const matchedCat = CATEGORY_PREFIXES[prefix];
      if (matchedCat) {
        return { categoryFilter: matchedCat, searchQuery: parts.slice(1).join(":").trim() };
      }
    }
    return { categoryFilter: null as CommandCategory | null, searchQuery: filter };
  }, [filter]);

  // 过滤后的命令
  const filtered = useMemo(() => {
    let cmds = getAllCommands();
    if (categoryFilter) {
      cmds = cmds.filter(c => c.category === categoryFilter);
    }
    if (!searchQuery) return cmds;
    const r = findCommands(searchQuery);
    const seen = new Set<string>();
    const merged: CommandDef[] = [];
    const push = (arr: CommandDef[]) => {
      for (const c of arr) { if (!seen.has(c.id)) { seen.add(c.id); merged.push(c); } }
    };
    push(r.exact);
    push(r.prefix);
    push(r.contains);
    push(r.fuzzy);
    return merged;
  }, [categoryFilter, searchQuery]);

  // 按分类分组
  const grouped = useMemo(() => {
    const map = new Map<CommandCategory, CommandDef[]>();
    for (const cmd of filtered) {
      const arr = map.get(cmd.category) || [];
      arr.push(cmd);
      map.set(cmd.category, arr);
    }
    const result: { label: string; commands: CommandDef[] }[] = [];
    for (const cat of CATEGORY_ORDER) {
      const cmds = map.get(cat);
      if (cmds && cmds.length > 0) {
        result.push({ label: getCategoryLabel(cat), commands: cmds });
      }
    }
    return result;
  }, [filtered]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setSelectedIdx(0);
  }, [filter]);

  // Scroll selected into view
  useEffect(() => {
    if (!listRef.current) return;
    const el = listRef.current.querySelector(".command-palette-item.selected") as HTMLElement | null;
    if (el) el.scrollIntoView({ block: "nearest" });
  }, [selectedIdx]);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIdx((p) => Math.min(p + 1, filtered.length - 1));
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIdx((p) => Math.max(p - 1, 0));
      return;
    }
    if (e.key === "Enter" && filtered[selectedIdx]) {
      e.preventDefault();
      onAction(filtered[selectedIdx].id);
      return;
    }
  };

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          className="command-palette-input"
          placeholder="Type a command... (c:config / s:session / d:debug)"
          value={filter}
          onChange={(e) => { setFilter(e.target.value); setSelectedIdx(0); }}
          onKeyDown={handleKeyDown}
        />
        <div className="command-palette-list" ref={listRef}>
          {categoryFilter && (
            <div className="command-palette-category-hint">
              Filtering: {getCategoryLabel(categoryFilter)} commands
              <button className="command-palette-clear-filter" onClick={() => setFilter("")}>
                ×
              </button>
            </div>
          )}
          {filtered.length === 0 ? (
            <div className="command-palette-empty">No commands found</div>
          ) : (
            <>
              {/* 分类模式：显示分类标题 */}
              {!searchQuery && !categoryFilter ? (
                grouped.map((group) => (
                  <div key={group.label}>
                    <div className="command-palette-group-header">{group.label}</div>
                    {group.commands.map((cmd, gi) => {
                      const idx = filtered.indexOf(cmd);
                      return (
                        <div
                          key={cmd.id}
                          className={`command-palette-item ${idx === selectedIdx ? "selected" : ""}`}
                          onClick={() => onAction(cmd.id)}
                          onMouseEnter={() => setSelectedIdx(idx)}
                        >
                          <div className="command-palette-item-left">
                            <span className="command-palette-label">{cmd.label}</span>
                            <span className="command-palette-desc">{cmd.description}</span>
                          </div>
                          {cmd.aliases.length > 0 && (
                            <span className="command-palette-shortcut">{cmd.aliases[0]}</span>
                          )}
                        </div>
                      );
                    })}
                  </div>
                ))
              ) : (
                // 搜索模式：平坦列表
                filtered.map((cmd, i) => (
                  <div
                    key={cmd.id}
                    className={`command-palette-item ${i === selectedIdx ? "selected" : ""}`}
                    onClick={() => onAction(cmd.id)}
                    onMouseEnter={() => setSelectedIdx(i)}
                  >
                    <div className="command-palette-item-left">
                      <span className="command-palette-label">{cmd.label}</span>
                      <span className="command-palette-desc">{cmd.description}</span>
                    </div>
                    {cmd.usage && cmd.usage !== `/${cmd.id}` && (
                      <span className="command-palette-shortcut">{cmd.usage}</span>
                    )}
                  </div>
                ))
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
