---
name: paper-review
description: 中英文论文全流程审阅与质量审计系统。当用户说"审稿"、"论文评审"、"帮我看看论文"、"模拟审稿"、"paper review"、"自检论文"、"论文质量检查"、"检查论文完整性"、"验证论文结构"、"gate 这篇论文"、"audit 我的 manuscript"、"deep review"、"quick audit"、"看看论文能不能投"、"查重自查"、"检查提交前准备"、或提到论文预提交检查、修改后再审、委员会评审、审稿人质询等场景时触发。整合了深度审计 (v4.2)、5 角色委员会评审、review-paper 6 维度评估框架、以及轻量级论文自检清单。支持 LaTeX/Typst/PDF 格式，中英文论文。
metadata:
  category: academic-writing
  tags: [audit, deep-review, paper, review, pdf, latex, typst, chinese, english, reviewer, gate, re-audit, self-review, committee, methodology, evidence-quality, GRADE, Cochrane-ROB]
  version: "4.2"
  last_updated: "2026-05-27"
  integrated:
    - paper-review v4.2
    - scientific-critical-thinking (methodology assessment, evidence quality)
argument-hint: "[paper.tex|paper.typ|paper.pdf] [--mode quick-audit|deep-review|gate|re-audit|polish] [--focus full|editor|theory|literature|methodology|logic] [--venue VENUE] [--previous-report PATH] [--literature-search] [--scholar-eval] [--format markdown|json]"
allowed-tools: Read, Glob, Grep, Bash(uv *), Task
---

# Paper Review Skill v4.2 (Merged)

`paper-review` merges three previous skills into one unified system:

- **paper-audit v4.2** (core) — deep-review-first audit, 5-mode workflow, 5-role academic committee, 16-dimension issue taxonomy, Python scripts
- **review-paper** — top-journal referee 6-dimension evaluation framework (Argument Structure, Identification Strategy, Econometric Specification, Literature Positioning, Writing Quality, Presentation) with 3-5 referee objections
- **paper-self-review** — lightweight pre-submission quality checklist (Structure, Logic Consistency, Citation Completeness, Figure/Table Quality, Writing Clarity) with section-by-section and final-verdict references

## Core Principle

This skill behaves like a serious reviewer at a top journal or conference: find technical, methodological, claim-level, and cross-section issues; keep script-backed findings separate from reviewer judgment; return structured issue bundles plus revision roadmaps.

**Do not use** for direct source editing, sentence rewriting, or compilation repair. Route those to format-specific writing skills instead.

## What This Skill Produces

| Mode | Output |
|------|--------|
| `quick-audit` | Fast submission-readiness screen with lightweight self-check checklist and script-backed findings |
| `deep-review` | Reviewer-style structured issue bundle with major/moderate/minor findings, plus 6-dimension referee report and 3-5 referee objections |
| `gate` | PASS/FAIL decision calibrated for submission blockers, with EIC desk-reject screening |
| `re-audit` | Compare current issue bundle against a previous audit with root-cause-aware status labels |
| `polish` | Precheck-only handoff into a polishing workflow |

For `deep-review`, the primary outputs are:
- `final_issues.json`
- `overall_assessment.txt`
- `review_report.md` (includes 6-dimension scoring and referee objections when in full mode)
- `revision_roadmap.md`

## Critical Rules

- Never rewrite the paper source unless the user explicitly switches to an editing skill.
- Never fabricate references, baselines, or reviewer evidence.
- Always distinguish `[Script]` from `[LLM]` findings.
- Always anchor reviewer findings to a quote, section, or exact textual location.
- Be conservative with OCR noise, formatting quirks, and obvious copy-editing trivia.
- Review like a careful reader: understand the author's intended meaning before flagging an issue.

## Mode Selection

| Requested intent | Mode |
|---|---|
| "check my paper", "quick audit", "submission readiness", "自检论文", "论文质量检查" | `quick-audit` |
| "review my paper", "simulate peer review", "harsh review", "deep review", "审稿", "论文评审", "模拟审稿" | `deep-review` |
| "is this ready to submit", "gate this submission", "blockers only", "gate 这篇论文", "看看论文能不能投" | `gate` |
| "did I fix these issues", "re-audit", "compare against old review", "修改后再审" | `re-audit` |
| "polish the writing, but only if safe" | `polish` |

