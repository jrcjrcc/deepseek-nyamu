# CodeWhale — 定制优化版

> 基于 [CodeWhale v0.8.46](https://github.com/Hmbown/CodeWhale) 的深度定制分支
> 融合 Claude Code 常用工作流，增强稳定性与中文体验

---

## 核心优化

### 🔧 12 项代码缺陷修复

| # | 问题 | 修复方式 |
|---|------|---------|
| 1 | Snapshot wipe 死循环（pack 文件未回收） | wipe 后追加 `git repack -ad` + `git gc --prune=now --aggressive` |
| 2 | 有界事件通道（256）阻塞引擎循环 | 通道容量 256 → 65536 |
| 3 | SQLite 无连接池/WAL/busy_timeout | 连接池化（`Arc<Mutex<Connection>>`），`init_schema()` 设置 PRAGMA |
| 4 | `block_in_place` + `block_on` 线程饥饿 | 改为 `tokio::task::spawn_blocking` |
| 5 | Snapshot restore 删除文件无日志 | 删除前输出 `tracing::warn!` 及逐文件日志 |
| 6 | Session JSON 无限膨胀 | 0 消息跳过写入、总文件大小硬性上限、超大 tool result 截断 |
| 7 | `cancel_task` / `finish_task` 竞态条件 | `Arc<Mutex<HashMap>>` 原子化状态转换 |
| 8 | Dream 整合无文件锁（并发 &dream 损坏记忆） | `ConsolidationLock::try_acquire()` 启动子代理前检测 |
| 9 | 全局工具写锁串行化所有调用 | 写锁统一改为读锁 |
| 10 | `std::env::set_var` 不安全 | 确认测试代码已有 `env_lock()` 保护（维持原样） |
| 11 | Git 子模块未检出时 `git add -A` 失败 | `git add -A --ignore-submodules` |
| 12 | MCP Server 子进程无健康检查 | 添加启动超时 + shutdown 等待子进程 |

### 🚀 指令系统增强

#### && 指令（记忆与实用工具）
借鉴 Claude Code 的 `&` 指令体系，将常用记忆和实用操作整合为快捷指令：

| 指令 | 功能 | 来源 |
|------|------|------|
| `&dream` | 运行 Dream 记忆整合，将会话知识持久化 | Claude Code 记忆系统 |
| `&tips` | 查看/记录使用技巧 | Claude Code 工作流 |
| `&traps` | 查看/记录踩坑记录 | Claude Code 工作流 |
| `&update` | 检查更新 | 原项目升级机制 |
| `&meng` | 中文别名：记忆整合 | Claude Code 习惯适配 |

#### ## 指令（快捷记录）
借鉴 Claude Code 的 `#` 约定，快速记录项目规则、约定和 TODO：
- 在对话中使用 `#` 前缀描述规则，自动追加到项目指令文件
- 支持 `记住`、`记录`、`记下` 等自然语言触发词

#### 三入口统一描述
`&` 命令在以下三处统一注册，不再遗漏：
- 帮助面板（F1/`/help`）
- 命令面板（Ctrl+P）
- 速选栏（`/` 斜杠菜单）

### 🌏 完整汉化

#### 280+ 本地化 MessageId 全覆盖
所有 `localization.rs` 中定义的 MessageId 均完成中文翻译，通过 `missing_message_ids(Locale::ZhCN)` 验证（返回空集）。

#### 引擎状态栏汉化（P0）
日常最频繁看到的引擎状态信息全部中文化：

| 原文 | 译文 |
|------|------|
| Auto-compacting context... | 正在自动压缩上下文... |
| Reached maximum steps | 已达到最大步骤数 |
| Request cancelled | 请求已取消 |
| Steer input accepted: {} | 转向输入已接受：{} |
| Restoring previous order... | 正在恢复上次对话顺序... |
| Executing tools sequentially... | 正在串行执行工具... |
| Approved tool call: {id} | 已批准工具调用：{id} |
| Denied tool call: {id} | 已拒绝工具调用：{id} |
| Mode changed to: {mode:?} | 模式已切换为：{mode:?} |
| Capacity refresh compaction failed... | 容量刷新压缩失败... |
| Capacity guardrail... | 容量护栏... |

#### 推理/思考标签汉化
| 原文 | 译文 |
|------|------|
| live | 思考中 |
| done | 已完成 |
| idle | 空闲 |
| More reasoning in Ctrl+O | 更多思考内容，按 Ctrl+O 查看 |
| Full reasoning in Ctrl+O | 完整思考内容，按 Ctrl+O 查看 |

#### 侧栏汉化
| 原文 | 译文 |
|------|------|
| Agents | 代理 |
| No agents | 无代理 |
| Recent tools | 最近工具 |
| Session | Session（会话） |
| Sidebar focus: work/tasks/agents/context/auto | 侧栏焦点：工作/任务/代理/上下文/自动 |
| Sidebar hidden | 侧栏已隐藏 |

#### 工具面板标签汉化
所有展开/折叠工具行中的键值标签：

名称 → 结果 → 查询词 → 来源 → 耗时 → 文件 → 目标 → 标题 → 路径 → 参数 → 命令 → 输出 → 提示词 → 文本 → 模式 → 模型

### 🎨 主题颜色撞色修复

`deepseek_theme.rs` — `Theme::dark()` / `Theme::light()`

| 字段 | 原颜色 | 新颜色 | 效果 |
|------|--------|--------|------|
| `tool_success_accent` | `TEXT_DIM` #8A96AE 灰 | `STATUS_SUCCESS` #4FD1C5 青绿 | 成功状态可辨识 |
| `plan_progress_color` | `STATUS_SUCCESS` 青绿 | `STATUS_INFO` #6AAEF2 天蓝 | 进行中≠已完成 |
| `plan_pending_color` | `TEXT_MUTED` | `TEXT_HINT` | 待处理与正文区分 |
| `plan_summary_color` | `TEXT_MUTED` | `TEXT_DIM` | 摘要与值文本区分 |
| `plan_explanation_color` | `TEXT_DIM` | `TEXT_MUTED` | 说明与标签颜色对调 |

### 🧹 编译修复

- `update.rs`：补充缺失的 `binary_prefix_for_exe` 函数（从 `crates/cli` 同步）
- `init_wizard.rs`：修正 Rust `format!` 中误用的 `%s` 占位符为 `{}`

---

## 设计思路

- **最小侵入修复**：不重构架构，只修正具体问题。事件通道未改为无界通道（需改动 80+ 调用点），而是大幅提高缓冲区至 65536
- **直替换免膨胀**：引擎状态消息直接替换源码中的英文字符串，避免新增 60 个 MessageId 变体
- **调色板不动**：不改 `palette.rs` 常量值，只改 `deepseek_theme.rs` 的语义映射，确保各主题一致性
- **编译安全**：每次修改后通过 `cargo check` 验证，最终 `cargo build --release` 零 error 通过

---

## 开源许可

基于 MIT License，保留原项目版权声明即可自由使用和分发。

原项目：[Hmbown/CodeWhale](https://github.com/Hmbown/CodeWhale)
