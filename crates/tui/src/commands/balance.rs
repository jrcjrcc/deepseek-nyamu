//! Balance: query the active provider's account balance or credit status.
//!
//! Makes a live HTTP request to the provider's balance API when available.

use crate::config::ApiProvider;
use crate::tui::app::App;

use super::CommandResult;

/// Query provider account balance / credits via the provider's API.
pub fn balance(app: &App) -> CommandResult {
    let provider = app.api_provider;

    match provider {
        ApiProvider::Deepseek | ApiProvider::DeepseekCN => query_deepseek_balance(),
        _ => CommandResult::message(format!(
            "暂不支持查询 {} 的余额。请在 provider 控制台查看。",
            provider.display_name()
        )),
    }
}

/// Query the DeepSeek balance endpoint via a blocking HTTP request.
fn query_deepseek_balance() -> CommandResult {
    // Resolve the API key from config or environment.
    let config = crate::config::Config::load(None, None).unwrap_or_default();
    let api_key = config
        .api_key
        .clone()
        .filter(|k| !k.is_empty())
        .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok().filter(|k| !k.is_empty()));

    let api_key = match api_key {
        Some(k) => k,
        None => {
            return CommandResult::error(
                "未配置 API Key。请先设置 config.toml 中的 api_key 或 DEEPSEEK_API_KEY 环境变量。",
            )
        }
    };

    let base_url = config
        .base_url
        .as_deref()
        .unwrap_or("https://api.deepseek.com");
    let url = format!("{base_url}/user/balance");

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CommandResult::error(format!("无法创建 HTTP 客户端：{e}")),
    };

    let resp = match client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            return CommandResult::error(format!("余额查询请求失败：{e}"));
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return CommandResult::error(format!("余额查询失败 (HTTP {status}): {body}"));
    }

    match resp.json::<serde_json::Value>() {
        Ok(json) => {
            // DeepSeek balance API shapes.
            if let Some(balance) = json.get("balance").and_then(|v| v.as_str()) {
                return CommandResult::message(format!("💰 余额：{balance}"));
            }
            if let Some(infos) = json.get("balance_infos").and_then(|v| v.as_array()) {
                if let Some(info) = infos.first() {
                    let total = info
                        .get("total_balance")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let currency = info
                        .get("currency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("CNY");
                    return CommandResult::message(format!("💰 余额：{total} {currency}"));
                }
            }
            CommandResult::message(format!(
                "💰 余额 API 返回（格式未知）：\n{}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            ))
        }
        Err(e) => CommandResult::error(format!("解析余额返回失败：{e}")),
    }
}
