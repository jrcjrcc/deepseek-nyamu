//! 移动端控制面板页面（HTML 嵌入）
//!
//! 提供 DeepWhale 在移动端浏览器中的控制界面，
//! 包含连接状态、会话管理、消息发送等基础功能。
//! 通过 app-server 的 HTTP API 提供服务。
//! 当前为预留实现。

#[allow(dead_code)]
pub const MOBILE_PAGE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<title>DeepWhale Mobile</title>
<style>
  :root {
    --bg: #FDFBF7; --text: #3D362D; --muted: #A69B8A;
    --accent: #FFB5C2; --border: #D9D2C5; --success: #7BC8A4;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, system-ui, sans-serif; background: var(--bg); color: var(--text); padding: 16px; }
  h1 { font-size: 1.3rem; margin-bottom: 4px; }
  .sub { color: var(--muted); font-size: 0.85rem; margin-bottom: 16px; }
  .card { background: white; border: 1px solid var(--border); border-radius: 12px; padding: 16px; margin-bottom: 12px; }
  .card h2 { font-size: 1rem; margin-bottom: 8px; }
  .stat-row { display: flex; justify-content: space-between; padding: 4px 0; font-size: 0.9rem; }
  .stat-label { color: var(--muted); }
  .stat-value { font-weight: 600; }
  .btn { display: block; width: 100%; padding: 12px; border: 1px solid var(--border); border-radius: 10px; background: white; font-size: 0.95rem; color: var(--text); cursor: pointer; margin-bottom: 8px; text-align: center; text-decoration: none; }
  .btn:active { background: var(--accent); }
  .btn-primary { background: var(--accent); border-color: var(--accent); font-weight: 600; }
  #status { padding: 8px 12px; border-radius: 8px; font-size: 0.85rem; margin-bottom: 12px; }
  .connected { background: #e8f5e9; color: #2e7d32; }
  .disconnected { background: #fbe9e7; color: #c62828; }
  .thread-item { padding: 8px 0; border-bottom: 1px solid var(--border); }
  .thread-item:last-child { border-bottom: none; }
  .thread-title { font-weight: 500; }
  .thread-meta { font-size: 0.8rem; color: var(--muted); }
  input, textarea { width: 100%; padding: 10px; border: 1px solid var(--border); border-radius: 8px; font-size: 0.9rem; margin-bottom: 8px; font-family: inherit; resize: vertical; }
</style>
</head>
<body>
  <h1>🐋 DeepWhale</h1>
  <div class="sub">Mobile Control Panel</div>
  <div id="status" class="disconnected">Connecting...</div>

  <div class="card">
    <h2>Threads</h2>
    <div id="threads">Loading...</div>
  </div>

  <div class="card">
    <h2>Send Message</h2>
    <textarea id="message" rows="3" placeholder="Type a message..."></textarea>
    <button class="btn btn-primary" onclick="sendMessage()">Send</button>
  </div>

  <div class="card">
    <h2>Quick Actions</h2>
    <a class="btn" href="/healthz" target="_blank">Health Check</a>
    <a class="btn" href="/v1/threads/summary" target="_blank">Thread Summary (JSON)</a>
  </div>

<script>
const API = (() => {
  const params = new URLSearchParams(location.search);
  const token = params.get('token') || '';
  const base = '/v1';
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = 'Bearer ' + token;

  async function get(path) {
    const r = await fetch(base + path, { headers });
    if (!r.ok) throw new Error(r.status + ' ' + (await r.text()).slice(0, 100));
    return r.json();
  }
  async function post(path, body) {
    const r = await fetch(base + path, { method: 'POST', headers, body: JSON.stringify(body) });
    if (!r.ok) throw new Error(r.status + ' ' + (await r.text()).slice(0, 100));
    return r.json();
  }
  return { get, post };
})();

async function init() {
  const status = document.getElementById('status');
  try {
    const health = await fetch('/healthz').then(r => r.json());
    status.textContent = 'Connected — ' + (health.service || 'DeepWhale');
    status.className = 'connected';
  } catch {
    status.textContent = 'Disconnected';
    status.className = 'disconnected';
  }
  loadThreads();
}
init();

async function loadThreads() {
  const el = document.getElementById('threads');
  try {
    const data = await API.get('/threads/summary');
    if (!data.threads || data.threads.length === 0) {
      el.innerHTML = '<div class="thread-item">No threads yet.</div>';
      return;
    }
    el.innerHTML = data.threads.map(t => `
      <div class="thread-item">
        <div class="thread-title">${escapeHtml(t.title || t.id)}</div>
        <div class="thread-meta">${t.id.slice(0, 12)}... · ${t.message_count || 0} messages</div>
      </div>
    `).join('');
  } catch (e) {
    el.innerHTML = 'Error: ' + escapeHtml(String(e));
  }
}

async function sendMessage() {
  const msg = document.getElementById('message');
  const text = msg.value.trim();
  if (!text) return;
  msg.value = '';
  try {
    const result = await API.post('/thread/message', { input: text });
    document.getElementById('threads').innerHTML =
      '<div class="thread-item">Sent! Response: ' + escapeHtml(String(result.output || result.status || 'ok').slice(0, 200)) + '</div>';
    setTimeout(loadThreads, 1000);
  } catch (e) {
    alert('Error: ' + e);
  }
}

function escapeHtml(str) {
  const d = document.createElement('div');
  d.textContent = str;
  return d.innerHTML;
}
</script>
</body>
</html>"##;

/// 获取移动端页面 HTML 字符串
pub fn get_mobile_page_html() -> String {
    MOBILE_PAGE_HTML.to_string()
}
