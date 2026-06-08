/**
 * SlashMenu —— 快捷命令菜单（/ 触发）- 增强版
 *
 * 移植自 CodeWhale 的命令系统：
 * - 60+ 命令，分类分组显示
 * - 三阶段模糊匹配（精确前缀 → 包含子串 → 子序列）
 * - 别名搜索支持
 * - 参数提示
 * - 无参数命令直接执行
 */

import { useState, useEffect, useRef, useMemo } from "react";
import {
  getAllCommands, findCommands, getCategoryLabel,
  CATEGORY_ORDER, type CommandDef, type CommandCategory,
} from "../lib/commands";

interface SlashMenuProps {
  visible: boolean;
  filter: string;
  onSelect: (command: string, args?: string) => void;
  onClose: () => void;
}

/** 按分类分组的匹配结果 */
interface GroupedResult {
  category: CommandCategory;
  label: string;
  commands: CommandDef[];
}

export function SlashMenu({ visible, filter, onSelect, onClose }: SlashMenuProps) {
  const [selected, setSelected] = useState(0);
  const ref = useRef<HTMLDivElement>(null);

  // 三阶段匹配结果
  const matchResult = useMemo(() => {
    if (!filter) {
      // 全部显示，按分类分组
      return getAllCommands();
    }
    const r = findCommands(filter);
    // 合并并去重：exact > prefix > contains > fuzzy
    const seen = new Set<string>();
    const merged: CommandDef[] = [];
    const push = (cmds: CommandDef[]) => {
      for (const c of cmds) {
        if (!seen.has(c.id)) { seen.add(c.id); merged.push(c); }
      }
    };
    push(r.exact);
    push(r.prefix);
    push(r.contains);
    push(r.fuzzy);
    return merged;
  }, [filter]);

  // 按分类分组
  const grouped = useMemo(() => {
    const map = new Map<CommandCategory, CommandDef[]>();
    for (const cmd of matchResult) {
      const arr = map.get(cmd.category) || [];
      arr.push(cmd);
      map.set(cmd.category, arr);
    }
    const result: GroupedResult[] = [];
    // 按 CATEGORY_ORDER 排序分类
    for (const cat of CATEGORY_ORDER) {
      const cmds = map.get(cat);
      if (cmds && cmds.length > 0) {
        result.push({ category: cat, label: getCategoryLabel(cat), commands: cmds });
      }
    }
    // 添加未在 CATEGORY_ORDER 中的分类
    for (const [cat, cmds] of map) {
      if (!result.find(r => r.category === cat)) {
        result.push({ category: cat, label: getCategoryLabel(cat), commands: cmds });
      }
    }
    return result;
  }, [matchResult]);

  // 平坦列表用于键盘导航
  const flatItems = useMemo(() => {
    const items: ({ kind: "header"; label: string } | { kind: "cmd"; cmd: CommandDef })[] = [];
    for (const group of grouped) {
      items.push({ kind: "header", label: group.label });
      for (const cmd of group.commands) {
        items.push({ kind: "cmd", cmd });
      }
    }
    return items;
  }, [grouped]);

  // 只计数可选择的命令项
  const selectableCount = flatItems.filter(i => i.kind === "cmd").length;

  useEffect(() => {
    setSelected(0);
  }, [filter]);

  useEffect(() => {
    if (!visible) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelected((i) => {
          let next = i + 1;
          while (next < flatItems.length && flatItems[next].kind === "header") next++;
          return Math.min(next, flatItems.length - 1);
        });
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelected((i) => {
          let prev = i - 1;
          while (prev >= 0 && flatItems[prev].kind === "header") prev--;
          return Math.max(prev, 0);
        });
      }
      if (e.key === "Enter") {
        const item = flatItems[selected];
        if (item && item.kind === "cmd") {
          e.preventDefault();
          const cmd = item.cmd;
          if (cmd.noArgAction) {
            onSelect(cmd.id);
          } else {
            onSelect(cmd.id, `/${cmd.id} `);
          }
          onClose();
        }
      }
      if (e.key === "Escape") { onClose(); }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [visible, flatItems, selected, onSelect, onClose]);

  // 确保选中项始终可见
  useEffect(() => {
    if (!ref.current || !visible) return;
    const selectedEl = ref.current.querySelector(".slash-item.selected") as HTMLElement | null;
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: "nearest" });
    }
  }, [selected, visible]);

  if (!visible || flatItems.length === 0) return null;

  return (
    <div className="slash-menu enhanced" ref={ref}>
      {flatItems.map((item, i) => {
        if (item.kind === "header") {
          return (
            <div key={`h-${item.label}`} className="slash-category-header">
              {item.label}
            </div>
          );
        }
        const cmd = item.cmd;
        const isSelected = i === selected;
        return (
          <div
            key={cmd.id}
            className={`slash-item ${isSelected ? "selected" : ""}`}
            onClick={(e) => {
              e.stopPropagation();
              if (cmd.noArgAction) {
                onSelect(cmd.id);
              } else {
                onSelect(cmd.id, `/${cmd.id} `);
              }
              onClose();
            }}
            onMouseEnter={() => setSelected(i)}
          >
            <span className="slash-label">{cmd.label}</span>
            <span className="slash-desc">{cmd.description}</span>
            {cmd.aliases.length > 0 && (
              <span className="slash-alias">{cmd.aliases.slice(0, 2).join(", ")}</span>
            )}
            {cmd.usage && cmd.usage !== `/${cmd.id}` && (
              <span className="slash-usage">{cmd.usage}</span>
            )}
          </div>
        );
      })}
    </div>
  );
}
