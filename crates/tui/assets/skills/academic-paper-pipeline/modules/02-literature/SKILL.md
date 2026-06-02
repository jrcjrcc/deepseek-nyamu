---
name: literature-review-synthesis
description: Conduct comprehensive, systematic literature reviews while mastering information synthesis, meta-analysis, and knowledge integration across sources. Search multiple academic databases (PubMed, arXiv, bioRxiv, Semantic Scholar, etc.), synthesize findings using diverse methodological approaches (narrative, thematic, meta-analytic, evidence mapping), and generate professionally formatted documents with verified citations in multiple citation styles (APA, Nature, Vancouver, etc.). Use for systematic reviews, meta-analyses, research synthesis, knowledge mapping, evidence gap analysis, and comprehensive literature searches across biomedical, scientific, and technical domains.
allowed-tools: Read Write Edit Bash
license: MIT license
metadata:
    skill-author: K-Dense Inc. / cc-polymath
---

# Literature Review and Research Synthesis

## Overview

Conduct systematic, comprehensive literature reviews following rigorous academic methodology. Search multiple literature databases, synthesize findings thematically, verify all citations for accuracy, and generate professional output documents in markdown and PDF formats.

This skill integrates knowledge from two complementary domains: (1) systematic literature review methodology with multi-database search, screening, citation verification, and document generation; and (2) advanced research synthesis approaches including narrative synthesis, thematic synthesis, meta-analysis, and evidence mapping. Together they form a complete pipeline for rigorous evidence synthesis.

This skill integrates with multiple scientific skills for database access (gget, bioservices, datacommons-client) and provides specialized tools for citation verification, result aggregation, and document generation.

## When to Use This Skill

Use this skill when:
- Conducting a systematic literature review for research or publication
- Synthesizing current knowledge on a specific topic across multiple sources
- Performing meta-analysis of quantitative research
- Scoping reviews to map the research landscape
- Writing the literature review section of a research paper or thesis
- Investigating the state of the art in a research domain
- Identifying research gaps and future directions
- Integrating qualitative findings across sources through thematic synthesis
- Creating knowledge maps and conceptual frameworks
- Evaluating evidence quality and strength using GRADE or similar frameworks
- Requiring verified citations and professional formatting

## Visual Enhancement with Scientific Schematics

**⚠️ MANDATORY: Every literature review MUST include at least 1-2 AI-generated figures using the scientific-schematics skill.**

This is not optional. Literature reviews without visual elements are incomplete. Before finalizing any document:
1. Generate at minimum ONE schematic or diagram (e.g., PRISMA flow diagram for systematic reviews)
2. Prefer 2-3 figures for comprehensive reviews (search strategy flowchart, thematic synthesis diagram, conceptual framework)

**How to generate figures:**
- Use the **scientific-schematics** skill to generate AI-powered publication-quality diagrams
- Simply describe your desired diagram in natural language
- Nano Banana Pro will automatically generate, review, and refine the schematic

**How to generate schematics:**
```bash
python scripts/generate_schematic.py "your diagram description" -o figures/output.png
```

The AI will automatically:
- Create publication-quality images with proper formatting
- Review and refine through multiple iterations
- Ensure accessibility (colorblind-friendly, high contrast)
- Save outputs in the figures/ directory

**When to add schematics:**
- PRISMA flow diagrams for systematic reviews
- Literature search strategy flowcharts
- Thematic synthesis diagrams
- Research gap visualization maps
- Citation network diagrams
- Conceptual framework illustrations
- Evidence maps showing study coverage and gaps
- Forest plots for meta-analysis results
- Any complex concept that benefits from visualization

For detailed guidance on creating schematics, refer to the scientific-schematics skill documentation.

---

## Core Workflow

Literature reviews follow a structured, multi-phase workflow:

### Phase 1: Planning and Scoping

1. **Define Research Question**: Use PICO framework (Population, Intervention, Comparison, Outcome) for clinical/biomedical reviews
   - Example: "What is the efficacy of CRISPR-Cas9 (I) for treating sickle cell disease (P) compared to standard care (C)?"

2. **Establish Scope and Objectives**:
   - Define clear, specific research questions
   - Determine review type (narrative, systematic, scoping, meta-analysis)
   - Set boundaries (time period, geographic scope, study types)

3. **Develop Search Strategy**:
   - Identify 2-4 main concepts from research question
   - List synonyms, abbreviations, and related terms for each concept
   - Plan Boolean operators (AND, OR, NOT) to combine terms
   - Select minimum 3 complementary databases

4. **Set Inclusion/Exclusion Criteria**:
   - Date range (e.g., last 10 years: 2015-2024)
   - Language (typically English, or specify multilingual)
   - Publication types (peer-reviewed, preprints, reviews)
   - Study designs (RCTs, observational, in vitro, etc.)
   - Document all criteria clearly

### Phase 2: Systematic Literature Search

