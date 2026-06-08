/**
 * PromptManager —— 个性设置和技能管理面板
 *
 * 通过 Tauri invoke 直接调用后端命令：
 * - list_personalities / set_personality / get_current_personality
 * - list_skills
 *
 * 用户可以选择 AI 的"人格"（影响回复风格），
 * 以及查看当前可用的技能列表。
 */
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SkillInfo {
  name: string;
  description: string;
  path: string;
}

export function PromptManager() {
  const [personalities, setPersonalities] = useState<string[]>([]);
  const [currentPersonality, setCurrentPersonality] = useState("");
  const [skills, setSkills] = useState<SkillInfo[]>([]);

  useEffect(() => {
    invoke<string[]>("list_personalities").then(setPersonalities).catch(console.error);
    invoke<string>("get_current_personality").then(setCurrentPersonality).catch(() => {});
    invoke<SkillInfo[]>("list_skills").then(setSkills).catch(console.error);
  }, []);

  const handleSelectPersonality = async (name: string) => {
    try {
      await invoke("set_personality", { name });
      setCurrentPersonality(name);
    } catch (e) {
      console.error("Failed to set personality:", e);
    }
  };

  return (
    <div className="prompt-manager">
      <section>
        <h3>Personality</h3>
        <p className="panel-hint">Choose how nyamuWhale communicates.</p>
        <div className="personality-list">
          {personalities.length === 0 && (
            <p className="panel-hint">No personalities found.</p>
          )}
          {personalities.map((p) => (
            <button
              key={p}
              className={`personality-btn ${p === currentPersonality ? "active" : ""}`}
              onClick={() => handleSelectPersonality(p)}
            >
              <span>{p.charAt(0).toUpperCase() + p.slice(1)}</span>
              {p === currentPersonality && <span className="hist-item__badge">active</span>}
            </button>
          ))}
        </div>
      </section>

      <section style={{ marginTop: 16 }}>
        <h3>Skills</h3>
        <p className="panel-hint">Available skills from skill directories.</p>
        {skills.length === 0 && (
          <p className="panel-hint">No skills found.</p>
        )}
        <div className="personality-list">
          {skills.map((s) => (
            <div key={s.name} className="personality-btn" style={{ cursor: "default" }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: "0.85rem" }}>{s.name}</div>
                <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", marginTop: 2 }}>
                  {s.description || "No description"}
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

