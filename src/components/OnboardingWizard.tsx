/**
 * OnboardingWizard —— 首次运行引导组件
 *
 * 检测到未配置 API key 时显示引导流程：
 * 1. 欢迎页面
 * 2. API Key 输入
 * 3. 完成
 */
import { useState } from "react";
import { Key, Check, ArrowRight, ArrowLeft } from "lucide-react";
import * as bridge from "../lib/bridge";

interface OnboardingWizardProps {
  onComplete: () => void;
}

type Step = "welcome" | "api-key" | "done";

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const [step, setStep] = useState<Step>("welcome");
  const [apiKey, setApiKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const handleSaveKey = async () => {
    if (!apiKey.trim()) {
      setError("Please enter an API key.");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await bridge.connectKey(apiKey.trim());
      setStep("done");
    } catch (e: any) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-card">
        {step === "welcome" && (
          <>
            <h1>Welcome to nyamuWhale</h1>
            <p className="onboarding-sub">
              AI-powered coding assistant built on the nyamu engine.
            </p>
            <div className="onboarding-features">
              <div className="onboarding-feature">35+ built-in tools</div>
              <div className="onboarding-feature">3 execution modes</div>
              <div className="onboarding-feature">18 LLM providers</div>
              <div className="onboarding-feature">Real-time streaming</div>
            </div>
            <button className="btn btn-primary" onClick={() => setStep("api-key")}>
              Get Started <ArrowRight size={16} />
            </button>
          </>
        )}

        {step === "api-key" && (
          <>
            <h2>Configure API Key</h2>
            <p className="onboarding-sub">
              Enter your DeepSeek API key to get started.
              You can also set the <code>DEEPSEEK_API_KEY</code> environment variable.
            </p>
            <div className="onboarding-input-group">
              <Key size={16} />
              <input
                type="password"
                className="onboarding-input"
                placeholder="sk-..."
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSaveKey()}
                autoFocus
              />
            </div>
            {error && <p className="onboarding-error">{error}</p>}
            <div className="onboarding-actions">
              <button className="btn btn-secondary" onClick={() => setStep("welcome")}>
                <ArrowLeft size={14} /> Back
              </button>
              <button className="btn btn-primary" onClick={handleSaveKey} disabled={saving}>
                {saving ? "Saving..." : "Save & Continue"}
              </button>
            </div>
          </>
        )}

        {step === "done" && (
          <>
            <div className="onboarding-done-icon"><Check size={32} /></div>
            <h2>All Set!</h2>
            <p className="onboarding-sub">
              Your API key has been saved. You're ready to start coding with AI.
            </p>
            <button className="btn btn-primary" onClick={onComplete}>
              Start Using nyamuWhale
            </button>
          </>
        )}
      </div>
    </div>
  );
}
