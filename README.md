# DeepWhale (nyamuWhale)

基于 **Tauri v2** 的 AI 编程助手桌面应用，React 前端 + Rust 后端，集成 nyamu 引擎核心能力。

## 功能

- **对话式编程助手** — LLM 驱动的代码生成、解释、重构
- **双模式运行** — GUI 桌面应用 + CLI 终端模式
- **沙箱执行** — 安全的 Shell 命令执行环境
- **LSP 诊断集成** — 实时代码错误检查
- **自动模型路由** — 根据任务复杂度自动选择模型
- **文件系统快照** — 支持操作撤销
- **工具调度** — 内置代码搜索、文件编辑、Shell 执行等工具
- **多会话管理** — 独立对话上下文，支持导出/导入
- **子代理** — 并行执行独立任务
- **SWE-bench 评估** — 支持软件工程基准测试导出
- **CLI 丰富子命令** — sessions, config, models, sandbox, serve, mcp-server 等

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 18 + TypeScript + Vite 6 |
| 桌面框架 | Tauri v2 |
| 后端 | Rust (edition 2024) |
| nyamu 引擎 | 12 个核心 crate（agent, core, config, provider, tools, state, mcp 等） |
| 包管理 | npm / Cargo workspace |

## 快速开始

```bash
# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## 项目结构

```
src/                  # React 前端源码
  components/         #  UI 组件（Composer, DiffView, Terminal 等）
  lib/                #  工具库（bridge, commands, theme, types）
  App.tsx             #  主应用组件
  styles.css          #  全局样式

src-tauri/            # Tauri + Rust 后端
  src/                #  Rust 源码
    commands.rs       #  Tauri IPC 命令
    engine.rs         #  核心 Agent 引擎
    cli.rs            #  CLI 子命令
    sandbox.rs        #  沙箱执行
    lsp/              #  LSP 客户端
    tools/            #  工具注册与分发
    snapshot/         #  文件系统快照
    rlm/              #  递归语言模型
  tauri.conf.json     #  Tauri 配置
  Cargo.toml          #  Rust 依赖

vendor/nyamu/         # nyamu 引擎依赖（vendored crate）
npm/                  # npm CLI 工具
.github/workflows/    # CI 配置
```

## 配置

环境变量：

- `RUST_LOG` — 日志级别（默认 `info`）

## 许可证

MIT
