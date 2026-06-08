/**
 * theme.ts —— 主题系统
 *
 * 三种模式：light / dark / auto（跟随系统）
 * 八套调色板：pastel, graphite, ember, aurora, midnight, sandstone, porcelain, glacier
 *
 * 持久化到 localStorage，通过 CSS data 属性应用：
 * - data-theme: "light" | "dark"
 * - data-theme-style: 调色板名称
 * - data-theme-scheme: 浏览器 prefers-color-scheme
 */

export type ThemeMode = "light" | "dark" | "auto";
export type ThemeStyle =
  | "pastel" | "graphite" | "ember" | "aurora"
  | "midnight" | "sandstone" | "porcelain" | "glacier";

export interface ThemeState {
  mode: ThemeMode;
  style: ThemeStyle;
}

const STORAGE_KEY_MODE = "nyamuwhale-theme";
const STORAGE_KEY_STYLE = "nyamuwhale-theme-style";
const DEFAULT_THEME: ThemeState = { mode: "light", style: "pastel" };

/** 将调色板映射到 light/dark 两套色值 */
export const STYLE_PALETTES: Record<ThemeStyle, { light: Record<string, string>; dark: Record<string, string> }> = {
  pastel: {
    light: {
      "--bg-primary": "#FDFBF7", "--bg-secondary": "#FFFCF8",
      "--bg-tertiary": "#F9F6F0", "--bg-surface": "#F4F1EB",
      "--bg-hover": "#EDE9E0",
      "--text-primary": "#3D362D", "--text-secondary": "#A69B8A", "--text-muted": "#8E8576",
      "--accent": "#FFB5C2", "--accent-hover": "#FFC5CF", "--accent-dim": "rgba(255,181,194,0.2)",
      "--border": "#D9D2C5", "--border-light": "#E5DFD3",
      "--user-msg-bg": "#FFF5F7", "--assistant-msg-bg": "#FAFAF8",
      "--success": "#7BC8A4", "--warning": "#F5C5A3", "--error": "#E07373",
    },
    dark: {
      "--bg-primary": "#1a1a2e", "--bg-secondary": "#16213e",
      "--bg-tertiary": "#1f1f35", "--bg-surface": "#252542",
      "--bg-hover": "#2a2a48",
      "--text-primary": "#e4e4e7", "--text-secondary": "#a1a1aa", "--text-muted": "#71717a",
      "--accent": "#FFB5C2", "--accent-hover": "#FFC5CF", "--accent-dim": "rgba(255,181,194,0.15)",
      "--border": "#3f3f5c", "--border-light": "#353550",
      "--user-msg-bg": "#1e1e38", "--assistant-msg-bg": "#1c1c30",
      "--success": "#7BC8A4", "--warning": "#F5C5A3", "--error": "#E07373",
    },
  },
  graphite: {
    light: {
      "--bg-primary": "#f8f9fa", "--bg-secondary": "#ffffff",
      "--bg-tertiary": "#f1f3f5", "--bg-surface": "#e9ecef",
      "--bg-hover": "#dee2e6",
      "--text-primary": "#212529", "--text-secondary": "#868e96", "--text-muted": "#adb5bd",
      "--accent": "#4dabf7", "--accent-hover": "#74c0fc", "--accent-dim": "rgba(77,171,247,0.2)",
      "--border": "#ced4da", "--border-light": "#dee2e6",
      "--user-msg-bg": "#e7f5ff", "--assistant-msg-bg": "#f8f9fa",
      "--success": "#51cf66", "--warning": "#fcc419", "--error": "#ff6b6b",
    },
    dark: {
      "--bg-primary": "#1e1e1e", "--bg-secondary": "#252525",
      "--bg-tertiary": "#2d2d2d", "--bg-surface": "#333333",
      "--bg-hover": "#3a3a3a",
      "--text-primary": "#e0e0e0", "--text-secondary": "#a0a0a0", "--text-muted": "#707070",
      "--accent": "#4dabf7", "--accent-hover": "#74c0fc", "--accent-dim": "rgba(77,171,247,0.15)",
      "--border": "#404040", "--border-light": "#353535",
      "--user-msg-bg": "#1a2630", "--assistant-msg-bg": "#232323",
      "--success": "#51cf66", "--warning": "#fcc419", "--error": "#ff6b6b",
    },
  },
  ember: {
    light: {
      "--bg-primary": "#fef9f0", "--bg-secondary": "#fffdf8",
      "--bg-tertiary": "#fdf3e0", "--bg-surface": "#f7e9d3",
      "--bg-hover": "#f0dcc4",
      "--text-primary": "#3d2e1e", "--text-secondary": "#b08a6a", "--text-muted": "#8c6e52",
      "--accent": "#e8590c", "--accent-hover": "#f76707", "--accent-dim": "rgba(232,89,12,0.2)",
      "--border": "#d4c1a8", "--border-light": "#e0cfba",
      "--user-msg-bg": "#fff4e6", "--assistant-msg-bg": "#fef9f0",
      "--success": "#2f9e44", "--warning": "#e8590c", "--error": "#c92a2a",
    },
    dark: {
      "--bg-primary": "#1a1410", "--bg-secondary": "#221b15",
      "--bg-tertiary": "#2a221b", "--bg-surface": "#332922",
      "--bg-hover": "#3d3026",
      "--text-primary": "#e0d5c8", "--text-secondary": "#a89078", "--text-muted": "#807060",
      "--accent": "#e8590c", "--accent-hover": "#f76707", "--accent-dim": "rgba(232,89,12,0.15)",
      "--border": "#4a3a2a", "--border-light": "#3d3026",
      "--user-msg-bg": "#2a1e14", "--assistant-msg-bg": "#1c1610",
      "--success": "#2f9e44", "--warning": "#e8590c", "--error": "#c92a2a",
    },
  },
  aurora: {
    light: {
      "--bg-primary": "#f0faf5", "--bg-secondary": "#f8fffb",
      "--bg-tertiary": "#e6f5ec", "--bg-surface": "#dceee2",
      "--bg-hover": "#cee6d6",
      "--text-primary": "#1e3a2f", "--text-secondary": "#6a9e82", "--text-muted": "#55806c",
      "--accent": "#099268", "--accent-hover": "#0ca678", "--accent-dim": "rgba(9,146,104,0.2)",
      "--border": "#b8d4c4", "--border-light": "#cce0d4",
      "--user-msg-bg": "#e0f5ec", "--assistant-msg-bg": "#f0faf5",
      "--success": "#2f9e44", "--warning": "#e67700", "--error": "#c92a2a",
    },
    dark: {
      "--bg-primary": "#0f1a16", "--bg-secondary": "#15221c",
      "--bg-tertiary": "#1a2a22", "--bg-surface": "#203328",
      "--bg-hover": "#263d30",
      "--text-primary": "#c8ddd2", "--text-secondary": "#80a890", "--text-muted": "#608070",
      "--accent": "#099268", "--accent-hover": "#0ca678", "--accent-dim": "rgba(9,146,104,0.15)",
      "--border": "#2a4a3a", "--border-light": "#203d30",
      "--user-msg-bg": "#142a20", "--assistant-msg-bg": "#101c16",
      "--success": "#2f9e44", "--warning": "#e67700", "--error": "#c92a2a",
    },
  },
  midnight: {
    light: {
      "--bg-primary": "#f0f2f8", "--bg-secondary": "#f8f9fd",
      "--bg-tertiary": "#e6e9f2", "--bg-surface": "#dde0ec",
      "--bg-hover": "#d0d4e4",
      "--text-primary": "#1e2235", "--text-secondary": "#6e7499", "--text-muted": "#585d80",
      "--accent": "#5c6bc0", "--accent-hover": "#7986cb", "--accent-dim": "rgba(92,107,192,0.2)",
      "--border": "#bcc2d4", "--border-light": "#cccfe0",
      "--user-msg-bg": "#e8ebf5", "--assistant-msg-bg": "#f0f2f8",
      "--success": "#66bb6a", "--warning": "#ffa726", "--error": "#ef5350",
    },
    dark: {
      "--bg-primary": "#121220", "--bg-secondary": "#181828",
      "--bg-tertiary": "#1e1e30", "--bg-surface": "#242438",
      "--bg-hover": "#2a2a40",
      "--text-primary": "#d0d0e0", "--text-secondary": "#8888a0", "--text-muted": "#686880",
      "--accent": "#5c6bc0", "--accent-hover": "#7986cb", "--accent-dim": "rgba(92,107,192,0.15)",
      "--border": "#303048", "--border-light": "#282840",
      "--user-msg-bg": "#1a1a30", "--assistant-msg-bg": "#141428",
      "--success": "#66bb6a", "--warning": "#ffa726", "--error": "#ef5350",
    },
  },
  sandstone: {
    light: {
      "--bg-primary": "#f8f4eb", "--bg-secondary": "#fefaf0",
      "--bg-tertiary": "#f0ebe0", "--bg-surface": "#e8e0d0",
      "--bg-hover": "#ddd5c4",
      "--text-primary": "#3a3228", "--text-secondary": "#9a8a70", "--text-muted": "#807058",
      "--accent": "#c49a5c", "--accent-hover": "#d4aa6c", "--accent-dim": "rgba(196,154,92,0.2)",
      "--border": "#c8bcac", "--border-light": "#d8cebe",
      "--user-msg-bg": "#f0e8d8", "--assistant-msg-bg": "#f8f4eb",
      "--success": "#7cb342", "--warning": "#d4a040", "--error": "#c04040",
    },
    dark: {
      "--bg-primary": "#1c1812", "--bg-secondary": "#242018",
      "--bg-tertiary": "#2c2820", "--bg-surface": "#343028",
      "--bg-hover": "#3d382e",
      "--text-primary": "#d8d0c0", "--text-secondary": "#988878", "--text-muted": "#786860",
      "--accent": "#c49a5c", "--accent-hover": "#d4aa6c", "--accent-dim": "rgba(196,154,92,0.15)",
      "--border": "#423a30", "--border-light": "#383028",
      "--user-msg-bg": "#282016", "--assistant-msg-bg": "#1e1a14",
      "--success": "#7cb342", "--warning": "#d4a040", "--error": "#c04040",
    },
  },
  porcelain: {
    light: {
      "--bg-primary": "#fafafa", "--bg-secondary": "#ffffff",
      "--bg-tertiary": "#f2f2f2", "--bg-surface": "#ececec",
      "--bg-hover": "#e4e4e4",
      "--text-primary": "#262626", "--text-secondary": "#8c8c8c", "--text-muted": "#737373",
      "--accent": "#9c89b8", "--accent-hover": "#b09ac8", "--accent-dim": "rgba(156,137,184,0.2)",
      "--border": "#d4d4d4", "--border-light": "#e0e0e0",
      "--user-msg-bg": "#f0ecf5", "--assistant-msg-bg": "#fafafa",
      "--success": "#9c89b8", "--warning": "#d4a0a0", "--error": "#b07070",
    },
    dark: {
      "--bg-primary": "#1a1a1a", "--bg-secondary": "#222222",
      "--bg-tertiary": "#2a2a2a", "--bg-surface": "#303030",
      "--bg-hover": "#383838",
      "--text-primary": "#d4d4d4", "--text-secondary": "#8c8c8c", "--text-muted": "#6a6a6a",
      "--accent": "#9c89b8", "--accent-hover": "#b09ac8", "--accent-dim": "rgba(156,137,184,0.15)",
      "--border": "#383838", "--border-light": "#303030",
      "--user-msg-bg": "#24202e", "--assistant-msg-bg": "#1c1c1c",
      "--success": "#9c89b8", "--warning": "#d4a0a0", "--error": "#b07070",
    },
  },
  glacier: {
    light: {
      "--bg-primary": "#eef5f8", "--bg-secondary": "#f5fafc",
      "--bg-tertiary": "#e2edf2", "--bg-surface": "#d8e6ec",
      "--bg-hover": "#ccdce4",
      "--text-primary": "#1a2e38", "--text-secondary": "#5a8a9a", "--text-muted": "#4a707a",
      "--accent": "#3b9ec9", "--accent-hover": "#54b0d8", "--accent-dim": "rgba(59,158,201,0.2)",
      "--border": "#aac4d0", "--border-light": "#c0d4de",
      "--user-msg-bg": "#dcecf5", "--assistant-msg-bg": "#eef5f8",
      "--success": "#5cb85c", "--warning": "#f0ad4e", "--error": "#d9534f",
    },
    dark: {
      "--bg-primary": "#0e1a20", "--bg-secondary": "#14222a",
      "--bg-tertiary": "#1a2a32", "--bg-surface": "#203238",
      "--bg-hover": "#263d44",
      "--text-primary": "#c0d8e0", "--text-secondary": "#7098a8", "--text-muted": "#587880",
      "--accent": "#3b9ec9", "--accent-hover": "#54b0d8", "--accent-dim": "rgba(59,158,201,0.15)",
      "--border": "#224048", "--border-light": "#1c343e",
      "--user-msg-bg": "#162a34", "--assistant-msg-bg": "#101c22",
      "--success": "#5cb85c", "--warning": "#f0ad4e", "--error": "#d9534f",
    },
  },
};

