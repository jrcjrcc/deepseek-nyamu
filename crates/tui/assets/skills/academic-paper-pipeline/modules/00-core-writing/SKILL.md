---
name: academic-writing-core
description: 统一学术论文写作全流程核心技能。涵盖科学论文写作（IMRAD结构）、ML/AI会议论文（NeurIPS/ICML/ICLR/ACL/AAAI/COLM）、系统会议论文（OSDI/SOSP/ASPLOS/NSDI/EuroSys）、报告规范与图表（CONSORT/STROBE/PRISMA）、引用管理（APA/AMA/Vancouver/Chicago/IEEE）、Python代码工具（IMRAD结构/统计报告/引用格式化/论证结构/审稿回应）、以及5阶段论文写作管线（从平台分析到提交准备）。当用户说"写论文"、"写学术论文"、"写科学论文"、"写 ML 论文"、"写系统论文"、"写期刊论文"、"NeurIPS 论文"、"OSDI 论文"、"规划论文"、"论文大纲"、"帮我规划一篇论文"、"设计论文结构"、"写一篇关于...的论文"、"论文审稿回应"等时触发。
allowed-tools: Read Write Edit Bash
license: MIT License (aggregated from multiple sources)
tags: [Academic Writing, Scientific Writing, ML Papers, Systems Papers, NeurIPS, ICML, ICLR, ACL, AAAI, COLM, OSDI, SOSP, ASPLOS, NSDI, EuroSys, LaTeX, IMRAD, Citations, CONSORT, STROBE, PRISMA, Peer Review, Literature Review, Research Pipeline, Python Tools, APA, AMA, Vancouver, Chicago, IEEE]
metadata:
    aggregated-from:
      - scientific-writing v1.0 (K-Dense Inc.)
      - ml-paper-writing v1.2.0 (Orchestra Research)
      - systems-paper-writing v1.1.0 (Orchestra Research)
      - academic-paper-pipeline (5-Phase pipeline, CoPaper.AI)
      - research-writing (code tools + peer review response)
---

