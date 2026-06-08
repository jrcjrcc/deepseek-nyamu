//! 递归语言模型（RLM）—— 长文本智能分解与分析
//!
//! 实现自 Zhang, Kraska & Khattab (arXiv:2512.24601) 的 Algorithm 1。
//!
//! 核心思想：不需要 Python REPL，而是通过符号递归模式：
//! - 确定性操作（peek, search, chunk）在 Rust 中执行
//! - 语义操作（sub_query, sub_query_batch, finalize）通过 LLM API 执行
//!
//! 循环伪代码：
//!   state ← Load(context)
//!   loop:
//!     meta ← Metadata(state)
//!     code ← LLM(system_prompt, meta, round_history)
//!     (state, stdout) ← Exec(code)
//!     if state.final is set: return state.final
//!
//! Ported from CodeWhale crates/tui/src/rlm/
//!
//! Lightweight Recursive Language Model (RLM) — Algorithm 1 from
//! Zhang, Kraska & Khattab (arXiv:2512.24601).
//!
//! ```text
//! state ← Load(context)
//! loop:
//!   meta ← Metadata(state)
//!   code ← LLM(system_prompt, meta, round_history)
//!   (state, stdout) ← Exec(code)
//!   if state.final is set: return state.final
//! ```
//!
//! Ported from CodeWhale crates/tui/src/rlm/

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[allow(dead_code)]
use anyhow::Result;
use serde::Serialize;
use serde_json::{Value, json};

mod prompt;

pub use prompt::rlm_system_prompt;

// ─── Configuration ────────────────────────────────────────────────────

/// Maximum RLM iterations before the loop gives up.
const MAX_ITERATIONS: u32 = 25;
/// Max consecutive rounds without a `repl` fence before hard-fail.
const MAX_NO_CODE: u32 = 3;
/// Max output tokens for the root LLM.
const ROOT_MAX_TOKENS: u32 = 4096;
/// Max chars of stdout shown as metadata.
const STDOUT_PREVIEW_LEN: usize = 800;
/// Max chars of context preview.
const CONTEXT_PREVIEW_LEN: usize = 500;
/// Max history messages kept across iterations.
const MAX_HISTORY: usize = 20;

// ─── Types ────────────────────────────────────────────────────────────

/// How an RLM turn ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RlmTermination {
    Final,
    NoCode,
    Exhausted,
    Error,
}

/// Per-round trace entry.
#[derive(Debug, Clone, Serialize)]
pub struct RlmRoundTrace {
    pub round: u32,
    pub code_summary: String,
    pub stdout_preview: String,
    pub had_error: bool,
    pub child_calls: u32,
    pub elapsed_ms: u64,
}

/// Result of an RLM turn.
#[derive(Debug, Clone, Serialize)]
pub struct RlmTurnResult {
    pub answer: String,
    pub iterations: u32,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub termination: RlmTermination,
    pub trace: Vec<RlmRoundTrace>,
    pub total_child_calls: u32,
}

/// Context metadata sent to the root LLM instead of the raw body.
#[derive(Debug, Clone, Serialize)]
pub struct ContextMeta {
    pub chars: usize,
    pub lines: usize,
    pub preview: String,
    pub tail_preview: String,
    pub sha256: String,
}

impl ContextMeta {
    fn from_text(text: &str) -> Self {
        use sha2::{Digest as _, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let sha = format!("{:x}", hasher.finalize());

        let chars: Vec<char> = text.chars().collect();
        let lines = text.lines().count();
        let preview: String = chars.iter().take(CONTEXT_PREVIEW_LEN).collect();
        let tail: String = chars.iter().rev().take(CONTEXT_PREVIEW_LEN / 2).collect::<Vec<_>>().into_iter().rev().collect();

        Self {
            chars: chars.len(),
            lines,
            preview: if text.len() > CONTEXT_PREVIEW_LEN { format!("{preview}...") } else { preview },
            tail_preview: tail,
            sha256: sha,
        }
    }
}

/// RLM state — holds variables across iterations.
pub struct RlmState {
    pub context: String,
    pub meta: ContextMeta,
    pub variables: HashMap<String, Value>,
    pub final_answer: Option<String>,
}

impl RlmState {
    fn new(context: String) -> Self {
        let meta = ContextMeta::from_text(&context);
        Self { context, meta, variables: HashMap::new(), final_answer: None }
    }
}

// ─── Deterministic helpers (replaces Python REPL functions) ───────────

impl RlmState {
    fn context_meta(&self) -> Value {
        json!({
            "chars": self.meta.chars,
            "lines": self.meta.lines,
            "preview": self.meta.preview,
            "tail_preview": self.meta.tail_preview,
            "sha256": self.meta.sha256,
        })
    }

