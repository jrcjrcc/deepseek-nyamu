/**
 * Transcript —— 消息列表（对话记录）
 *
 * 功能：
 * 1. 遍历渲染所有消息
 * 2. 自动滚动到底部（监听 messages 变化）
 * 3. 在等待 AI 回复时显示 ThinkingBar
 * 4. 右键菜单（复制/删除消息）
 */
import { useEffect, useRef, useState, useCallback } from "react";
import { Message } from "./Message";
import { ContextMenu, messageContextMenuItems } from "./ContextMenu";
import { ThinkingBar } from "./ThinkingBar";
import type { Message as MessageType } from "../lib/bridge";

interface TranscriptProps {
  messages: MessageType[];
  onDeleteMessage?: (index: number) => void;
  liveReasoning?: string;
  isWaiting?: boolean;
}

export function Transcript({ messages, onDeleteMessage, liveReasoning, isWaiting }: TranscriptProps) {
  const bottomRef = useRef<HTMLDivElement>(null);
  const [contextMenu, setContextMenu] = useState<{
    x: number; y: number; index: number;
  } | null>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleContextMenu = useCallback((e: React.MouseEvent, index: number) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, index });
  }, []);

  // Close context menu on Escape (ContextMenu handles outside-click itself)
  useEffect(() => {
    if (!contextMenu) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setContextMenu(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [contextMenu]);

  return (
    <div className="transcript">
      {messages.map((msg, i) => (
        <div
          key={i}
          onContextMenu={(e) => handleContextMenu(e, i)}
          style={{ userSelect: "none" }}
        >
          <Message message={msg} />
        </div>
      ))}
      {isWaiting && liveReasoning && (
        <div className="thinking-bar-container" onContextMenu={(e) => e.preventDefault()}>
          <ThinkingBar reasoning={liveReasoning} />
        </div>
      )}
      <div ref={bottomRef} />

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={messageContextMenuItems(
            messages[contextMenu.index]?.content || "",
            undefined,
            undefined,
            onDeleteMessage
              ? () => onDeleteMessage(contextMenu.index)
              : undefined,
          )}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}
