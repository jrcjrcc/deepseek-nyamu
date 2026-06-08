/**
 * Message —— 单条消息渲染组件
 *
 * 根据消息的 role 渲染不同样式：
 * - user：用户消息（带 ">" 前缀标记）
 * - assistant：AI 回复（可折叠的推理链 + Markdown 正文 + 复制按钮）
 * - tool_call：工具调用卡片（运行中/完成/失败三种状态）
 *
 * 推理链（reasoning）默认折叠，点击 "Thought process" 展开。
 */
import { useState } from "react";
import { ChevronRight, Copy, Check, Loader, Brain } from "lucide-react";
import type { Message as MessageType, ToolCall } from "../lib/bridge";
import { Markdown } from "./Markdown";
import * as bridge from "../lib/bridge";

interface MessageProps {
  message: MessageType;
}

export function Message({ message }: MessageProps) {
  const isUser = message.role === "user";
  const isTool = message.role === "tool_call";
  const isAssistant = message.role === "assistant";
  const [copied, setCopied] = useState(false);
  const [reasoningOpen, setReasoningOpen] = useState(false);

  const copyContent = () => {
    bridge.writeClipboard(message.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (isTool) {
    return (
      <div className="msg msg--tool">
        {message.tool_calls?.map((tc, i) => (
          <ToolCallCard key={i} call={tc} />
        ))}
      </div>
    );
  }

  return (
    <div className={`msg ${isUser ? "msg--user" : "msg--assistant"}`}>
      {isUser && <span className="msg__caret">›</span>}

      {isAssistant && message.reasoning && (
        <div className="reasoning">
          <button className="reasoning__toggle" onClick={() => setReasoningOpen((v) => !v)}>
            <ChevronRight
              className={`reasoning__chevron ${reasoningOpen ? "reasoning__chevron--open" : ""}`}
              size={12}
            />
            Thought process
          </button>
          {reasoningOpen && <div className="reasoning__body">{message.reasoning}</div>}
        </div>
      )}

      <div className="msg__body">
        {message.content ? <Markdown content={message.content} /> : null}
      </div>

      {isAssistant && message.content && (
        <div className="msg__actions">
          <button className="icon-btn" onClick={copyContent} title="Copy">
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>
        </div>
      )}
    </div>
  );
}

function ToolCallCard({ call }: { call: ToolCall }) {
  const [expanded, setExpanded] = useState(false);
  const [outputExpanded, setOutputExpanded] = useState(false);

  const isGit = call.name.startsWith("git_");
  const isGithub = call.name.startsWith("github_");
  const isWebSearch = call.name === "web_search";
  const isFetch = call.name === "fetch_url";
  const isExec = call.name === "exec_shell";
  const isTodo = call.name === "todo_write" || call.name === "todo_list" || call.name === "update_todo";
  const isPlan = call.name === "update_plan";
  const isNotify = call.name === "notify";
  const isRemember = call.name === "remember";
  const isDiagnostics = call.name === "diagnostics";
  const isRead = call.name === "read_file";
  const isWrite = call.name === "write_file" || call.name === "edit_file";

  // Try to parse input as JSON for display
  let parsedInput: Record<string, string> | null = null;
  try { const p = JSON.parse(call.input); if (typeof p === "object") parsedInput = p; } catch {}

  // Detect if output contains a diff (unified diff format)
  const hasDiff = call.output && (
    call.output.includes("--- ") || call.output.includes("+++ ") || call.output.includes("@@ -")
  );

  // Detect if output contains error
  const hasError = call.status === "failed" || (
    call.output && (call.output.includes("Error:") || call.output.includes("error:") || call.output.includes("Traceback"))
  );

  const getToolIcon = () => {
    if (isGit || isGithub) return "⌚";
    if (isWebSearch) return "🔍";
    if (isFetch) return "🌐";
    if (isExec) return "$";
    if (isTodo) return "☑";
    if (isPlan) return "📋";
    if (isRead) return "📄";
    if (isWrite) return "✏";
    if (isNotify) return "🔔";
    if (isDiagnostics) return "⚙";
    return "⚙";
  };

  return (
    <div className={`toolcall-card toolcall-${call.status}`}>
      <div className="toolcall-header" onClick={() => setExpanded(!expanded)}>
        {call.status === "running" ? (
          <Loader size={14} className="spin" />
        ) : call.status === "completed" ? (
          <span className="toolcall-ok">✓</span>
        ) : (
          <span className="toolcall-fail">✗</span>
        )}
        <span className="toolcall-name">
          <span className="toolcall-icon">{getToolIcon()}</span>
          {call.name}
        </span>
        {parsedInput && isExec && parsedInput.command && (
          <code className="toolcall-preview">{parsedInput.command.slice(0, 60)}{parsedInput.command.length > 60 ? "..." : ""}</code>
        )}
        {parsedInput && isRead && parsedInput.path && (
          <code className="toolcall-preview">{parsedInput.path}</code>
        )}
        {parsedInput && isWebSearch && parsedInput.query && (
          <span className="toolcall-preview">"{parsedInput.query.slice(0, 40)}"</span>
        )}
        <span className={`toolcall-status-badge toolcall-badge-${call.status}`}>{call.status}</span>
        <ChevronRight size={14} className={`toolcall-chevron ${expanded ? "toolcall-chevron--open" : ""}`} />
      </div>
      {expanded && (
        <div className="toolcall-detail">
          <div className="toolcall-section">
            <strong>Input:</strong>
            {parsedInput ? (
              <div className="toolcall-input-fields">
                {Object.entries(parsedInput).map(([k, v]) => (
                  <div key={k} className="toolcall-field">
                    <span className="toolcall-field-key">{k}:</span>
                    <span className="toolcall-field-value">{typeof v === "string" && v.length > 200 ? v.slice(0, 200) + "..." : JSON.stringify(v)}</span>
                  </div>
                ))}
              </div>
            ) : (
              <pre>{call.input}</pre>
            )}
          </div>
          {call.output && (
            <div className="toolcall-section">
              <strong>Output:</strong>
              {hasDiff ? (
                <pre className="toolcall-output toolcall-output-diff">{call.output.length > 1000 ? call.output.slice(0, 1000) + "\n... [truncated]" : call.output}</pre>
              ) : hasError ? (
                <pre className="toolcall-output toolcall-output-error">{call.output.length > 800 ? call.output.slice(0, 800) + "\n... [truncated]" : call.output}</pre>
              ) : (
                <pre className="toolcall-output">{call.output.length > 500 ? call.output.slice(0, 500) + "\n... [truncated]" : call.output}</pre>
              )}
              {call.output.length > 500 && !outputExpanded && (
                <button className="toolcall-expand-btn" onClick={(e) => { e.stopPropagation(); setOutputExpanded(true); }}>
                  Show full output ({call.output.length} chars)
                </button>
              )}
              {outputExpanded && (
                <pre className="toolcall-output toolcall-output-full">{call.output}</pre>
              )}
            </div>
          )}
          {call.status === "failed" && (
            <div className="toolcall-section toolcall-error-section">
              <strong>Error Details</strong>
              <pre className="toolcall-output-error">{call.output || "Unknown error"}</pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
