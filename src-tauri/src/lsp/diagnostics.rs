//! LSP 诊断数据结构与渲染器
//!
//! 定义 Diagnostic 结构体和 Severity 枚举（Error / Warning / Information / Hint），
//! 以及将诊断列表渲染为 Markdown 代码块的 render_blocks 函数。
//! 渲染结果以 `---\n<diagnostics file="...">` 格式注入对话上下文。
//!
//! Ported from CodeWhale crates/tui/src/lsp/diagnostics.rs

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error, Warning, Information, Hint,
}

impl Severity {
    #[must_use]
    pub fn from_lsp(code: Option<i64>) -> Option<Self> {
        match code? {
            1 => Some(Self::Error),
            2 => Some(Self::Warning),
            3 => Some(Self::Information),
            4 => Some(Self::Hint),
            _ => None,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Information => "INFO",
            Self::Hint => "HINT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: u32,
    pub column: u32,
    pub severity: Severity,
    pub message: String,
}

impl Diagnostic {
    fn render_message(&self) -> String {
        self.message.lines().next().unwrap_or("").trim().to_string()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiagnosticBlock {
    pub file: PathBuf,
    pub items: Vec<Diagnostic>,
}

#[allow(dead_code)]
impl DiagnosticBlock {
    #[must_use]
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }
        let file_attr = self.file.display();
        let mut out = format!("<diagnostics file=\"{file_attr}\">\n");
        for item in &self.items {
            out.push_str(&format!(
                "  {} [{}:{}] {}\n",
                item.severity.label(),
                item.line,
                item.column,
                item.render_message(),
            ));
        }
        out.push_str("</diagnostics>");
        out
    }

    pub fn truncate(&mut self, max_per_file: usize) {
        if self.items.len() > max_per_file {
            self.items.truncate(max_per_file);
        }
    }
}

#[must_use]
#[allow(dead_code)]
pub fn render_blocks(blocks: &[DiagnosticBlock]) -> String {
    let mut chunks = Vec::new();
    for block in blocks {
        let rendered = block.render();
        if !rendered.is_empty() {
            chunks.push(rendered);
        }
    }
    chunks.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_block_format() {
        let block = DiagnosticBlock {
            file: PathBuf::from("src/foo.rs"),
            items: vec![
                Diagnostic { line: 12, column: 8, severity: Severity::Error, message: "missing semicolon".to_string() },
                Diagnostic { line: 13, column: 1, severity: Severity::Error, message: "expected `,`".to_string() },
            ],
        };
        let r = block.render();
        assert!(r.contains("<diagnostics file=\"src/foo.rs\">"));
        assert!(r.contains("ERROR [12:8] missing semicolon"));
        assert!(r.ends_with("</diagnostics>"));
    }

    #[test]
    fn empty_block_renders_empty() {
        let block = DiagnosticBlock { file: PathBuf::from("foo.rs"), items: vec![] };
        assert!(block.render().is_empty());
    }
}
