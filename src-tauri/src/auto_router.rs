//! 自动模型路由 —— 根据请求复杂度智能选择模型
//!
//! 两层架构：
//! 1. 快速确定性启发式（基于关键字 + 输入长度判断）
//! 2. 可选：对模糊情况调用 Flash 模型进行二次判断
//!
//! 当配置中 model 设为 "auto" 时触发此模块。
//! 端口自 CodeWhale crates/tui/src/commands/config.rs
//!
//! Fin auto-routing — selects model and reasoning effort per turn.
//!
//! Two-layer architecture:
//! 1. Fast deterministic heuristic (keyword + length based)
//! 2. Optional Flash router call for ambiguous cases
//!
//! Ported from CodeWhale crates/tui/src/commands/config.rs

use std::time::Duration;

#[allow(dead_code)]
/// Auto-select a model based on request complexity.
pub fn auto_model_heuristic(input: &str, _current_model: &str) -> String {
    auto_model_heuristic_with_bias(input, _current_model, false)
}

#[allow(dead_code)]
/// `auto_model_heuristic` with optional cost-saving bias.
pub fn auto_model_heuristic_with_bias(
    input: &str,
    _current_model: &str,
    cost_saving: bool,
) -> String {
    auto_model_heuristic_selection_with_bias(input, _current_model, cost_saving).model
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutoModelHeuristicConfidence {
    Decisive,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AutoModelHeuristicSelection {
    model: String,
    confidence: AutoModelHeuristicConfidence,
}

fn auto_model_heuristic_selection_with_bias(
    input: &str,
    _current_model: &str,
    cost_saving: bool,
) -> AutoModelHeuristicSelection {
    let len = input.chars().count();
    let lower = input.to_lowercase();
    let borderline_pro_keywords: &[&str] = &[
        "implement",
        "analyze",
        "\u{5b9e}\u{73b0}", // 实现
        "\u{5206}\u{6790}", // 分析
        "\u{5be6}\u{73fe}", // 實現
    ];
    let strong_match = COMPLEX_KEYWORDS
        .iter()
        .any(|kw| !borderline_pro_keywords.contains(kw) && lower.contains(kw));
    let borderline_match = borderline_pro_keywords.iter().any(|kw| lower.contains(kw));
    let pro_match = strong_match || (!cost_saving && borderline_match);
    if pro_match {
        return AutoModelHeuristicSelection {
            model: "deepseek-v4-pro".to_string(),
            confidence: AutoModelHeuristicConfidence::Decisive,
        };
    }
    // Short messages → Flash
    if len < 100 {
        return AutoModelHeuristicSelection {
            model: "deepseek-v4-flash".to_string(),
            confidence: AutoModelHeuristicConfidence::Decisive,
        };
    }
    // Long complex requests → Pro.
    let long_threshold = if cost_saving { 1_000 } else { 500 };
    if len > long_threshold {
        return AutoModelHeuristicSelection {
            model: "deepseek-v4-pro".to_string(),
            confidence: AutoModelHeuristicConfidence::Decisive,
        };
    }

    // Ambiguous — use Flash but let the Flash router decide if available
    AutoModelHeuristicSelection {
        model: "deepseek-v4-flash".to_string(),
        confidence: AutoModelHeuristicConfidence::Ambiguous,
    }
}

/// Keywords that escalate auto-mode model selection to deepseek-v4-pro.
const COMPLEX_KEYWORDS: &[&str] = &[
    // English
    "refactor", "architecture", "design", "debug", "security",
    "review", "audit", "migrate", "optimize", "rewrite",
    "implement", "analyze",
    // Simplified Chinese
    "\u{91cd}\u{6784}", // 重构
    "\u{67b6}\u{6784}", // 架构
    "\u{8bbe}\u{8ba1}", // 设计
    "\u{8c03}\u{8bd5}", // 调试
    "\u{5b89}\u{5168}", // 安全
    "\u{5ba1}\u{67e5}", // 审查
    "\u{5ba1}\u{8ba1}", // 审计
    "\u{8fc1}\u{79fb}", // 迁移
    "\u{4f18}\u{5316}", // 优化
    "\u{91cd}\u{5199}", // 重写
    "\u{5b9e}\u{73b0}", // 实现
    "\u{5206}\u{6790}", // 分析
    // Traditional Chinese
    "\u{91cd}\u{69cb}", // 重構
    "\u{67b6}\u{69cb}", // 架構
    "\u{8a2d}\u{8a08}", // 設計
    "\u{8abf}\u{8a66}", // 調試
    "\u{5be9}\u{67e5}", // 審查
    "\u{5be9}\u{8a08}", // 審計
    "\u{9077}\u{79fb}", // 遷移
    "\u{512a}\u{5316}", // 優化
    "\u{91cd}\u{5beb}", // 重寫
    "\u{5be6}\u{73fe}", // 實現
];

/// Result from the Flash router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoRouteRecommendation {
    pub model: String,
    pub reasoning_effort: Option<String>, // "off" | "high" | "max"
}

/// Source of the auto-route decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoRouteSource {
    FlashRouter,
    Heuristic,
}

