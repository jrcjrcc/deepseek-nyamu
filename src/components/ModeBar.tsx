/**
 * ModeBar —— 模式选择器（Agent / Plan / Yolo）
 *
 * 三种操作模式：
 * - Agent：自主执行，写操作需要审批（默认模式）
 * - Plan：只读调查模式，禁止文件写入和 Shell 执行
 * - Yolo：完全自主，所有工具自动批准
 *
 * 点击当前激活的模式标签可显示模式说明信息提示。
 */
import { Zap, ClipboardCheck, Target, Info } from "lucide-react";
import { useState } from "react";

export type Mode = "agent" | "plan" | "yolo";

interface ModeBarProps {
  mode: Mode;
  onCycleMode: () => void;
}

const MODE_ICONS: Record<Mode, React.ReactNode> = {
  agent: <Target size={14} />,
  plan: <ClipboardCheck size={14} />,
  yolo: <Zap size={14} />,
};

const MODE_COLORS: Record<Mode, string> = {
  agent: "#C9B1E8",
  plan: "#80D0C0",
  yolo: "#FABF8C",
};

const MODE_DESC: Record<Mode, string> = {
  agent: "Autonomous task execution with tools. Writes require approval.",
  plan: "Read-only investigation mode. No file writes or shell execution.",
  yolo: "Full autonomy. All tools auto-approved, no confirmation prompts.",
};

export function ModeBar({ mode, onCycleMode }: ModeBarProps) {
  const [showInfo, setShowInfo] = useState(false);
  const modes: Mode[] = ["agent", "plan", "yolo"];

  return (
    <div className="mode-bar">
      <span className="mode-bar-label">Mode</span>
      <div className="mode-bar-tabs">
        {modes.map((m) => (
          <button
            key={m}
            className={`mode-tab ${m === mode ? "active" : ""}`}
            style={m === mode ? { borderColor: MODE_COLORS[m], color: MODE_COLORS[m] } : {}}
            onClick={() => {
              if (m === mode) { setShowInfo(!showInfo); return; }
              let current = mode;
              while (current !== m) {
                current = modes[(modes.indexOf(current) + 1) % modes.length];
              }
              onCycleMode();
            }}
          >
            {MODE_ICONS[m]}
            <span>{m.charAt(0).toUpperCase() + m.slice(1)}</span>
          </button>
        ))}
      </div>
      <button className="mode-cycle-btn" onClick={onCycleMode} title="Cycle mode (Shift+Tab)">
        <span className="mode-kbd">⇧⇥</span>
      </button>
      {showInfo && (
        <div className="mode-info" onClick={() => setShowInfo(false)}>
          {MODE_DESC[mode]}
        </div>
      )}
    </div>
  );
}