Legacy aliases still work for one compatibility cycle:
- `self-check` -> `quick-audit`
- `review` -> `deep-review`

## Committee Focus Routing (deep-review)

For `deep-review`, use the **Academic Pre-Review Committee** by default. This is a 5-role review pass:

1. Editor (desk-reject screen)
2. Reviewer 1 (theory contribution)
3. Reviewer 3 (literature dialogue / gap)
4. Reviewer 2 (methodology transparency)
5. Reviewer 4 (logic chain)

If the user requests a single dimension, run only the matching committee role(s).

If `--focus ...` is provided, it overrides keyword inference:
- `--focus full` (default)
- `--focus editor|theory|literature|methodology|logic`

Keyword map (English + Chinese):
- editor: "desk reject", "pre-screen", "editor", "EIC", "主编", "预筛", "初筛"
- theory: "theory", "contribution", "novelty", "theoretical dialogue", "理论", "贡献", "创新性"
- literature: "related work", "literature", "research gap", "citation", "文献", "综述", "Research Gap", "引用"
- methodology: "methods", "sample", "coding", "data", "design", "SRQR", "方法", "样本", "编码", "数据", "研究设计", "透明度"
- logic: "logic", "argument", "causal", "structure", "论证", "因果", "逻辑", "结构"

Output language: match the user's request language. If ambiguous, match the paper language.

## Review Standard

Read these references before running reviewer-style work:

1. `references/REVIEW_CRITERIA.md`
2. `references/DEEP_REVIEW_CRITERIA.md`
3. `references/CHECKLIST.md`
4. `references/CONSOLIDATION_RULES.md`
5. `references/ISSUE_SCHEMA.md`

### 16-Dimension Issue Taxonomy (from paper-audit deep-review)

1. formula / derivation errors
2. notation inconsistency
3. prose vs formal object mismatch
4. numerical inconsistency
5. missing justification
6. overclaim or claim inaccuracy
7. ambiguity that can mislead a careful reader
8. underspecified methods / missing information
9. internal contradiction
10. self-consistency of standards
11. table structure violations
12. abstract structural incompleteness
13. theory contribution deficiency
14. qualitative methodology opacity
15. pseudo-innovation / straw man
16. paragraph-level argument incoherence

### 6-Dimension Evaluation Framework (from review-paper)

For `deep-review` mode, also evaluate the paper across these referee-style dimensions (integrated as a structured reference for the committee):

#### Dimension 1: Argument Structure
- Is the research question clearly stated?
- Does the introduction motivate the question effectively?
- Is the logical flow sound (question -> method -> results -> conclusion)?
- Are the conclusions supported by the evidence?
- Are limitations acknowledged?

#### Dimension 2: Identification Strategy
- Is the causal claim credible?
- What are the key identifying assumptions? Are they stated explicitly?
- Are there threats to identification (omitted variables, reverse causality, measurement error)?
- Are robustness checks adequate?
- Is the estimator appropriate for the research design?

#### Dimension 3: Econometric / Methodological Specification
- Correct standard errors (clustered? robust? bootstrap?)?
- Appropriate functional form?
- Sample selection issues?
- Multiple testing concerns?
- Are point estimates economically meaningful (not just statistically significant)?

#### Dimension 4: Literature Positioning
- Are the key papers cited?
- Is prior work characterized accurately?
- Is the contribution clearly differentiated from existing work?
- Any missing citations that a referee would flag?

#### Dimension 5: Writing Quality
- Clarity and concision
- Academic tone
- Consistent notation throughout
- Abstract effectively summarizes the paper
- Tables and figures are self-contained (clear labels, notes, sources)

#### Dimension 6: Presentation
- Are tables and figures well-designed?
- Is notation consistent throughout?
- Are there any typos, grammatical errors, or formatting issues?
- Is the paper the right length for the contribution?

### Lightweight Self-Check Checklist (from paper-self-review, for quick-audit mode)

For `quick-audit` mode, use this checklist alongside the script-backed audit:

```
Paper Quality Checklist:
- [ ] Abstract includes problem, method, results, contributions
- [ ] Introduction clearly states research motivation
- [ ] Method is reproducible
- [ ] Results support conclusions
- [ ] Discussion addresses limitations
- [ ] All figures/tables have captions
- [ ] Citations are complete and accurate
```