1. **Multi-Database Search**:

   Select databases appropriate for the domain:

   **Biomedical & Life Sciences:**
   - Use `gget` skill: `gget search pubmed "search terms"` for PubMed/PMC
   - Use `gget` skill: `gget search biorxiv "search terms"` for preprints
   - Use `bioservices` skill for ChEMBL, KEGG, UniProt, etc.

   **General Scientific Literature:**
   - Search arXiv via direct API (preprints in physics, math, CS, q-bio)
   - Search Semantic Scholar via API (200M+ papers, cross-disciplinary)
   - Use Google Scholar for comprehensive coverage (manual or careful scraping)

   **Specialized Databases:**
   - Use `gget alphafold` for protein structures
   - Use `gget cosmic` for cancer genomics
   - Use `datacommons-client` for demographic/statistical data
   - Use specialized databases as appropriate for the domain

2. **Document Search Parameters**:
   ```markdown
   ## Search Strategy
   
   ### Database: PubMed
   - **Date searched**: 2024-10-25
   - **Date range**: 2015-01-01 to 2024-10-25
   - **Search string**:
   ```
     ("CRISPR"[Title] OR "Cas9"[Title])
     AND ("sickle cell"[MeSH] OR "SCD"[Title/Abstract])
     AND 2015:2024[Publication Date]
     ```
   - **Results**: 247 articles
     ```

   Repeat for each database searched.

3. **Export and Aggregate Results**:
   - Export results in JSON format from each database
   - Combine all results into a single file
   - Use `scripts/search_databases.py` for post-processing:
     ```bash
     python search_databases.py combined_results.json \
       --deduplicate \
       --format markdown \
       --output aggregated_results.md
     ```

### Phase 3: Screening and Selection

1. **Deduplication**:
   ```bash
   python search_databases.py results.json --deduplicate --output unique_results.json
   ```
   - Removes duplicates by DOI (primary) or title (fallback)
   - Document number of duplicates removed

2. **Title Screening**:
   - Review all titles against inclusion/exclusion criteria
   - Exclude obviously irrelevant studies
   - Document number excluded at this stage

3. **Abstract Screening**:
   - Read abstracts of remaining studies
   - Apply inclusion/exclusion criteria rigorously
   - Document reasons for exclusion

4. **Full-Text Screening**:
   - Obtain full texts of remaining studies
   - Conduct detailed review against all criteria
   - Document specific reasons for exclusion
   - Record final number of included studies

5. **Create PRISMA Flow Diagram**:
   ```
   Initial search: n = X
   ├─ After deduplication: n = Y
   ├─ After title screening: n = Z
   ├─ After abstract screening: n = A
   └─ Included in review: n = B
   ```

### Phase 4: Data Extraction and Quality Assessment

1. **Extract Key Data** from each included study:
   - Study metadata (authors, year, journal, DOI)
   - Study design and methods
   - Sample size and population characteristics
   - Key findings and results
   - Limitations noted by authors
   - Funding sources and conflicts of interest

2. **Assess Study Quality**:
   - **For RCTs**: Use Cochrane Risk of Bias tool
   - **For observational studies**: Use Newcastle-Ottawa Scale
   - **For systematic reviews**: Use AMSTAR 2
   - **For overall evidence quality**: Use GRADE framework (see Synthesis Quality Assessment section)
   - Rate each study: High, Moderate, Low, or Very Low quality
   - Consider excluding very low-quality studies

3. **Organize by Themes**:
   - Identify 3-5 major themes across studies
   - Group studies by theme (studies may appear in multiple themes)
   - Note patterns, consensus, and controversies

### Phase 5: Synthesis and Analysis

1. **Create Review Document** from template:
   ```bash
   cp assets/review_template.md my_literature_review.md
   ```

2. **Write Thematic Synthesis** (NOT study-by-study summaries):
   - Organize Results section by themes or research questions
   - Synthesize findings across multiple studies within each theme
   - Compare and contrast different approaches and results
   - Identify consensus areas and points of controversy
   - Highlight the strongest evidence

   Example structure:
   ```markdown
   #### 3.3.1 Theme: CRISPR Delivery Methods
   
   Multiple delivery approaches have been investigated for therapeutic
   gene editing. Viral vectors (AAV) were used in 15 studies^1-15^ and
   showed high transduction efficiency (65-85%) but raised immunogenicity
   concerns^3,7,12^. In contrast, lipid nanoparticles demonstrated lower
   efficiency (40-60%) but improved safety profiles^16-23^.
   ```

3. **Critical Analysis**:
   - Evaluate methodological strengths and limitations across studies
   - Assess quality and consistency of evidence
   - Identify knowledge gaps and methodological gaps
   - Note areas requiring future research

4. **Write Discussion**:
   - Interpret findings in broader context
   - Discuss clinical, practical, or research implications
   - Acknowledge limitations of the review itself
   - Compare with previous reviews if applicable
   - Propose specific future research directions

