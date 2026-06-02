import os, sys
os.chdir(os.path.dirname(os.path.dirname(__file__)))
with open('crates/tui/src/tui/history.rs', encoding='utf-8') as f:
    t = f.read()
# Add #[allow(dead_code)] before static COLOR_DEPTH
t = t.replace(
    'static COLOR_DEPTH: std::sync::OnceLock<palette::ColorDepth> = std::sync::OnceLock::new();',
    '#[allow(dead_code)]\nstatic COLOR_DEPTH: std::sync::OnceLock<palette::ColorDepth> = std::sync::OnceLock::new();'
)
# Add #[allow(dead_code)] before fn cached_color_depth
t = t.replace(
    'fn cached_color_depth() -> palette::ColorDepth {',
    '#[allow(dead_code)]\nfn cached_color_depth() -> palette::ColorDepth {'
)
with open('crates/tui/src/tui/history.rs', 'w', encoding='utf-8') as f:
    f.write(t)
print('OK: replaced', t.count('#[allow(dead_code)]'))