Additional section-level checks (see `references/SECTION-CHECKLIST.md`):
- **Abstract**: problem clear, method identifiable, main result quantified, contribution stated
- **Introduction**: research gap explicit, question/hypothesis visible, contribution list concrete
- **Method**: enough detail to reproduce, assumptions named, key design choices justified
- **Results**: numbers support claims, uncertainty/significance reported, baselines and ablations interpretable
- **Discussion**: limitations named, failure cases acknowledged, next-step implications reasonable

Verdict buckets (see `references/FINAL-VERDICT.md`):
- `ready with minor edits`
- `needs moderate revision`
- `not ready for submission`

Always report: top 3 blocking issues, top 3 polish issues, missing evidence/citations, figure/table risks.

## Referee Objections (from review-paper, for deep-review mode)

For `deep-review` mode in full committee configuration, after the 6-dimension evaluation is complete, generate **3-5 referee objections** — the tough questions a top referee would raise. Each objection should include:

- **Question**: The specific hard question a referee would ask
- **Why it matters**: Why this could be fatal to the paper's contribution
- **How to address it**: Suggested response or additional analysis

These objections should be integrated into `review_report.md` under a dedicated "Referee Objections" section.

## Methodology & Evidence Quality Assessment

This section integrates the `scientific-critical-thinking` methodology evaluation framework into the paper review process.

### Methodology Critique

Evaluate research methodology across these dimensions:

**Research Design:**
- Is the design appropriate for the research question? (RCT vs observational vs qualitative)
- Are there threats to internal validity (selection bias, history, maturation, testing effects)?
- Is the study adequately powered? Sample size justification?

**Experimental Validity:**
- Construct validity: do the measures actually capture the intended constructs?
- External validity: can findings be generalized beyond the study sample?
- Ecological validity: does the experimental setting reflect real-world conditions?

### Statistical Validity Assessment

Check for common statistical issues:

| Issue | What to Look For |
|-------|-----------------|
| Multiple comparisons | Are corrections applied (Bonferroni, FDR)? |
| P-hacking | Are analyses pre-registered? Any post-hoc decisions? |
| Effect size reporting | Are effect sizes and confidence intervals reported, not just p-values? |
| Assumption violations | Are normality, homoscedasticity, independence assumptions checked? |
| Outlier handling | Are exclusion criteria pre-specified and reported? |
| Missing data | Is the amount and pattern of missing data reported? How was it handled? |

### Bias Identification

Apply structured bias assessment frameworks:

**Cochrane Risk of Bias (ROB) Tool** — for clinical trials:
1. Random sequence generation (selection bias)
2. Allocation concealment (selection bias)
3. Blinding of participants and personnel (performance bias)
4. Blinding of outcome assessment (detection bias)
5. Incomplete outcome data (attrition bias)
6. Selective reporting (reporting bias)
7. Other sources of bias

**ROBINS-I** — for non-randomized studies:
1. Confounding
2. Selection of participants
3. Classification of interventions
4. Deviations from intended interventions
5. Missing data
6. Measurement of outcomes
7. Selection of reported result

### Evidence Quality Frameworks

**GRADE (Grading of Recommendations Assessment, Development and Evaluation):**

| Level | Definition |
|-------|-----------|
| **High** | Further research is very unlikely to change confidence |
| **Moderate** | Further research is likely to have an important impact |
| **Low** | Further research is very likely to change confidence |
| **Very Low** | Any estimate is very uncertain |

Factors that lower quality: risk of bias, inconsistency, indirectness, imprecision, publication bias.
Factors that raise quality: large effect, dose-response gradient, plausible confounding would reduce effect.

### Integration into Review Workflow

- For **deep-review** mode: The methodology committee role (`committee_methodology_agent.md`) should apply these frameworks when evaluating methodology transparency and statistical validity.
- For **quick-audit** mode: Include evidence quality levels (using GRADE) in the assessment of each major finding.
- For **gate** mode: Flag fatal methodological flaws as blockers (e.g., unaddressed confounding, underpowered study, p-hacking).

## Workflow

### Common Step 0

Parse `$ARGUMENTS` and infer the mode if the user did not provide one. State the inferred mode before running commands if you had to infer it.