impl AutoRouteSource {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            AutoRouteSource::FlashRouter => "flash-router",
            AutoRouteSource::Heuristic => "heuristic",
        }
    }
}

/// Final auto-route decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoRouteSelection {
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub source: AutoRouteSource,
}

/// System prompt for the Flash router.
pub const AUTO_MODEL_ROUTER_SYSTEM_PROMPT: &str = "\
You are the codewhale auto-routing classifier. Return only compact JSON: \
{\"model\":\"deepseek-v4-flash|deepseek-v4-pro\",\"thinking\":\"off|high|max\"}. \
Use deepseek-v4-flash for trivial, conversational, status, or single-step work. \
Use deepseek-v4-pro for coding, debugging, release work, multi-step tasks, high-risk decisions, \
tool-heavy work, ambiguous requests, or anything that benefits from deeper reasoning. \
Use thinking off only for trivial no-tool answers, high for ordinary reasoning, and max for \
agentic, coding, multi-file, release, architecture, debugging, security, tool-heavy, or uncertain work.";

/// Cost-saving bias for the Flash router.
pub const AUTO_MODEL_ROUTER_COST_SAVING_ADDENDUM: &str = "\
\n\nCost-saving mode is ON. Prefer deepseek-v4-flash for any request that is \
not unmistakably agentic, multi-step, architecture/design, security review, \
debugging, or otherwise clearly out of Flash's capability. Resolve ambiguous \
cases in favour of deepseek-v4-flash, not deepseek-v4-pro.";

/// Run the auto-route decision: heuristic first, then Flash router if ambiguous.
pub async fn resolve_auto_route(
    api_key: &str,
    base_url: &str,
    latest_request: &str,
    recent_context: &str,
    cost_saving: bool,
) -> AutoRouteSelection {
    let heuristic =
        auto_model_heuristic_selection_with_bias(latest_request, "", cost_saving);
    if heuristic.confidence == AutoModelHeuristicConfidence::Decisive {
        return auto_route_from_heuristic(latest_request, heuristic);
    }

    match auto_route_flash_recommendation(
        api_key,
        base_url,
        latest_request,
        recent_context,
        cost_saving,
    )
    .await
    {
        Ok(Some(recommendation)) => AutoRouteSelection {
            model: recommendation.model,
            reasoning_effort: recommendation.reasoning_effort,
            source: AutoRouteSource::FlashRouter,
        },
        Ok(None) | Err(_) => auto_route_from_heuristic(latest_request, heuristic),
    }
}

fn auto_route_from_heuristic(
    _latest_request: &str,
    heuristic: AutoModelHeuristicSelection,
) -> AutoRouteSelection {
    // Default to high reasoning for heuristic decisions
    AutoRouteSelection {
        model: heuristic.model,
        reasoning_effort: Some("high".to_string()),
        source: AutoRouteSource::Heuristic,
    }
}

