/**
 * SettingsPanel —— 完整设置面板
 *
 * 8 个标签页：Models, Providers, Permissions, Sandbox,
 * Agent, Appearance, Network, Updates
 */
import { useState, useEffect } from "react";
import { X } from "lucide-react";
import * as bridge from "../lib/bridge";
import {
  ThemeMode, ThemeStyle, STYLE_NAMES, STYLE_PALETTES,
  loadTheme, saveTheme, applyTheme, resolveThemeMode,
} from "../lib/theme";

interface SettingsPanelProps {
  onClose: () => void;
}

type SettingsTab = "appearance" | "models" | "providers" | "permissions" | "sandbox" | "agent" | "network" | "updates";

const TABS: { id: SettingsTab; label: string }[] = [
  { id: "appearance", label: "Appearance" },
  { id: "models", label: "Models" },
  { id: "providers", label: "Providers" },
  { id: "permissions", label: "Permissions" },
  { id: "sandbox", label: "Sandbox" },
  { id: "agent", label: "Agent" },
  { id: "network", label: "Network" },
  { id: "updates", label: "Updates" },
];

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const [tab, setTab] = useState<SettingsTab>("appearance");

  return (
    <div className="settings-panel-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="icon-btn" onClick={onClose}><X size={16} /></button>
        </div>
        <div className="settings-body">
          <div className="settings-tabs">
            {TABS.map((t) => (
              <button
                key={t.id}
                className={`settings-tab ${tab === t.id ? "active" : ""}`}
                onClick={() => setTab(t.id)}
              >
                {t.label}
              </button>
            ))}
          </div>
          <div className="settings-content">
            {tab === "appearance" && <AppearanceTab />}
            {tab === "models" && <ModelsTab />}
            {tab === "providers" && <ProvidersTab />}
            {tab === "permissions" && <PermissionsTab />}
            {tab === "sandbox" && <SandboxTab />}
            {tab === "agent" && <AgentTab />}
            {tab === "network" && <NetworkTab />}
            {tab === "updates" && <UpdatesTab />}
          </div>
        </div>
      </div>
    </div>
  );
}

function AppearanceTab() {
  const [theme, setThemeState] = useState<{ mode: ThemeMode; style: ThemeStyle }>(() => loadTheme());

  const update = (partial: Partial<{ mode: ThemeMode; style: ThemeStyle }>) => {
    const next = { ...theme, ...partial };
    setThemeState(next);
    saveTheme(next);
    applyTheme(next);
  };

  const resolved = resolveThemeMode(theme.mode);
  const palette = STYLE_PALETTES[theme.style][resolved];

  return (
    <div className="settings-tab-content">
      <h3>Appearance</h3>
      <div className="setting-group">
        <label>Theme Mode</label>
        <div className="setting-options">
          {(["light", "dark", "auto"] as ThemeMode[]).map((m) => (
            <button
              key={m}
              className={`setting-option ${theme.mode === m ? "active" : ""}`}
              onClick={() => update({ mode: m })}
            >
              {m === "light" ? "☀ Light" : m === "dark" ? "☾ Dark" : "◐ Auto"}
            </button>
          ))}
        </div>
      </div>
      <div className="setting-group">
        <label>Color Palette</label>
        <div className="palette-grid">
          {STYLE_NAMES.map((s) => (
            <button
              key={s.value}
              className={`palette-card ${theme.style === s.value ? "active" : ""}`}
              onClick={() => update({ style: s.value })}
            >
              <div className="palette-swatch">
                <span style={{ background: palette["--accent"], width: 12, height: 12, borderRadius: "50%", display: "inline-block" }} />
                <span style={{ background: palette["--bg-primary"], width: 12, height: 12, borderRadius: "50%", border: "1px solid var(--border)", display: "inline-block" }} />
              </div>
              <span className="palette-name">{s.label}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function ModelsTab() {
  const [models, setModels] = useState<bridge.ModelInfo[]>([]);

  useEffect(() => {
    bridge.listModels().then(setModels).catch(() => {});
  }, []);

  const handleSelect = async (name: string) => {
    await bridge.setModel(name);
    setModels((prev) => prev.map((m) => ({ ...m, active: m.name === name })));
  };

  return (
    <div className="settings-tab-content">
      <h3>Models</h3>
      <p className="panel-hint">Select the default model for new conversations.</p>
      <div className="model-list">
        {models.map((m) => (
          <label key={m.name} className="model-item">
            <input
              type="radio"
              name="model"
              checked={m.active}
              onChange={() => handleSelect(m.name)}
            />
            <span className="model-name">{m.name}</span>
            {m.active && <span className="badge active">active</span>}
          </label>
        ))}
      </div>
    </div>
  );
}

function ProvidersTab() {
  const [config, setConfig] = useState<string>("");

  useEffect(() => {
    bridge.getConfig().then(setConfig).catch(() => setConfig("{}"));
  }, []);

  return (
    <div className="settings-tab-content">
      <h3>Providers</h3>
      <p className="panel-hint">API providers and model endpoints.</p>
      <div className="config-view">
        <pre>{config}</pre>
      </div>
      <p className="panel-hint" style={{ marginTop: 8 }}>Edit providers via the configuration file.</p>
    </div>
  );
}

function PermissionsTab() {
  return (
    <div className="settings-tab-content">
      <h3>Permissions</h3>
      <p className="panel-hint">Tool approval rules come from the ExecPolicy engine.</p>
      <div className="info-cards">
        <div className="info-card"><strong>Agent mode</strong><span>All tools available, approval per config</span></div>
        <div className="info-card"><strong>Plan mode</strong><span>Read-only tools only</span></div>
        <div className="info-card"><strong>YOLO mode</strong><span>All tools auto-approved</span></div>
      </div>
    </div>
  );
}

function SandboxTab() {
  return (
    <div className="settings-tab-content">
      <h3>Sandbox</h3>
      <p className="panel-hint">Execution sandbox settings for shell commands.</p>
      <div className="info-cards">
        <div className="info-card"><strong>Status</strong><span>Sandbox active</span></div>
        <div className="info-card"><strong>Network</strong><span>Enabled</span></div>
      </div>
    </div>
  );
}

function AgentTab() {
  return (
    <div className="settings-tab-content">
      <h3>Agent Settings</h3>
      <p className="panel-hint">Agent behavior parameters.</p>
      <div className="info-cards">
        <div className="info-card"><strong>Max tokens</strong><span>16384</span></div>
        <div className="info-card"><strong>Reasoning effort</strong><span>max</span></div>
      </div>
    </div>
  );
}

function NetworkTab() {
  return (
    <div className="settings-tab-content">
      <h3>Network</h3>
      <p className="panel-hint">Proxy and network settings.</p>
      <p className="panel-hint">Configured via system environment or config file.</p>
    </div>
  );
}

function UpdatesTab() {
  return (
    <div className="settings-tab-content">
      <h3>Updates</h3>
      <p className="panel-hint">Check for new versions of nyamuWhale.</p>
      <div className="info-cards">
        <div className="info-card"><strong>Version</strong><span>0.8.53</span></div>
      </div>
    </div>
  );
}