### `quick-audit`

1. Run:
   ```bash
   uv run python -B "$SKILL_DIR/scripts/audit.py" <paper> --mode quick-audit ...
   ```
2. Present a concise report:
   - `Submission Blockers` first
   - then `Quality Improvements`
   - then checklist items from the lightweight self-check checklist (Section Checklist + Final Verdict)
   - mark quick-audit findings with `[Script]` provenance
3. If the user clearly wants reviewer-depth critique after the quick screen, escalate to `deep-review`.

### `deep-review`

Use this as the default reviewer-style path. In full committee mode, the review should incorporate the 6-dimension evaluation framework (Argument Structure, Identification Strategy, Econometric Specification, Literature Positioning, Writing Quality, Presentation) and produce 3-5 referee objections alongside the standard audit output.

#### Phase 1: Prepare workspace

Run:

```bash
uv run python -B "$SKILL_DIR/scripts/prepare_review_workspace.py" <paper> --output-dir ./review_results
```

This creates:
- `full_text.md`
- `metadata.json`
- `section_index.json`
- `claim_map.json`
- `paper_summary.md`
- `sections/*.md`
- `comments/`
- `references/` (minimal copies for reviewer agents)
- `committee/` (committee reviewer artifacts)

#### Phase 2: Phase 0 automated audit

Run:

```bash
uv run python -B "$SKILL_DIR/scripts/audit.py" <paper> --mode deep-review ...
```

Treat this as **Phase 0 only**. It supplies script-backed context and scores, not the final review.

#### Phase 3: Committee + Review Lanes

##### Phase 3A: Academic Pre-Review Committee (default)

Decide committee focus:
- If `--focus ...` is provided, use it.
- Otherwise infer from the user request using the keyword map in "Committee Focus Routing".
- If nothing matches, default to `full` (all five roles).

Dispatch the committee reviewers (in this exact order) and have them write artifacts into the workspace:

1. `agents/committee_editor_agent.md`
   - write: `committee/editor.md`
   - write: `comments/committee_editor.json`
2. `agents/committee_theory_agent.md`
   - write: `committee/theory.md`
   - write: `comments/committee_theory.json`
3. `agents/committee_literature_agent.md`
   - write: `committee/literature.md`
   - write: `comments/committee_literature.json`
4. `agents/committee_methodology_agent.md`
   - write: `committee/methodology.md`
   - write: `comments/committee_methodology.json`
5. `agents/committee_logic_agent.md`
   - write: `committee/logic.md`
   - write: `comments/committee_logic.json`

If subagents are unavailable, run the committee reviewers inline, but keep the same file outputs.

Then write: `committee/consensus.md`
- include: overall score (1-10), ordered priorities, and the top 3 issues to fix first
- scoring formula:
  - start at 9.0
  - subtract: `1.5 * (# major) + 0.7 * (# moderate) + 0.2 * (# minor)`
  - floor at 1.0
  - if Editor verdict is Desk Reject, cap at 4.0

Note: `render_deep_review_report.py` automatically embeds `committee/*.md` into `review_report.md` when present.

##### Phase 3B: 6-Dimension Evaluation + Referee Objections (from review-paper)

For full committee `deep-review`, perform an additional structured evaluation pass using the 6-Dimension Evaluation Framework (see above). This should be done by the committee as a collective assessment after individual reviews. The output covers:

- Ratings (1-5) for each of the 6 dimensions
- 3-5 referee objections (hard questions a top referee would ask)
- These are written into `review_report.md` under "6-Dimension Evaluation" and "Referee Objections" sections

##### Phase 3C: Section and cross-cutting review lanes (coverage)

Read:
- `references/SUBAGENT_TEMPLATES.md`
- `references/REVIEW_LANE_GUIDE.md`

Then dispatch reviewer tasks for:
- section lanes
  - introduction / related work
  - methods
  - results
  - discussion / conclusion
  - appendix, if present
- cross-cutting lanes
  - claims vs evidence
  - notation and numeric consistency
  - evaluation fairness and reproducibility
  - self-standard consistency
  - prior-art and novelty grounding

Each lane writes a JSON array into `comments/`.