### Phase 6: Citation Verification

**CRITICAL**: All citations must be verified for accuracy before final submission.

1. **Verify All DOIs**:
   ```bash
   python scripts/verify_citations.py my_literature_review.md
   ```

   This script:
   - Extracts all DOIs from the document
   - Verifies each DOI resolves correctly
   - Retrieves metadata from CrossRef
   - Generates verification report
   - Outputs properly formatted citations

2. **Review Verification Report**:
   - Check for any failed DOIs
   - Verify author names, titles, and publication details match
   - Correct any errors in the original document
   - Re-run verification until all citations pass

3. **Format Citations Consistently**:
   - Choose one citation style and use throughout (see `references/citation_styles.md`)
   - Common styles: APA, Nature, Vancouver, Chicago, IEEE
   - Use verification script output to format citations correctly
   - Ensure in-text citations match reference list format

### Phase 7: Document Generation

1. **Generate PDF**:
   ```bash
   python scripts/generate_pdf.py my_literature_review.md \
     --citation-style apa \
     --output my_review.pdf
   ```

   Options:
   - `--citation-style`: apa, nature, chicago, vancouver, ieee
   - `--no-toc`: Disable table of contents
   - `--no-numbers`: Disable section numbering
   - `--check-deps`: Check if pandoc/xelatex are installed

2. **Review Final Output**:
   - Check PDF formatting and layout
   - Verify all sections are present
   - Ensure citations render correctly
   - Check that figures/tables appear properly
   - Verify table of contents is accurate

3. **Quality Checklist**:
   - [ ] All DOIs verified with verify_citations.py
   - [ ] Citations formatted consistently
   - [ ] PRISMA flow diagram included (for systematic reviews)
   - [ ] Search methodology fully documented
   - [ ] Inclusion/exclusion criteria clearly stated
   - [ ] Results organized thematically (not study-by-study)
   - [ ] Quality assessment completed
   - [ ] Limitations acknowledged
   - [ ] References complete and accurate
   - [ ] PDF generates without errors

---

## Core Synthesis Methodologies

When synthesizing findings from the included studies, select the approach(es) best suited to your data type and research question. Each methodology serves a distinct purpose.

### 1. Narrative Synthesis

**Purpose**: Qualitative integration of findings using words and text. Suitable when studies are too heterogeneous for statistical pooling.

**Process**:
```
1. Define scope and research questions
2. Search and select studies systematically
3. Extract key findings and themes
4. Organize by theoretical framework or chronology
5. Identify patterns, contradictions, gaps
6. Synthesize into coherent narrative
```

**Example Structure**:
```markdown
## Research Question
How does X influence Y in Z contexts?

## Synthesis Findings

### Theme 1: Direct Effects
- Study A (2023): Found positive correlation (r=0.65, p<0.01)
- Study B (2022): Confirmed effect in different population
- Study C (2021): Mixed results, moderated by factor Q

### Theme 2: Mediating Mechanisms
- Study D identifies pathway through M
- Study E challenges this, proposes alternative

### Theme 3: Contextual Factors
- Effect stronger in setting X
- No effect observed in setting Y

## Synthesis Conclusion
Evidence suggests X→Y relationship is robust but context-dependent...
```

### 2. Meta-Analysis

**Purpose**: Statistical integration of quantitative results across studies.

