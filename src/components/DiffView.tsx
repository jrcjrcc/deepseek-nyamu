/**
 * DiffView —— 统一 Diff 渲染组件
 *
 * 解析并渲染标准 unified diff 格式：
 * - ---/+++ 文件头
 * - @@ 块标
 * - -/+ 行（删除/新增）
 * - 上下文行
 *
 * 支持折叠超过 50 行的块，显示文件名标签。
 */
import { useMemo, useState } from "react";

interface DiffViewProps {
  diff: string;
  maxLines?: number;
}

interface DiffHunk {
  oldStart: number;
  newStart: number;
  lines: { type: "add" | "del" | "ctx"; text: string; oldLine: number; newLine: number }[];
}

interface DiffFile {
  oldFile: string;
  newFile: string;
  hunks: DiffHunk[];
}

export function DiffView({ diff, maxLines = 50 }: DiffViewProps) {
  const parsed = useMemo(() => parseDiff(diff), [diff]);

  if (!parsed || parsed.length === 0) {
    return <div className="diff-empty">No diff to display</div>;
  }

  return (
    <div className="diff-view">
      {parsed.map((file, fi) => (
        <DiffFileBlock key={fi} file={file} maxLines={maxLines} />
      ))}
    </div>
  );
}

function DiffFileBlock({ file, maxLines }: { file: DiffFile; maxLines: number }) {
  const [collapsed, setCollapsed] = useState(false);
  const totalLines = file.hunks.reduce((s, h) => s + h.lines.length, 0);
  const canCollapse = totalLines > maxLines;

  return (
    <div className="diff-file">
      <div className="diff-file-header" onClick={() => canCollapse && setCollapsed((p) => !p)}>
        <span className="diff-file-name">{file.newFile}</span>
        {file.oldFile !== file.newFile && (
          <span className="diff-file-rename">{file.oldFile} →</span>
        )}
        <span className="diff-file-stats">
          +{file.hunks.reduce((s, h) => s + h.lines.filter((l) => l.type === "add").length, 0)}
          &nbsp;-{file.hunks.reduce((s, h) => s + h.lines.filter((l) => l.type === "del").length, 0)}
        </span>
        {canCollapse && (
          <span className="diff-collapse-btn">{collapsed ? "Expand" : "Collapse"}</span>
        )}
      </div>
      {!collapsed && file.hunks.map((hunk, hi) => (
        <div key={hi} className="diff-hunk">
          <div className="diff-hunk-header">
            @@ -{hunk.oldStart},{hunk.lines.length} +{hunk.newStart},{hunk.lines.length} @@
          </div>
          {hunk.lines.map((line, li) => (
            <div key={li} className={`diff-line diff-line-${line.type}`}>
              <span className="diff-line-num diff-line-num-old">{line.oldLine || ""}</span>
              <span className="diff-line-num diff-line-num-new">{line.newLine || ""}</span>
              <span className="diff-line-prefix">
                {line.type === "add" ? "+" : line.type === "del" ? "-" : " "}
              </span>
              <span className="diff-line-text">{line.text}</span>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

function parseDiff(diff: string): DiffFile[] {
  const lines = diff.split("\n");
  const files: DiffFile[] = [];
  let current: DiffFile | null = null;
  let currentHunk: DiffHunk | null = null;
  let oldLine = 0;
  let newLine = 0;

  for (const raw of lines) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;

    if (line.startsWith("--- ")) {
      // new file
      if (current && currentHunk) {
        current.hunks.push(currentHunk);
        currentHunk = null;
      }
      if (current) files.push(current);
      current = { oldFile: line.slice(4), newFile: "", hunks: [] };
      continue;
    }
    if (line.startsWith("+++ ")) {
      if (current) current.newFile = line.slice(4);
      continue;
    }
    if (line.startsWith("@@")) {
      if (current && currentHunk) {
        current.hunks.push(currentHunk);
      }
      const m = line.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
      currentHunk = { oldStart: m ? parseInt(m[1]) : 0, newStart: m ? parseInt(m[2]) : 0, lines: [] };
      oldLine = currentHunk.oldStart;
      newLine = currentHunk.newStart;
      continue;
    }
    if (!current || !currentHunk) continue;

    if (line.startsWith("+")) {
      currentHunk.lines.push({ type: "add", text: line.slice(1), oldLine: 0, newLine: newLine });
      newLine++;
    } else if (line.startsWith("-")) {
      currentHunk.lines.push({ type: "del", text: line.slice(1), oldLine: oldLine, newLine: 0 });
      oldLine++;
    } else {
      currentHunk.lines.push({ type: "ctx", text: line.slice(1), oldLine, newLine });
      oldLine++;
      newLine++;
    }
  }

  // flush last
  if (current && currentHunk) current.hunks.push(currentHunk);
  if (current) files.push(current);

  return files;
}