If subagents are unavailable, use the built-in deterministic fallback lane pass in `scripts/audit.py` so the workflow still writes lane-compatible JSON into `comments/` before consolidation.

#### Phase 4: Consolidation

Run:

```bash
uv run python -B "$SKILL_DIR/scripts/consolidate_review_findings.py" <review_dir>
uv run python -B "$SKILL_DIR/scripts/verify_quotes.py" <review_dir> --write-back
uv run python -B "$SKILL_DIR/scripts/render_deep_review_report.py" <review_dir>
```

Consolidation rules:
- merge exact duplicates
- keep distinct paper-level consequences separate even if they share a root cause
- preserve singleton findings unless clearly false positive
- assign `comment_type`, `severity`, `confidence`, and `root_cause_key`

#### Phase 5: Present result

Summarize:
- 1 short paragraph overall assessment
- counts of major / moderate / minor issues
- 3 highest-priority revision items
- 6-dimension rating summary table (if full committee mode)
- 3-5 referee objections summary (if full committee mode)
- path to `review_report.md` and `final_issues.json`

### `gate`

1. Run:
   ```bash
   uv run python -B "$SKILL_DIR/scripts/audit.py" <paper> --mode gate ...
   ```
2. **EIC Screening** (Phase 0.5): Read `agents/editor_in_chief_agent.md` and perform the editor-in-chief desk-reject screening on the paper's title, abstract, and introduction. This evaluates pitch quality, venue fit, fatal flaws, and presentation baseline. A desk-reject verdict is a gate blocker.
3. Report PASS/FAIL.
4. Present EIC screening results first (verdict + score + justification).
5. List blockers next.
6. Keep advisory items separate from blockers.
7. For IEEE pseudocode checks, make it explicit which issues are mandatory and which are only IEEE-safe recommendations.

### `re-audit`

1. Requires `--previous-report PATH`.
2. Run:
   ```bash
   uv run python -B "$SKILL_DIR/scripts/audit.py" <paper> --mode re-audit --previous-report <path> ...
   ```
3. If both old and new `final_issues.json` bundles are available, also run:
   ```bash
   uv run python -B "$SKILL_DIR/scripts/diff_review_issues.py" <old_final_issues.json> <new_final_issues.json>
   ```
4. Present:
   - root-cause-aware status labels: `FULLY_ADDRESSED`, `PARTIALLY_ADDRESSED`, `NOT_ADDRESSED`, `NEW`
   - use structured prior issue bundles when available, but still accept Markdown previous reports

### `polish`

1. Run the audit precheck:
   ```bash
   uv run python -B "$SKILL_DIR/scripts/audit.py" <paper> --mode polish ...
   ```
2. If blockers exist, stop and report them.
3. Only proceed into polishing if the precheck is safe.

## Output Contract

For `deep-review`, the final issue schema is:

```json
{
  "title": "short issue title",
  "quote": "exact quote from paper",
  "explanation": "why this matters and what remains problematic",
  "comment_type": "methodology|claim_accuracy|presentation|missing_information",
  "severity": "major|moderate|minor",
  "confidence": "high|medium|low",
  "source_kind": "script|llm",
  "source_section": "methods",
  "related_sections": ["results", "appendix"],
  "root_cause_key": "shared-normalized-key",
  "review_lane": "claims_vs_evidence",
  "gate_blocker": false,
  "quote_verified": true
}
```

Always prefer:
- exact quotes over vague paraphrase
- evidence-backed findings over style commentary
- issue bundle + roadmap over raw script dumps

## References

