//! `&` 命令集 — 参考 CCPlugins 设计的助手命令
//!
//! 所有命令均通过 `SendMessage` 让模型执行，不写 Rust 逻辑。
//! 每个命令是一个函数，返回格式化后的 prompt。

use super::CommandResult;
use crate::tui::app::{App, AppAction};

// ── &format ──────────────────────────────────────────────────

pub fn format(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[格式化] 正在检测格式化工具并格式化代码...",
        AppAction::SendMessage(String::from(
            "检测项目使用的格式化工具并格式化代码。\n\n\
             1. 检查项目配置文件（.rustfmt.toml、.prettierrc、.editorconfig 等）\n\
             2. 运行检测到的格式化工具（rustfmt、prettier、black 等）\n\
             3. 只格式化当前修改的文件，避免无关变更\n\
             4. 如果没有配置格式化工具，建议适合项目类型的格式化工\n\
             5. 格式化完成后显示变更摘要\n\
             \n\
             不要安装新的格式化工具，只用项目已有的。",
        )),
    )
}

// ── &scaffold ────────────────────────────────────────────────

pub fn scaffold(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[脚手架] 正在分析项目模式并生成功能结构...",
        AppAction::SendMessage(format!(
            "生成完整的功能结构，基于项目现有模式。\n\n\
             参数：{}\n\n\
             1. 分析项目已有代码模式（入口、模块组织、命名规范）\n\
             2. 按项目结构规划新功能的文件组织\n\
             3. 生成所有必要文件（源码、测试、配置）\n\
             4. 确保新代码符合项目的代码规范\n\
             \n\
             用 `exec_shell` 执行构建和测试验证新功能可用。\n\
             先分析项目结构再动手，不要预设框架。",
            arg.unwrap_or("（未指定）")
        )),
    )
}

// ── &test ────────────────────────────────────────────────────

pub fn test(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[测试] 正在运行测试并分析结果...",
        AppAction::SendMessage(String::from(
            "运行测试并智能分析失败原因。\n\n\
             1. 检测项目的测试框架和命令\n\
             2. 运行测试\n\
             3. 如果有失败：\n\
                - 分析失败原因（断言失败、编译错误、超时等）\n\
                - 查看相关代码和测试代码\n\
                - 提出修复方案\n\
             4. 输出测试结果摘要\n\
             \n\
             用 `exec_shell` 执行测试命令。",
        )),
    )
}

// ── &implement ───────────────────────────────────────────────

pub fn implement(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[导入] 正在分析来源并适配代码...",
        AppAction::SendMessage(format!(
            "从指定来源导入代码并适配到当前项目。\n\n\
             来源：{}\n\n\
             1. 如果是 URL：用 `fetch_url` 或 `web_search` 获取代码\n\
             2. 如果是本地路径：用 `read_file` 读取\n\
             3. 理解代码的功能和逻辑\n\
             4. 将代码适配到当前项目的架构和风格\n\
             5. 集成必要的依赖\n\
             6. 创建对应测试\n\
             7. 验证适配后的代码能正常工作\n\
             \n\
             不要直接复制，要根据项目模式调整。",
            arg.unwrap_or("（未指定）")
        )),
    )
}

// ── &refactor ────────────────────────────────────────────────

pub fn refactor(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[重构] 正在分析代码结构并制定重构计划...",
        AppAction::SendMessage(format!(
            "智能重构代码，保持功能不变的前提下改善结构和可维护性。\n\n\
             范围：{}\n\n\
             1. 分析指定代码的问题（重复、复杂度、耦合等）\n\
             2. 制定重构计划\n\
             3. 逐步执行重构，每次变更后验证\n\
             4. 确保测试仍能通过\n\
             5. 去重和简化\n\
             \n\
             **关键规则：** 每次修改后都验证，不改功能。",
            arg.unwrap_or("（全部代码）")
        )),
    )
}

// ── &security-scan ───────────────────────────────────────────

pub fn security_scan(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[安全扫描] 正在分析安全漏洞...",
        AppAction::SendMessage(format!(
            "执行安全漏洞扫描。\n\n\
             范围：{}\n\n\
             检查以下方面：\n\
             1. 注入漏洞（SQL、命令、XSS 等）\n\
             2. 敏感信息泄露（密钥、密码、token）\n\
             3. 不安全的依赖版本\n\
             4. 权限和认证问题\n\
             5. 输入验证不足\n\
             6. 不安全的文件操作\n\
             7. 常见安全反模式\n\
             \n\
             对每个发现的问题：\n\
             - 描述风险等级（高/中/低）\n\
             - 指出具体代码位置\n\
             - 给出修复建议\n\
             \n\
             用 `grep_files` 和 `read_file` 分析代码。",
            arg.unwrap_or("（全部代码）")
        )),
    )
}

