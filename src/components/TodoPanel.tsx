/**
 * TodoPanel —— 置顶任务列表组件
 *
 * 在 Composer 上方显示当前 todo_write 输出的任务列表。
 * 支持：
 * - 复选框标记完成/未完成
 * - 进度条显示
 * - 可收起/展开
 * - 可关闭
 */
import { useState } from "react";
import { CheckSquare, ChevronDown, ChevronRight, X, ListTodo } from "lucide-react";

export interface TodoItem {
  id: string;
  description: string;
  status: "pending" | "completed";
}

interface TodoPanelProps {
  items: TodoItem[];
  onToggle?: (id: string) => void;
  onClose?: () => void;
}

export function TodoPanel({ items, onToggle, onClose }: TodoPanelProps) {
  const [collapsed, setCollapsed] = useState(false);

  if (items.length === 0) return null;

  const total = items.length;
  const done = items.filter((t) => t.status === "completed").length;
  const progress = total > 0 ? Math.round((done / total) * 100) : 0;

  return (
    <div className="todo-panel">
      <div className="todo-panel-header">
        <button className="todo-collapse-btn" onClick={() => setCollapsed((p) => !p)}>
          {collapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
        </button>
        <ListTodo size={14} className="todo-icon" />
        <span className="todo-title">Tasks</span>
        <span className="todo-progress-text">{done}/{total}</span>
        <div className="todo-progress-bar">
          <div className="todo-progress-fill" style={{ width: `${progress}%` }} />
        </div>
        {onClose && (
          <button className="todo-close-btn" onClick={onClose} title="Close task list">
            <X size={12} />
          </button>
        )}
      </div>
      {!collapsed && (
        <div className="todo-list">
          {items.map((item) => (
            <label key={item.id} className="todo-item">
              <input
                type="checkbox"
                className="todo-checkbox"
                checked={item.status === "completed"}
                onChange={() => onToggle?.(item.id)}
              />
              <span className={`todo-desc ${item.status === "completed" ? "todo-done" : ""}`}>
                {item.description}
              </span>
            </label>
          ))}
        </div>
      )}
    </div>
  );
}
