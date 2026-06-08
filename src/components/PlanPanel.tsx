/**
 * PlanPanel —— 计划状态面板
 *
 * 展示 LLM 通过 update_plan 工具创建的会话执行计划。
 * 实时响应 plan:updated 事件，显示计划步骤和状态。
 */
import { useState, useEffect, useCallback } from "react";
import * as bridge from "../lib/bridge";

interface PlanStep {
  text?: string;
  status?: string;
  notes?: string;
  [key: string]: any;
}

interface Plan {
  id: string;
  title?: string;
  steps?: PlanStep[];
  current_step?: number;
  status?: string;
  [key: string]: any;
}

function getStepIcon(status?: string): string {
  switch (status) {
    case "completed": return "✓";
    case "in_progress": return "●";
    case "failed": return "✗";
    default: return "○";
  }
}

export function PlanPanel() {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchPlans = useCallback(() => {
    bridge.getPlans()
      .then((data: any[]) => {
        setPlans(data || []);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  // Initial fetch + poll fallback
  useEffect(() => {
    fetchPlans();
    const interval = setInterval(fetchPlans, 3000);
    return () => clearInterval(interval);
  }, [fetchPlans]);

  // Listen for real-time plan:updated events
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bridge.onPlanUpdated(() => {
      fetchPlans();
    }).then((u) => { unlisten = u; });
    return () => { if (unlisten) unlisten(); };
  }, [fetchPlans]);

  if (loading) {
    return (
      <div className="panel-content">
        <h3>Plan</h3>
        <p className="panel-hint">Loading plans...</p>
      </div>
    );
  }

  if (plans.length === 0) {
    return (
      <div className="panel-content">
        <h3>Plan</h3>
        <p className="panel-hint">No plans yet. Ask the AI to create a plan.</p>
      </div>
    );
  }

  return (
    <div className="panel-content">
      <h3>Plan</h3>
      <div className="plan-panel">
        {plans.map((plan) => (
          <div key={plan.id} className="plan-card">
            {/* Plan header */}
            {plan.title && (
              <div className="plan-header">
                <span className="plan-title">{plan.title}</span>
                <span className={`plan-status-badge plan-status-${plan.status || "active"}`}>
                  {plan.status || "active"}
                </span>
              </div>
            )}

            {/* Explanation (if any, from the first step that has text but no status) */}
            {plan.explanation && (
              <div className="plan-explanation">{plan.explanation}</div>
            )}

            {/* Steps list */}
            {plan.steps && plan.steps.length > 0 && (
              <div className="plan-steps">
                {plan.steps.map((step, idx) => {
                  const stepStatus = step.status || "pending";
                  const isCurrent = plan.current_step !== undefined && idx === plan.current_step - 1;
                  return (
                    <div
                      key={idx}
                      className={`plan-step plan-step-${stepStatus} ${isCurrent ? "plan-step-current" : ""}`}
                    >
                      <span className="plan-step-icon">{getStepIcon(stepStatus)}</span>
                      <div className="plan-step-body">
                        <span className="plan-step-text">{step.text || step.step || `Step ${idx + 1}`}</span>
                        {step.notes && (
                          <span className="plan-step-notes">{step.notes}</span>
                        )}
                      </div>
                      {isCurrent && <span className="plan-step-marker">current</span>}
                    </div>
                  );
                })}
              </div>
            )}

            {/* Empty plan */}
            {(!plan.steps || plan.steps.length === 0) && (
              <p className="plan-empty">Plan has no steps defined.</p>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
