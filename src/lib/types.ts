/**
 * 类型统一导出入口
 *
 * 将 bridge.ts 中定义的所有数据类型重新导出，
 * 方便其他组件通过 `import type { Message } from "../lib/types"` 引用，
 * 而不需要关心具体类型定义在哪个模块。
 */
export type {
  SessionInfo,
  ToolCall,
  Message,
  ReasoningEvent,
  TokenEvent,
  ToolStartEvent,
  ToolEndEvent,
} from "./bridge";