export const STYLE_NAMES: { value: ThemeStyle; label: string }[] = [
  { value: "pastel", label: "Pastel Dream" },
  { value: "graphite", label: "Graphite" },
  { value: "ember", label: "Ember" },
  { value: "aurora", label: "Aurora" },
  { value: "midnight", label: "Midnight" },
  { value: "sandstone", label: "Sandstone" },
  { value: "porcelain", label: "Porcelain" },
  { value: "glacier", label: "Glacier" },
];

/** 从 localStorage 或默认值加载主题状态 */
export function loadTheme(): ThemeState {
  let mode: ThemeMode = DEFAULT_THEME.mode;
  let style: ThemeStyle = DEFAULT_THEME.style;
  try {
    const m = localStorage.getItem(STORAGE_KEY_MODE);
    if (m === "light" || m === "dark" || m === "auto") mode = m;
    const s = localStorage.getItem(STORAGE_KEY_STYLE);
    if (s && Object.keys(STYLE_PALETTES).includes(s)) style = s as ThemeStyle;
  } catch { /* ignore */ }
  return { mode, style };
}

/** 保存主题状态到 localStorage */
export function saveTheme(state: ThemeState) {
  try {
    localStorage.setItem(STORAGE_KEY_MODE, state.mode);
    localStorage.setItem(STORAGE_KEY_STYLE, state.style);
  } catch { /* ignore */ }
}

/** 根据 mode + 系统偏好 解析出实际 light/dark */
export function resolveThemeMode(mode: ThemeMode): "light" | "dark" {
  if (mode === "auto") {
    if (typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches) {
      return "dark";
    }
    return "light";
  }
  return mode;
}

/** 将主题状态应用到 document.documentElement */
export function applyTheme(state: ThemeState) {
  const resolved = resolveThemeMode(state.mode);
  const palette = STYLE_PALETTES[state.style];
  const vars = palette[resolved];
  const root = document.documentElement;

  root.setAttribute("data-theme", resolved);
  root.setAttribute("data-theme-style", state.style);
  root.setAttribute("data-theme-scheme", state.mode);

  // 应用 CSS 变量
  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }
}

/** 检测系统主题变化，返回取消监听函数 */
export function listenSystemTheme(callback: (isDark: boolean) => void): () => void {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = (e: MediaQueryListEvent) => callback(e.matches);
  mq.addEventListener("change", handler);
  return () => mq.removeEventListener("change", handler);
}
