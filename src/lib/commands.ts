/**
 * commands.ts —— 集中命令注册表
 *
 * 移植自 CodeWhale 的 60+ "/" 命令系统。
 * 每个命令定义其 id、别名、分类、描述、处理方式。
 * 供 SlashMenu、CommandPalette、Composer 共同消费。
 */

export type CommandCategory =
  | "core" | "session" | "config"
  | "debug" | "project" | "skills";

/** 命令处理方式 */
export type CommandHandler = "frontend" | "bridge" | "engine";

export interface CommandDef {
  id: string;
  label: string;
  aliases: string[];
  category: CommandCategory;
  description: string;
  usage?: string;
  handler: CommandHandler;
  /** 无参数时直接执行，不插入输入框 */
  noArgAction?: boolean;
}

const COMMANDS: CommandDef[] = [
  /* ── Core ─────────────────────────────────────────────── */
  {
    id: "help", label: "/help", aliases: ["?", "bangzhu", "帮助"],
    category: "core", description: "显示帮助信息",
    usage: "/help [command]", handler: "frontend", noArgAction: true,
  },
  {
    id: "clear", label: "/clear", aliases: ["qingping", "清除"],
    category: "core", description: "清空当前对话",
    usage: "/clear", handler: "frontend", noArgAction: true,
  },
  {
    id: "exit", label: "/exit", aliases: ["quit", "q", "tuichu", "退出"],
    category: "core", description: "退出应用程序",
    usage: "/exit", handler: "frontend", noArgAction: true,
  },
  {
    id: "model", label: "/model", aliases: ["moxing", "模型"],
    category: "core", description: "切换或查看当前模型",
    usage: "/model [name]", handler: "bridge",
  },
  {
    id: "effort", label: "/effort", aliases: ["推理"],
    category: "core", description: "设置推理力度 (off/low/medium/high/max)",
    usage: "/effort <off|low|medium|high|max>", handler: "bridge",
  },
  {
    id: "provider", label: "/provider", aliases: ["提供商"],
    category: "core", description: "切换活跃提供商和/或模型",
    usage: "/provider [name] [model]", handler: "engine",
  },
  {
    id: "agent", label: "/agent", aliases: ["daili", "代理"],
    category: "core", description: "打开持久化子代理会话",
    usage: "/agent [0-3] <task>", handler: "engine",
  },
  {
    id: "subagents", label: "/subagents", aliases: ["agents", "zhinengti", "智能体"],
    category: "core", description: "列出子代理状态",
    usage: "/subagents", handler: "frontend", noArgAction: true,
  },
  {
    id: "memory", label: "/memory", aliases: ["记忆"],
    category: "core", description: "查看或管理持久化用户记忆文件",
    usage: "/memory [show|path|clear|edit|help]", handler: "bridge", noArgAction: true,
  },
  {
    id: "note", label: "/note", aliases: ["笔记"],
    category: "core", description: "管理工作区笔记",
    usage: "/note [add|list|show|edit|remove|clear|path]", handler: "engine",
  },
  {
    id: "attach", label: "/attach", aliases: ["image", "media", "fujian", "附件"],
    category: "core", description: "附加图片或媒体文件",
    usage: "/attach <path>", handler: "engine",
  },
  {
    id: "task", label: "/task", aliases: ["tasks", "任务"],
    category: "core", description: "管理后台任务",
    usage: "/task [add <prompt>|list|show <id>|cancel <id>]", handler: "engine",
  },
  {
    id: "hooks", label: "/hooks", aliases: ["hook", "gouzi", "钩子"],
    category: "core", description: "列出已配置的生命周期钩子",
    usage: "/hooks [list|events]", handler: "frontend", noArgAction: true,
  },
  {
    id: "mcp", label: "/mcp", aliases: [],
    category: "core", description: "管理 MCP 服务器",
    usage: "/mcp [init|add|enable|disable|remove|validate|reload]", handler: "engine",
  },
  {
    id: "network", label: "/network", aliases: ["网络"],
    category: "core", description: "管理网络允许/禁止规则",
    usage: "/network [list|allow <host>|deny <host>|remove <host>]", handler: "engine",
  },
  {
    id: "feedback", label: "/feedback", aliases: ["反馈"],
    category: "core", description: "生成 GitHub 反馈 URL",
    usage: "/feedback [bug|feature|security]", handler: "frontend", noArgAction: true,
  },
  {
    id: "links", label: "/links", aliases: ["dashboard", "api", "lianjie", "链接"],
    category: "core", description: "显示 DeepSeek 仪表盘和文档链接",
    usage: "/links", handler: "frontend", noArgAction: true,
  },
  {
    id: "home", label: "/home", aliases: ["stats", "overview", "zhuye", "首页"],
    category: "core", description: "显示首页仪表盘",
    usage: "/home", handler: "frontend", noArgAction: true,
  },
  {
    id: "workspace", label: "/workspace", aliases: ["cwd", "工作区"],
    category: "core", description: "显示或切换当前工作区路径",
    usage: "/workspace [path]", handler: "frontend", noArgAction: true,
  },

  /* ── Session ──────────────────────────────────────────── */
  {
    id: "rename", label: "/rename", aliases: ["gaiming", "chongmingming", "重命名"],
    category: "session", description: "重命名当前会话",
    usage: "/rename <new title>", handler: "bridge",
  },
  {
    id: "save", label: "/save", aliases: ["保存"],
    category: "session", description: "将会话保存到文件",
    usage: "/save [path]", handler: "bridge",
  },
  {
    id: "fork", label: "/fork", aliases: ["branch", "分支"],
    category: "session", description: "将当前对话分支到新会话",
    usage: "/fork", handler: "frontend", noArgAction: true,
  },
  {
    id: "new", label: "/new", aliases: ["新会话"],
    category: "session", description: "开始新的会话",
    usage: "/new [--force]", handler: "frontend", noArgAction: true,
  },
  {
    id: "sessions", label: "/sessions", aliases: ["resume", "会话"],
    category: "session", description: "打开会话历史选择器",
    usage: "/sessions [show|prune <days>]", handler: "frontend", noArgAction: true,
  },
  {
    id: "load", label: "/load", aliases: ["jiazai", "加载"],
    category: "session", description: "从文件加载会话",
    usage: "/load [path]", handler: "bridge",
  },
  {
    id: "compact", label: "/compact", aliases: ["yasuo", "压缩"],
    category: "session", description: "触发上下文压缩释放空间",
    usage: "/compact", handler: "bridge", noArgAction: true,
  },
  {
    id: "purge", label: "/purge", aliases: ["qingchu", "清除"],
    category: "session", description: "让 AI 手术刀式清理对话历史",
    usage: "/purge", handler: "engine", noArgAction: true,
  },
  {
    id: "export", label: "/export", aliases: ["daochu", "导出"],
    category: "session", description: "将会话导出为 Markdown",
    usage: "/export [path]", handler: "bridge",
  },
  {
    id: "relay", label: "/relay", aliases: ["batonpass", "接力"],
    category: "session", description: "创建接力会话获得新线程",
    usage: "/relay [focus]", handler: "engine",
  },
  {
    id: "context", label: "/context", aliases: ["ctx", "上下文"],
    category: "session", description: "打开会话上下文检查器",
    usage: "/context", handler: "frontend", noArgAction: true,
  },

  /* ── Config ───────────────────────────────────────────── */
  {
    id: "config", label: "/config", aliases: ["配置"],
    category: "config", description: "查看配置",
    usage: "/config", handler: "bridge", noArgAction: true,
  },
  {
    id: "mode", label: "/mode", aliases: ["jihua", "zidong", "模式"],
    category: "config", description: "切换模式 (agent/plan/yolo)",
    usage: "/mode [agent|plan|yolo|1|2|3]", handler: "frontend",
  },
  {
    id: "theme", label: "/theme", aliases: ["主题"],
    category: "config", description: "切换主题或打开主题选择器",
    usage: "/theme [name]", handler: "frontend", noArgAction: true,
  },
  {
    id: "verbose", label: "/verbose", aliases: ["详细"],
    category: "config", description: "切换实时思考过程显示",
    usage: "/verbose [on|off]", handler: "frontend",
  },
  {
    id: "trust", label: "/trust", aliases: ["xinren", "信任"],
    category: "config", description: "管理工作区信任列表",
    usage: "/trust [on|off|add <path>|remove <path>|list]", handler: "engine",
  },
  {
    id: "logout", label: "/logout", aliases: ["退出登录"],
    category: "config", description: "清除 API Key 并回到设置",
    usage: "/logout", handler: "frontend", noArgAction: true,
  },
  {
    id: "settings", label: "/settings", aliases: ["设置"],
    category: "config", description: "打开设置面板",
    usage: "/settings", handler: "frontend", noArgAction: true,
  },
  {
    id: "status", label: "/status", aliases: ["状态"],
    category: "config", description: "显示运行时会话状态",
    usage: "/status", handler: "frontend", noArgAction: true,
  },
  {
    id: "statusline", label: "/statusline", aliases: ["状态栏"],
    category: "config", description: "配置状态栏显示项",
    usage: "/statusline", handler: "frontend",
  },
  {
    id: "profile", label: "/profile", aliases: ["dangan", "档案"],
    category: "config", description: "切换配置档案",
    usage: "/profile <name>", handler: "engine",
  },

  /* ── Debug ────────────────────────────────────────────── */
  {
    id: "tokens", label: "/tokens", aliases: ["token"],
    category: "debug", description: "显示会话 Token 用量",
    usage: "/tokens", handler: "bridge", noArgAction: true,
  },
  {
    id: "system", label: "/system", aliases: ["xitong", "系统"],
    category: "debug", description: "显示当前系统提示词",
    usage: "/system", handler: "bridge", noArgAction: true,
  },
  {
    id: "edit", label: "/edit", aliases: ["编辑"],
    category: "debug", description: "修改并重新发送上一条消息",
    usage: "/edit", handler: "frontend", noArgAction: true,
  },
  {
    id: "diff", label: "/diff", aliases: ["差异"],
    category: "debug", description: "显示会话开始以来的文件变更",
    usage: "/diff", handler: "bridge", noArgAction: true,
  },
  {
    id: "change", label: "/change", aliases: ["changelog"],
    category: "debug", description: "显示最新更新日志",
    usage: "/change [version]", handler: "bridge", noArgAction: true,
  },
  {
    id: "undo", label: "/undo", aliases: ["撤销"],
    category: "debug", description: "移除最后一条消息对",
    usage: "/undo", handler: "frontend", noArgAction: true,
  },
  {
    id: "retry", label: "/retry", aliases: ["chongshi", "重试"],
    category: "debug", description: "重试最后一次请求",
    usage: "/retry", handler: "frontend", noArgAction: true,
  },
  {
    id: "cost", label: "/cost", aliases: ["费用"],
    category: "debug", description: "显示会话费用详情",
    usage: "/cost", handler: "bridge", noArgAction: true,
  },
  {
    id: "balance", label: "/balance", aliases: ["余额"],
    category: "debug", description: "检查活跃提供商账户余额",
    usage: "/balance", handler: "bridge", noArgAction: true,
  },
  {
    id: "cache", label: "/cache", aliases: ["缓存"],
    category: "debug", description: "显示缓存命中率统计",
    usage: "/cache [count|inspect|stats|zones|warmup]", handler: "bridge", noArgAction: true,
  },
  {
    id: "translate", label: "/translate", aliases: ["translation", "翻译"],
    category: "debug", description: "切换输出翻译",
    usage: "/translate", handler: "frontend", noArgAction: true,
  },

  /* ── Project ──────────────────────────────────────────── */
  {
    id: "review", label: "/review", aliases: ["shencha", "审查"],
    category: "project", description: "对文件、diff 或 PR 进行代码审查",
    usage: "/review <target>", handler: "engine",
  },
  {
    id: "skills", label: "/skills", aliases: ["jinengliebiao", "技能列表"],
    category: "project", description: "列出可用技能",
    usage: "/skills [--remote|sync|<prefix>]", handler: "bridge", noArgAction: true,
  },
  {
    id: "skill", label: "/skill", aliases: ["jineng", "技能"],
    category: "project", description: "激活或管理技能",
    usage: "/skill <name|install <spec>|update <name>|uninstall <name>>", handler: "engine",
  },
  {
    id: "restore", label: "/restore", aliases: ["恢复"],
    category: "project", description: "将工作区回滚到之前的快照",
    usage: "/restore [N]", handler: "engine",
  },
  {
    id: "init", label: "/init", aliases: [],
    category: "project", description: "生成项目 AGENTS.md",
    usage: "/init", handler: "engine", noArgAction: true,
  },
  {
    id: "lsp", label: "/lsp", aliases: [],
    category: "project", description: "切换 LSP 诊断",
    usage: "/lsp [on|off|status]", handler: "bridge",
  },
  {
    id: "share", label: "/share", aliases: ["分享"],
    category: "project", description: "将会话导出为可分享的 URL",
    usage: "/share", handler: "engine", noArgAction: true,
  },
  {
    id: "hunt", label: "/hunt", aliases: ["goal", "mubiao", "目标"],
    category: "project", description: "设置会话目标",
    usage: "/hunt <quarry> [budget: N]", handler: "engine",
  },
];