// ── &predict-issues ──────────────────────────────────────────

pub fn predict_issues(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[预测] 正在分析潜在问题...",
        AppAction::SendMessage(String::from(
            "提前预测项目可能遇到的问题并评估修复时间。\n\n\
             1. 分析代码库中常见的故障模式\n\
             2. 检测潜在的：\n\
                - 性能瓶颈\n\
                - 并发问题\n\
                - 边界条件遗漏\n\
                - 错误处理缺失\n\
                - 资源泄漏\n\
                - 可扩展性问题\n\
             3. 对每个潜在问题：\n\
                - 描述出现的条件\n\
                - 评估影响范围\n\
                - 预估修复时间\n\
                - 建议预防措施\n\
             \n\
             专注于非显而易见的深层次问题。",
        )),
    )
}

// ── &remove-comments ─────────────────────────────────────────

pub fn remove_comments(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[清理注释] 正在分析注释质量...",
        AppAction::SendMessage(format!(
            "清理无用注释，保留有价值的文档。\n\n\
             范围：{}\n\n\
             规则：\n\
             1. 删除以下注释：\n\
                - 自动生成的样板注释（// TODO: fix me 等无意义内容）\n\
                - 注释掉的死代码\n\
                - 显而易见的注释（// 加 1）\n\
                - 过时的注释（与代码不符）\n\
             2. 保留以下注释：\n\
                - 解释复杂逻辑的注释\n\
                - 公共 API 文档\n\
                - 安全相关说明\n\
                - 标记为 FIXME、HACK、XXX 的关键注释\n\
             3. 提升有问题的注释：\n\
                - 把「干什么」的注释改为「为什么这么干」\n\
             \n\
             用 `edit_file` 或 `apply_patch` 进行修改。",
            arg.unwrap_or("（全部文件）")
        )),
    )
}

// ── &fix-imports ─────────────────────────────────────────────

pub fn fix_imports(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[修复导入] 正在扫描断裂的导入...",
        AppAction::SendMessage(String::from(
            "修复重构后断裂的 import/use/require 语句。\n\n\
             1. 扫描所有源文件中的导入语句\n\
             2. 检测以下问题：\n\
                - 引用已移动/重命名的模块或文件\n\
                - 循环依赖\n\
                - 未使用的导入\n\
                - 缺少的导入\n\
             3. 对每个断裂的导入：\n\
                - 查找目标文件的新位置\n\
                - 更新导入路径\n\
                - 验证修复后的代码能通过编译（用 `exec_shell`）\n\
             \n\
             每次修改后验证编译/语法正确性。",
        )),
    )
}

// ── &find-todos ──────────────────────────────────────────────

pub fn find_todos(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[查找 TODO] 正在扫描代码...",
        AppAction::SendMessage(String::from(
            "查找并整理代码中所有 TODO/FIXME/HACK/XXX 标记。\n\n\
             1. 用 `grep_files` 搜索 TODO、FIXME、HACK、XXX、NOTE\n\
             2. 按类型分类：\n\
                - TODO：待实现功能\n\
                - FIXME：已知问题需修复\n\
                - HACK：临时方案需改进\n\
                - XXX：危险/需要注意\n\
                - NOTE：重要说明\n\
             3. 输出清单：\n\
                - 文件路径和行号\n\
                - TODO 原文\n\
                - 分类和紧急程度\n\
             4. 统计总数并按目录分组",
        )),
    )
}

// ── &create-todos ────────────────────────────────────────────

pub fn create_todos(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[添加 TODO] 正在分析代码上下文...",
        AppAction::SendMessage(String::from(
            "根据代码分析自动添加带上下文的 TODO 注释。\n\n\
             1. 分析代码中明显缺失的部分：\n\
                - 空函数/方法体\n\
                - 未处理的错误情形\n\
                - 缺失的边界条件检查\n\
                - 硬编码值需提取为配置\n\
                - 缺失的日志或监控\n\
                - 未实现的接口方法\n\
             2. 为每个缺失添加 TODO：\n\
                - 描述需要做什么\n\
                - 说明为什么需要（上下文）\n\
                - 参考相关代码\n\
             3. 不要添加无关或臆想的 TODO\n\
             \n\
             用 `edit_file` 添加注释。",
        )),
    )
}