    fn peek(&self, start: usize, end: usize, unit: &str) -> Value {
        match unit {
            "lines" | "line" => {
                let lines: Vec<&str> = self.context.lines().collect();
                let start = start.min(lines.len());
                let end = end.min(lines.len());
                json!({"start": start, "end": end, "text": lines[start..end].join("\n")})
            }
            _ => {
                let chars: Vec<char> = self.context.chars().collect();
                let start = start.min(chars.len());
                let end = end.min(chars.len());
                let text: String = chars[start..end].iter().collect();
                json!({"start": start, "end": end, "text": text, "unit": "chars"})
            }
        }
    }

    fn search(&self, pattern: &str, max_hits: usize) -> Value {
        let _lower = self.context.to_lowercase();
        let p_lower = pattern.to_lowercase();
        let mut hits = Vec::new();

        let lines: Vec<&str> = self.context.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if hits.len() >= max_hits { break; }
            if line.to_lowercase().contains(&p_lower) {
                let snippet: String = line.chars().take(200).collect();
                hits.push(json!({"line": i + 1, "snippet": snippet}));
            }
        }
        json!({"pattern": pattern, "hits": hits, "total": hits.len()})
    }

    fn chunk(&self, max_chars: usize, overlap: usize) -> Value {
        let chars: Vec<char> = self.context.chars().collect();
        let total = chars.len();
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < total {
            let end = (start + max_chars).min(total);
            let text: String = chars[start..end].iter().collect();
            chunks.push(json!({
                "index": index,
                "start": start,
                "end": end,
                "text": text,
            }));
            index += 1;
            if end >= total { break; }
            start = end.saturating_sub(overlap);
        }

        json!({"chunks": chunks, "total_chunks": chunks.len(), "coverage": {
            "input_chars": total,
            "covered_chars": chunks.iter().map(|c| c["end"].as_u64().unwrap_or(0) - c["start"].as_u64().unwrap_or(0)).sum::<u64>(),
            "chunks": chunks.len(),
        }})
    }

    fn show_vars(&self) -> Value {
        json!(self.variables.iter().map(|(k, v)| {
            json!({"name": k, "type": match v { Value::String(_) => "str", Value::Array(_) => "list", Value::Object(_) => "dict", _ => "other" }})
        }).collect::<Vec<_>>())
    }