| File | Source | Purpose |
|---|---|---|
| `references/REVIEW_CRITERIA.md` | paper-audit | top-level audit scoring and mapping |
| `references/DEEP_REVIEW_CRITERIA.md` | paper-audit | deep-review-specific issue taxonomy (16 dimensions) and leniency rules |
| `references/CONSOLIDATION_RULES.md` | paper-audit | deduplication and root-cause merge policy |
| `references/ISSUE_SCHEMA.md` | paper-audit | canonical JSON schema |
| `references/REVIEW_LANE_GUIDE.md` | paper-audit | section lanes and cross-cutting lanes |
| `references/SUBAGENT_TEMPLATES.md` | paper-audit | reviewer task templates |
| `references/QUICK_REFERENCE.md` | paper-audit | CLI and mode cheat sheet |
| `references/CHECKLIST.md` | paper-audit | general audit checklist |
| `references/AUDIT_GUIDE.md` | paper-audit | audit guide |
| `references/CHANGELOG.md` | paper-audit | version history |
| `references/FORBIDDEN_TERMS.md` | paper-audit | terms to avoid |
| `references/LITERATURE_GROUNDING_GUIDE.md` | paper-audit | literature grounding methodology |
| `references/POLISH_GUIDE.md` | paper-audit | polish mode guide |
| `references/QUALITATIVE_STANDARDS.md` | paper-audit | qualitative research standards |
| `references/SCHOLAR_EVAL_GUIDE.md` | paper-audit | scholar evaluation guide |
| `references/SCORING_SYSTEMS.md` | paper-audit | venue-specific scoring systems |
| `references/TROUBLESHOOTING.md` | paper-audit | troubleshooting guide |
| `references/VENUE_RULES.md` | paper-audit | venue-specific submission rules |
| `references/editorial_decision_standards.md` | paper-audit | editorial decision standards |
| `references/quality_rubrics.md` | paper-audit | quality rubrics |
| `references/SECTION-CHECKLIST.md` | paper-self-review | section-by-section review questions for quick-audit |
| `references/FINAL-VERDICT.md` | paper-self-review | submission readiness verdict buckets and blocking issues |

## Scripts

| Script | Purpose |
|---|---|
| `scripts/audit.py` | Phase 0 audit and mode entrypoint |
| `scripts/prepare_review_workspace.py` | create deep-review workspace |
| `scripts/build_claim_map.py` | extract headline claims and closure targets |
| `scripts/consolidate_review_findings.py` | deduplicate comment JSONs |
| `scripts/verify_quotes.py` | verify exact quote presence |
| `scripts/render_deep_review_report.py` | render final Markdown report |
| `scripts/diff_review_issues.py` | compare old vs new issue bundles |
| `scripts/pdf_parser.py` | PDF parsing utilities |
| `scripts/parsers.py` | general parsing utilities |
| `scripts/report_generator.py` | report generation utilities |
| `scripts/detect_language.py` | language detection |
| `scripts/scoring_model.py` | ML-based scoring model |
| `scripts/check_citations.py` | citation checking |
| `scripts/check_references.py` | reference validation |
| `scripts/literature_search.py` | literature search |
| `scripts/literature_compare.py` | literature comparison |
| `scripts/scholar_eval.py` | scholar evaluation |
| `scripts/visual_check.py` | visual element checking |
| `scripts/models/scoring_model.json` | scoring model weights |

## Agents

Committee agents (deep-review default):
- `agents/committee_editor_agent.md`
- `agents/committee_theory_agent.md`
- `agents/committee_literature_agent.md`
- `agents/committee_methodology_agent.md`
- `agents/committee_logic_agent.md`

Section and cross-cutting lane agents:
- `agents/section_reviewer_agent.md`
- `agents/claims_evidence_reviewer_agent.md`
- `agents/notation_consistency_reviewer_agent.md`
- `agents/evaluation_fairness_reviewer_agent.md`
- `agents/self_consistency_reviewer_agent.md`
- `agents/prior_art_reviewer_agent.md`
- `agents/synthesis_agent.md`
- `agents/editor_in_chief_agent.md` — EIC desk-reject screener (used in `gate` mode)

Specialized deep-review agents:
- `agents/critical_reviewer_agent.md` — devil's advocate with C3-C5 checks
- `agents/domain_reviewer_agent.md` — domain expertise with A1-A7 assessments
- `agents/methodology_reviewer_agent.md` — methodology rigor with B3-B10 checks
- `agents/literature_reviewer_agent.md` — evidence-based literature verification (optional, `--literature-search`)

## Examples

- "Review this manuscript like a serious conference reviewer and tell me the biggest validity risks."
- "帮我审一下这篇论文，用 6 维度框架全面评估，再加 3 个审稿人质询问题。"
- "Run a quick audit on `paper.tex` and tell me what blocks submission."
- "Gate this IEEE submission and separate blockers from recommendations."
- "Re-audit this revision against my previous report."
- "自检一下我的论文，用 checklist 逐项检查完整性。"
- "模拟审稿，deep review 模式，全委员会评审。"