**Python Implementation**:
```python
import pandas as pd
import numpy as np
from scipy import stats
import matplotlib.pyplot as plt

class MetaAnalysis:
    """Fixed and random effects meta-analysis"""

    def __init__(self, effect_sizes, standard_errors, study_labels):
        self.effects = np.array(effect_sizes)
        self.se = np.array(standard_errors)
        self.labels = study_labels
        self.weights = None
        self.pooled_effect = None

    def fixed_effects(self):
        """Fixed effects meta-analysis"""
        # Inverse variance weighting
        self.weights = 1 / (self.se ** 2)
        self.pooled_effect = np.sum(self.weights * self.effects) / np.sum(self.weights)
        pooled_se = np.sqrt(1 / np.sum(self.weights))

        # 95% CI
        ci_lower = self.pooled_effect - 1.96 * pooled_se
        ci_upper = self.pooled_effect + 1.96 * pooled_se

        # Heterogeneity statistics
        Q = np.sum(self.weights * (self.effects - self.pooled_effect) ** 2)
        df = len(self.effects) - 1
        I2 = max(0, ((Q - df) / Q) * 100)

        return {
            'pooled_effect': self.pooled_effect,
            'se': pooled_se,
            'ci_lower': ci_lower,
            'ci_upper': ci_upper,
            'Q': Q,
            'I2': I2,
            'p_heterogeneity': 1 - stats.chi2.cdf(Q, df)
        }

    def random_effects(self, tau2_method='DL'):
        """Random effects meta-analysis (DerSimonian-Laird)"""
        # First get fixed effects for Q statistic
        fixed = self.fixed_effects()
        Q = fixed['Q']
        df = len(self.effects) - 1

        # Estimate between-study variance (tau^2)
        C = np.sum(self.weights) - np.sum(self.weights ** 2) / np.sum(self.weights)
        tau2 = max(0, (Q - df) / C)

        # Random effects weights
        re_weights = 1 / (self.se ** 2 + tau2)
        pooled_effect = np.sum(re_weights * self.effects) / np.sum(re_weights)
        pooled_se = np.sqrt(1 / np.sum(re_weights))

        ci_lower = pooled_effect - 1.96 * pooled_se
        ci_upper = pooled_effect + 1.96 * pooled_se

        return {
            'pooled_effect': pooled_effect,
            'se': pooled_se,
            'ci_lower': ci_lower,
            'ci_upper': ci_upper,
            'tau2': tau2,
            'I2': fixed['I2']
        }

    def forest_plot(self, results, title='Meta-Analysis Forest Plot'):
        """Create forest plot visualization"""
        fig, ax = plt.subplots(figsize=(10, len(self.effects) + 2))

        ci_lower = self.effects - 1.96 * self.se
        ci_upper = self.effects + 1.96 * self.se

        y_pos = np.arange(len(self.effects))
        ax.errorbar(self.effects, y_pos,
                   xerr=[self.effects - ci_lower, ci_upper - self.effects],
                   fmt='s', markersize=8, capsize=5, label='Studies')

        pooled_y = len(self.effects) + 0.5
        ax.errorbar([results['pooled_effect']], [pooled_y],
                   xerr=[[results['pooled_effect'] - results['ci_lower']],
                         [results['ci_upper'] - results['pooled_effect']]],
                   fmt='D', markersize=12, capsize=8,
                   color='red', label='Pooled Effect')

        ax.axvline(x=0, color='black', linestyle='--', linewidth=1)
        ax.set_yticks(list(y_pos) + [pooled_y])
        ax.set_yticklabels(list(self.labels) + ['Pooled'])
        ax.set_xlabel('Effect Size')
        ax.set_title(title)
        ax.legend()
        ax.grid(axis='x', alpha=0.3)

        return fig

# Example usage
studies = [
    ('Study A', 0.45, 0.12),
    ('Study B', 0.62, 0.15),
    ('Study C', 0.38, 0.10),
    ('Study D', 0.51, 0.14),
    ('Study E', 0.55, 0.11)
]

labels = [s[0] for s in studies]
effects = [s[1] for s in studies]
ses = [s[2] for s in studies]

meta = MetaAnalysis(effects, ses, labels)
fixed_results = meta.fixed_effects()
random_results = meta.random_effects()

print(f"Fixed Effects: {fixed_results['pooled_effect']:.3f} "
      f"[{fixed_results['ci_lower']:.3f}, {fixed_results['ci_upper']:.3f}]")
print(f"I² = {fixed_results['I2']:.1f}%")
print(f"\nRandom Effects: {random_results['pooled_effect']:.3f} "
      f"[{random_results['ci_lower']:.3f}, {random_results['ci_upper']:.3f}]")

# Create forest plot
fig = meta.forest_plot(random_results)
plt.savefig('forest_plot.png', dpi=300, bbox_inches='tight')
```

### 3. Thematic Synthesis

**Purpose**: Integrate qualitative findings across studies by identifying, coding, and organizing themes.