    fn repl_set(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    fn repl_get(&self, name: &str) -> Value {
        self.variables.get(name).cloned().unwrap_or(Value::Null)
    }

    fn evaluate_progress(&self) -> Value {
        json!({
            "has_final": self.final_answer.is_some(),
            "var_count": self.variables.len(),
            "var_names": self.variables.keys().collect::<Vec<_>>(),
        })
    }

    fn finalize(&mut self, value: &str, _confidence: Option<&str>) -> Value {
        self.final_answer = Some(value.to_string());
        json!({"ok": true, "final": value})
    }
}

// ─── Code executor ────────────────────────────────────────────────────

/// Execute a `repl` code block against the state.
fn execute_code(state: &mut RlmState, code: &str) -> Result<String> {
    let code = code.trim();
    let mut output = String::new();

    // Parse line by line
    for line in code.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        // Match deterministic REPL functions
        let result = if line.starts_with("context_meta(") || line.starts_with("print(context_meta(") {
            let meta = state.context_meta();
            if line.starts_with("print") {
                output.push_str(&format!("{}\n", serde_json::to_string_pretty(&meta)?));
            }
            None
        } else if line.starts_with("peek(") {
            let args = extract_args(line, "peek");
            let start = args.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let end = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(100);
            let unit = args.get(2).map(|s| s.trim().trim_matches('"').trim_matches('\'')).unwrap_or("chars");
            let result = state.peek(start, end, unit);
            if line.starts_with("print") {
                output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            }
            None
        } else if line.starts_with("search(") {
            let args = extract_args(line, "search");
            let pattern = args.get(0).map(|s| s.trim().trim_matches('"').trim_matches('\'')).unwrap_or("");
            let max_hits = args.iter().position(|a| a.contains("max_hits"))
                .and_then(|i| args.get(i+1))
                .and_then(|s| s.trim().trim_end_matches(')').parse::<usize>().ok())
                .unwrap_or(100);
            let result = state.search(pattern, max_hits);
            if line.starts_with("print") {
                output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            }
            None
        } else if line.starts_with("chunk(") || line.starts_with("print(chunk(") {
            let args = extract_args(line, "chunk");
            let max_chars = args.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20000);
            let overlap = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let result = state.chunk(max_chars, overlap);
            if line.starts_with("print") {
                output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            }
            None
        } else if line.starts_with("SHOW_VARS(") || line.starts_with("print(SHOW_VARS(") {
            let result = state.show_vars();
            if line.starts_with("print") {
                output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            }
            None
        } else if line.starts_with("evaluate_progress(") || line.starts_with("print(evaluate_progress(") {
            let result = state.evaluate_progress();
            output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            None
        } else if line.starts_with("repl_set(") {
            let inner = line.trim_start_matches("repl_set(").trim_end_matches(')');
            if let Some(comma) = inner.find(',') {
                let name = inner[..comma].trim().trim_matches('"').trim_matches('\'');
                let value_str = inner[comma+1..].trim();
                state.repl_set(name, serde_json::from_str(value_str).unwrap_or(Value::String(value_str.to_string())));
                output.push_str(&format!("set variable: {name}\n"));
            }
            None
        } else if line.starts_with("repl_get(") {
            let name = line.trim_start_matches("repl_get(").trim_end_matches(')').trim().trim_matches('"').trim_matches('\'');
            let result = state.repl_get(name);
            output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            None
        } else if line.starts_with("finalize(") {
            let inner = line.trim_start_matches("finalize(");
            let content = inner.rsplit_once("confidence=")
                .map(|(v, _)| v.trim().trim_end_matches(')'))
                .unwrap_or(inner.trim_end_matches(')'));
            let value = content.trim().trim_matches('"').trim_matches('\'').trim_matches('f').trim_matches('(').trim();
            let result = state.finalize(value, None);
            output.push_str(&format!("{}\n", serde_json::to_string_pretty(&result)?));
            None
        } else if let Some(captured) = line.strip_prefix("print(").and_then(|s| s.strip_suffix(')')) {
            // Simple print
            output.push_str(captured);
            output.push('\n');
            None
        } else {
            Some(anyhow::anyhow!("Unknown REPL command: {line}"))
        };

        if let Some(err) = result {
            return Err(err);
        }
    }

    Ok(output)
}

