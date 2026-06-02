---
name: academic-paper-pipeline
description: "统一论文写作全流程技能。涵盖论文写作（IMRAD结构/ML会议/系统会议）、 论文审稿与质量审计（deep-review/quick-audit/gate/re-audit/polish 5模式）、 系统性文献综述、引用管理（BibTeX/APA/MLA/Chicago/Vancouver/IEEE）、 引用验证防伪、开放获取检索、LaTeX排版与编译、论文图表绘制、 中英文写作AI痕迹消除、科研基金申请、科学假设生成与实验设计。 用户说\"写论文\"、\"审稿\"、\"文献综述\"、\"引用管理\"、\"LaTeX\"、\"论文图表\"、 \"去AI味\"、\"基金申请\"、\"假设生成\"、\"paper review\"、\"literature review\"、 \"citation\"、\"latex\"、\"写作\"、\"降重\"、\"降AIGC\"、\"图表\"、\"plotting\"等时触发。"
allowed-tools: Read Write Edit Bash
tags: [Academic Writing, Scientific Writing, Paper Writing, Paper Review, Literature Review, Citation Management, LaTeX, Academic Plotting, Writing Humanizer, Research Grants, Hypothesis Generation, NeurIPS, ICML, ICLR, ACL, AAAI, COLM, OSDI, SOSP, ASPLOS, NSDI, EuroSys, IMRAD]
license: MIT License (aggregated from multiple sources)
metadata:
  version: "1.0.0"
  aggregated-from:
    - paper-writing
    - academic-paper
    - paper-review v4.2
    - latex-document
    - academic-citation-manager
    - academic-plotting
    - literature-review
    - writing-humanizer
    - scientific-critical-thinking
    - citation-verification
    - finding-open-access-papers
    - hypothesis-generation
    - research-grants
    - research-writing (cc-polymath)
    - research-synthesis (cc-polymath)
---

# Academic Paper Pipeline — 统一论文写作全流程

本技能将 15 个独立的论文写作相关 skill 合并为一个统一管线。根据你的指令自动路由到对应功能模块。

---

## 快速路由

| 你说 | 路由到 |
|------|--------|
| "写一篇关于...的论文"、"帮我写论文"、"论文大纲"、"NeurIPS/ICML/OSDI 论文" | `modules/00-core-writing/` — 论文写作 |
| "帮我审稿"、"检查论文质量"、"gate 这篇论文"、"模拟审稿" | `modules/01-review/` — 论文审稿与审计 |
| "做文献综述"、"systematic review"、"综合文献" | `modules/02-literature/` — 文献综述 |
| "管理引用"、"BibTeX"、"APA 格式"、"验证引用"、"找开放获取" | `modules/03-citations/` — 引用管理 |
| "排版论文"、"LaTeX 编译"、"PDF 操作"、"简历/PPT/海报" | `modules/04-latex/` — LaTeX 排版 |
| "画论文图表"、"Figure 1"、"architecture diagram"、"数据可视化" | `modules/05-plotting/` — 论文图表 |
| "去 AI 味"、"降 AIGC 检测率"、"humanize"、"降重" | `modules/06-humanizer/` — 写作人工化 |
| "写基金申请书"、"NSF/NIH  proposal"、"科研资助申请" | `modules/07-grants/` — 基金申请 |
| "生成研究假设"、"实验设计"、"提出假设" | `modules/08-hypothesis/` — 假设生成 |

---

## 各模块简介

### 00 — 论文写作 (Core Writing)
合并自 `paper-writing`、`academic-paper`、`research-writing`。涵盖：
- **通用核心**：IMRAD 结构、各章节写作指导、引用格式、写作原则
- **ML 会议论文**：NeurIPS/ICML/ICLR/ACL/AAAI/COLM 专用指导、LaTeX 模板、审稿人视角
- **系统会议论文**：OSDI/SOSP/ASPLOS/NSDI/EuroSys 12页管线蓝图、段落级 blueprint
- **5 阶段管线**：平台分析 → 理论框架 → 大纲优化 → 逐章写作(质量门控≥16/20) → 质量控制(≥56/70)
- **报告规范**：CONSORT/STROBE/PRISMA/STARD/TRIPOD