/* ─── 公共 API ─────────────────────────────────────── */

export function getAllCommands(): CommandDef[] {
  return COMMANDS;
}

export function getCommandsByCategory(): Record<CommandCategory, CommandDef[]> {
  const map: Record<string, CommandDef[]> = {};
  for (const cmd of COMMANDS) {
    if (!map[cmd.category]) map[cmd.category] = [];
    map[cmd.category].push(cmd);
  }
  return map as Record<CommandCategory, CommandDef[]>;
}

export interface MatchResult {
  exact: CommandDef[];
  prefix: CommandDef[];
  contains: CommandDef[];
  fuzzy: CommandDef[];
}

/**
 * 三阶段模糊匹配：
 * 1. 精确前缀 — 命令 id 或别名以 filter 开头
 * 2. 包含子串 — 命令 id 或别名包含 filter
 * 3. 子序列匹配 — filter 的字符按顺序出现在 id 中（不连续）
 */
export function findCommands(filter: string): MatchResult {
  if (!filter.trim()) {
    return { exact: COMMANDS, prefix: [], contains: [], fuzzy: [] };
  }
  const f = filter.toLowerCase().trim();

  const exact: CommandDef[] = [];
  const prefix: CommandDef[] = [];
  const contains: CommandDef[] = [];
  const fuzzy: CommandDef[] = [];
  const seen = new Set<string>();

  const addIfNotSeen = (cmd: CommandDef, target: CommandDef[]) => {
    if (!seen.has(cmd.id)) {
      seen.add(cmd.id);
      target.push(cmd);
    }
  };

  for (const cmd of COMMANDS) {
    const searchables = [cmd.id.toLowerCase(), ...cmd.aliases.map(a => a.toLowerCase())];
    const isExact = searchables.some(s => s === f);
    const isPrefix = !isExact && searchables.some(s => s.startsWith(f));
    const isContains = !isExact && !isPrefix && searchables.some(s => s.includes(f));
    const isFuzzy = !isExact && !isPrefix && !isContains &&
      searchables.some(s => isSubsequence(f, s));

    if (isExact) addIfNotSeen(cmd, exact);
    else if (isPrefix) addIfNotSeen(cmd, prefix);
    else if (isContains) addIfNotSeen(cmd, contains);
    else if (isFuzzy) addIfNotSeen(cmd, fuzzy);
  }

  return { exact, prefix, contains, fuzzy };
}

/** 检查 chars 是否是 target 的子序列（不连续的字符在 target 中按顺序出现） */
function isSubsequence(chars: string, target: string): boolean {
  let ci = 0;
  for (let ti = 0; ti < target.length && ci < chars.length; ti++) {
    if (target[ti] === chars[ci]) ci++;
  }
  return ci === chars.length;
}

/** 根据 id 查找命令 */
export function findCommandById(id: string): CommandDef | undefined {
  const f = id.toLowerCase();
  return COMMANDS.find(
    cmd => cmd.id === f || cmd.aliases.some(a => a.toLowerCase() === f)
  );
}

/** 根据分类名获取中文标题 */
export const CATEGORY_LABELS: Record<CommandCategory, string> = {
  core: "核心",
  session: "会话",
  config: "配置",
  debug: "调试",
  project: "项目",
  skills: "技能",
};

/** 根据分类获取中文标题 */
export function getCategoryLabel(cat: CommandCategory): string {
  return CATEGORY_LABELS[cat] || cat;
}

/** 获取所有分类（按顺序） */
export const CATEGORY_ORDER: CommandCategory[] = [
  "core", "session", "config", "debug", "project", "skills",
];
