//! 技能（Skill）管理模块
//!
//! Skill 是从特定目录（~/.deepseek/skills/、~/.nyamu/skills/ 等）加载的
//! SKILL.md 文件，每个 Skill 包含一个提示词模板，注入到系统提示词中。
//!
//! SkillStore 管理技能的启用/禁用状态，持久化到 ~/.nyamu/skills-state.json。
//!
//! 支持 CLI 命令：`deepwhale --cli skill list/enable/disable`
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub path: String,
    pub enabled: bool,
}

/// Persists skill enable/disable state to a JSON file.
pub struct SkillStore {
    path: PathBuf,
    enabled: HashMap<String, bool>,
}

impl SkillStore {
    /// Default path at `~/.nyamu/skills-state.json`.
    pub fn new() -> Self {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_default();
        let path = PathBuf::from(home).join(".nyamu").join("skills-state.json");
        SkillStore {
            path,
            enabled: HashMap::new(),
        }
    }

    /// Load state from the JSON file. Returns an empty store if the file does not exist.
    pub fn load(&self) -> Self {
        let content = match std::fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => {
                return SkillStore {
                    path: self.path.clone(),
                    enabled: HashMap::new(),
                };
            }
        };
        let enabled: HashMap<String, bool> =
            serde_json::from_str(&content).unwrap_or_default();
        SkillStore {
            path: self.path.clone(),
            enabled,
        }
    }

    /// Save state to the JSON file.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create ~/.nyamu directory for skills-state")?;
        }
        let content = serde_json::to_string_pretty(&self.enabled)
            .context("Failed to serialize skill state")?;
        std::fs::write(&self.path, content)
            .context("Failed to write skills-state.json")?;
        Ok(())
    }

    /// Check whether a skill is enabled. Skills default to enabled if not recorded.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.get(name).copied().unwrap_or(true)
    }

    /// Set the enabled state for a skill and persist immediately.
    pub fn set_enabled(&self, name: &str, enabled: bool) -> Result<()> {
        let mut map = self.enabled.clone();
        map.insert(name.to_string(), enabled);
        let store = SkillStore {
            path: self.path.clone(),
            enabled: map,
        };
        store.save()?;
        Ok(())
    }

    /// Return a list of skill names that are enabled.
    #[allow(dead_code)]
    pub fn list_enabled(&self) -> Vec<String> {
        self.enabled
            .iter()
            .filter(|&(_, &v)| v)
            .map(|(k, _)| k.clone())
            .collect()
    }
}

/// Enable a skill by name. Persists the setting.
pub fn enable_skill(name: &str) -> Result<()> {
    let store = SkillStore::new().load();
    store.set_enabled(name, true)
}

/// Disable a skill by name. Persists the setting.
pub fn disable_skill(name: &str) -> Result<()> {
    let store = SkillStore::new().load();
    store.set_enabled(name, false)
}

fn scan_dir(dir: &PathBuf) -> Vec<SkillInfo> {
    let store = SkillStore::new().load();
    let mut skills = Vec::new();
    if !dir.exists() {
        return skills;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    if let Some(name) = path.file_name() {
                        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                        // Extract first line as description
                        let desc = content.lines()
                            .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        let name_str = name.to_string_lossy().to_string();
                        skills.push(SkillInfo {
                            name: name_str.clone(),
                            description: desc,
                            path: skill_md.to_string_lossy().to_string(),
                            enabled: store.is_enabled(&name_str),
                        });
                    }
                }
            }
        }
    }
    skills
}

/// Scan all skill directories for SKILL.md files.
pub fn list_skills() -> Vec<SkillInfo> {
    let mut skills = Vec::new();

    // Check common skill locations
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    for subdir in &[".deepseek/skills", ".nyamu/skills", ".claude/skills"] {
        let path = home.join(subdir);
        skills.extend(scan_dir(&path));
    }

    skills
}
