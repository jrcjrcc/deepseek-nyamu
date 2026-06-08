/**
 * DeepWhale 前端入口
 *
 * 挂载 React 应用到 DOM 的根节点 (#root)。
 * 使用 StrictMode 以便在开发阶段捕获潜在的副作用问题。
 */
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
