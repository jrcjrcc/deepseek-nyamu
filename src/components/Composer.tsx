/**
 * Composer —— 消息输入组件（增强版）
 *
 * 功能：
 * 1. Enter 发送（Shift+Enter 换行）
 * 2. / 触发器弹出命令菜单（SlashMenu）
 * 3. Enter 时检测 /cmd 格式并路由到命令调度器
 * 4. 文本域自动伸缩
 * 5. 等待状态禁用输入
 */
import { useState, useRef, KeyboardEvent, useEffect } from "react";
import { Send, PanelLeft, PanelRight } from "lucide-react";
import { SlashMenu } from "./SlashMenu";
import { findCommandById } from "../lib/commands";

interface ComposerProps {
  onSubmit: (content: string) => void;
  isWaiting: boolean;
  onToggleHistory: () => void;
  onToggleRightSidebar?: () => void;
  /** 命令调度器：若提供，/ 命令将由它处理而非直接发送 */
  executeCommand?: (id: string, args: string) => Promise<string | void>;
}

export function Composer({ onSubmit, isWaiting, onToggleHistory, onToggleRightSidebar, executeCommand }: ComposerProps) {
  const [input, setInput] = useState("");
  const [showSlash, setShowSlash] = useState(false);
  const [slashFilter, setSlashFilter] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // 暴露 setInput 给外部（/edit 命令使用）
  useEffect(() => {
    (window as any).__composerSetInput = setInput;
    return () => { delete (window as any).__composerSetInput; };
  }, []);

  const handleSubmit = async () => {
    const trimmed = input.trim();
    if (!trimmed || isWaiting) return;

    // 检测是否为 / 命令
    if (trimmed.startsWith("/")) {
      const spaceIdx = trimmed.indexOf(" ");
      const cmdName = spaceIdx > 0 ? trimmed.slice(1, spaceIdx) : trimmed.slice(1);
      const args = spaceIdx > 0 ? trimmed.slice(spaceIdx + 1) : "";

      const cmd = findCommandById(cmdName);
      if (cmd && executeCommand) {
        setInput("");
        setShowSlash(false);
        if (textareaRef.current) {
          textareaRef.current.style.height = "auto";
        }
        await executeCommand(cmd.id, args);
        return;
      }
    }

    // 普通消息或未识别的命令，直接提交
    onSubmit(trimmed);
    setInput("");
    setShowSlash(false);
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (showSlash && (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Escape" || e.key === "Enter")) {
      // Let SlashMenu handle these
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleInput = (value: string) => {
    setInput(value);
    // Detect "/" at the start of a word
    const cursor = textareaRef.current?.selectionStart || value.length;
    const beforeCursor = value.slice(0, cursor);
    const lastSpace = beforeCursor.lastIndexOf(" ");
    const currentWord = lastSpace >= 0 ? beforeCursor.slice(lastSpace + 1) : beforeCursor;
    if (currentWord.startsWith("/") && currentWord.length > 1) {
      setShowSlash(true);
      setSlashFilter(currentWord.slice(1));
    } else if (value === "/") {
      setShowSlash(true);
      setSlashFilter("");
    } else {
      setShowSlash(false);
    }

    // Auto-resize
    const el = textareaRef.current;
    if (el) {
      el.style.height = "auto";
      el.style.height = Math.min(el.scrollHeight, 240) + "px";
    }
  };

  const handleSlashSelect = (cmdId: string, insertText?: string) => {
    if (insertText) {
      setInput(insertText);
    } else {
      // 无参数命令：直接执行
      if (executeCommand) {
        setInput("");
        executeCommand(cmdId, "");
      }
    }
    setShowSlash(false);
    textareaRef.current?.focus();
  };

  return (
    <div className="composer-container">
      <div className="composer-toolbar">
        <button className="toolbar-btn" onClick={onToggleHistory} title="Toggle history">
          <PanelLeft size={18} />
        </button>
        <div style={{ flex: 1 }} />
        {onToggleRightSidebar && (
          <button className="toolbar-btn" onClick={onToggleRightSidebar} title="Toggle sidebar">
            <PanelRight size={18} />
          </button>
        )}
      </div>
      <div className="composer-input-row" style={{ position: "relative" }}>
        <SlashMenu
          visible={showSlash}
          filter={slashFilter}
          onSelect={handleSlashSelect}
          onClose={() => setShowSlash(false)}
        />
        <textarea
          ref={textareaRef}
          className="composer-input"
          placeholder="Send a message... (@ file, / command)"
          value={input}
          onChange={(e) => handleInput(e.target.value)}
          onKeyDown={handleKeyDown}
          rows={1}
          disabled={isWaiting}
        />
        <button
          className="composer-send-btn"
          onClick={handleSubmit}
          disabled={!input.trim() || isWaiting}
        >
          {isWaiting ? <span className="spinner" /> : <Send size={18} />}
        </button>
      </div>
    </div>
  );
}
