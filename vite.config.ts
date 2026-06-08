/**
 * Vite 构建配置
 *
 * 开发服务器配置：
 * - 端口 1420（与 Tauri 后端约定端口）
 * - strictPort：端口被占用时不自动回退
 * - HMR 通过 WebSocket 端口 1421（支持局域网访问，需设置 TAURI_DEV_HOST）
 * - 忽略 src-tauri/ 目录的变动（由 Tauri 热重载管理）
 */
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],

  // Tauri 开发时禁用 Vite 的自动清除屏幕
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    // 如果设置了 TAURI_DEV_HOST，监听局域网地址（用于移动端/远程调试）
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tauri 的 Rust 文件由 cargo watch 管理，不需要 Vite 监听
      ignored: ["**/src-tauri/**"],
    },
  },
}));