**Process**:
```python
from collections import defaultdict
import pandas as pd

class ThematicSynthesis:
    """Synthesize themes across qualitative studies"""

    def __init__(self):
        self.studies = []
        self.themes = defaultdict(list)

    def add_study(self, study_id, findings):
        """Add study findings

        Args:
            study_id: Study identifier
            findings: List of (finding, theme) tuples
        """
        self.studies.append(study_id)
        for finding, theme in findings:
            self.themes[theme].append({
                'study': study_id,
                'finding': finding
            })

    def get_theme_matrix(self):
        """Create study x theme presence matrix"""
        themes = list(self.themes.keys())
        matrix = []

        for study in self.studies:
            row = []
            for theme in themes:
                has_theme = any(f['study'] == study for f in self.themes[theme])
                row.append(1 if has_theme else 0)
            matrix.append(row)

        return pd.DataFrame(matrix,
                          index=self.studies,
                          columns=themes)

    def synthesize_theme(self, theme_name):
        """Synthesize findings for a specific theme"""
        findings = self.themes[theme_name]

        synthesis = {
            'theme': theme_name,
            'n_studies': len(set(f['study'] for f in findings)),
            'n_findings': len(findings),
            'findings_by_study': defaultdict(list)
        }

        for finding in findings:
            synthesis['findings_by_study'][finding['study']].append(
                finding['finding']
            )

        return synthesis

    def generate_report(self):
        """Generate synthesis report"""
        report = []
        report.append("# Thematic Synthesis Report\n")
        report.append(f"Total Studies: {len(self.studies)}\n")
        report.append(f"Total Themes: {len(self.themes)}\n\n")

        # Theme frequency
        report.append("## Theme Prevalence\n")
        for theme, findings in sorted(self.themes.items(),
                                    key=lambda x: len(x[1]),
                                    reverse=True):
            n_studies = len(set(f['study'] for f in findings))
            report.append(f"- **{theme}**: {n_studies} studies, "
                        f"{len(findings)} findings\n")

        report.append("\n## Detailed Synthesis\n")
        for theme in sorted(self.themes.keys()):
            synthesis = self.synthesize_theme(theme)
            report.append(f"\n### {theme}\n")
            report.append(f"Present in {synthesis['n_studies']} studies\n\n")

            for study, findings in synthesis['findings_by_study'].items():
                report.append(f"**{study}**:\n")
                for finding in findings:
                    report.append(f"- {finding}\n")
                report.append("\n")

        return ''.join(report)

# Example usage
synth = ThematicSynthesis()

synth.add_study('Smith2023', [
    ('Participants valued flexibility', 'Flexibility'),
    ('Trust was essential for engagement', 'Trust'),
    ('Time constraints were barrier', 'Barriers')
])

synth.add_study('Jones2022', [
    ('Flexibility enabled participation', 'Flexibility'),
    ('Communication breakdowns reduced trust', 'Trust'),
    ('Technology issues created frustration', 'Barriers')
])

synth.add_study('Lee2023', [
    ('Schedule flexibility was key benefit', 'Flexibility'),
    ('Clear expectations built trust', 'Trust')
])

# Generate outputs
matrix = synth.get_theme_matrix()
print(matrix)
print("\n" + synth.generate_report())
```

### 4. Evidence Mapping

**Purpose**: Visualize the research landscape and identify under-studied areas.

**Approach**:
```python
import networkx as nx
import matplotlib.pyplot as plt

class EvidenceMap:
    """Create evidence map of research domain"""

    def __init__(self):
        self.graph = nx.DiGraph()

    def add_study(self, study_id, population, intervention,
                 outcome, quality='medium'):
        """Add study to evidence map"""
        pop_node = f"POP: {population}"
        int_node = f"INT: {intervention}"
        out_node = f"OUT: {outcome}"

        self.graph.add_node(pop_node, type='population')
        self.graph.add_node(int_node, type='intervention')
        self.graph.add_node(out_node, type='outcome')

        self.graph.add_edge(pop_node, int_node,
                          study=study_id, quality=quality)
        self.graph.add_edge(int_node, out_node,
                          study=study_id, quality=quality)

    def identify_gaps(self):
        """Identify under-researched areas"""
        gaps = {
            'populations': [],
            'interventions': [],
            'outcomes': [],
            'combinations': []
        }

        for node in self.graph.nodes():
            degree = self.graph.degree(node)
            node_type = self.graph.nodes[node]['type']

            if degree < 2:
                gaps[f"{node_type}s"].append(node)

        return gaps

    def visualize(self):
        """Create evidence map visualization"""
        pos = nx.spring_layout(self.graph, k=2, iterations=50)

        colors = []
        for node in self.graph.nodes():
            node_type = self.graph.nodes[node]['type']
            if node_type == 'population':
                colors.append('lightblue')
            elif node_type == 'intervention':
                colors.append('lightgreen')
            else:
                colors.append('lightyellow')

        plt.figure(figsize=(12, 8))
        nx.draw(self.graph, pos, node_color=colors,
               with_labels=True, node_size=3000,
               font_size=8, arrows=True)

        plt.title('Evidence Map')
        return plt.gcf()
```

---

## Synthesis Quality Assessment

### GRADE Framework

**Criteria for Evidence Quality**:
1. **Risk of bias**: Study design and execution quality
2. **Inconsistency**: Heterogeneity across studies
3. **Indirectness**: Relevance to question
4. **Imprecision**: Sample size and confidence intervals
5. **Publication bias**: Missing studies

**Rating System**:
```
High:     Further research unlikely to change confidence
Moderate: Further research likely important to confidence
Low:      Further research very likely important
Very Low: Very uncertain about the estimate
```

### Quality Assessment Tools (by Study Type)
- **For RCTs**: Cochrane Risk of Bias tool
- **For observational studies**: Newcastle-Ottawa Scale
- **For systematic reviews**: AMSTAR 2
- **For overall body of evidence**: GRADE

---

## Synthesis Patterns

### Effective Synthesis
```
✓ Systematic search strategy documented
✓ Inclusion/exclusion criteria clear
✓ Quality assessment of each study
✓ Appropriate synthesis method for data type
✓ Heterogeneity acknowledged and explored
✓ Limitations discussed
✓ Practical implications stated
✓ Research gaps identified
```