### 01 — 论文审稿与审计 (Paper Review)
完整保留 `paper-review` v4.2 全部功能，并整合 `scientific-critical-thinking` 方法论评估。
- **5 种模式**：quick-audit / deep-review / gate / re-audit / polish
- **5 角色委员会**：Editor + Theory + Literature + Methodology + Logic
- **16 维度问题分类** + Python 自动化审计脚本
- **科研方法论评估**：GRADE / Cochrane ROB 框架

### 02 — 文献综述 (Literature Review)
合并自 `literature-review`、`research-synthesis`。
- 系统性文献综述（PubMed/arXiv/bioRxiv/Semantic Scholar）
- 元分析、范围综述、叙事综述
- 信息综合与知识整合框架
- PRISMA 流程图、主题综合图

### 03 — 引用管理 (Citation Management)
合并自 `academic-citation-manager`、`citation-verification`、`finding-open-access-papers`。
- 格式管理：BibTeX/APA/MLA/Chicago/Vancouver/IEEE 互转
- 引用验证：防 AI 伪造引用（~40% 错误率），Google Scholar 核实
- 开放获取：Unpaywall API 一键找免费全文

### 04 — LaTeX 排版 (LaTeX Document)
完整保留 `latex-document-skill-main` 全部功能。
- 50+ 文档类型支持：论文/学位论文/简历/Beamer/海报/书籍/试卷/信函
- 30+ 脚本：编译/图表/PDF操作/引文提取/字数统计/查重
- 28 份参考指南 + 60+ 示例 PNG + 模板

### 05 — 论文图表 (Academic Plotting)
精简自 `academic-plotting`。
- Gemini 生成架构图/系统流程图
- matplotlib/seaborn 生成数据图表（折线/柱状/散点/热力图）
- 自动根据数据特征选图类型

### 06 — 写作人工化 (Writing Humanizer)
完整保留 `writing-humanizer`。
- 英文 AI 模式检测与重写（481 条规则）
- 中文学术降 AIGC 检测率（17 类模式）
- 标点符号审查（破折号零容忍）

### 07 — 基金申请 (Research Grants)
精简自 `research-grants`。
- NSF/NIH/DOE/DARPA/NSTC 基金申请
- 机构特定格式、评审标准、预算编制

### 08 — 假设生成 (Hypothesis Generation)
精简自 `hypothesis-generation`。
- 科学假设公式化生成
- 实验设计、竞争性解释、预测开发

---

## 论文写作全流程

```
选题确定 ──→ 文献综述 ──→ 假设生成 ──→ 论文写作 ──→ 图表绘制 ──→ LaTeX排版 ──→ 引用管理 ──→ 审稿自检 ──→ 降AI味 ──→ 投稿
  │             │             │             │             │             │             │             │             │
  └─ 07-grants  └─ 02-lit     └─ 08-hypo    └─ 00-write   └─ 05-plot    └─ 04-latex   └─ 03-cite    └─ 01-review  └─ 06-human
  (基金申请)     (文献综述)    (假设生成)    (论文写作)    (论文图表)    (LaTeX排版)   (引用管理)    (论文审稿)    (写作人工化)
```

每个阶段可按需独立使用，也可顺序执行完整管线。

---

## 目录结构

```
academic-paper-pipeline/
├── SKILL.md                       ← 本文件：主入口路由
├── modules/
│   ├── 00-core-writing/           ← 论文写作
│   ├── 01-review/                 ← 论文审稿（含脚本/agent/参考）
│   ├── 02-literature/             ← 文献综述
│   ├── 03-citations/              ← 引用管理
│   ├── 04-latex/                  ← LaTeX 排版（含脚本/模板/示例）
│   ├── 05-plotting/               ← 论文图表
│   ├── 06-humanizer/              ← 写作人工化
│   ├── 07-grants/                 ← 基金申请
│   └── 08-hypothesis/             ← 假设生成
```