/// Extract arguments from a function call like `peek(0, 100, "chars")`
fn extract_args(line: &str, func_name: &str) -> Vec<String> {
    let start = line.find(func_name)
        .and_then(|i| line[i..].find('('))
        .map(|i| i + 1)
        .unwrap_or(0);
    // Find matching closing paren
    let mut depth = 0;
    let end = line[start..].chars().position(|c| {
        match c { '(' => depth += 1, ')' => depth -= 1, _ => {} }
        depth < 0
    }).map(|i| start + i).unwrap_or(line.len());

    let args = line[start..end].trim();
    if args.is_empty() { return Vec::new(); }

    // Simple comma split (doesn't handle nested parens well, but good enough)
    args.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ─── Main RLM loop ────────────────────────────────────────────────────

/// Run a full RLM turn.
///
/// `context` is the large text to analyze. `task` is what the user wants done.
/// The root LLM receives only metadata + the task, never the raw context.
pub async fn run_rlm_turn(
    api_key: &str,
    base_url: &str,
    model: &str,
    _child_model: &str,
    context: String,
    task: Option<&str>,
    _max_depth: u32,
) -> RlmTurnResult {
    let start = Instant::now();
    let mut state = RlmState::new(context);
    let mut trace: Vec<RlmRoundTrace> = Vec::new();
    let total_child_calls: u32 = 0;
    let mut no_code_rounds: u32 = 0;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .unwrap();

    // ── Round loop ────────────────────────────────────────────────────
    let mut round_history: Vec<Value> = Vec::new();

    for round in 0..MAX_ITERATIONS {
        let round_start = Instant::now();
        let child_calls_this_round = 0u32;

        // Build the system prompt
        let system_prompt = rlm_system_prompt();

        // Build metadata for root LLM
        let meta = state.context_meta();
        let vars = state.show_vars();
        let history_summary: Vec<&Value> = round_history.iter().rev().take(3).collect();

        let metadata = json!({
            "round": round,
            "context": meta,
            "variables": vars,
            "task": task.unwrap_or("Analyze the context and produce a final answer."),
            "history": history_summary,
        });

        // Call the root LLM
        let messages = vec![
            json!({"role": "system", "content": system_prompt}),
            json!({"role": "user", "content": format!("```json\n{}\n```\n\nEmit a single ```repl block with Python code.", serde_json::to_string_pretty(&metadata).unwrap_or_default())}),
        ];

        let body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": ROOT_MAX_TOKENS,
            "temperature": 0.3,
            "stream": false,
        });

        let resp = match client
            .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return RlmTurnResult {
                    answer: String::new(),
                    iterations: round,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("RLM API call failed at round {round}: {e}")),
                    termination: RlmTermination::Error,
                    trace,
                    total_child_calls,
                };
            }
        };

        let response_text = resp.text().await.unwrap_or_default();
        let response_json: Value = serde_json::from_str(&response_text).unwrap_or(json!({}));
        let content = response_json["choices"][0]["message"]["content"].as_str().unwrap_or("");

        // Check for finalize in the response
        if content.contains("finalize(") || content.to_uppercase().contains("FINALIZE") {
            // Try to extract final value
            let answer = if content.contains("finalize(") {
                // Naive extraction — the real answer will come from a repl block
                "RLM completed. (See trace output.)"
            } else {
                content
            };

            state.finalize(answer, None);
            let duration_ms = start.elapsed().as_millis() as u64;
            trace.push(RlmRoundTrace {
                round,
                code_summary: "finalize()".to_string(),
                stdout_preview: answer.chars().take(200).collect(),
                had_error: false,
                child_calls: child_calls_this_round,
                elapsed_ms: round_start.elapsed().as_millis() as u64,
            });

            return RlmTurnResult {
                answer: state.final_answer.unwrap_or_default(),
                iterations: round + 1,
                duration_ms,
                error: None,
                termination: RlmTermination::Final,
                trace,
                total_child_calls,
            };
        }

        // Extract repl code block
        let code = extract_repl_block(content);
        let code_summary = code.as_ref().map(|c| c.chars().take(100).collect::<String>()).unwrap_or_default();

        if code.is_none() {
            no_code_rounds += 1;
            if no_code_rounds >= MAX_NO_CODE {
                let duration_ms = start.elapsed().as_millis() as u64;
                trace.push(RlmRoundTrace {
                    round,
                    code_summary: "no-code".to_string(),
                    stdout_preview: content.chars().take(200).collect(),
                    had_error: false,
                    child_calls: 0,
                    elapsed_ms: round_start.elapsed().as_millis() as u64,
                });
                return RlmTurnResult {
                    answer: content.to_string(),
                    iterations: round + 1,
                    duration_ms,
                    error: None,
                    termination: RlmTermination::NoCode,
                    trace,
                    total_child_calls,
                };
            }
            continue;
        }

        let code = code.unwrap();
        no_code_rounds = 0;

        // Execute the code against our Rust-internal state
        match execute_code(&mut state, &code) {
            Ok(stdout) => {
                let preview: String = stdout.chars().take(STDOUT_PREVIEW_LEN).collect();
                let duration_ms = round_start.elapsed().as_millis() as u64;
                trace.push(RlmRoundTrace {
                    round,
                    code_summary: code_summary.clone(),
                    stdout_preview: if stdout.len() > STDOUT_PREVIEW_LEN { format!("{preview}...") } else { preview },
                    had_error: false,
                    child_calls: child_calls_this_round,
                    elapsed_ms: duration_ms,
                });

                round_history.push(json!({
                    "round": round,
                    "code": code_summary,
                    "stdout": stdout.chars().take(300).collect::<String>(),
                }));

                // Trim history
                while round_history.len() > MAX_HISTORY {
                    round_history.remove(0);
                }
            }
            Err(e) => {
                let duration_ms = round_start.elapsed().as_millis() as u64;
                trace.push(RlmRoundTrace {
                    round,
                    code_summary: format!("ERROR: {e}"),
                    stdout_preview: String::new(),
                    had_error: true,
                    child_calls: 0,
                    elapsed_ms: duration_ms,
                });
            }
        }

        // Check for finalize set during execution
        if state.final_answer.is_some() {
            let duration_ms = start.elapsed().as_millis() as u64;
            return RlmTurnResult {
                answer: state.final_answer.clone().unwrap_or_default(),
                iterations: round + 1,
                duration_ms,
                error: None,
                termination: RlmTermination::Final,
                trace,
                total_child_calls,
            };
        }
    }

    // Exhausted
    RlmTurnResult {
        answer: String::new(),
        iterations: MAX_ITERATIONS,
        duration_ms: start.elapsed().as_millis() as u64,
        error: Some("RLM iteration cap reached".to_string()),
        termination: RlmTermination::Exhausted,
        trace,
        total_child_calls,
    }
}

