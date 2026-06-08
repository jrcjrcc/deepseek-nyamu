/**
 * ContextMenu —— 右键上下文菜单（增强版）
 *
 * 支持：
 * - 菜单项图标、危险操作样式
 * - 分隔线（separator）
 * - 快捷键提示（shortcut）
 * - 自动调整位置避免溢出视口
 * - 外部点击 / Escape 关闭
 *
 * messageContextMenuItems 提供消息操作的菜单项构建。
 */
import { useEffect, useRef } from "react";
import { Copy, Trash2, Pencil, FileCode, FileText } from "lucide-react";
import * as bridge from "../lib/bridge";

export interface MenuItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  danger?: boolean;
  separator?: boolean;
  shortcut?: string;
  action?: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: MenuItem[];
  onClose: () => void;
}

export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent && e.key === "Escape") {
        onClose();
        return;
      }
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    window.addEventListener("mousedown", handler);
    window.addEventListener("keydown", handler);
    return () => {
      window.removeEventListener("mousedown", handler);
      window.removeEventListener("keydown", handler);
    };
  }, [onClose]);

  const visibleItems = items.filter(i => !i.separator);
  const adjustedX = Math.min(x, window.innerWidth - 180);
  const adjustedY = Math.min(y, window.innerHeight - visibleItems.length * 36 - 16);

  return (
    <div
      ref={ref}
      className="context-menu"
      style={{ left: adjustedX, top: adjustedY }}
    >
      {items.map((item, i) => {
        if (item.separator) {
          return <div key={i} className="context-separator" />;
        }
        return (
          <button
            key={item.id}
            className={`context-item${item.danger ? " context-item--danger" : ""}`}
            onClick={() => { item.action?.(); onClose(); }}
          >
            {item.icon && <span className="context-icon">{item.icon}</span>}
            <span className="context-label">{item.label}</span>
            {item.shortcut && <span className="context-shortcut">{item.shortcut}</span>}
          </button>
        );
      })}
    </div>
  );
}

// Helper to build context menu items for a message (enhanced)
export function messageContextMenuItems(
  content: string,
  onCopy?: () => void,
  onEdit?: () => void,
  onDelete?: () => void,
): MenuItem[] {
  const items: MenuItem[] = [];

  items.push({
    id: "copy",
    label: "Copy message",
    icon: <Copy size={14} />,
    shortcut: "⌘C",
    action: onCopy ? onCopy : (() => bridge.writeClipboard(content)),
  });

  items.push({
    id: "copy-raw",
    label: "Copy as raw text",
    icon: <FileCode size={14} />,
    action: () => bridge.writeClipboard(content),
  });

  if (onEdit || onDelete) {
    items.push({
      id: "separator-1",
      separator: true,
      label: "",
    });
  }

  if (onEdit) {
    items.push({
      id: "edit",
      label: "Edit message",
      icon: <Pencil size={14} />,
      action: onEdit,
    });
  }

  if (onDelete) {
    items.push({
      id: "delete",
      label: "Delete message",
      icon: <Trash2 size={14} />,
      danger: true,
      action: onDelete,
    });
  }

  return items;
}