// ── &fix-todos ───────────────────────────────────────────────

pub fn fix_todos(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[实现 TODO] 正在分析 TODO 上下文...",
        AppAction::SendMessage(format!(
            "智能实现代码中的 TODO 修复。\n\n\
             范围：{}\n\n\
             1. 用 `grep_files` 找到相关 TODO\n\
             2. 对每个 TODO：\n\
                - 阅读 TODO 上下文了解需求\n\
                - 设计实现方案\n\
                - 实现功能\n\
                - 验证代码能通过编译/测试\n\
                - 移除已实现的 TODO 标记\n\
             3. 先实现简单的 TODO，复杂的单独讨论\n\
             \n\
             每次修改后立即验证。",
            arg.unwrap_or("（自动检测范围内 TODO）")
        )),
    )
}

// ── &understand ──────────────────────────────────────────────

pub fn understand(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[架构分析] 正在分析项目架构...",
        AppAction::SendMessage(String::from(
            "分析整个项目的架构和设计模式。\n\n\
             Phase 1：项目发现\n\
             - 读取 README、配置文件、文档\n\
             - 检测技术栈\n\
             - 了解项目结构\n\n\
             Phase 2：架构分析\n\
             - 入口点：主文件、路由、初始化\n\
             - 核心模块：业务逻辑组织\n\
             - 数据层：数据库、模型、存储\n\
             - API 层：路由、控制器、接口\n\
             - 前端：组件、视图、模板\n\
             - 配置：环境设置、常量和配置\n\
             - 测试：测试结构和策略\n\n\
             Phase 3：模式识别\n\
             - 命名规范\n\
             - 模块依赖关系\n\
             - 数据流模式\n\
             - 错误处理策略\n\n\
             输出结构化架构文档，不要空泛描述。",
        )),
    )
}

// ── &explain (explain-like-senior) ────────────────────────────

pub fn explain(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[讲解] 正在分析代码...",
        AppAction::SendMessage(format!(
            "以资深工程师的视角讲解代码。\n\n\
             目标代码：{}\n\n\
             讲解内容包括：\n\
             1. 这段代码在做什么（业务层面）\n\
             2. 为什么这样设计（设计决策和 trade-off）\n\
             3. 潜在的问题和改进空间\n\
             4. 相关的设计模式和原则\n\
             5. 与其他模块的交互方式\n\
             6. 测试策略和边界条件\n\n\
             风格：\n\
             - 不要解释语法（假设听众是资深工程师）\n\
             - 关注设计思路而非实现细节\n\
             - 指出值得注意的地方和未来优化的方向\n\
             - 如果代码有明显问题，直接指出",
            arg.unwrap_or("（选中的代码）")
        )),
    )
}

// ── &contributing ────────────────────────────────────────────

pub fn contributing(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[贡献分析] 正在分析项目就绪度...",
        AppAction::SendMessage(String::from(
            "分析项目的贡献就绪度。\n\n\
             检查以下方面：\n\n\
             1. 文档就绪度\n\
                - 有没有 README（安装、使用、贡献指南）\n\
                - 有没有 CONTRIBUTING.md\n\
                - 有没有 API 文档/代码注释\n\
                - 有没有变更日志（CHANGELOG）\n\n\
             2. 项目设置\n\
                - 有没有明确的构建和测试流程\n\
                - 有没有 CI/CD 配置\n\
                - 有没有代码规范配置（格式化、lint）\n\n\
             3. 代码质量\n\
                - 测试覆盖率如何\n\
                - 代码风格是否一致\n\
                - 有没有明显的技术债务\n\n\
             4. 改进建议\n\
                - 列出最优先改进的 3-5 项\n\
                - 每项说明理由和实施建议",
        )),
    )
}

// ── &make-it-pretty ──────────────────────────────────────────

pub fn make_it_pretty(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[美化] 正在分析代码可读性...",
        AppAction::SendMessage(format!(
            "不改变功能，只提升代码可读性和整洁度。\n\n\
             范围：{}\n\n\
             检查并改进：\n\
             1. 命名：变量/函数/类名是否表达意图\n\
             2. 结构：函数是否过长、是否需要拆分\n\
             3. 一致性：是否遵循项目的命名和风格\n\
             4. 简化：是否有冗余的条件判断或重复代码\n\
             5. 注释：是否解释了「为什么」而非「是什么」\n\
             6. 格式：缩进、空格、空行是否一致\n\n\
             每次修改后用 `exec_shell` 验证编译/语法正确。\n\
             不改功能逻辑。",
            arg.unwrap_or("（当前焦点代码）")
        )),
    )
}

