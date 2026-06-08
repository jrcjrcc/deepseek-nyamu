/**
 * WorkspacePanel —— 工作区文件浏览和变更列表（带右键菜单）
 *
 * 两个子面板：
 * - Files：文件树浏览器（懒加载展开/折叠 + 右键操作）
 * - Changes：当前会话的文件变更快照列表
 */
import { useState, useEffect, useCallback } from "react";
import { File, Folder, FolderOpen, ChevronRight, ChevronDown, FileText, Clock, Copy, Code, ExternalLink } from "lucide-react";
import * as bridge from "../lib/bridge";
import { ContextMenu, type MenuItem } from "./ContextMenu";

interface WorkspacePanelProps {
  workspacePath?: string;
}

interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  children?: FileNode[];
}

export function WorkspacePanel({ workspacePath }: WorkspacePanelProps) {
  const [view, setView] = useState<"files" | "changes">("files");

  return (
    <div className="workspace-panel">
      <div className="workspace-tabs">
        <button className={`workspace-tab ${view === "files" ? "active" : ""}`} onClick={() => setView("files")}>
          <FileText size={12} /> Files
        </button>
        <button className={`workspace-tab ${view === "changes" ? "active" : ""}`} onClick={() => setView("changes")}>
          <Clock size={12} /> Changes
        </button>
      </div>
      <div className="workspace-content">
        {view === "files" ? (
          <FileTree workspacePath={workspacePath} />
        ) : (
          <SessionChanges />
        )}
      </div>
    </div>
  );
}

function FileTree({ workspacePath }: { workspacePath?: string }) {
  const [tree, setTree] = useState<FileNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [contextMenu, setContextMenu] = useState<{
    x: number; y: number; node: FileNode;
  } | null>(null);

  const loadTree = useCallback(async () => {
    const root = workspacePath || (await tryGetWorkspace());
    if (!root) {
      setError("No workspace selected");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const result = await bridge.listDirectoryTree(root);
      setTree(result.entries || []);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [workspacePath]);

  useEffect(() => { loadTree(); }, [loadTree]);

  // Close context menu on Escape (ContextMenu handles outside-click itself)
  useEffect(() => {
    if (!contextMenu) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setContextMenu(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [contextMenu]);

  const handleNodeContextMenu = useCallback((e: React.MouseEvent, node: FileNode) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, node });
  }, []);

  if (error) return <p className="panel-hint">{error}</p>;
  if (loading) return <p className="panel-hint">Loading...</p>;

  return (
    <div className="file-tree">
      {tree.length === 0 ? (
        <p className="panel-hint">No files found</p>
      ) : (
        tree.map((node) => (
          <FileTreeNode
            key={node.path}
            node={node}
            depth={0}
            onContextMenu={handleNodeContextMenu}
          />
        ))
      )}

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={buildFileMenuItems(contextMenu.node)}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}

function FileTreeNode({
  node, depth, onContextMenu,
}: {
  node: FileNode; depth: number;
  onContextMenu: (e: React.MouseEvent, node: FileNode) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [children, setChildren] = useState<FileNode[] | null>(
    node.children || null
  );

  const handleToggle = async () => {
    if (!node.is_dir) return;
    if (expanded) { setExpanded(false); return; }
    if (!children) {
      try {
        const result = await bridge.listDirectoryTree(node.path);
        setChildren(result.entries || []);
      } catch { setChildren([]); }
    }
    setExpanded(true);
  };

  return (
    <div>
      <div
        className="file-tree-node"
        style={{ paddingLeft: depth * 16 + 8 }}
        onClick={handleToggle}
        onContextMenu={(e) => onContextMenu(e, node)}
      >
        {node.is_dir ? (
          <>
            {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            {expanded ? <FolderOpen size={14} /> : <Folder size={14} />}
          </>
        ) : (
          <>
            <span style={{ width: 12 }} />
            <File size={14} />
          </>
        )}
        <span className="file-tree-name">{node.name}</span>
      </div>
      {expanded && children && (
        <div>
          {children.length === 0 ? (
            <div className="file-tree-empty" style={{ paddingLeft: depth * 16 + 28 }}>Empty</div>
          ) : (
            children.map((child) => (
              <FileTreeNode key={child.path} node={child} depth={depth + 1} onContextMenu={onContextMenu} />
            ))
          )}
        </div>
      )}
    </div>
  );
}

function buildFileMenuItems(node: FileNode): MenuItem[] {
  const items: MenuItem[] = [];

  if (node.is_dir) {
    items.push({
      id: "copy-path",
      label: "Copy directory path",
      icon: <Copy size={14} />,
      action: () => bridge.writeClipboard(node.path),
    });
  } else {
    items.push({
      id: "open-file",
      label: "Open file",
      icon: <ExternalLink size={14} />,
      action: async () => {
        try {
          const content = await bridge.readFileContent(node.path);
          // Create a downloadable blob
          const blob = new Blob([content], { type: "text/plain" });
          // Use clipboard for smaller files
          if (content.length < 100000) {
            await bridge.writeClipboard(content);
          }
        } catch {}
      },
    });
    items.push({
      id: "copy-path",
      label: "Copy file path",
      icon: <Copy size={14} />,
      action: () => bridge.writeClipboard(node.path),
    });
    items.push({
      id: "copy-content",
      label: "Copy file content",
      icon: <Code size={14} />,
      action: async () => {
        try {
          const content = await bridge.readFileContent(node.path);
          if (content.length < 500000) {
            await bridge.writeClipboard(content);
          }
        } catch {}
      },
    });
  }

  return items;
}

function SessionChanges() {
  const [changes, setChanges] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    bridge.getSessionChanges().then(setChanges).catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <p className="panel-hint">Loading changes...</p>;

  return (
    <div className="session-changes">
      {changes.length === 0 ? (
        <p className="panel-hint">No file changes in this session.</p>
      ) : (
        changes.map((c, i) => (
          <div key={i} className="change-item">
            <span className="change-id">{c.id?.slice(0, 8)}</span>
            {c.label && <span className="change-label">{c.label}</span>}
          </div>
        ))
      )}
    </div>
  );
}

async function tryGetWorkspace(): Promise<string | null> {
  try {
    await bridge.listDirectoryTree(".");
    return ".";
  } catch {
    return null;
  }
}
