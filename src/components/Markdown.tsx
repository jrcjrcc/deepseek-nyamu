/**
 * Markdown —— Markdown 渲染组件（含代码语法高亮）
 *
 * 基于 react-markdown，插件支持：
 * - remark-gfm：表格、任务列表等 GFM 扩展
 * - remark-math + rehype-katex：LaTeX 数学公式
 * - highlight.js：代码块语法高亮（自动检测语言）
 * - 自定义代码块渲染（CodeBlock 组件，含语言标签、复制按钮和高亮）
 * - 表格自动包裹（响应式滚动）
 */
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { useEffect, useRef, useState } from "react";
import { Copy, Check } from "lucide-react";
import hljs from "highlight.js";
import "highlight.js/styles/github-dark.css";
import * as bridge from "../lib/bridge";

interface MarkdownProps {
  content: string;
}

export function Markdown({ content }: MarkdownProps) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm, remarkMath]}
      rehypePlugins={[rehypeKatex]}
      components={{
        code({ className, children, ...props }) {
          const match = /language-(\w+)/.exec(className || "");
          const code = String(children).replace(/\n$/, "");
          if (match) {
            return <CodeBlock language={match[1]} code={code} />;
          }
          return <code className={className} {...props}>{children}</code>;
        },
        pre({ children }) {
          return <>{children}</>;
        },
        table({ children }) {
          return (
            <div className="table-wrapper">
              <table>{children}</table>
            </div>
          );
        },
      }}
    >
      {content}
    </ReactMarkdown>
  );
}

function CodeBlock({ language, code }: { language: string; code: string }) {
  const [copied, setCopied] = useState(false);
  const codeRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (codeRef.current) {
      hljs.highlightElement(codeRef.current);
    }
  }, [code]);

  const copy = () => {
    bridge.writeClipboard(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="code-block">
      <div className="code-block-header">
        <span className="code-lang">{language}</span>
        <button className="icon-btn" onClick={copy} title="Copy code">
          {copied ? <Check size={14} /> : <Copy size={14} />}
        </button>
      </div>
      <pre><code ref={codeRef} className={`language-${language}`}>{code}</code></pre>
    </div>
  );
}