// ── &session-start ───────────────────────────────────────────

pub fn session_start(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[开始会话] 正在初始化...",
        AppAction::SendMessage(String::from(
            "开始记录会话。\n\n\
             1. 读取当前项目的指令文件（WHALE.md、AGENTS.md、CLAUDE.md）\n\
             2. 了解当前 git 状态（分支、未提交变更）\n\
             3. 记录会话目标：\n\
                - 本次会话要完成什么\n\
                - 当前进度\n\
                - 关键约束\n\
             4. 在 `.codewhale/sessions/` 下初始化会话日志\n\
             \n\
             用 `git_status` 和 `git_diff` 了解当前状态。",
        )),
    )
}

// ── &session-end ─────────────────────────────────────────────

pub fn session_end(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[结束会话] 正在总结...",
        AppAction::SendMessage(String::from(
            "总结并保存当前会话。\n\n\
             1. 总结本次会话完成的工作\n\
             2. 记录未完成的事项和后续步骤\n\
             3. 记录关键决策和原因\n\
             4. 记录有用的命令或代码片段\n\
             5. 保存到 `.codewhale/sessions/`\n\
             \n\
             用 `write_file` 将会话日志写入 `.codewhale/sessions/`。",
        )),
    )
}

// ── &docs ─────────────────────────────────────────────────────

pub fn docs(_app: &App, arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[文档] 正在分析文档需求...",
        AppAction::SendMessage(format!(
            "智能管理项目文档。\n\n\
             参数：{}\n\n\
             根据参数执行不同操作：\n\
             - 无参数：检查文档现状，列出缺失和过时的文档\n\
             - 文件名：读取并改进指定文档\n\
             - \"update\"：更新所有过时的文档（对照代码变更）\n\
             - \"create <name>\"：创建新文档\n\
             - \"check\"：检查文档与代码的同步程度\n\n\
             文档质量标准：\n\
             - 准确描述当前代码行为\n\
             - 包含示例\n\
             - 说明为什么而非只是是什么\n\
             - 保持简洁",
            arg.unwrap_or("（检查文档现状）")
        )),
    )
}

// ── &remember ──────────────────────────────────────────────

pub fn remember(_app: &App, arg: Option<&str>) -> CommandResult {
    let text = arg.map(str::trim).filter(|s| !s.is_empty());
    let Some(text) = text else {
        return CommandResult::error("用法：&remember <要记住的规则/约定/TODO>");
    };
    CommandResult::with_message_and_action(
        "[记录] 正在追加到指令文件...",
        AppAction::SendMessage(format!(
            "将以下内容追加到项目的指令文件（WHALE.md / AGENTS.md / CLAUDE.md，按优先级选择已存在的）。\n\n             内容：{}\n\n             1. 检测哪个指令文件存在（WHALE.md > AGENTS.md > CLAUDE.md）\n             2. 读取现有内容\n             3. 将内容润色整理后放到合适章节\n             4. 用 `write_file` 写回\n\n             格式要求：\n             - 规则/约定放 ## 代码规范 或新建 ## 项目约定\n             - TODO 放 ## 待办\n             - 保持与现有内容风格一致",
            text
        )),
    )
}

// ── &todos-to-issues ─────────────────────────────────────────

pub fn todos_to_issues(_app: &App, _arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "[TODO→Issue] 正在转换...",
        AppAction::SendMessage(String::from(
            "将代码中的 TODO 转换为 GitHub Issue 格式。\n\n\
             1. 用 `grep_files` 收集所有 TODO/FIXME/HACK/XXX\n\
             2. 对每个标记：\n\
                - 提取描述和上下文\n\
                - 确定合理的标签（enhancement/bug/tech-debt）\n\
                - 评估优先级\n\
                - 生成 Issue 格式的标题和正文\n\
             3. 输出为以下格式（不创建实际 Issue，除非用户要求）：\n\
                ```\n\
                ## TODO: <标题>\n\
                - 位置：`文件:行号`\n\
                - 描述：<TODO 内容>\n\
                - 建议：<实现建议>\n\
                - 优先级：<高/中/低>\n\
                ```\n\
             4. 统计分类：\n\
                - 按模块分组\n\
                - 按优先级分组",
        )),
    )
}