### Ineffective Synthesis
```
✗ Cherry-picking favorable studies
✗ Mixing apples and oranges without justification
✗ Ignoring study quality differences
✗ Over-generalizing from limited evidence
✗ Failing to report null findings
✗ Missing recent relevant studies
✗ Synthesis method unclear or inappropriate
```

---

## Database-Specific Search Guidance

### PubMed / PubMed Central

Access via `gget` skill:
```bash
# Search PubMed
gget search pubmed "CRISPR gene editing" -l 100

# Search with filters
# Use PubMed Advanced Search Builder to construct complex queries
# Then execute via gget or direct Entrez API
```

**Search tips**:
- Use MeSH terms: `"sickle cell disease"[MeSH]`
- Field tags: `[Title]`, `[Title/Abstract]`, `[Author]`
- Date filters: `2020:2024[Publication Date]`
- Boolean operators: AND, OR, NOT
- See MeSH browser: https://meshb.nlm.nih.gov/search

### bioRxiv / medRxiv

Access via `gget` skill:
```bash
gget search biorxiv "CRISPR sickle cell" -l 50
```

**Important considerations**:
- Preprints are not peer-reviewed
- Verify findings with caution
- Check if preprint has been published (CrossRef)
- Note preprint version and date

### arXiv

Access via direct API or WebFetch:
```python
# Example search categories:
# q-bio.QM (Quantitative Methods)
# q-bio.GN (Genomics)
# q-bio.MN (Molecular Networks)
# cs.LG (Machine Learning)
# stat.ML (Machine Learning Statistics)

# Search format: category AND terms
search_query = "cat:q-bio.QM AND ti:\"single cell sequencing\""
```

### Semantic Scholar

Access via direct API (requires API key, or use free tier):
- 200M+ papers across all fields
- Excellent for cross-disciplinary searches
- Provides citation graphs and paper recommendations
- Use for finding highly influential papers

### Specialized Biomedical Databases

Use appropriate skills:
- **ChEMBL**: `bioservices` skill for chemical bioactivity
- **UniProt**: `gget` or `bioservices` skill for protein information
- **KEGG**: `bioservices` skill for pathways and genes
- **COSMIC**: `gget` skill for cancer mutations
- **AlphaFold**: `gget alphafold` for protein structures
- **PDB**: `gget` or direct API for experimental structures

### Citation Chaining

Expand search via citation networks:

1. **Forward citations** (papers citing key papers):
   - Use Google Scholar "Cited by"
   - Use Semantic Scholar or OpenAlex APIs
   - Identifies newer research building on seminal work

2. **Backward citations** (references from key papers):
   - Extract references from included papers
   - Identify highly cited foundational work
   - Find papers cited by multiple included studies

---

## Citation Style Guide

Detailed formatting guidelines are in `references/citation_styles.md`. Quick reference:

### APA (7th Edition)
- In-text: (Smith et al., 2023)
- Reference: Smith, J. D., Johnson, M. L., & Williams, K. R. (2023). Title. *Journal*, *22*(4), 301-318. https://doi.org/10.xxx/yyy

### Nature
- In-text: Superscript numbers^1,2^
- Reference: Smith, J. D., Johnson, M. L. & Williams, K. R. Title. *Nat. Rev. Drug Discov.* **22**, 301-318 (2023).

### Vancouver
- In-text: Superscript numbers^1,2^
- Reference: Smith JD, Johnson ML, Williams KR. Title. Nat Rev Drug Discov. 2023;22(4):301-18.

**Always verify citations** with verify_citations.py before finalizing.

---

## Prioritizing High-Impact Papers (CRITICAL)

**Always prioritize influential, highly-cited papers from reputable authors and top venues.** Quality matters more than quantity in literature reviews.

### Citation Count Thresholds

Use citation counts to identify the most impactful papers:

| Paper Age | Citation Threshold | Classification |
|-----------|-------------------|----------------|
| 0-3 years | 20+ citations | Noteworthy |
| 0-3 years | 100+ citations | Highly Influential |
| 3-7 years | 100+ citations | Significant |
| 3-7 years | 500+ citations | Landmark Paper |
| 7+ years | 500+ citations | Seminal Work |
| 7+ years | 1000+ citations | Foundational |

### Journal and Venue Tiers

Prioritize papers from higher-tier venues:

- **Tier 1 (Always Prefer):** Nature, Science, Cell, NEJM, Lancet, JAMA, PNAS, Nature Medicine, Nature Biotechnology
- **Tier 2 (Strong Preference):** High-impact specialized journals (IF>10), top conferences (NeurIPS, ICML for ML/AI)
- **Tier 3 (Include When Relevant):** Respected specialized journals (IF 5-10)
- **Tier 4 (Use Sparingly):** Lower-impact peer-reviewed venues

### Author Reputation Assessment

Prefer papers from:
- **Senior researchers** with high h-index (>40 in established fields)
- **Leading research groups** at recognized institutions (Harvard, Stanford, MIT, Oxford, etc.)
- **Authors with multiple Tier-1 publications** in the relevant field
- **Researchers with recognized expertise** (awards, editorial positions, society fellows)

