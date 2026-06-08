//! 提示词组装模块
//!
//! 本模块负责构建 DeepWhale 的系统提示词（System Prompt），
//! 将多个提示词组件按规则组合成最终的完整提示词。
//!
//! 主要功能：
//! - 加载宪法（base.md）作为基础系统提示
//! - 加载模式特定提示（modes/ 目录）
//! - 管理个性化角色设定（personalities/ 目录）
//! - 加载已启用的技能定义（SKILL.md）
//! - 将以上所有组件拼接为最终系统提示

use std::path::PathBuf;
use std::sync::Mutex;

/// 当前激活的角色设定文本（线程安全的全局状态）
///
/// 通过 `Mutex` 保护，可在多线程环境下安全读取和更新。
static CURRENT_PERSONALITY: Mutex<String> = Mutex::new(String::new());

/// 获取 prompts 目录的路径
///
/// 路径结构：`{项目根}/vendor/nyamu/prompts/`
/// 从 `CARGO_MANIFEST_DIR`（src-tauri）向上退一级到项目根，
/// 然后依次进入 vendor/nyamu/prompts。
fn prompts_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // src-tauri → nyamuwhale（项目根）
    p.push("vendor");
    p.push("nyamu");
    p.push("prompts");
    p
}

/// 加载宪法文件（base.md）作为系统提示的基础内容
///
/// 宪法定义了 AI 助手的基本行为准则和身份设定。
/// 如果文件不存在或读取失败，返回一个默认的简短身份描述作为回退。
pub fn load_constitution() -> String {
    let path = prompts_dir().join("base.md");
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        "You are DeepWhale, a helpful AI coding assistant.".to_string()
    })
}

/// 设置当前激活的角色设定
///
/// 从 `personalities/` 目录下加载指定名称的 Markdown 文件作为角色设定内容。
/// 角色设定文件命名规则：`{name}.md`
///
/// # 参数
/// * `name` — 角色设定名称（不含 .md 后缀）
///
/// # 返回值
/// * `Ok(())` — 加载成功
/// * `Err(String)` — 角色设定文件未找到或读取失败
///
/// # 示例
/// ```ignore
/// set_personality("calm")?;  // 加载 personalities/calm.md
/// set_personality("playful")?; // 加载 personalities/playful.md
/// ```
pub fn set_personality(name: &str) -> Result<(), String> {
    let path = prompts_dir().join("personalities").join(format!("{}.md", name));
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Personality '{name}' not found: {e}"))?;
    let mut p = CURRENT_PERSONALITY.lock().unwrap();
    *p = content;
    Ok(())
}

/// 获取当前激活的角色设定文本内容
///
/// 返回之前通过 [`set_personality`] 设置的完整文本内容。
/// 如果未设置角色，返回空字符串。
pub fn get_personality() -> String {
    CURRENT_PERSONALITY.lock().unwrap().clone()
}

/// 获取当前激活的角色设定名称
///
/// 通过比对 CURRENT_PERSONALITY 内容与 personalities/ 目录下的文件，
/// 返回匹配的角色设定文件名（不含 .md 后缀）。
pub fn get_personality_name() -> String {
    let current = CURRENT_PERSONALITY.lock().unwrap().clone();
    if current.is_empty() {
        return String::new();
    }
    let dir = prompts_dir().join("personalities");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.trim() == current.trim() {
                        if let Some(name) = path.file_stem() {
                            return name.to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
    }
    String::new()
}

/// 列出所有可用的角色设定
///
/// 扫描 `personalities/` 目录下的所有 `.md` 文件，
/// 返回文件名的 stem（不含扩展名）列表。
pub fn list_personalities() -> Vec<String> {
    let dir = prompts_dir().join("personalities");
    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.path().file_stem() {
                list.push(name.to_string_lossy().to_string());
            }
        }
    }
    list
}

/// 加载已启用技能的内容
///
/// 扫描以下目录下的 `SKILL.md` 文件：
/// - `~/.deepseek/skills/`
/// - `~/.nyamu/skills/`
/// - `~/.claude/skills/`
///
/// 仅加载在 `SkillStore` 中标记为已启用的技能，
/// 每个技能的内容以 `---\nSkill: {名称}\n{内容}` 格式组装。
fn load_skills_content() -> String {
    let mut parts = Vec::new();
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    let store = crate::skills::SkillStore::new().load();
    for subdir in &[".deepseek/skills", ".nyamu/skills", ".claude/skills"] {
        let dir = home.join(subdir);
        if !dir.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy();
                            // 仅注入已启用的技能
                            if !store.is_enabled(&name_str) {
                                continue;
                            }
                            if let Ok(content) = std::fs::read_to_string(&skill_md) {
                                parts.push(format!("---\nSkill: {}\n{}", name_str, content));
                            }
                        }
                    }
                }
            }
        }
    }
    parts.join("\n")
}

/// 加载模式特定的提示词
///
/// 从 `modes/` 目录下加载指定模式的 Markdown 文件。
/// 模式文件用于在不同的运行模式下（如 agent、plan、yolo 等）
/// 提供额外的行为指导。如果文件不存在，返回空字符串。
///
/// # 参数
/// * `mode` — 模式名称（如 "agent"、"plan"、"yolo"）
fn load_mode_prompt(mode: &str) -> String {
    let path = prompts_dir().join("modes").join(format!("{}.md", mode));
    std::fs::read_to_string(&path).unwrap_or_default()
}

/// 构建完整的系统提示词（向后兼容版本）
///
/// 调用 `build_system_prompt_for_mode` 但不指定 mode 参数，
/// 与旧版 API 保持兼容。
///
/// # 参数
/// * `personality` — 可选的初始角色设定名称
///
/// # 返回值
/// 组装完成的系统提示词字符串
#[allow(dead_code)]
pub fn build_system_prompt(personality: Option<&str>) -> String {
    build_system_prompt_for_mode(personality, None)
}

/// 构建指定模式的完整系统提示词
///
/// 系统提示词由以下组件按顺序拼接而成，组件之间以两个换行符分隔：
///
/// 1. **宪法（Constitution）** — 基础行为准则（必须）
/// 2. **模式提示（Mode Prompt）** — 特定运行模式的额外指导（可选）
/// 3. **角色设定（Personality）** — 个性化角色设定（可选）
/// 4. **技能定义（Skills）** — 已启用技能的功能定义（可选）
///
/// # 参数
/// * `personality` — 可选的初始角色设定名称，如果提供则自动调用 `set_personality`
/// * `mode` — 可选的运行模式名称，用于加载模式特定提示
///
/// # 返回值
/// 组装完成的完整系统提示词字符串
pub fn build_system_prompt_for_mode(personality: Option<&str>, mode: Option<&str>) -> String {
    let mut parts = Vec::new();
    // 1. 加载宪法（基础系统提示）
    parts.push(load_constitution());
    // 2. 加载模式特定提示（如 agent/plan/yolo）
    if let Some(m) = mode {
        let mode_text = load_mode_prompt(m);
        if !mode_text.is_empty() {
            parts.push(mode_text);
        }
    }
    // 3. 设置并加载角色设定
    if let Some(p_name) = personality {
        let _ = set_personality(p_name);
    }
    // 默认：不加载角色设定。使用 `set_personality("calm")` 启用。
    let p_text = get_personality();
    if !p_text.is_empty() {
        parts.push(p_text);
    }
    // 4. 加载已启用技能的内容
    let skills = load_skills_content();
    if !skills.is_empty() {
        parts.push(skills);
    }
    parts.join("\n\n")
}