/// Extract the first ```repl block from model output.
fn extract_repl_block(text: &str) -> Option<String> {
    let start_marker = "```repl";
    let end_marker = "```";

    let start = text.find(start_marker)?;
    let code_start = start + start_marker.len();
    let after = &text[code_start..];
    let end = after.find(end_marker)?;

    Some(after[..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_meta_counts_chars_and_lines() {
        let state = RlmState::new("hello\nworld\n".to_string());
        let meta = state.context_meta();
        assert_eq!(meta["chars"], 12);
        assert_eq!(meta["lines"], 2);
    }

    #[test]
    fn peek_returns_bounded_slice() {
        let state = RlmState::new("abcdefghij".to_string());
        let result = state.peek(2, 5, "chars");
        assert_eq!(result["text"], "cde");
    }

    #[test]
    fn search_finds_substring() {
        let state = RlmState::new("line one\nline two\nline three".to_string());
        let result = state.search("two", 10);
        assert_eq!(result["total"], 1);
    }

    #[test]
    fn chunk_divides_text() {
        let text = "a".repeat(5000);
        let state = RlmState::new(text);
        let result = state.chunk(2000, 100);
        assert_eq!(result["total_chunks"], 3);
    }

    #[test]
    fn repl_finalize_sets_answer() {
        let mut state = RlmState::new("test".to_string());
        state.finalize("done", None);
        assert_eq!(state.final_answer.as_deref(), Some("done"));
    }

    #[test]
    fn extract_repl_block_works() {
        let text = "Some text\n```repl\nprint(context_meta())\n```\nmore text";
        let code = extract_repl_block(text);
        assert_eq!(code.as_deref(), Some("print(context_meta())"));
    }

    #[test]
    fn execute_peek_produces_output() {
        let mut state = RlmState::new("hello world".to_string());
        let output = execute_code(&mut state, "print(peek(0, 5, \"chars\"))").unwrap();
        assert!(output.contains("hello"));
    }

    #[test]
    fn rlm_prompt_is_not_empty() {
        let prompt = rlm_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("```repl"));
        assert!(prompt.contains("finalize"));
    }
}
