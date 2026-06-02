# Delete legacy sub-agent tool structs that were superseded by agent_eval/agent_close
import os

os.chdir(os.path.dirname(os.path.dirname(__file__)))

with open('crates/tui/src/tools/subagent/mod.rs', encoding='utf-8') as f:
    lines = f.readlines()

# Find markers (0-based line numbers)
r_close = None
for i, l in enumerate(lines):
    if '/// Tool to close a running sub-agent' in l and r_close is None:
        r_close = i
    if '/// Tool to resume an existing sub-agent' in l:
        r_resume = i
    if '// === Sub-agent Execution ===' in l:
        r_exec = i

# r_close = line where real AgentCloseTool starts
# Delete from line 2547 (result tool) to before r_close (keep close tool)
del lines[2547:r_close]

# New positions after first deletion
r_resume = None
r_exec = None
for i, l in enumerate(lines):
    if '/// Tool to resume an existing sub-agent' in l:
        r_resume = i
    if '// === Sub-agent Execution ===' in l:
        r_exec = i

# Delete from resume tool to before exec section
del lines[r_resume:r_exec]

with open('crates/tui/src/tools/subagent/mod.rs', 'w', encoding='utf-8') as f:
    f.writelines(lines)

print(f'Done. File now has {len(lines)} lines')
