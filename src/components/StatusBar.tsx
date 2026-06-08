/**
 * StatusBar —— 底部状态栏（增强版）
 *
 * 展示：
 * - 连接状态
 * - 模型选择器 + 推理力度选择器
 * - 上下文仪表（用量进度条）
 * - Token 用量 + 缓存命中率 + 费用
 * - 耗时显示
 */
import { useState, useEffect, useRef } from "react";
import type { UsageEvent, ModelInfo } from "../lib/bridge";
import * as bridge from "../lib/bridge";

interface StatusBarProps {
  sessionCount: number;
  isWaiting: boolean;
  modelName?: string;
  reasoningEffort?: string;
}

export function StatusBar({ sessionCount, isWaiting, modelName, reasoningEffort }: StatusBarProps) {
  const [usage, setUsage] = useState<UsageEvent | null>(null);
  const [totalCost, setTotalCost] = useState(0);
  const [totalTokens, setTotalTokens] = useState({ in: 0, out: 0 });
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [showEffortPicker, setShowEffortPicker] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [effort, setEffort] = useState(reasoningEffort || "max");
  const timerRef = useRef<ReturnType<typeof setInterval>>();
  const modelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge.onUsage((event: UsageEvent) => {
      setUsage(event);
      setTotalCost((prev) => prev + event.cost_dollars);
      setTotalTokens((prev) => ({
        in: prev.in + event.input_tokens,
        out: prev.out + event.output_tokens,
      }));
    }).then((u) => { unlisten = u; });

    bridge.listModels().then(setModels).catch(() => {});

    return () => { if (unlisten) unlisten(); };
  }, []);

  // Elapsed timer
  useEffect(() => {
    if (isWaiting) {
      setElapsed(0);
      timerRef.current = setInterval(() => setElapsed((p) => p + 1), 1000);
    } else {
      clearInterval(timerRef.current);
    }
    return () => clearInterval(timerRef.current);
  }, [isWaiting]);

  // Close model picker on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (modelRef.current && !modelRef.current.contains(e.target as Node)) {
        setShowModelPicker(false);
        setShowEffortPicker(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const fmtTokens = (n: number) => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return n.toString();
  };

  const fmtCost = (n: number) => {
    if (n < 0.001) return `${(n * 100_000).toFixed(0)}μ$ `;
    return `$${n.toFixed(4)}`;
  };

  const fmtTime = (s: number) => {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m > 0 ? `${m}m ${sec}s` : `${sec}s`;
  };

  const activeModel = models.find((m) => m.active)?.name || modelName || "deepseek-v4-flash";
  const cacheTotal = usage ? usage.cache_hit_tokens + usage.cache_miss_tokens : 0;
  const cacheRate = usage && usage.input_tokens > 0
    ? ((usage.cache_hit_tokens / (usage.input_tokens + cacheTotal)) * 100).toFixed(0)
    : "0";

  // Context gauge: percentage of 128K context used
  const contextPct = Math.min(100, Math.round((totalTokens.in + totalTokens.out) / 1280));
  const effortLevels = ["off", "low", "medium", "high", "max"];

  const handleModelChange = async (name: string) => {
    await bridge.setModel(name);
    setShowModelPicker(false);
    setModels((prev) => prev.map((m) => ({ ...m, active: m.name === name })));
  };

  const handleEffortChange = async (level: string) => {
    await bridge.setEffort(level);
    setEffort(level);
    setShowEffortPicker(false);
  };

  return (
    <div className="status-bar">
      <div className="status-section">
        <span className={`status-indicator ${isWaiting ? "working" : ""}`} />
        <span>{isWaiting ? "Working..." : "Ready"}</span>
        {isWaiting && <span className="status-elapsed">{fmtTime(elapsed)}</span>}
      </div>

      <div className="status-section status-center">
        {/* Context gauge */}
        <div className="context-gauge" title={`Context: ~${contextPct}% used`}>
          <div className="context-gauge-fill" style={{ width: `${contextPct}%` }} />
        </div>

        {/* Model picker */}
        <div className="status-picker-wrapper" ref={modelRef}>
          <span className="status-chip status-clickable" onClick={() => { setShowModelPicker(!showModelPicker); setShowEffortPicker(false); }}>
            {activeModel}
          </span>
          {showModelPicker && (
            <div className="status-dropdown">
              {models.map((m) => (
                <button key={m.name} className={`status-dropdown-item ${m.active ? "active" : ""}`} onClick={() => handleModelChange(m.name)}>
                  {m.name}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Effort picker */}
        <div className="status-picker-wrapper">
          <span className="status-chip status-clickable" onClick={() => { setShowEffortPicker(!showEffortPicker); setShowModelPicker(false); }}>
            effort: {effort}
          </span>
          {showEffortPicker && (
            <div className="status-dropdown">
              {effortLevels.map((e) => (
                <button key={e} className={`status-dropdown-item ${effort === e ? "active" : ""}`} onClick={() => handleEffortChange(e)}>
                  {e}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Tokens */}
        {usage && (
          <>
            <span className="status-sep">|</span>
            <span className="status-chip" title={`In: ${usage.input_tokens} | Out: ${usage.output_tokens}`}>
              {fmtTokens(usage.input_tokens)}→{fmtTokens(usage.output_tokens)}t
            </span>
            <span className="status-chip cache-chip" title="Cache hit rate">${cacheRate}% hit</span>
            <span className="status-chip cost-chip">{fmtCost(totalCost)}</span>
          </>
        )}
      </div>

      <div className="status-section">
        <span>{sessionCount} session{sessionCount !== 1 ? "s" : ""}</span>
        <span className="status-sep">路</span>
        <span>nyamuWhale</span>
      </div>
    </div>
  );
}