async fn auto_route_flash_recommendation(
    api_key: &str,
    base_url: &str,
    latest_request: &str,
    recent_context: &str,
    cost_saving: bool,
) -> Result<Option<AutoRouteRecommendation>, String> {
    let mut router_system = AUTO_MODEL_ROUTER_SYSTEM_PROMPT.to_string();
    if cost_saving {
        router_system.push_str(AUTO_MODEL_ROUTER_COST_SAVING_ADDENDUM);
    }

    let prompt = format!(
        "Session mode: agent\n\nRecent context:\n{}\n\nLatest user request:\n{}\n\nReturn JSON only.",
        if recent_context.trim().is_empty() {
            "No prior context."
        } else {
            recent_context
        },
        truncate_for_auto_router(latest_request, 4_000)
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let body = serde_json::json!({
        "model": "deepseek-v4-flash",
        "messages": [
            {"role": "system", "content": router_system},
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 96,
        "temperature": 0.0,
        "reasoning_effort": "off",
        "stream": false
    });

    let response = tokio::time::timeout(
        Duration::from_secs(4),
        client
            .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send(),
    )
    .await
    .map_err(|e| format!("Flash router timed out: {e}"))?
    .map_err(|e| format!("Flash router request failed: {e}"))?;

    let status = response.status();
    let text = response.text().await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Flash router returned HTTP {status}: {text}"));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse response JSON: {e}"))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("");

    Ok(parse_auto_route_recommendation(content))
}

/// Parse the Flash router's JSON-only response.
pub fn parse_auto_route_recommendation(raw: &str) -> Option<AutoRouteRecommendation> {
    let json = extract_first_json_object(raw)?;
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let model = value.get("model").and_then(serde_json::Value::as_str)?;
    let model = normalize_auto_route_model(model)?;
    let reasoning_effort = value
        .get("thinking")
        .or_else(|| value.get("reasoning_effort"))
        .or_else(|| value.get("effort"))
        .and_then(serde_json::Value::as_str)
        .and_then(parse_auto_route_reasoning_effort);

    Some(AutoRouteRecommendation {
        model: model.to_string(),
        reasoning_effort: reasoning_effort.map(|e| e.to_string()),
    })
}

fn extract_first_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (end >= start).then_some(&raw[start..=end])
}

fn normalize_auto_route_model(model: &str) -> Option<&'static str> {
    match model.trim().to_ascii_lowercase().as_str() {
        "deepseek-v4-pro" | "v4-pro" | "pro" => Some("deepseek-v4-pro"),
        "deepseek-v4-flash" | "v4-flash" | "flash" => Some("deepseek-v4-flash"),
        _ => None,
    }
}

fn parse_auto_route_reasoning_effort(effort: &str) -> Option<&'static str> {
    match effort.trim().to_ascii_lowercase().as_str() {
        "off" | "disabled" | "none" | "false" => Some("off"),
        "low" | "minimal" => Some("high"),
        "medium" | "mid" => Some("high"),
        "high" => Some("high"),
        "max" | "maximum" | "xhigh" => Some("max"),
        _ => None,
    }
}

fn truncate_for_auto_router(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_short_message_uses_flash() {
        let result = auto_model_heuristic("hello", "");
        assert_eq!(result, "deepseek-v4-flash");
    }

    #[test]
    fn heuristic_long_message_uses_pro() {
        let long = "a".repeat(600);
        let result = auto_model_heuristic(&long, "");
        assert_eq!(result, "deepseek-v4-pro");
    }

    #[test]
    fn heuristic_refactor_keyword_uses_pro() {
        let result = auto_model_heuristic("please refactor this module", "");
        assert_eq!(result, "deepseek-v4-pro");
    }

    #[test]
    fn heuristic_debug_keyword_uses_pro() {
        let result = auto_model_heuristic("debug the crash", "");
        assert_eq!(result, "deepseek-v4-pro");
    }

    #[test]
    fn parse_route_recommendation_pro_max() {
        let rec = parse_auto_route_recommendation(
            r#"{"model":"deepseek-v4-pro","thinking":"max"}"#
        );
        assert!(rec.is_some());
        let rec = rec.unwrap();
        assert_eq!(rec.model, "deepseek-v4-pro");
        assert_eq!(rec.reasoning_effort.as_deref(), Some("max"));
    }

    #[test]
    fn parse_route_recommendation_flash_off() {
        let rec = parse_auto_route_recommendation(
            r#"route: {"model":"flash","reasoning_effort":"off"}"#
        );
        assert!(rec.is_some());
        let rec = rec.unwrap();
        assert_eq!(rec.model, "deepseek-v4-flash");
        assert_eq!(rec.reasoning_effort.as_deref(), Some("off"));
    }

    #[test]
    fn parse_route_recommendation_invalid_model_fails() {
        let rec = parse_auto_route_recommendation(
            r#"{"model":"some-other-model","thinking":"max"}"#
        );
        assert!(rec.is_none());
    }
}
