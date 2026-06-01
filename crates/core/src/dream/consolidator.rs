//! Dream consolidation sub‑agent prompt generation.
//!
//! The prompt instructs a child sub‑agent (spawned by the TUI layer)
//! to perform the four‑phase memory integration:
//!
//! 1. **Orient**       – list the memory directory, read the index.
//! 2. **Gather**       – read logs & existing memories for new signal.
//! 3. **Consolidate**  – merge new information into topic files.
//! 4. **Prune**        – update the index, trim oversized files.

use std::path::PathBuf;

use chrono::Utc;

use super::config::DreamConfig;

/// Tools the Dream sub‑agent is permitted to call.
pub const ALLOWED_TOOLS: &[&str] = &[
    "read_file",
    "write_file",
    "edit_file",
    "grep_files",
    "list_dir",
];

/// Resolve the two possible log directories from the user's home dir.
fn log_directories() -> (PathBuf, PathBuf) {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    let codewhale_logs = home.join(".codewhale").join("logs");
    let deepseek_logs = home.join(".deepseek").join("logs");
    (codewhale_logs, deepseek_logs)
}

/// Build the system prompt for the Dream consolidation sub‑agent.
pub fn build_prompt(config: &DreamConfig) -> String {
    let memory_dir = config.resolved_memory_dir().display().to_string();
    let max_size = config.max_memory_file_size;
    let tool_list = ALLOWED_TOOLS.join(", ");
    let today = Utc::now().format("%Y-%m-%d");
    let (codewhale_logs, deepseek_logs) = log_directories();
    let codewhale_logs = codewhale_logs.display().to_string();
    let deepseek_logs = deepseek_logs.display().to_string();

    format!(
        r#"## Dream — 记忆整合助理

你是 CodeWhale 的记忆整合助理。你的任务是对记忆目录中的文件进行自动化整合。

### 约束

- **记忆目录**: `{memory_dir}`
- **可用工具**: {tool_list}
- **受限**: 只有 `{tool_list}` 可用，不得调用其他工具
- **单文件大小上限**: {max_size} 字节
- **总记忆上限**: 所有主题文件合计不超过 521KB。超出时从最旧的文件开始裁剪
- **工作方式**: 只读/写记忆目录内的文件，不得触碰项目代码

### 文件命名规则

- **traps.md** — 所有项目的通用踩坑记录
- **tips.md** — 所有项目的通用技巧配置
- **`<project-name>.md`** — 按项目命名的独立文件，如 `codewhale.md`、`claude-code.md`。从日志中识别项目名称后创建/更新对应文件
- **MEMORY.md** — 索引文件（自动维护，不要手动编辑）

### 四阶段流程

#### 阶段 1: Orient（定位）
1. 用 `list_dir` 列出 `{memory_dir}`、`{codewhale_logs}` 和 `{deepseek_logs}` 目录的内容
2. 用 `read_file` 读取 `{memory_dir}/MEMORY.md`（如果存在），了解现有的记忆索引结构
3. 用 `read_file` 读取每个已有的主题记忆文件，了解已有知识

#### 阶段 2: Gather（搜集）
1. 读取自上次整合后新增的日志文件——**需要检查两个位置**：
   - `{codewhale_logs}/`（首选，`~/.codewhale/logs`）
   - `{deepseek_logs}/`（次选，`~/.deepseek/logs`，实际运行时日志可能写在这里）
   日志文件名格式为 `tui-YYYY-MM-DD-*.log`（如 `tui-2026-06-01-12345.log`）
2. 从日志中提取新的知识点、决策记录、配置技巧、错误修复方案
3. 用 `grep_files` 在已有记忆文件中搜索重复或相关主题
4. **识别项目归属**：每条知识点属于哪个项目（如 CodeWhale、Claude Code、Lean 等），记录到对应项目文件中

#### 阶段 3: Consolidate（整合）
1. 所有文件（新建和已有的）必须包含 YAML frontmatter：
   ```
   ---
   description: 该主题的简短且具体的描述（用于检索匹配，务必准确描述含哪些内容）
   ---
   ```
2. **按项目分类**：
   - 知识点属于哪个项目 → 写入对应的 `<project-name>.md`
   - 通用陷阱/技巧 → 写入 `traps.md`/`tips.md`
3. **description 同步更新**：
   - 创建新文件 → 写 frontmatter + 内容
   - 更新已有文件：
     - 如果缺少 frontmatter → 补上
     - **新增内容后，立即更新 `description` 字段**，把新增的知识点关键词加进去，确保 RAG 能匹配到新内容
     - 例如 traps.md 新增了 "snapshot" 相关内容 → description 加入 snapshot 关键词
4. 新信息插入到文件开头，旧内容往后移（时间倒序，最新的在最前面）
5. 文件结构示例（时间倒序）：
   ```
   ---
   description: 已知陷阱和踩坑记录，含 Snapshot、工具调用、路径访问等常见问题
   ---
   
   ## 已知陷阱
   
   2026-06-01: 最新发现的问题
   ...
   2026-05-28: 较早的发现
   ...
   2026-05-20: 最早的记录
   ```
6. 处理日期：将相对日期（如"昨天"、"上周三"）转为绝对日期（如 "{today}"）
7. 删除 {today} 两个月以前的信息（即 {today} 之前超过 60 天的旧记录）
8. 每个文件不得超过 {max_size} 字节。如果超出，从文件尾部（旧内容）开始裁剪

#### 阶段 4: Prune（裁剪）
1. 确保每个主题文件不超过大小限制
2. 更新 `{memory_dir}/MEMORY.md` 索引文件：
   ```
   # 记忆索引
   
   ## 主题文件
   - traps.md: 已知陷阱和踩坑记录，含 Snapshot、工具调用等
   - tips.md: 常用技巧和配置
   - codewhale.md: CodeWhale 项目架构、配置、工作区信息
   - claude-code.md: Claude Code 项目相关知识
   
   上次更新: YYYY-MM-DD
   文件总数: N
   ```

### 完成标准
- 所有新信息已合并到对应的项目文件或通用文件中
- 每个文件的 frontmatter description 反映了最新内容
- MEMORY.md 索引已更新
- 已清理过时信息
- 所有文件在大小限制内

完成时输出一条摘要，说明本次整合了哪些信息、更新了哪些文件。"#
    )
}