<!--
  Portions of this skill include content aggregated from:
  - academic-paper-skills (MIT License) — collected by CoPaper.AI (https://copaper.ai)
  - research-writing skill — writing code tools and peer review response framework
-->

# Paper Writing: Unified Academic Writing Core

This skill combines four complementary bodies of academic writing guidance into a single unified skill:

| Sub-Skill | Scope | Target Venues |
|-----------|-------|---------------|
| **通用核心 (General Core)** | IMRAD structure, section writing, citations, writing principles, outline-to-prose workflow | Scientific journals, any academic paper |
| **ML 会议论文 (ML Conference Papers)** | ML-specific structure, citation verification, LaTeX templates, reviewer perspective | NeurIPS, ICML, ICLR, ACL, AAAI, COLM |
| **系统会议论文 (Systems Conference Papers)** | Paragraph-level blueprints, page allocation, writing patterns, systems-specific checklists | OSDI, SOSP, ASPLOS, NSDI, EuroSys |
| **报告规范与图表 (Reporting Guidelines & Figures)** | CONSORT/STROBE/PRISMA/STARD/TRIPOD, figure/table design, visual best practices | All study types |
| **代码工具与审稿回应 (Code Tools & Peer Review)** | Python implementations of IMRAD structure, statistical reporting, citation formatting, argument structure, and peer review response | All academic writing |
| **5阶段写作管线 (5-Phase Pipeline)** | Platform analysis → literature search → outline optimization → systematic writing → quality control | Philosopy/Interdisciplinary papers (PhilArchive, arXiv) |

---

## Overview

**This is the core skill for academic paper writing**—combining AI-driven deep research with well-formatted written outputs. Every document produced should be backed by comprehensive literature search and verified citations.

Scientific writing is a process for communicating research with precision and clarity. Write manuscripts using appropriate structure for your venue, with proper citations, figures/tables, and reporting guidelines.

**Critical Principle: Always write in full paragraphs with flowing prose. Never submit bullet points in the final manuscript.** Use a two-stage process: first create section outlines with key points using research-lookup, then convert those outlines into complete paragraphs.

### When to Use This Skill

**Writing & Revision:**
- Writing or revising any section of a scientific manuscript
- Drafting ML/AI papers for NeurIPS, ICML, ICLR, ACL, AAAI, COLM
- Writing systems papers for OSDI, SOSP, ASPLOS, NSDI, EuroSys
- Structuring a paper using IMRAD or venue-specific formats
- Formatting citations and references in specific styles
- Creating, formatting, or improving figures and tables
- Applying study-specific reporting guidelines
- Drafting abstracts meeting journal/conference requirements
- Preparing manuscripts for submission to specific venues
- Improving writing clarity, conciseness, and precision
- Addressing reviewer comments and revising manuscripts
- Converting between conference formats for resubmission

**Planning & Research:**
- Design a research paper from initial idea to structured outline
- Identify a suitable publication platform
- Conduct systematic literature search
- Identify research gaps with evidence
- Assess originality of your research idea
- Predict potential impact

**Quality Assurance:**
- Validate each chapter before proceeding
- Check cross-chapter coherence
- Perform final quality assessment
- Verify submission readiness
- Prepare submission package

**Triggers:**
- "写论文" / "write a paper on [topic]"
- "规划论文" / "plan a paper on [topic]"
- "论文大纲" / "help me design a paper about [subject]"
- "写学术论文" / "compose a full manuscript"
- "帮我规划一篇论文" / "identify research gaps in [field]"
- "设计论文结构" / "is this idea original?"
- "What platform should I submit to?"
- "I have an outline, help me write the paper"
- "论文审稿回应" / "respond to reviewer comments"

---

# Part I: 通用核心 (General Core)

Derived from scientific-writing skill. This section covers the universal principles applicable to all academic writing.

## IMRAD Format

Guide papers through the standard Introduction, Methods, Results, And Discussion structure used across most scientific disciplines:

- **Introduction**: Establish research context, identify gaps, state objectives
- **Methods**: Detail study design, populations, procedures, and analysis approaches
- **Results**: Present findings objectively without interpretation
- **Discussion**: Interpret results, acknowledge limitations, propose future directions

**Alternative Structures**: Review articles (narrative, systematic, scoping), case reports and case series, meta-analyses and pooled analyses, theoretical/modeling papers, methods papers and protocols.

## Section-Specific Writing Guidance

### Abstract Composition

Craft concise, standalone summaries (100-250 words) that capture the paper's purpose, methods, results, and conclusions. Support both structured abstracts (with labeled sections) and unstructured single-paragraph formats.

**Abstract Format Rule:**
- NEVER use labeled sections (Background:, Methods:, Results:, Conclusions:)
- ALWAYS write as flowing paragraph(s) with natural transitions
- Exception: Only use structured format if journal explicitly requires it in author guidelines

### Introduction Development

Build compelling introductions that:
- Establish the research problem's importance
- Review relevant literature systematically
- Identify knowledge gaps or controversies
- State clear research questions or hypotheses
- Explain the study's novelty and significance

### Methods Documentation

Ensure reproducibility through:
- Detailed participant/sample descriptions
- Clear procedural documentation
- Statistical methods with justification
- Equipment and materials specifications
- Ethical approval and consent statements

### Results Presentation

Present findings with:
- Logical flow from primary to secondary outcomes
- Integration with figures and tables
- Statistical significance with effect sizes
- Objective reporting without interpretation

### Discussion Construction

Synthesize findings by:
- Relating results to research questions
- Comparing with existing literature
- Acknowledging limitations honestly
- Proposing mechanistic explanations
- Suggesting practical implications and future research

## Citation and Reference Management

### Major Citation Styles

- **AMA (American Medical Association)**: Numbered superscript citations, common in medicine
- **Vancouver**: Numbered citations in square brackets, biomedical standard
- **APA (American Psychological Association)**: Author-date in-text citations, common in social sciences
- **Chicago**: Notes-bibliography or author-date, humanities and sciences
- **IEEE**: Numbered square brackets, engineering and computer science

### Best Practices

- Cite primary sources when possible
- Include recent literature (last 5-10 years for active fields)
- Balance citation distribution across introduction and discussion
- Verify all citations against original sources
- Use reference management software (Zotero, Mendeley, EndNote)

## Writing Principles and Style

### Clarity
- Use precise, unambiguous language
- Define technical terms and abbreviations at first use
- Maintain logical flow within and between paragraphs
- Use active voice when appropriate for clarity

### Conciseness
- Eliminate redundant words and phrases
- Favor shorter sentences (15-20 words average)
- Remove unnecessary qualifiers
- Respect word limits strictly

### Accuracy
- Report exact values with appropriate precision
- Use consistent terminology throughout
- Distinguish between observations and interpretations
- Acknowledge uncertainty appropriately

### Objectivity
- Present results without bias
- Avoid overstating findings or implications
- Acknowledge conflicting evidence
- Maintain professional, neutral tone

## Writing Process: From Outline to Full Paragraphs

**CRITICAL: Always write in full paragraphs, never submit bullet points in scientific papers.**

### Stage 1: Create Section Outlines with Key Points

When starting a new section:
1. Use research-lookup to gather relevant literature and data
2. Create a structured outline with bullet points marking:
   - Main arguments or findings to present
   - Key studies to cite
   - Data points and statistics to include
   - Logical flow and organization
3. These bullet points serve as scaffolding—they are NOT the final manuscript

**Example outline (Introduction section):**
```
- Background: AI in drug discovery gaining traction
  * Cite recent reviews (Smith 2023, Jones 2024)
  * Traditional methods are slow and expensive
- Gap: Limited application to rare diseases
  * Only 2 prior studies (Lee 2022, Chen 2023)
  * Small datasets remain a challenge
- Our approach: Transfer learning from common diseases
  * Novel architecture combining X and Y
- Study objectives: Validate on 3 rare disease datasets
```

### Stage 2: Convert Key Points to Full Paragraphs

Once the outline is complete, expand each bullet point into proper prose:

1. **Transform bullet points into complete sentences** with subjects, verbs, and objects
2. **Add transitions** between sentences and ideas (however, moreover, in contrast, subsequently)
3. **Integrate citations naturally** within sentences, not as lists
4. **Expand with context and explanation** that bullet points omit
5. **Ensure logical flow** from one sentence to the next within each paragraph
6. **Vary sentence structure** to maintain reader engagement

**Key Differences Between Outlines and Final Text:**

| Outline (Planning Stage) | Final Manuscript |
|--------------------------|------------------|
| Bullet points and fragments | Complete sentences and paragraphs |
| Telegraphic notes | Full explanations with context |
| List of citations | Citations integrated into prose |
| Abbreviated ideas | Developed arguments with transitions |
| For your eyes only | For publication and peer review |

**Common Mistakes to Avoid:**

- Never leave bullet points in the final manuscript
- Never submit lists where paragraphs should be
- Don't use numbered or bulleted lists in Results or Discussion sections (except for specific cases like study hypotheses or inclusion criteria)
- Don't write sentence fragments or incomplete thoughts
- Do use occasional lists only in Methods (e.g., inclusion/exclusion criteria, materials lists)
- Do ensure every section flows as connected prose
- Do read paragraphs aloud to check for natural flow

**When Lists ARE Acceptable (Limited Cases):**

Lists may appear in scientific papers only in specific contexts:
- **Methods**: Inclusion/exclusion criteria, materials and reagents, participant characteristics
- **Supplementary Materials**: Extended protocols, equipment lists, detailed parameters
- **Never in**: Abstract, Introduction, Results, Discussion, Conclusions

## Professional Report Formatting (Non-Journal Documents)

For research reports, technical reports, white papers, and other professional documents that are NOT journal manuscripts, use the `scientific_report.sty` LaTeX style package.

**When to Use Professional Report Formatting:**
- Research reports and technical reports
- White papers and policy briefs
- Grant reports and progress reports
- Industry reports and technical documentation
- Internal research summaries
- Feasibility studies and project deliverables

**When NOT to Use (Use Venue-Specific Formatting Instead):**
- Journal manuscripts → Use venue-specific templates
- Conference papers → Use venue-specific templates
- Academic theses → Use institutional templates

**The `scientific_report.sty` Style Package Provides:**

| Feature | Description |
|---------|-------------|
| Typography | Helvetica font family for modern, professional appearance |
| Color Scheme | Professional blues, greens, and accent colors |
| Box Environments | Colored boxes for key findings, methods, recommendations, limitations |
| Tables | Alternating row colors, professional headers |
| Figures | Consistent caption formatting |
| Scientific Commands | Shortcuts for p-values, effect sizes, confidence intervals |

**Box Environments for Content Organization:**

```latex
% Key findings (blue) - for major discoveries
\begin{keyfindings}[Title]
Content with key findings and statistics.
\end{keyfindings}

% Methodology (green) - for methods highlights
\begin{methodology}[Study Design]
Description of methods and procedures.
\end{methodology}

% Recommendations (purple) - for action items
\begin{recommendations}[Clinical Implications]
\begin{enumerate}
    \item Specific recommendation 1
    \item Specific recommendation 2
\end{enumerate}
\end{recommendations}

% Limitations (orange) - for caveats and cautions
\begin{limitations}[Study Limitations]
Description of limitations and their implications.
\end{limitations}
```

**Professional Table Formatting:**

```latex
\begin{table}[htbp]
\centering
\caption{Table caption.}
\begin{tabular}{lcc}
\toprule
Variable & Group A & Group B \\
\midrule
Metric 1 & Value & Value \\
Metric 2 & Value & Value \\
\bottomrule
\end{tabular}
\end{table}
```

---

# Part II: ML 会议论文 (ML Conference Papers)

Derived from ml-paper-writing skill. Expert-level guidance for writing publication-ready papers targeting **NeurIPS, ICML, ICLR, ACL, AAAI, COLM**. Combines writing philosophy from top researchers (Nanda, Farquhar, Karpathy, Lipton, Steinhardt) with practical tools: LaTeX templates, citation verification APIs, and conference checklists.

## Core Philosophy: Collaborative Writing

Paper writing is collaborative, but the assistant should be proactive in delivering drafts. The typical workflow starts with a research repository containing code, results, and experimental artifacts. The role is to:

1. **Understand the project** by exploring the repo, results, and existing documentation
2. **Deliver a complete first draft** when confident about the contribution
3. **Search literature** using web search and APIs to find relevant citations
4. **Refine through feedback cycles** when the scientist provides input
5. **Ask for clarification** only when genuinely uncertain about key decisions

**Key Principle**: Be proactive. If the repo and results are clear, deliver a full draft. Don't block waiting for feedback on every section—scientists are busy. Produce something concrete they can react to, then iterate based on their response.

## The Narrative Principle

The single most critical insight: Your paper is not a collection of experiments—it's a story with one clear contribution supported by evidence.

Every successful ML paper centers on what Neel Nanda calls "the narrative": a short, rigorous, evidence-based technical story with a takeaway readers care about.

**Three Pillars (must be crystal clear by end of introduction):**

| Pillar | Description | Example |
|--------|-------------|---------|
| **The What** | 1-3 specific novel claims within cohesive theme | "We prove that X achieves Y under condition Z" |
| **The Why** | Rigorous empirical evidence supporting claims | Strong baselines, experiments distinguishing hypotheses |
| **The So What** | Why readers should care | Connection to recognized community problems |

**If you cannot state your contribution in one sentence, you don't yet have a paper.**

## Paper Structure Workflow

### Workflow: Writing a Complete Paper (Iterative)

```
Paper Writing Progress:
- [ ] Step 1: Define the one-sentence contribution (with scientist)
- [ ] Step 2: Draft Figure 1 → get feedback → revise
- [ ] Step 3: Draft abstract → get feedback → revise
- [ ] Step 4: Draft introduction → get feedback → revise
- [ ] Step 5: Draft methods → get feedback → revise
- [ ] Step 6: Draft experiments → get feedback → revise
- [ ] Step 7: Draft related work → get feedback → revise
- [ ] Step 8: Draft limitations → get feedback → revise
- [ ] Step 9: Complete paper checklist (required)
- [ ] Step 10: Final review cycle and submission
```

### Step 1: Define the One-Sentence Contribution

This step requires explicit confirmation from the scientist. Before writing anything, articulate and verify:
- What is the single thing your paper contributes?
- What was not obvious or present before your work?

### Step 2: Draft Figure 1

Figure 1 deserves special attention—many readers skip directly to it.
- Convey core idea, approach, or most compelling result
- Use vector graphics (PDF/EPS for plots)
- Write captions that stand alone without main text
- Ensure readability in black-and-white

### Step 3: Write Abstract (5-Sentence Formula, from Sebastian Farquhar)

```
1. What you achieved: "We introduce...", "We prove...", "We demonstrate..."
2. Why this is hard and important
3. How you do it (with specialist keywords for discoverability)
4. What evidence you have
5. Your most remarkable number/result
```

Delete generic openings like "Large language models have achieved remarkable success..."

### Step 4: Write Introduction (1-1.5 pages max)

Must include:
- 2-4 bullet contribution list (max 1-2 lines each in two-column format)
- Clear problem statement
- Brief approach overview
- Methods should start by page 2-3 maximum

### Step 5: Methods Section

Enable reimplementation:
- Conceptual outline or pseudocode
- All hyperparameters listed
- Architectural details sufficient for reproduction
- Present final design decisions; ablations go in experiments

### Step 6: Experiments Section

For each experiment, explicitly state:
- What claim it supports
- How it connects to main contribution
- Experimental setting (details in appendix)
- What to observe: "the blue line shows X, which demonstrates Y"

Requirements:
- Error bars with methodology (standard deviation vs standard error)
- Hyperparameter search ranges
- Compute infrastructure (GPU type, total hours)
- Seed-setting methods

### Step 7: Related Work

Organize methodologically, not paper-by-paper:

**Good:** "One line of work uses Floogledoodle's assumption [refs] whereas we use Doobersnoddle's assumption because..."

**Bad:** "Snap et al. introduced X while Crackle et al. introduced Y."

Cite generously—reviewers likely authored relevant papers.

### Step 8: Limitations Section (REQUIRED)

All major conferences require this. Counter-intuitively, honesty helps:
- Reviewers are instructed not to penalize honest limitation acknowledgment
- Pre-empt criticisms by identifying weaknesses first
- Explain why limitations don't undermine core claims

## Writing Philosophy for Top ML Conferences

> "A paper is a short, rigorous, evidence-based technical story with a takeaway readers care about." — Neel Nanda

### Key Sources

| Source | Key Contribution |
|--------|-----------------|
| **Neel Nanda** (Google DeepMind) | The Narrative Principle, What/Why/So What framework |
| **Sebastian Farquhar** (DeepMind) | 5-sentence abstract formula |
| **Gopen & Swan** | 7 principles of reader expectations |
| **Zachary Lipton** | Word choice, eliminating hedging |
| **Jacob Steinhardt** (UC Berkeley) | Precision, consistent terminology |
| **Ethan Perez** (Anthropic) | Micro-level clarity tips |
| **Andrej Karpathy** | Single contribution focus |

### Time Allocation (From Neel Nanda)

Spend approximately **equal time** on each of:
1. The abstract
2. The introduction
3. The figures
4. Everything else combined

**Why?** Most reviewers form judgments before reaching your methods. Readers encounter your paper as: **title → abstract → introduction → figures → maybe the rest.**

### Sentence-Level Clarity (Gopen & Swan's 7 Principles)

| Principle | Rule | Example |
|-----------|------|---------|
| **Subject-verb proximity** | Keep subject and verb close | "The model, which was trained on..., achieves" → "The model achieves... after training on..." |
| **Stress position** | Place emphasis at sentence ends | "Accuracy improves by 15% when using attention" → "When using attention, accuracy improves by **15%**" |
| **Topic position** | Put context first, new info after | "Given these constraints, we propose..." |
| **Old before new** | Familiar info → unfamiliar info | Link backward, then introduce new |
| **One unit, one function** | Each paragraph makes one point | Split multi-point paragraphs |
| **Action in verb** | Use verbs, not nominalizations | "We performed an analysis" → "We analyzed" |
| **Context before new** | Set stage before presenting | Explain before showing equation |

### Micro-Level Tips (Ethan Perez)

- **Minimize pronouns**: "This shows..." → "This result shows..."
- **Verbs early**: Position verbs near sentence start
- **Unfold apostrophes**: "X's Y" → "The Y of X" (when awkward)
- **Delete filler words**: "actually," "a bit," "very," "really," "basically," "quite," "essentially"

### Word Choice (Zachary Lipton)

- **Be specific**: "performance" → "accuracy" or "latency" (say what you mean)
- **Eliminate hedging**: Drop "may" and "can" unless genuinely uncertain
- **Avoid incremental vocabulary**: "combine," "modify," "expand" → "develop," "propose," "introduce"
- **Delete intensifiers**: "provides *very* tight approximation" → "provides tight approximation"

### Precision Over Brevity (Jacob Steinhardt)

- **Consistent terminology**: Different terms for same concept creates confusion. Pick one and stick with it.
- **State assumptions formally**: Before theorems, list all assumptions explicitly
- **Intuition + rigor**: Provide intuitive explanations alongside formal proofs

### What Reviewers Actually Read

| Paper Section | % Reviewers Who Read | Implication |
|---------------|---------------------|-------------|
| Abstract | 100% | Must be perfect |
| Introduction | 90%+ (skimmed) | Front-load contribution |
| Figures | Examined before methods | Figure 1 is critical |
| Methods | Only if interested | Don't bury the lede |
| Appendix | Rarely | Put only supplementary details |

## Reviewer Evaluation Criteria

Reviewers assess papers on four dimensions:

| Criterion | What Reviewers Look For |
|-----------|------------------------|
| **Quality** | Technical soundness, well-supported claims |
| **Clarity** | Clear writing, reproducible by experts |
| **Significance** | Community impact, advances understanding |
| **Originality** | New insights (doesn't require new method) |

**Scoring (NeurIPS 6-point scale):**
- 6: Strong Accept - Groundbreaking, flawless
- 5: Accept - Technically solid, high impact
- 4: Borderline Accept - Solid, limited evaluation
- 3: Borderline Reject - Solid but weaknesses outweigh
- 2: Reject - Technical flaws
- 1: Strong Reject - Known results or ethics issues

## CRITICAL: Never Hallucinate Citations

**This is the most important rule in academic writing with AI assistance.**

### The Problem
AI-generated citations have a **~40% error rate**. Hallucinated references—papers that don't exist, wrong authors, incorrect years, fabricated DOIs—are a serious form of academic misconduct that can result in desk rejection or retraction.

### The Rule
**NEVER generate BibTeX entries from memory. ALWAYS fetch programmatically.**

| Action | Correct | Wrong |
|--------|---------|-------|
| Adding a citation | Search API → verify → fetch BibTeX | Write BibTeX from memory |
| Uncertain about a paper | Mark as `[CITATION NEEDED]` | Guess the reference |
| Can't find exact paper | Note: "placeholder - verify" | Invent similar-sounding paper |

### Citation Verification Workflow

```
Citation Verification (MANDATORY for every citation):
- [ ] Step 1: Search using Exa MCP or Semantic Scholar API
- [ ] Step 2: Verify paper exists in 2+ sources (Semantic Scholar + arXiv/CrossRef)
- [ ] Step 3: Retrieve BibTeX via DOI (programmatically, not from memory)
- [ ] Step 4: Verify the claim you're citing actually appears in the paper
- [ ] Step 5: Add verified BibTeX to bibliography
- [ ] Step 6: If ANY step fails → mark as placeholder, inform scientist
```

### When You Can't Verify a Citation

```latex
% EXPLICIT PLACEHOLDER - requires human verification
\cite{PLACEHOLDER_author2024_verify_this}  % TODO: Verify this citation exists
```

**Always tell the scientist**: "I've marked [X] citations as placeholders that need verification. I could not confirm these papers exist."

## Conference Requirements Quick Reference

### ML/AI Conferences

| Conference | Page Limit | Extra for Camera-Ready | Key Requirement |
|------------|------------|------------------------|------------------|
| **NeurIPS 2025** | 9 pages | +0 | Mandatory checklist, lay summary for accepted |
| **ICML 2026** | 8 pages | +1 | Broader Impact Statement required |
| **ICLR 2026** | 9 pages | +1 | LLM disclosure required, reciprocal reviewing |
| **ACL 2025** | 8 pages (long) | varies | Limitations section mandatory |
| **AAAI 2026** | 7 pages | +1 | Strict style file adherence |
| **COLM 2025** | 9 pages | +1 | Focus on language models |

**Universal Requirements:**
- Double-blind review (anonymize submissions)
- References don't count toward page limit
- Appendices unlimited but reviewers not required to read
- LaTeX required for all venues

## Using LaTeX Templates Properly

### Workflow: Starting a New Paper from Template

Always copy the entire template directory first, then write within it.

```
Template Setup Checklist:
- [ ] Step 1: Copy entire template directory to new project
- [ ] Step 2: Verify template compiles as-is (before any changes)
- [ ] Step 3: Read the template's example content to understand structure
- [ ] Step 4: Replace example content section by section
- [ ] Step 5: Keep template comments/examples as reference until done
- [ ] Step 6: Clean up template artifacts only at the end
```

**Template Pitfalls to Avoid:**

| Pitfall | Problem | Solution |
|---------|---------|----------|
| Copying only `main.tex` | Missing `.sty`, won't compile | Copy entire directory |
| Modifying `.sty` files | Breaks conference formatting | Never edit style files |
| Adding random packages | Conflicts, breaks template | Only add if necessary |
| Deleting template content too early | Lose formatting reference | Keep as comments until done |
| Not compiling frequently | Errors accumulate | Compile after each section |

### Quick Template Reference

| Conference | Main File | Key Style File |
|------------|-----------|----------------|
| NeurIPS 2025 | `main.tex` | `neurips.sty` |
| ICML 2026 | `example_paper.tex` | `icml2026.sty` |
| ICLR 2026 | `iclr2026_conference.tex` | `iclr2026_conference.sty` |
| ACL | `acl_latex.tex` | `acl.sty` |
| AAAI 2026 | `aaai2026-unified-template.tex` | `aaai2026.sty` |
| COLM 2025 | `colm2025_conference.tex` | `colm2025_conference.sty` |

## Conference Resubmission & Format Conversion

### ML/AI Conversions

| From → To | Page Change | Key Adjustments |
|-----------|-------------|------------------|
| NeurIPS → ICML | 9 → 8 pages | Cut 1 page, add Broader Impact if missing |
| ICML → ICLR | 8 → 9 pages | Can expand experiments, add LLM disclosure |
| NeurIPS → ACL | 9 → 8 pages | Restructure for NLP conventions, add Limitations |
| ICLR → AAAI | 9 → 7 pages | Significant cuts needed, strict style adherence |
| Any → COLM | varies → 9 | Reframe for language model focus |

### Content Migration Rules

**Never copy LaTeX preambles between templates.** Instead:
1. Start fresh with target template
2. Copy ONLY content sections from old paper
3. Paste into target template structure

When cutting pages: Move detailed proofs to appendix, condense related work, combine similar experiments, tighten writing.
When expanding: Add ablation studies, expand limitations, include additional baselines.

---

# Part III: 系统会议论文 (Systems Conference Papers)

Derived from systems-paper-writing skill. Fine-grained structural guidance for writing **10-12 page systems papers** targeting top systems venues: OSDI, SOSP, ASPLOS, NSDI, and EuroSys.

## Authoritative Sources

This blueprint synthesizes guidance from established systems researchers:

1. **Levin & Redell** — "How (and How Not) to Write a Good Systems Paper" (SOSP'83 PC Chairs)
2. **Irene Zhang** (MSR/UW) — "Hints on how to write an SOSP paper" (SOSP/OSDI PC)
3. **Gernot Heiser** (UNSW, seL4) — Style Guide + Paper Writing Talk
4. **Timothy Roscoe** (ETH Zurich) — "Writing reviews for systems conferences"
5. **Mike Dahlin** (UT Austin/Google) — "Giving a Conference Talk"
6. **Yi Ding** — "How to write good systems papers?"
7. **hzwer & DingXiaoH** — WritingAIPaper

## 12-Page Systems Paper Blueprint

### Overview: Page Allocation

| Section | Pages | Purpose |
|---------|-------|---------|
| Abstract | ~0.25 | 150-250 words, 5-sentence structure |
| S1 Introduction | 1.5-2 | Problem → Gap → Insight → Contributions |
| S2 Background & Motivation | 1-1.5 | Terms + Production observations |
| S3 Design | 3-4 | Architecture + Module details + Alternatives |
| S4 Implementation | 0.5-1 | Prototype details, LOC, key engineering |
| S5 Evaluation | 3-4 | Setup + End-to-end + Microbenchmarks + Scalability |
| S6 Related Work | 1 | Grouped by methodology, explicit comparison |
| S7 Conclusion | 0.5 | 3-sentence summary |
| **Total** | **~12** | Submission: 12 pages strict (USENIX) / 11 pages (ACM ASPLOS) |

### Abstract (150-250 words, 5 sentences)

```
Sentence 1: Problem context and importance
Sentence 2: Gap in existing approaches
Sentence 3: Key insight or thesis ("X is better for Y in environment Z")
Sentence 4: Summary of approach and key results
Sentence 5: Broader impact or availability
```

### S1 Introduction (1.5-2 pages)

**Paragraph structure**:

1. **Problem statement** (~0.5 page) — Establish the domain and why it matters. Use concrete numbers.
2. **Gap analysis** (~0.5 page) — Enumerate specific gaps G1-Gn in existing systems. Each gap is one sentence with evidence.
3. **Key insight** (1 paragraph) — The thesis statement: "X is better for applications Y running in environment Z." (Irene Zhang formula)
4. **Contributions** (~0.5 page) — Numbered list of 3-5 concrete contributions. Each contribution is testable and maps to a section.

**Writing pattern**: hzwer Move 1 (Establish territory) → Move 2 (Find niche) → Move 3 (Occupy niche).

### S2 Background & Motivation (1-1.5 pages)

1. **Technical background** (~0.5 page) — Define terms and systems the reader needs. Follow Gernot Heiser's "define-before-use" principle.
2. **Production observations** (~0.5-1 page) — Present Observation 1, 2, 3 from real data or measurements. Each observation leads to a design insight.

### S3 Design (3-4 pages)

1. **System architecture overview** (~0.5 page) — Architecture diagram first. One-paragraph walkthrough of major components and data flow.
2. **Module-by-module design** (~2-2.5 pages) — Each subsection: what the module does, the design choice made, alternatives considered, and why this choice wins.
3. **Design alternatives and trade-offs** (~0.5-1 page) — For each major decision, explicitly discuss what was not chosen and why.

### S4 Implementation (0.5-1 page)

1. **Prototype description** — Language, framework, LOC, integration with existing systems.
2. **Key engineering decisions** — Non-obvious implementation choices worth documenting.

### S5 Evaluation (3-4 pages)

1. **Experimental setup** (~0.5 page) — Hardware, baselines, workloads, metrics. Enough detail to reproduce.
2. **End-to-end comparison** (~1-1.5 pages) — X vs baselines for application Y on environment Z.
3. **Microbenchmarks / Ablation** (~1-1.5 pages) — Isolate each design decision's contribution.
4. **Scalability** (~0.5 page) — Show behavior as problem size, cluster size, or load increases.

**Critical rule** (Irene Zhang): State every experimental conclusion **three times**:
- Section opening: hypothesis ("We expect X to outperform Y because...")
- Section closing: conclusion ("Results show X outperforms Y by Z%")
- Figure caption: evidence ("Figure N shows X achieves Z% better throughput than Y")

### S6 Related Work (1 page)

- Group by **methodology or approach**, not by individual papers.
- For each group: what they do, what limitation remains, how your work differs.
- Use a comparison table when comparing 4+ systems on specific dimensions.

### S7 Conclusion (0.5 page)

Three sentences (Irene Zhang formula):
1. The hypothesis / problem addressed
2. The solution approach
3. The key result

## Writing Patterns

Four reusable patterns for structuring systems papers:

### Pattern 1: Gap Analysis
Enumerate gaps G1-Gn in Introduction → map to answers A1-An in Design. Creates a clear contract with the reader.

### Pattern 2: Observation-Driven
Present production observations (O1-O3) in Motivation → derive design insights → build system around insights. Effective when you have real workload data.

### Pattern 3: Contribution List
Numbered contributions in Introduction, each mapping to a section. Readers (and reviewers) can track claims through the paper.

### Pattern 4: Thesis Formula (Irene Zhang)
Structure the entire paper around: "X is better for applications Y running in environment Z." Introduction states it, Design explains how, Evaluation proves it.

## Conference Differences

| Venue | Format | Submission Limit | Camera-Ready | References |
|-------|--------|-----------------|--------------|------------|
| OSDI | USENIX | 12 pages | 14 pages | Unlimited |
| NSDI | USENIX | 12 pages | 14 pages | Unlimited |
| SOSP | ACM SIGOPS | 12 pages (tech content) | — | Unlimited |
| ASPLOS | ACM SIGPLAN | 11 pages | 13 pages | Unlimited |
| EuroSys | ACM | 12 pages | — | Unlimited |

Based on 2025/2026 CFPs. Verify current limits before submission.

## Writing Philosophy for Systems Papers

### Manage Reader State (Gernot Heiser)
Treat the reader's cognitive load like an OS managing process state. Never introduce a concept without context. Never reference something defined later without a forward pointer.

### Six-Dimensional Quality (Levin & Redell)
Self-check against: **Original Ideas**, **Reality** (is it built?), **Lessons** (what did you learn?), **Choices** (alternatives discussed?), **Context** (related work fair?), **Presentation** (clear writing?).

### Page-One Figure (hzwer)
Include a figure on the first page that captures the core idea. Reviewers form first impressions from the title, abstract, and page-one figure.

## Quick Checklist for Systems Papers

- [ ] Thesis statement follows "X is better for Y in Z" formula
- [ ] Introduction has numbered contributions (3-5)
- [ ] Each contribution maps to a paper section
- [ ] Design discusses alternatives for every major choice
- [ ] Every eval conclusion stated 3 times (hypothesis, result, caption)
- [ ] Related work grouped by methodology, not individual papers
- [ ] Page budget within venue limits
- [ ] All citations verified programmatically (no hallucinated references)

## Common Issues and Solutions (Systems Papers)

| Issue | Solution |
|-------|----------|
| Paper feels like a "feature list" | Restructure around thesis formula: X better for Y in Z |
| Evaluation lacks depth | Add ablation experiments isolating each design decision |
| Reviewers say "incremental" | Strengthen gap analysis: make G1-Gn crisper with evidence |
| Design section too long | Move implementation details to S4, keep S3 at design level |
| Motivation feels weak | Add production observations with concrete numbers |
| Related work reads like a bibliography | Group by approach, add explicit differentiation |

## Academic Integrity Requirements

### Citation Discipline
- **Never generate citations from memory.** Use the citation verification workflow from Part II (Semantic Scholar / DBLP / CrossRef APIs).
- Mark unverified references as `[CITATION NEEDED]`.

### Prohibition of Fabrication
- Do NOT fabricate production observations, traces, deployment experiences, or experimental results.
- Do NOT generate fake venue rules, paper metadata, or best-paper claims.
- Do NOT copy paragraph-level text from reference papers. This skill provides **structural guidance**, not copy-paste templates.

### LLM Disclosure
- Some venues require disclosure of substantial LLM use in writing or ideation. Check each venue's AI policy in the current CFP.

### Temporal Validity
- Venue rules (page limits, format, AI policies) change annually. All venue information is based on 2025/2026 CFPs. **Always verify against the current year's CFP.**

---

# Part IV: 报告规范与图表 (Reporting Guidelines & Figures)

Derived from scientific-writing skill. Covers study-specific reporting standards, figure/table design principles, and visual best practices.

## Reporting Guidelines by Study Type

Ensure completeness and transparency by following established reporting standards.

### Key Guidelines

- **CONSORT**: Randomized controlled trials
- **STROBE**: Observational studies (cohort, case-control, cross-sectional)
- **PRISMA**: Systematic reviews and meta-analyses
- **STARD**: Diagnostic accuracy studies
- **TRIPOD**: Prediction model studies
- **ARRIVE**: Animal research
- **CARE**: Case reports
- **SQUIRE**: Quality improvement studies
- **SPIRIT**: Study protocols for clinical trials
- **CHEERS**: Economic evaluations

Each guideline provides checklists ensuring all critical methodological elements are reported.

## Figures and Tables

Create effective data visualizations that enhance comprehension.

### When to Use Tables vs. Figures

- **Tables**: Precise numerical data, complex datasets, multiple variables requiring exact values
- **Figures**: Trends, patterns, relationships, comparisons best understood visually

### Design Principles

- Make each table/figure self-explanatory with complete captions
- Use consistent formatting and terminology across all display items
- Label all axes, columns, and rows with units
- Include sample sizes (n) and statistical annotations
- Follow the "one table/figure per 1000 words" guideline
- Avoid duplicating information between text, tables, and figures

### Common Figure Types

- Bar graphs: Comparing discrete categories
- Line graphs: Showing trends over time
- Scatterplots: Displaying correlations
- Box plots: Showing distributions and outliers
- Heatmaps: Visualizing matrices and patterns

### Tables in LaTeX (ML Papers)

Use `booktabs` LaTeX package for professional tables:

```latex
\usepackage{booktabs}
\begin{tabular}{lcc}
\toprule
Method & Accuracy \uparrow{} & Latency \downarrow{} \\
\midrule
Baseline & 85.2 & 45ms \\
\textbf{Ours} & \textbf{92.1} & 38ms \\
\bottomrule
\end{tabular}
```

**Rules:**
- Bold best value per metric
- Include direction symbols (\uparrow{} higher is better, \downarrow{} lower is better)
- Right-align numerical columns
- Consistent decimal precision

### Figures (ML Conference Standards)

- **Vector graphics** (PDF, EPS) for all plots and diagrams
- **Raster** (PNG 600 DPI) only for photographs
- Use **colorblind-safe palettes** (Okabe-Ito or Paul Tol)
- Verify **grayscale readability** (8% of men have color vision deficiency)
- **No title inside figure**—the caption serves this function
- **Self-contained captions**—reader should understand without main text

## Common Issues and Solutions (ML Papers)

| Issue | Solution |
|-------|----------|
| Abstract too generic | Delete first sentence if it could be prepended to any ML paper. Start with your specific contribution. |
| Introduction exceeds 1.5 pages | Split background into Related Work. Front-load contribution bullets. |
| Experiments lack explicit claims | Add sentence before each experiment: "This experiment tests whether [specific claim]..." |
| Reviewers find paper hard to follow | Add explicit signposting, use consistent terminology, include self-contained figure captions |
| Missing statistical significance | Always include error bars, number of runs, statistical tests if comparing methods |

---

# Part V: 代码工具与审稿回应 (Code Tools & Peer Review Response)

This section provides Python implementations of common academic writing structures and a framework for responding to peer review. These tools can be used programmatically to generate outlines, format statistics, manage citations, structure arguments, and compose review response letters.

## IMRAD Structure (Python Implementation)

```python
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class PaperSection:
    """Define paper section structure"""
    title: str
    purpose: str
    key_elements: List[str]
    common_mistakes: List[str]
    word_count_range: tuple

class IMRADStructure:
    """Standard research paper structure"""

    def __init__(self):
        self.sections = self._define_sections()

    def _define_sections(self):
        return {
            'Title': PaperSection(
                title='Title',
                purpose='Concisely communicate main finding',
                key_elements=[
                    'Specific (not vague)',
                    'Informative (conveys key message)',
                    'Indexable (includes key terms)',
                    '10-15 words typical'
                ],
                common_mistakes=[
                    'Too broad or vague',
                    'Missing key variables',
                    'Clickbait-style teasing'
                ],
                word_count_range=(10, 15)
            ),
            'Abstract': PaperSection(
                title='Abstract',
                purpose='Standalone summary of entire paper',
                key_elements=[
                    'Background (1-2 sentences)',
                    'Objective/hypothesis (1 sentence)',
                    'Methods (2-3 sentences)',
                    'Results (2-3 sentences)',
                    'Conclusions (1-2 sentences)'
                ],
                common_mistakes=[
                    'References or citations',
                    'Unexplained abbreviations',
                    'Vague results (no numbers)',
                    'Overstating conclusions'
                ],
                word_count_range=(150, 250)
            ),
            'Introduction': PaperSection(
                title='Introduction',
                purpose='Establish importance and rationale',
                key_elements=[
                    'Opening: Broad importance of topic',
                    'Narrowing: What is known',
                    'Gap: What is unknown',
                    'Purpose: What this study does',
                    'Hypothesis/Aim: Specific predictions'
                ],
                common_mistakes=[
                    'Starting too broad ("Since ancient times...")',
                    'Missing gap statement',
                    'No clear hypothesis',
                    'Too long or tangential'
                ],
                word_count_range=(500, 1000)
            ),
            'Methods': PaperSection(
                title='Methods',
                purpose='Enable replication',
                key_elements=[
                    'Design overview',
                    'Participants/Sample',
                    'Materials/Measures',
                    'Procedure',
                    'Analysis plan'
                ],
                common_mistakes=[
                    'Insufficient detail for replication',
                    'Missing ethical approval',
                    'No justification for sample size',
                    'Analysis decisions post-hoc'
                ],
                word_count_range=(800, 1500)
            ),
            'Results': PaperSection(
                title='Results',
                purpose='Report findings objectively',
                key_elements=[
                    'Descriptive statistics first',
                    'Each hypothesis tested',
                    'Statistics reported in full',
                    'Figures and tables',
                    'Unexpected findings noted'
                ],
                common_mistakes=[
                    'Interpreting (save for Discussion)',
                    'Incomplete statistics',
                    'Cherry-picking results',
                    'Poor figure quality'
                ],
                word_count_range=(800, 1500)
            ),
            'Discussion': PaperSection(
                title='Discussion',
                purpose='Interpret findings and implications',
                key_elements=[
                    'Summary of key findings',
                    'Interpretation in context of literature',
                    'Theoretical implications',
                    'Practical implications',
                    'Limitations',
                    'Future directions',
                    'Conclusion'
                ],
                common_mistakes=[
                    'Repeating Results verbatim',
                    'Overgeneralizing',
                    'Ignoring limitations',
                    'Introducing new data'
                ],
                word_count_range=(1000, 2000)
            )
        }

    def generate_outline(self):
        """Create paper outline"""
        outline = "# Research Paper Outline (IMRAD)\n\n"

        for section_name, section in self.sections.items():
            outline += f"## {section.title}\n"
            outline += f"**Purpose**: {section.purpose}\n\n"
            outline += f"**Word count**: {section.word_count_range[0]}-"
            outline += f"{section.word_count_range[1]} words\n\n"
            outline += "**Key elements**:\n"
            for elem in section.key_elements:
                outline += f"- [ ] {elem}\n"
            outline += "\n**Avoid**:\n"
            for mistake in section.common_mistakes:
                outline += f"- {mistake}\n"
            outline += "\n---\n\n"

        return outline

    def check_section(self, section_name: str, text: str):
        """Check if section meets requirements"""
        section = self.sections.get(section_name)
        if not section:
            return "Section not found"

        word_count = len(text.split())
        issues = []

        # Check word count
        if word_count < section.word_count_range[0]:
            issues.append(f"Too short: {word_count} words "
                        f"(minimum {section.word_count_range[0]})")
        elif word_count > section.word_count_range[1] * 1.2:
            issues.append(f"Quite long: {word_count} words "
                        f"(typical max {section.word_count_range[1]})")

        return {
            'section': section_name,
            'word_count': word_count,
            'target_range': section.word_count_range,
            'issues': issues if issues else ['Within guidelines']
        }

# Example
structure = IMRADStructure()
print(structure.generate_outline())
```

## Statistical Reporting (APA Style)

```python
from typing import Dict, Any

class StatisticalReporter:
    """Format statistical results in APA style"""

    @staticmethod
    def t_test(result: Dict[str, Any]) -> str:
        """Format t-test results"""
        # Result should have: t_statistic, df, p_value, mean_1, mean_2, cohens_d

        # Format p-value
        if result['p_value'] < 0.001:
            p_str = "p < .001"
        else:
            p_str = f"p = {result['p_value']:.3f}"

        # Build sentence
        report = (
            f"The {'paired-samples' if result.get('paired') else 'independent-samples'} "
            f"t-test revealed a "
            f"{'significant' if result['significant'] else 'non-significant'} "
            f"difference between groups "
            f"(M₁ = {result['mean_1']:.2f}, M₂ = {result['mean_2']:.2f}), "
            f"t({result['df']}) = {result['t_statistic']:.2f}, "
            f"{p_str}, d = {result['cohens_d']:.2f}."
        )

        return report

    @staticmethod
    def correlation(result: Dict[str, Any]) -> str:
        """Format correlation results"""
        # Result should have: r, p_value, n

        if result['p_value'] < 0.001:
            p_str = "p < .001"
        else:
            p_str = f"p = {result['p_value']:.3f}"

        report = (
            f"There was a {'significant' if result['significant'] else 'non-significant'} "
            f"{'positive' if result['r'] > 0 else 'negative'} correlation "
            f"between the variables, "
            f"r({result['n'] - 2}) = {result['r']:.2f}, {p_str}."
        )

        return report

    @staticmethod
    def regression(result: Dict[str, Any]) -> str:
        """Format regression results"""
        # Result should have: r_squared, f_statistic, df_model, df_resid, p_value

        if result['p_value'] < 0.001:
            p_str = "p < .001"
        else:
            p_str = f"p = {result['p_value']:.3f}"

        report = (
            f"The regression model was "
            f"{'significant' if result['significant'] else 'non-significant'}, "
            f"F({result['df_model']}, {result['df_resid']}) = "
            f"{result['f_statistic']:.2f}, {p_str}, "
            f"R² = {result['r_squared']:.2f}, "
            f"accounting for {result['r_squared']*100:.1f}% of the variance."
        )

        return report

    @staticmethod
    def create_results_table(results: list, table_title: str) -> str:
        """Create APA-style results table"""
        table = f"Table X\n\n{table_title}\n\n"
        table += "| Variable | M | SD | 1 | 2 | 3 |\n"
        table += "|----------|---|----|----|----|\n"

        for i, var in enumerate(results, 1):
            table += f"| {var['name']} | {var['mean']:.2f} | "
            table += f"{var['sd']:.2f} |"

            # Add correlations
            for j in range(len(results)):
                if j < i - 1:
                    table += f" {var['correlations'][j]:.2f} |"
                elif j == i - 1:
                    table += " — |"
                else:
                    table += " |"

            table += "\n"

        table += "\n*Note*. N = XX. * p < .05, ** p < .01, *** p < .001\n"

        return table

# Example
reporter = StatisticalReporter()

t_result = {
    't_statistic': 2.45,
    'df': 58,
    'p_value': 0.017,
    'mean_1': 45.2,
    'mean_2': 40.8,
    'cohens_d': 0.63,
    'significant': True,
    'paired': False
}

print(reporter.t_test(t_result))
```

## Citation Management (Python Implementation)

```python
from dataclasses import dataclass
from typing import List, Optional
from enum import Enum

class CitationStyle(Enum):
    APA = "apa"
    MLA = "mla"
    CHICAGO = "chicago"

@dataclass
class Reference:
    """Store reference information"""
    authors: List[str]
    year: int
    title: str
    journal: Optional[str] = None
    volume: Optional[int] = None
    issue: Optional[int] = None
    pages: Optional[str] = None
    doi: Optional[str] = None
    book_title: Optional[str] = None
    publisher: Optional[str] = None
    url: Optional[str] = None

class CitationFormatter:
    """Format citations and references"""

    @staticmethod
    def format_authors_apa(authors: List[str], in_text: bool = False) -> str:
        """Format author names in APA style"""
        if len(authors) == 1:
            return authors[0]
        elif len(authors) == 2:
            return f"{authors[0]} & {authors[1]}"
        elif in_text and len(authors) >= 3:
            return f"{authors[0]} et al."
        elif len(authors) <= 20:
            return ", ".join(authors[:-1]) + f", & {authors[-1]}"
        else:
            # 21+ authors
            return ", ".join(authors[:19]) + f", ... {authors[-1]}"

    @staticmethod
    def in_text_citation_apa(ref: Reference, page: Optional[str] = None) -> str:
        """Generate APA in-text citation"""
        author_str = CitationFormatter.format_authors_apa(ref.authors, in_text=True)

        if page:
            return f"({author_str}, {ref.year}, p. {page})"
        else:
            return f"({author_str}, {ref.year})"

    @staticmethod
    def reference_apa(ref: Reference) -> str:
        """Generate APA reference list entry"""
        # Authors
        ref_str = CitationFormatter.format_authors_apa(ref.authors, in_text=False)

        # Year
        ref_str += f" ({ref.year}). "

        # Title
        ref_str += f"{ref.title}. "

        # Journal article
        if ref.journal:
            ref_str += f"*{ref.journal}*"
            if ref.volume:
                ref_str += f", *{ref.volume}*"
            if ref.issue:
                ref_str += f"({ref.issue})"
            if ref.pages:
                ref_str += f", {ref.pages}"
            ref_str += ". "

        # Book
        if ref.book_title:
            ref_str += f"*{ref.book_title}*. "
            if ref.publisher:
                ref_str += f"{ref.publisher}. "

        # DOI or URL
        if ref.doi:
            ref_str += f"https://doi.org/{ref.doi}"
        elif ref.url:
            ref_str += f"{ref.url}"

        return ref_str

    @staticmethod
    def generate_reference_list(refs: List[Reference]) -> str:
        """Create formatted reference list"""
        # Sort alphabetically by first author
        sorted_refs = sorted(refs, key=lambda r: r.authors[0].split()[-1])

        ref_list = "# References\n\n"
        for ref in sorted_refs:
            ref_list += CitationFormatter.reference_apa(ref) + "\n\n"

        return ref_list

# Example
refs = [
    Reference(
        authors=['Smith, J. A.', 'Johnson, B. C.'],
        year=2023,
        title='Remote work and wellbeing: A systematic review',
        journal='Journal of Occupational Health Psychology',
        volume=28,
        issue=3,
        pages='234-256',
        doi='10.1037/ocp0000123'
    ),
    Reference(
        authors=['Lee, K.', 'Chen, M.', 'Park, S.'],
        year=2022,
        title='Work-life balance in the digital age',
        book_title='Organizational Psychology',
        publisher='Academic Press'
    )
]

formatter = CitationFormatter()
print("In-text:", formatter.in_text_citation_apa(refs[0]))
print("\nReference list:")
print(formatter.generate_reference_list(refs))
```

## Argument Structure

```python
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class ArgumentStructure:
    """Structure for academic argument"""
    claim: str
    evidence: List[str]
    reasoning: str
    counterargument: Optional[str] = None
    rebuttal: Optional[str] = None

    def format_paragraph(self) -> str:
        """Generate well-structured paragraph"""
        para = f"{self.claim} "

        # Evidence
        for i, ev in enumerate(self.evidence, 1):
            para += f"{ev} "

        # Reasoning
        para += f"{self.reasoning} "

        # Address counterargument if present
        if self.counterargument:
            para += f"While some argue that {self.counterargument}, "
            if self.rebuttal:
                para += f"{self.rebuttal} "

        return para

# Example
arg = ArgumentStructure(
    claim="Remote work poses challenges for work-life balance.",
    evidence=[
        "Smith et al. (2023) found that 65% of remote workers reported "
        "difficulty separating work and personal time.",
        "Lee and Chen (2022) observed increased overtime among remote workers."
    ],
    reasoning="The physical proximity of work materials and lack of spatial "
             "boundaries may contribute to this blurring of domains.",
    counterargument="remote work provides flexibility that could improve balance",
    rebuttal="the flexibility paradox shows that autonomy without structure "
             "can increase rather than decrease work-life conflict "
             "(Park et al., 2021)"
)

print(arg.format_paragraph())
```

## Responding to Peer Review

### Review Response Framework

```python
from dataclasses import dataclass
from typing import List

@dataclass
class ReviewComment:
    """Represent reviewer comment"""
    reviewer: str
    comment_number: int
    comment: str
    response: str
    action_taken: str
    location: str

class ReviewResponse:
    """Organize response to peer review"""

    def __init__(self, manuscript_title: str):
        self.title = manuscript_title
        self.responses = []

    def add_response(self, response: ReviewComment):
        """Add response to reviewer comment"""
        self.responses.append(response)

    def generate_response_letter(self) -> str:
        """Create point-by-point response letter"""
        letter = f"# Response to Reviewers\n\n"
        letter += f"## Manuscript: {self.title}\n\n"
        letter += "Dear Editor and Reviewers,\n\n"
        letter += ("We thank the editor and reviewers for their thoughtful "
                  "comments. We have carefully addressed each point and "
                  "believe the manuscript is substantially improved. "
                  "Below we provide point-by-point responses.\n\n")

        # Group by reviewer
        by_reviewer = {}
        for resp in self.responses:
            if resp.reviewer not in by_reviewer:
                by_reviewer[resp.reviewer] = []
            by_reviewer[resp.reviewer].append(resp)

        for reviewer, comments in by_reviewer.items():
            letter += f"## {reviewer}\n\n"

            for comment in sorted(comments, key=lambda x: x.comment_number):
                letter += f"### Comment {comment.comment_number}\n\n"
                letter += f"*{comment.comment}*\n\n"
                letter += "**Response**: "
                letter += f"{comment.response}\n\n"
                letter += "**Action taken**: "
                letter += f"{comment.action_taken} "
                letter += f"({comment.location})\n\n"
                letter += "---\n\n"

        letter += ("We hope these revisions address the concerns raised. "
                  "We look forward to your decision.\n\n")
        letter += "Sincerely,\n[Authors]\n")

        return letter

# Example
response = ReviewResponse("Remote Work and Wellbeing Study")

response.add_response(ReviewComment(
    reviewer="Reviewer 1",
    comment_number=1,
    comment="The sample size seems small for the claims made. "
            "Can you justify this?",
    response="We appreciate this concern. We have added a power analysis "
             "to the Methods section demonstrating that our sample of "
             "N=120 provides 80% power to detect medium effects (d=0.5). "
             "We have also softened claims in the Discussion to acknowledge "
             "that replication with larger samples is needed.",
    action_taken="Added power analysis to Methods; revised Discussion claims",
    location="Methods p.8, Discussion p.15"
))

response.add_response(ReviewComment(
    reviewer="Reviewer 1",
    comment_number=2,
    comment="The measure of work-life balance is not validated. "
            "This is a significant limitation.",
    response="We agree this is an important point. We have added a new "
             "section to Methods describing the scale validation process, "
             "including factor analysis (α=0.85) and convergent validity "
             "with established measures (r=0.67, p<.001). We have also "
             "added this as a limitation since the scale has not been "
             "validated in other samples.",
    action_taken="Added validation evidence to Methods; noted limitation",
    location="Methods p.9-10, Discussion p.16"
))

print(response.generate_response_letter())
```

### Key Principles for Responding to Reviewers

1. **Always be respectful**: Thank reviewers for their time and insight, even if you disagree
2. **Answer every point**: Do not skip any comment, even minor ones
3. **Quote the comment**: Include the reviewer's original comment before your response
4. **Be specific about changes**: State exactly what was changed and where
5. **Disagree professionally**: If you disagree, provide evidence and reasoning
6. **Highlight improvements**: Frame responses in terms of how the paper improved
7. **Separate by reviewer**: Organize responses by reviewer and comment number

---

# 备选工作流: 5阶段论文写作管线 (5-Phase Paper Pipeline)

<!-- Source: academic-paper-skills (MIT License, collected by CoPaper.AI) -->

This section provides a complete end-to-end framework for producing academic papers — from strategic planning through submission-ready manuscript. It combines two complementary workflows into a single 5-phase pipeline:

1. **Strategic Planning** (Phases 1-3): Platform selection, AI-driven literature search, research gap identification, originality assessment, and optimized outline design.
2. **Systematic Writing** (Phases 4-5): Chapter-by-chapter writing with iterative quality control, final evaluation, and submission package preparation.

**Output**: A complete, submission-ready manuscript with comprehensive quality documentation.

## Workflow Overview

```
Phase 1: PLATFORM ANALYSIS (Target Selection + Style Learning)
    ↓
Phase 2: THEORETICAL FRAMEWORK (AI-Driven Gap Identification)
    ↓
Phase 3: OUTLINE OPTIMIZATION (Quality-Controlled Design)
    ↓
Phase 4: SYSTEMATIC WRITING (Chapter-by-Chapter + Quality Gates)
    ↓
Phase 5: QUALITY CONTROL (Final Validation + Submission Prep)
    ↓
Output: Submission-Ready Manuscript + Quality Reports
```

**Quality Gates**: 3 validation checkpoints in planning phases + after each chapter + final paper evaluation.

## Required Input

### For Planning (Phases 1-3):
- **Core research idea or topic** (brief description)
- **Target platform** (optional — if unclear, will recommend)
- **Field/discipline** (philosophy, cognitive science, interdisciplinary, etc.)
- **Optional**: Any papers you already know about

### For Writing (Phases 4-5):
An optimized detailed outline (generated from Phase 3, or equivalent) specifying:
- Chapter titles and word counts
- Subsection structure (to 3rd level)
- Content guidance for each section
- Key citations to include
- Argument structure notes

---

## Phase 1: Platform Analysis

### Goal
Identify the optimal submission platform and understand its writing standards through systematic sample paper analysis.

### Input Required from User
- **Core research idea or topic** (brief description)
- **Target platform** (optional - if unclear, will recommend)
- **Field/discipline** (philosophy, cognitive science, interdisciplinary, etc.)

### Workflow

#### Step 1.1: Platform Selection (If Needed)

If target platform unclear:

1. **List candidate platforms** based on research content:
   - **PhilArchive/PhilPapers**: Philosophy papers, phenomenology, metaphysics
   - **arXiv (cs.AI, q-bio.NC)**: Computational, neuroscience, AI-related
   - **PhilSci-Archive**: Philosophy of science, formal methods
   - **PsyArXiv**: Psychology, cognitive science
   - **SocArXiv**: Social sciences, interdisciplinary

2. **Evaluate each platform**:
   - Subject area alignment (does your topic fit?)
   - Methodology match (philosophical/empirical/computational)
   - Acceptance criteria
   - Typical review timeline

3. **Provide recommendation** with reasoning

4. **Decision Point 1**: You confirm platform or suggest alternative

#### Step 1.2: Sample Paper Search (AI-Driven, Quality-Controlled)

Conduct **multi-dimensional search** for 8-10 representative papers:

**Search Strategy**:

**Time Dimension**:
- Recent (last 6 months): 3 papers - capture current trends
- Current (1-2 years): 3 papers - established standards
- Classic (highly cited): 2 papers - quality benchmarks

**Relevance Dimension**:
- Use keyword combinations from your topic
- Score each paper 0-10 for relevance
- Retain only papers scoring ≥7/10

**Diversity Dimension**:
- Multiple authors (≥5 unique)
- Different research perspectives
- Varied paper lengths

**Tools Used**:
- Exa MCP (semantic search)
- Tavily MCP (web search)
- Platform-specific search (PhilPapers, arXiv)

**Quality Validation**:
After search, generate evaluation report validating:
- Sample quality metrics
- Time distribution check
- Relevance statistics
- Diversity assessment
- Pass/Fail recommendation

**Quality Gate 1** (Must Pass):
- Sample papers ≥8
- Time distribution balanced
- Average relevance ≥8/10
- Unique authors ≥5

**If Failed**: Re-search with adjusted criteria

#### Step 1.3: Writing Standards Extraction

From the 8-10 sample papers, extract:

**Structural Patterns**:
- Abstract structure (Problem→Method→Results→Contribution?)
- Chapter organization (how many sections? typical flow?)
- Average proportions (Intro 15%, Main 70%, Conclusion 15%?)

**Style Patterns**:
- First-person vs passive voice usage
- How arguments are structured
- Citation density and format
- Use of technical terminology

**Format Specifications**:
- Typical word count range
- Reference count range
- Section heading conventions

**Output**: `[Platform]_Writing_Standards_Guide.md`

---

## Phase 2: Theoretical Framework

### Goal
AI-driven systematic literature search, research gap identification, and originality assessment.

### Input Required from User
- **Core research question/thesis** (your main argument)
- **Background context** (why you're interested in this)
- **Optional**: Any papers you already know about

### Workflow

#### Step 2.1: Literature Search (AI-Driven, Fully Automated)

**Multi-Round Search Strategy**:

**Round 1: Direct Search (Primary Literature)**
1. **Extract core concepts** from your idea (3-5 concepts)
2. **Generate keyword combinations** (10-15 combinations)
   - Concept + concept
   - Concept + method
   - Include synonyms and disciplinary variants
3. **Search each combination** using Exa/Tavily
4. **Collect 30-50 candidate papers**
5. **Quality filter**: Retain top 20 papers (relevance ≥7/10)

**Round 2: Expanded Search (Adjacent Fields)**
1. **Extract new keywords** from Round 1 papers
2. **Search adjacent disciplines**
3. **Collect 10-20 bridging papers**

**Round 3: Classic Literature (Foundational Works)**
1. **Identify highly-cited papers** (>100 citations)
2. **Track citations** from Round 1-2 papers
3. **Collect 5-10 foundational papers**

**Total Literature Base**: 35-50 papers

#### Step 2.2: Research Gap Identification (AI Analysis)

Using collected literature, automatically identify 3-5 research gaps:

**Gap Identification Methods**:

1. **Concept Mapping**:
   - Plot papers on Concept × Method matrix
   - Identify white spaces (unexplored combinations)

2. **Problem-Solution Analysis**:
   - What problems does literature address?
   - What limitations do authors acknowledge?
   - What questions remain unanswered?

3. **Temporal Analysis**:
   - What was once studied but abandoned?
   - What emerged recently but unexplored?

**Gap Types**:
- **Complete gaps**: No existing research
- **Partial gaps**: Preliminary work only, needs development
- **Controversy gaps**: Competing theories, no resolution

**For Each Gap, Document**:
- Clear definition (50-100 words)
- Evidence (3-5 citations showing gap exists)
- Significance assessment (High/Medium/Low)
- Feasibility assessment (Can you address it?)

**Quality Gate 2** (Must Pass):
- Literature base ≥20 papers
- Identified gaps ≥3
- Each gap has ≥3 evidence citations
- At least 1 high-significance gap

**If Failed**: Continue search or pivot research direction

**Output**: `Literature_Review_Report.md` + `Research_Gap_Analysis.md`

#### Step 2.3: Originality Assessment (AI Analysis)

Automatically assess your idea's originality:

**Step 1: Similarity Analysis**
- Compare your idea with top 15 most similar papers
- Create similarity matrix (topic/method/conclusion overlap)
- Calculate overall similarity percentage

**Interpretation**:
- >80%: High similarity, needs repositioning
- 50-80%: Moderate, emphasize differences
- <50%: Good originality, proceed

**Step 2: Innovation Classification**

Identify which innovation types apply (need ≥2):
1. **Methodological**: New approach to known problem
2. **Theoretical**: New framework or model
3. **Application**: Existing theory to new domain
4. **Integrative**: Synthesizing separate literatures

**Step 3: Impact Prediction (1-10 scale)**

**Scoring Criteria**:
- **Gap Importance** (5 points): Core vs. peripheral problem?
- **Generalizability** (3 points): Widely applicable?
- **Explanatory Power** (2 points): Resolves existing puzzles?

**Target**: ≥7/10 for good impact potential

**Output**: `Originality_Assessment_Report.md`

#### Step 2.4: Core Concepts Discussion (Interactive)

**Decision Point 2**: Based on literature analysis:
1. **Propose 3-5 core concepts** to emphasize
2. **Explain rationale** (based on gap analysis + literature frequency)
3. **Ask for your feedback**: Agree? Adjust? Add?

This ensures the paper focuses on the right concepts to maximize contribution.

---

## Phase 3: Outline Optimization

### Goal
Design a structured, review-ready outline optimized from a reviewer's perspective.

### Input
- Literature analysis from Phase 2
- Core concepts (confirmed in Step 2.4)
- Platform standards from Phase 1

### Workflow

#### Step 3.1: Initial Structure Design

Based on platform standards:

1. **Design chapter structure**:
   - Abstract
   - Introduction (with subsections)
   - Main body (3-5 chapters, each with subsections)
   - Conclusion

2. **Allocate word counts**:
   - Introduction: 15-20% of total
   - Main body: 60-70% of total
   - Conclusion: 10-15% of total

3. **Determine argument flow**:
   - Logical progression of ideas
   - Where to introduce concepts
   - Where to address objections

**Output**: `Initial_Outline_Draft.md`

#### Step 3.2: Reviewer-Perspective Self-Assessment

Evaluate the outline as if you were a platform reviewer, using **7 dimensions** (5 points each, 35 total):

1. **Argument Clarity** (1-5): Is the thesis clear? Are supporting arguments identifiable?
2. **Argument Completeness** (1-5): Any logical gaps or jumps? All premises justified?
3. **Literature Support** (1-5): Expected citation count (40+ for philosophy)? Key works covered?
4. **Methodological Clarity** (1-5): Approach explicit? Method justified?
5. **Originality Expression** (1-5): Contribution clear? Differentiated from existing work?
6. **Organization** (1-5): Logical flow? Proportions balanced?
7. **Platform Fit** (1-5): Matches platform style? Meets format requirements?

**Scoring**:
- Total: X/35
- Passing threshold: ≥28/35 (80%)

**Requirement**: Must identify at least 3-5 specific issues with concrete improvement suggestions.

**Output**: `Reviewer_Assessment_Report.md`

#### Step 3.3: Optimization Recommendations (Data-Driven)

For each dimension scoring <4/5, provide:
- **Issue Description**: What specific problem exists?
- **Severity** (High/Medium/Low)
- **Concrete Solution**: Specific actionable fix
- **Expected Improvement**: How much will this raise the score?

**Prioritization**:
1. All high-severity issues first
2. Then medium-severity
3. Then low-severity (optional)

**Decision Point 3**: Present recommendations; you decide:
- Accept (implement all)
- Selective (choose which to implement)
- Modify (adjust recommendations)

#### Step 3.4: Final Outline Generation

After implementing approved optimizations, produce:

**Detailed Outline Structure**:
```markdown
# [Paper Title]

## Abstract (250 words)
- [Key points to cover]

## 1. Introduction (1,500 words)
### 1.1 The Puzzle (400 words)
### 1.2 Existing Approaches (600 words)
### 1.3 This Paper's Contribution (500 words)

## 2. [Main Chapter] (1,200 words)
### 2.1 [Section] (400 words)
...
[Complete structure to 3rd-level headings]

## References
- [Expected 40-60 sources]
```

**Quality Gate 3** (Must Pass):
- Reviewer score ≥28/35 (80%)
- All high-severity issues resolved
- Word allocations sum to target total
- Platform conformity ≥70%

**If Failed**: Redesign outline addressing identified issues

**Final Output**: `Optimized_Detailed_Outline.md`

---

## Phase 4: Systematic Writing

### Goal
Write complete manuscript chapter-by-chapter with iterative quality control.

### Workflow

#### Step 4.1: Writing Environment Setup

Before writing:

1. **Verify outline completeness**:
   - All chapters specified with word counts
   - Content guidance provided
   - Argument structure clear

2. **Load reference documents**:
   - Platform writing standards (from Phase 1)
   - Section writing guides
   - Quality standards

3. **Create writing tracker**:
   ```markdown
   # Writing Progress Tracker
   - [ ] Abstract (250 words)
   - [ ] Chapter 1: Introduction (1,500 words)
   - [ ] Chapter 2: [Title] (1,200 words)
   ...
   - [ ] Conclusion (1,000 words)
   - [ ] References
   ```

**Decision Point 4**: Confirm outline and standards loaded, ready to begin writing.

#### Step 4.2: Chapter-by-Chapter Writing

**For each chapter, follow this sequence**:

##### A. Pre-Writing Review
Before writing chapter N:
1. **Review outline specification** for this chapter:
   - Word count target
   - Subsection structure
   - Content guidance
   - Key arguments/citations

2. **Review previous chapter** (if N>1):
   - Last paragraph of chapter N-1
   - Key concepts introduced
   - Promises to fulfill

##### B. Writing Execution

Write the chapter following:

**Content principles**:
- **Follow outline exactly**: Respect structure and word counts
- **Include specified citations**: Use literature from Phase 2
- **Maintain platform style**: Match voice and terminology (from Phase 1)

**Quality targets (pre-emptive)**:
- Argument quality: Clear thesis, justified premises
- Citation quality: All claims supported, proper format
- Clarity: Precise prose, terms defined, good transitions
- Structure: Logical flow, proper proportions
- Style conformity: Match platform conventions

**Output**: Complete chapter draft

##### C. Post-Writing Evaluation

After completing chapter draft:

1. **Perform 5-dimension assessment**:
   - **Argument Quality** (1-4): Thesis clear? Premises justified? Objections addressed?
   - **Citation Quality** (1-4): All claims cited? Format consistent? Key literature included?
   - **Clarity & Readability** (1-4): Prose clear? Terms defined? Transitions smooth?
   - **Structure & Flow** (1-4): Logical progression? Proper proportions? Follows outline?
   - **Platform Conformity** (1-4): Style match? Voice consistent? Format correct?

2. **Generate quality report**:
   - Total score (X/20)
   - Pass/fail (threshold: ≥16/20)
   - Weak dimensions identified
   - Specific revision recommendations

**Quality Gate 4A** (After Each Chapter):
- Score ≥16/20 (80%)
- All dimensions ≥3/4 (or revisions implemented)
- Word count within ±10% of target

**If Failed**: Implement revisions before proceeding to next chapter

##### D. Iteration (If Needed)

If chapter scores <16/20:

1. **Identify weak dimension(s)**: Which scored <3/4?
2. **Implement targeted revisions**:
   - Argument quality issue → Add justifications, address objections
   - Citation quality issue → Add supporting citations, fix format
   - Clarity issue → Simplify prose, add definitions, improve transitions
   - Structure issue → Reorganize paragraphs, adjust proportions
   - Style issue → Adjust voice, terminology, format
3. **Re-evaluate**: Generate new quality report
4. **Repeat until passing** (typically 1-2 iterations)

**Important**: Do not proceed to next chapter until current chapter passes quality gate.

##### E. Chapter Completion

Once chapter passes quality gate:
1. **Mark chapter complete** in writing tracker
2. **Note key concepts introduced** (for coherence check later)
3. **Preview next chapter** requirements

**Decision Point 5** (After Major Chapters): After completing each main body chapter:
- Present completed chapter summary
- Show quality score
- Ask: Proceed to next chapter or revise further?

#### Step 4.3: Writing Sequence

**Recommended order**:
1. **Introduction** (write first) — Establishes thesis and roadmap
2. **Main Body Chapters** (in outline order) — Follow outline sequence
3. **Conclusion** (write after main body) — Synthesizes findings
4. **Abstract** (write last) — Summarizes complete paper
5. **References** (compile throughout) — Format according to platform standards

#### Step 4.4: Cross-Chapter Coherence Check

After all chapters written, before final evaluation:
1. **Terminology consistency**: Extract key terms, verify consistent usage
2. **Argument flow**: Verify chapter N+1 builds on chapter N
3. **Citation patterns**: Check for uneven citation distribution

**Output**: Cross-chapter coherence report identifying any inconsistencies

---

## Phase 5: Quality Control

### Goal
Perform comprehensive final evaluation and prepare submission-ready manuscript.

### Workflow

#### Step 5.1: Content Completeness Check

Using structured checklist, verify:

**Structural Completeness**:
- [ ] Abstract (250-300 words)
- [ ] Introduction with all required elements
- [ ] All outlined main chapters present
- [ ] Conclusion with all required elements
- [ ] References section formatted correctly

**Content Completeness**:
- [ ] All introduction promises fulfilled
- [ ] All claims supported by evidence or argument
- [ ] All technical terms defined
- [ ] All objections addressed
- [ ] All limitations acknowledged

**Citation Completeness**:
- [ ] Every citation in text has bibliography entry
- [ ] Every bibliography entry cited in text
- [ ] Citation format consistent throughout

**Format Completeness**:
- [ ] Section numbering consistent
- [ ] Heading hierarchy logical
- [ ] Figure/table captions (if applicable)

**Output**: Completeness checklist report

#### Step 5.2: Final 7-Dimension Evaluation

**7 Dimensions** (10 points each, 70 total):

1. **Overall Argument Quality** (1-10): Thesis clarity, chapter integration, logical completeness
2. **Literature Integration** (1-10): Citation count, key literature, critical engagement
3. **Clarity & Accessibility** (1-10): Prose clarity, complex ideas explained
4. **Originality & Contribution** (1-10): Clear contribution, advance over literature
5. **Methodological Rigor** (1-10): Method explicit, appropriate, limitations acknowledged
6. **Structure & Organization** (1-10): Logical flow, balanced proportions, seamless transitions
7. **Platform & Style Conformity** (1-10): Style matches platform, format correct

**Scoring Process**:
1. Evaluate each dimension (1-10)
2. Provide detailed notes for each
3. List any specific issues

**Quality Gate 5** (Final):
- Score ≥56/70 (80%)
- All completeness checklist items complete
- All high-priority issues addressed

**If Failed**: Implement revisions and re-evaluate

#### Step 5.3: Revision Implementation (If Needed)

If final score <56/70 or completeness incomplete:
1. **Prioritize revisions**: HIGH → MEDIUM → LOW
2. **Implement systematically**: Address high-priority first
3. **Re-evaluate**: Generate new final report, verify score ≥56/70
4. **Iterate until passing**

**Decision Point 6**: After final evaluation:
- Present final score and assessment
- Show submission readiness status
- Recommend: Submit immediately / Implement optional improvements / Required revisions

#### Step 5.4: Submission Package Preparation

Once final evaluation passes:

1. **Platform-specific checklist**:
   - **PhilArchive/PhilPapers**: PDF format, Abstract <500 words, metadata complete
   - **arXiv**: LaTeX or PDF format, Abstract <1920 characters, category selection correct
   - **PhilSci-Archive**: PDF format, subject classification, keywords (3-5)

2. **Generate final outputs**:
   - Formatted manuscript (PDF or LaTeX)
   - Abstract (separate file if needed)
   - Metadata file
   - Cover letter (if applicable)

3. **Pre-submission verification**:
   - Re-read complete paper
   - Check all formatting
   - Verify all links/citations work
   - Proofread for typos

**Output**: Complete submission package ready for platform upload

---

## Complete Output Package

Upon completion of all 5 phases, you receive:

### Planning Documentation (from Phases 1-3)
1. **`[Platform]_Writing_Standards_Guide.md`** — Platform style patterns, structural templates
2. **`Sample_Papers_Evaluation_Report.md`** — 8-10 analyzed papers, quality metrics
3. **`Literature_Review_Report.md`** — 35-50 core papers, organized by theme
4. **`Research_Gap_Analysis.md`** — 3-5 identified gaps with evidence
5. **`Originality_Assessment_Report.md`** — Similarity analysis, innovation classification
6. **`Reviewer_Assessment_Report.md`** — 7-dimension scores, optimization recommendations
7. **`Optimized_Detailed_Outline.md`** — Complete structure to 3rd-level headings

### Quality Reports (from Phases 4-5)
8. **Chapter Quality Reports** (one per chapter) — 5-dimension scores, pass/fail status
9. **Cross-Chapter Coherence Report** — Terminology consistency, argument flow
10. **Final Evaluation Report** — 7-dimension assessment, completeness checklist

### Manuscript Files (from Phases 4-5)
11. **Complete Manuscript** — All chapters integrated, submission-ready
12. **Abstract** (separate file) — Standalone 250-300 words
13. **Metadata Document** — Title, keywords, classification

### Supporting Documentation
14. **Writing Progress Tracker** — All chapters completed, quality scores logged
15. **Citation List** — All references, formatted for platform

---

## Decision Points (Interactive)

This pipeline has **6 key decision points** for user input:

### Decision Point 1: Platform Selection (Phase 1, Step 1.1)
**I provide**: Platform analysis + recommendation → **You decide**: Accept or suggest alternative

### Decision Point 2: Core Concepts (Phase 2, Step 2.4)
**I provide**: 3-5 proposed core concepts + rationale → **You decide**: Confirm, adjust, or supplement

### Decision Point 3: Optimization Acceptance (Phase 3, Step 3.3)
**I provide**: Prioritized improvements + recommendations → **You decide**: Accept, select, or modify

### Decision Point 4: Writing Commencement (Phase 4, Step 4.1)
**I provide**: Loaded outline, standards, writing plan → **You confirm**: Ready to begin

### Decision Point 5: Chapter Completion (Phase 4, Step 4.2.E)
**I provide**: Completed chapter with quality score → **You decide**: Proceed, revise, or adjust

### Decision Point 6: Final Submission (Phase 5, Step 5.3)
**I provide**: Final evaluation report + readiness assessment → **You decide**: Submit, improve, or revise

---

## Example Usage

### User Request
"I want to write a philosophy paper about self-continuity during sleep, arguing that narrative compression maintains identity across sleep-wake cycles."

### Pipeline Response

**Phase 1: Platform Analysis**
1. Analyzing topic... Recommended platform: **PhilArchive** (philosophy of mind focus)
2. Searching sample papers... Found 10 candidates, 8 meet quality standards
3. Extracting writing patterns... Style guide generated

**Phase 2: Theoretical Framework**
1. Conducting literature search: 45 papers in literature base
2. Identifying gaps... 4 gaps found (compression mechanism, functional explanation, philosophical implications, integration)
3. Assessing originality... 62% similarity, integrative innovation, impact score: 8/10
4. **Decision Point 2**: Core concepts confirmed

**Phase 3: Outline Optimization**
1. Initial outline designed: 6 chapters, 9,600 words
2. Reviewer assessment: 26/35 (below threshold) → issues identified
3. Optimizations applied: New score 30/35 (passes)

**Phase 4: Systematic Writing**
1. Chapter-by-chapter with quality gates (threshold ≥16/20)
2. All chapters pass → Cross-chapter coherence verified

**Phase 5: Quality Control**
1. Final evaluation: 60/70 (85.7%) PASS
2. Submission package generated

**Output**: Submission-ready manuscript + comprehensive quality reports

---

## Common Issues and Solutions (5-Phase Pipeline)

### Issue 1: Chapter Fails Quality Gate
**Symptom**: Score <16/20 after writing chapter
**Solution**: Review weak dimension(s), implement specific recommendations, re-evaluate
**Prevention**: Follow section guides closely during initial writing

### Issue 2: Inconsistent Style Across Chapters
**Symptom**: Some chapters feel different in tone or voice
**Solution**: Run cross-chapter coherence check, identify inconsistencies, revise to match dominant style
**Prevention**: Reference platform standards before writing each chapter

### Issue 3: Low Final Score (<56/70)
**Symptom**: Paper fails final quality gate
**Solution**: Focus on dimensions scoring <7/10, implement high-priority revisions, re-evaluate
**Common causes**: Insufficient literature integration, unclear contribution, poor coherence

### Issue 4: Completeness Checklist Incomplete
**Symptom**: Missing required elements
**Solution**: Review which category has incomplete items, add missing elements, re-run check
**Prevention**: Use writing tracker throughout

---

# References & Resources

## Reference Documents

| Document | Contents |
|----------|----------|
| [references/imrad_structure.md](references/imrad_structure.md) | Detailed IMRAD structure guidance |
| [references/citation_styles.md](references/citation_styles.md) | Comprehensive citation style guides |
| [references/figures_tables.md](references/figures_tables.md) | Detailed figure/table best practices |
| [references/reporting_guidelines.md](references/reporting_guidelines.md) | Full reporting guideline checklists |
| [references/writing_principles.md](references/writing_principles.md) | In-depth writing principles |
| [references/writing-guide.md](references/writing-guide.md) | Gopen & Swan 7 principles, micro-tips, word choice |
| [references/citation-workflow.md](references/citation-workflow.md) | Citation APIs, Python code, BibTeX management |
| [references/checklists.md](references/checklists.md) | NeurIPS/ICML/ICLR/ACL checklists |
| [references/reviewer-guidelines.md](references/reviewer-guidelines.md) | Evaluation criteria, scoring, rebuttals |
| [references/section-blueprints.md](references/section-blueprints.md) | Per-section paragraph templates for systems papers |
| [references/writing-patterns.md](references/writing-patterns.md) | Four systems writing patterns with examples |
| [references/checklist.md](references/checklist.md) | Systems paper pre-submission checklist |
| [references/systems-conferences.md](references/systems-conferences.md) | Systems conference deadlines and rules |

## LaTeX Templates

- **ML/AI**: ICML, ICLR, NeurIPS, ACL/EMNLP, AAAI, COLM templates
- **Systems**: OSDI, NSDI, ASPLOS, SOSP templates

## Key External Sources

**Writing Philosophy:**
- [Neel Nanda: How to Write ML Papers](https://www.alignmentforum.org/posts/eJGptPbbFPZGLpjsp/highly-opinionated-advice-on-how-to-write-ml-papers)
- [Farquhar: How to Write ML Papers](https://sebastianfarquhar.com/on-research/2024/11/04/how_to_write_ml_papers/)
- [Gopen & Swan: Science of Scientific Writing](https://cseweb.ucsd.edu/~swanson/papers/science-of-writing.pdf)
- [Lipton: Heuristics for Scientific Writing](https://www.approximatelycorrect.com/2018/01/29/heuristics-technical-scientific-writing-machine-learning-perspective/)
- [Perez: Easy Paper Writing Tips](https://ethanperez.net/easy-paper-writing-tips/)

**Systems Papers:**
- Levin & Redell — "How (and How Not) to Write a Good Systems Paper"
- Irene Zhang — "Hints on how to write an SOSP paper"
- Gernot Heiser — Style Guide + Paper Writing Talk
- Timothy Roscoe — "Writing reviews for systems conferences"

**APIs:** [Semantic Scholar](https://api.semanticscholar.org/api-docs/) | [CrossRef](https://www.crossref.org/documentation/retrieve-metadata/rest-api/) | [arXiv](https://info.arxiv.org/help/api/basics.html)

**ML/AI Venues:** [NeurIPS](https://neurips.cc/) | [ICML](https://icml.cc/) | [ICLR](https://iclr.cc/) | [ACL](https://www.aclweb.org/)

**Systems Venues:** [OSDI](https://www.usenix.org/conferences/osdi) | [SOSP](https://sosp.org/) | [ASPLOS](https://www.asplos-conference.org/) | [NSDI](https://www.usenix.org/conferences/nsdi) | [EuroSys](https://eurosys.org/)
