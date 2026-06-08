//! 语言检测与 LSP 服务器映射表
//!
//! 通过文件扩展名检测编程语言（Rust / Go / Python / TypeScript 等），
//! 并返回对应的 LSP 服务器命令（如 rust-analyzer、gopls、pyright 等）。
//!
//! Language 枚举的 detect 方法使用固定字典匹配扩展名，
//! server_for 函数返回静态注册的 LSP 服务器命令。
//!
//! Ported from CodeWhale crates/tui/src/lsp/registry.rs

use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust, Go, Python, TypeScript, JavaScript, Java, Vue, C, Cpp, Other,
}

impl Language {
    #[must_use]
    pub fn as_key(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Go => "go",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Java => "java",
            Language::Vue => "vue",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Other => "other",
        }
    }

    #[must_use]
    pub fn language_id(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Go => "go",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Java => "java",
            Language::Vue => "vue",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Other => "plaintext",
        }
    }
}

#[must_use]
pub fn detect_language(path: &Path) -> Language {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext.to_ascii_lowercase(),
        None => return Language::Other,
    };
    match ext.as_str() {
        "rs" => Language::Rust,
        "go" => Language::Go,
        "py" | "pyi" => Language::Python,
        "ts" | "tsx" => Language::TypeScript,
        "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
        "java" => Language::Java,
        "vue" => Language::Vue,
        "c" | "h" => Language::C,
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Language::Cpp,
        _ => Language::Other,
    }
}

#[must_use]
pub fn server_for(lang: Language) -> Option<(&'static str, &'static [&'static str])> {
    match lang {
        Language::Rust => Some(("rust-analyzer", &[])),
        Language::Go => Some(("gopls", &["serve"])),
        Language::Python => Some(("pyright-langserver", &["--stdio"])),
        Language::TypeScript | Language::JavaScript => Some(("typescript-language-server", &["--stdio"])),
        Language::Java => Some(("jdtls", &[])),
        Language::Vue => Some(("vue-language-server", &["--stdio"])),
        Language::C | Language::Cpp => Some(("clangd", &[])),
        Language::Other => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_rust_extension() {
        assert_eq!(detect_language(&PathBuf::from("foo.rs")), Language::Rust);
    }
    #[test]
    fn detects_unknown_as_other() {
        assert_eq!(detect_language(&PathBuf::from("notes.txt")), Language::Other);
    }
    #[test]
    fn server_for_rust_is_rust_analyzer() {
        let (cmd, args) = server_for(Language::Rust).expect("rust has a server");
        assert_eq!(cmd, "rust-analyzer");
        assert!(args.is_empty());
    }
}