### Identifying Seminal Papers

For any topic, identify foundational work by:
1. **High citation count** (typically 500+ for papers 5+ years old)
2. **Frequently cited by other included studies** (appears in many reference lists)
3. **Published in Tier-1 venues** (Nature, Science, Cell family)
4. **Written by field pioneers** (often cited as establishing concepts)

---

## Best Practices

### 1. Planning Phase
- Register protocol (PROSPERO for systematic reviews)
- Define clear research questions (PICO framework)
- Specify inclusion/exclusion criteria a priori
- Plan synthesis approach before seeing results

### 2. Search Strategy
- Use multiple databases (minimum 3): Ensures comprehensive coverage
- Include preprint servers: Captures latest unpublished findings
- Include grey literature (conference proceedings, dissertations)
- Hand-search key journals for completeness
- Document everything: Search strings, dates, result counts for reproducibility
- Test and refine: Run pilot searches, review results, adjust search terms
- Sort by citations: When available, sort results by citation count to surface influential work first

### 3. Screening and Selection
- Use clear criteria: Document inclusion/exclusion criteria before screening
- Screen systematically: Title → Abstract → Full text
- Document exclusions: Record reasons for excluding studies
- Consider dual screening: For systematic reviews, have two reviewers screen independently

### 4. Data Extraction
- Use standardized extraction forms
- Dual extraction for quality control
- Extract both results and methods details
- Contact authors for missing data
- Track extraction decisions

### 5. Synthesis
- Organize thematically: Group by themes, NOT by individual studies
- Synthesize across studies: Compare, contrast, identify patterns
- Choose method appropriate for data type (narrative, thematic, meta-analytic)
- Test for heterogeneity before pooling quantitative data
- Explore sources of heterogeneity
- Conduct sensitivity analyses
- Be critical: Evaluate quality and consistency of evidence
- Identify gaps: Note what's missing or understudied
- Assess publication bias (funnel plots for meta-analysis)

### 6. Quality and Reproducibility
- Assess study quality: Use appropriate quality assessment tools
- GRADE the overall body of evidence
- Verify all citations: Run verify_citations.py script
- Document methodology: Provide enough detail for others to reproduce
- Follow guidelines: Use PRISMA for systematic reviews

### 7. Writing
- Be objective: Present evidence fairly, acknowledge limitations
- Be systematic: Follow structured template
- Be specific: Include numbers, statistics, effect sizes where available
- Be clear: Use clear headings, logical flow, thematic organization
- Follow PRISMA guidelines for structure

---

## Common Pitfalls to Avoid

1. **Single database search**: Misses relevant papers; always search 3+ databases
2. **No search documentation**: Makes review irreproducible; document all searches
3. **Study-by-study summary**: Lacks synthesis; organize thematically instead
4. **Unverified citations**: Leads to errors; always run verify_citations.py
5. **Too broad search**: Yields thousands of irrelevant results; refine with specific terms
6. **Too narrow search**: Misses relevant papers; include synonyms and related terms
7. **Ignoring preprints**: Misses latest findings; include bioRxiv, medRxiv, arXiv
8. **No quality assessment**: Treats all evidence equally; assess and report quality
9. **Publication bias**: Only positive results published; note potential bias, search grey literature
10. **Outdated search**: Field evolves rapidly; clearly state search date
11. **Scope creep**: Question becomes too broad during review; return to original PICO
12. **Data overload**: Drowning in extracted information; use structured extraction tools
13. **Apples to oranges**: Pooling incomparable studies; use narrative synthesis or subgroup analysis
14. **Cherry picking**: Selective reporting of themes; use systematic coding, frequency analysis
15. **Failing to report null findings**: Report all results regardless of direction

---

## Templates and Tools

### PRISMA Flow Diagram Template
```
Records identified through database searching (n=)
  ↓
Records after duplicates removed (n=)
  ↓
Records screened (n=) → Records excluded (n=)
  ↓
Full-text assessed (n=) → Full-text excluded, with reasons (n=)
  ↓
Studies included in synthesis (n=)
  ↓
Studies included in meta-analysis (n=)
```

### Synthesis Summary Table
```markdown
| Study | Year | Design | N | Population | Outcome | Effect Size | Quality |
|-------|------|--------|---|------------|---------|-------------|---------|
| A     | 2023 | RCT    |100| Adults     | Anxiety | d=-0.45     | High    |
| B     | 2022 | Cohort |250| Teens      | Anxiety | d=-0.32     | Medium  |
```

---

## Example Workflow

Complete workflow for a biomedical literature review:

```bash
# 1. Create review document from template
cp assets/review_template.md crispr_sickle_cell_review.md

# 2. Search multiple databases using appropriate skills
# - Use gget skill for PubMed, bioRxiv
# - Use direct API access for arXiv, Semantic Scholar
# - Export results in JSON format

# 3. Aggregate and process results
python scripts/search_databases.py combined_results.json \
  --deduplicate \
  --rank citations \
  --year-start 2015 \
  --year-end 2024 \
  --format markdown \
  --output search_results.md \
  --summary

# 4. Screen results and extract data
# - Manually screen titles, abstracts, full texts
# - Extract key data into the review document
# - Organize by themes

# 5. Write the review following template structure
# - Introduction with clear objectives
# - Detailed methodology section
# - Results organized thematically
# - Critical discussion
# - Clear conclusions

# 6. Verify all citations
python scripts/verify_citations.py crispr_sickle_cell_review.md

# Review the citation report
cat crispr_sickle_cell_review_citation_report.json

# Fix any failed citations and re-verify
python scripts/verify_citations.py crispr_sickle_cell_review.md

# 7. Generate professional PDF
python scripts/generate_pdf.py crispr_sickle_cell_review.md \
  --citation-style nature \
  --output crispr_sickle_cell_review.pdf

# 8. Review final PDF and markdown outputs
```

---

## Integration with Other Skills

This skill works seamlessly with other scientific skills:

### Database Access Skills
- **gget**: PubMed, bioRxiv, COSMIC, AlphaFold, Ensembl, UniProt
- **bioservices**: ChEMBL, KEGG, Reactome, UniProt, PubChem
- **datacommons-client**: Demographics, economics, health statistics

### Analysis Skills
- **pydeseq2**: RNA-seq differential expression (for methods sections)
- **scanpy**: Single-cell analysis (for methods sections)
- **anndata**: Single-cell data (for methods sections)
- **biopython**: Sequence analysis (for background sections)

### Visualization Skills
- **matplotlib**: Generate figures and plots for review
- **seaborn**: Statistical visualizations

### Writing Skills
- **brand-guidelines**: Apply institutional branding to PDF
- **internal-comms**: Adapt review for different audiences

### Research Methods Skills
- **research-design**: Formulating synthesis research questions
- **data-analysis**: Statistical techniques for meta-analysis
- **quantitative-methods**: Understanding primary study methods
- **qualitative-methods**: Synthesizing qualitative findings
- **research-writing**: Writing synthesis reports and reviews
- **data-collection**: Systematic search and extraction protocols

---

## Resources

### Bundled Resources

**Scripts:**
- `scripts/verify_citations.py`: Verify DOIs and generate formatted citations
- `scripts/generate_pdf.py`: Convert markdown to professional PDF
- `scripts/search_databases.py`: Process, deduplicate, and format search results

**References:**
- `references/citation_styles.md`: Detailed citation formatting guide (APA, Nature, Vancouver, Chicago, IEEE)
- `references/database_strategies.md`: Comprehensive database search strategies

**Assets:**
- `assets/review_template.md`: Complete literature review template with all sections

### External Resources

**Guidelines:**
- PRISMA (Systematic Reviews): http://www.prisma-statement.org/
- Cochrane Handbook: https://training.cochrane.org/handbook
- AMSTAR 2 (Review Quality): https://amstar.ca/
- PROSPERO (Protocol Registration): https://www.crd.york.ac.uk/prospero/

**Tools:**
- MeSH Browser: https://meshb.nlm.nih.gov/search
- PubMed Advanced Search: https://pubmed.ncbi.nlm.nih.gov/advanced/
- Boolean Search Guide: https://www.ncbi.nlm.nih.gov/books/NBK3827/

**Citation Styles:**
- APA Style: https://apastyle.apa.org/
- Nature Portfolio: https://www.nature.com/nature-portfolio/editorial-policies/reporting-standards
- NLM/Vancouver: https://www.nlm.nih.gov/bsd/uniform_requirements.html

---

## Dependencies

### Required Python Packages
```bash
pip install requests  # For citation verification
```

### Required System Tools
```bash
# For PDF generation
brew install pandoc  # macOS
apt-get install pandoc  # Linux

# For LaTeX (PDF generation)
brew install --cask mactex  # macOS
apt-get install texlive-xetex  # Linux
```

Check dependencies:
```bash
python scripts/generate_pdf.py --check-deps
```

---

## Summary

This literature-review-synthesis skill provides:

1. **Systematic methodology** following academic best practices
2. **Multi-database integration** via existing scientific skills
3. **Diverse synthesis approaches** (narrative, thematic, meta-analytic, evidence mapping)
4. **Evidence quality assessment** using GRADE and study-specific tools
5. **Citation verification** ensuring accuracy and credibility
6. **Professional output** in markdown and PDF formats
7. **Comprehensive guidance** covering the entire review process
8. **Quality assurance** with verification and validation tools
9. **Reproducibility** through detailed documentation requirements

Conduct thorough, rigorous literature reviews that meet academic standards and provide comprehensive synthesis of current knowledge in any domain.
