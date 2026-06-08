/**
 * ThinkingBar —— AI 推理过程实时展示组件
 *
 * 当 AI 正在进行内部推理/思考时，显示实时的推理文本流。
 * 展示 Token 计数器，让用户了解推理过程的长度和进度。
 * 推理完成后此组件消失，推理内容移交到 Message 组件的 reasoning 折叠区。
 */
import { Loader } from "lucide-react";

interface ThinkingBarProps {
  reasoning: string;
}

export function ThinkingBar({ reasoning }: ThinkingBarProps) {
  if (!reasoning) return null;

  return (
    <div className="thinking-bar">
      <div className="thinking-bar-header">
        <Loader size={14} className="spin" />
        <span>Thinking...</span>
        <span className="thinking-bar-token-count">{reasoning.length} tokens</span>
      </div>
      <div className="thinking-bar-content">{reasoning}</div>
    </div>
  );
}
