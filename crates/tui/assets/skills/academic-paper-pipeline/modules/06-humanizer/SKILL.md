---
name: writing-humanizer
description: 中英文写作人工化与 AI 痕迹消除。当用户说"去 AI 味"、"降 AIGC 检测率"、"remove AI writing patterns"、"remove AI-isms"、"humanize"、"让文字不像 AI 写的"、"知网 AIGC"、"降重"、"论文查重 AI"、"改得像人写的"、"去 AI 痕迹"、"audit for AI tells"、"clean up AI writing"、"人工化重写"时触发。支持两类工作流：(1) 英文 AI 模式检测与重写（481 行规则库，覆盖词汇/句式/结构/节奏）；(2) 中文学术 AIGC 降检测率（17 类中文 AI 痕迹模式，五步闭环工作流）；(3) 标点符号审查（统计破折号数量，零容忍）。
---

# Writing Humanizer — AI Writing Pattern Removal

本技能合并了英文 AI 写作模式移除和中文学术降 AIGC 检测两个方向的能力，并加入标点符号审查。

**选择工作流：**
- **英文** → 使用 Part 1（避免 AI 写作模式）
- **中文** → 使用 Part 2（中文学术降 AIGC）
- **标点审查** → 两种语言都使用 Part 3

---

# Part 1: Avoid AI Writing — Audit & Rewrite

You are editing content to remove AI writing patterns ("AI-isms") that make text sound machine-generated.

## Modes

This skill operates in one of two modes:

**`rewrite`** (default) — Flag AI-isms and rewrite the text to fix them.

**`detect`** — Flag AI-isms only. No rewriting. Use this mode when:
- The writer wants to see what's flagged and decide what to fix themselves
- The flagged patterns might be intentional (AI patterns aren't always bad — they can be effective in small doses)
- You're auditing text you don't want altered (published content, someone else's writing, reference material)
- You want a quick scan without waiting for a full rewrite

Trigger detect mode when the user says "detect," "flag only," "audit only," "just flag," "scan," "what AI patterns are in this," or similar. Default to rewrite mode if not specified.

---

In **rewrite** mode, your job is to:

1. **Audit it**: identify every AI-ism present, citing the specific text
2. **Rewrite it**: return a clean version with all AI-isms removed
3. **Show a diff summary**: briefly list what you changed and why

In **detect** mode, your job is to:

1. **Audit it**: identify every AI-ism present, citing the specific text
2. **Assess it**: note which flags are clear problems vs. patterns that may be intentional or effective in context

---

## What to remove or fix

### Formatting
- **Em dashes (— and --)**: Replace with commas, periods, parentheses, or rewrite as two sentences. Target: zero. Hard max: one per 1,000 words. This applies to headings and section titles too, not just body prose. Catch both the Unicode em dash (—) and the double-hyphen substitute (--).
- **Bold overuse**: Strip bold from most phrases. One bolded phrase per major section at most, or none. If something's important enough to bold, restructure the sentence to lead with it instead.
- **Emoji in headers**: Remove entirely. No `## 🚀 What This Means`. Exception: social posts may use one or two emoji sparingly — at the end of a line, never mid-sentence.
- **Excessive bullet lists**: Convert bullet-heavy sections into prose paragraphs. Bullets only for genuinely list-like content (feature comparisons, step-by-step instructions, API parameters).

### Sentence structure
- **"It's not X — it's Y" / "This isn't about X, it's about Y"**: Rewrite as a direct positive statement. Max one per piece, and only if it serves the argument.
- **Hollow intensifiers**: Cut `genuine`, `real` (as in "a real improvement"), `truly`, `quite frankly`, `to be honest`, `let's be clear`, `it's worth noting that`. Just state the fact.
- **Vague endorsement ("worth [verb]ing")**: Cut or replace `worth reading`, `worth paying attention to`, `worth a look`, `worth exploring`, `worth checking out`, `worth your time`. These substitute a generic thumbs-up for a specific reason. Say *why* something matters instead.
- **Hedging**: Cut `perhaps`, `could potentially`, `it's important to note that`, `to be clear`. Make the point directly.
- **Missing bridge sentences**: Each paragraph should connect to the last. If paragraphs could be rearranged without the reader noticing, add connective tissue.
- **Compulsive rule of three**: Vary groupings. Use two items, four items, or a full sentence instead of triads. Max one "adjective, adjective, and adjective" pattern per piece.

### Words and phrases to replace

Words are organized into three tiers based on how reliably they signal AI-generated text. This tiered approach — adapted from [brandonwise/humanizer](https://github.com/brandonwise/humanizer)'s vocabulary research — reduces false positives on words that are fine in isolation but suspicious in clusters.

- **Tier 1 — Always flag.** These words appear 5–20x more often in AI text than human text. Replace on sight.
- **Tier 2 — Flag in clusters.** Individually fine, but two or more in the same paragraph is a strong AI signal. Flag when they appear together.
- **Tier 3 — Flag by density.** Common words that AI simply overuses. Only flag when they make up a noticeable fraction of the text (roughly 3%+ of total words).

#### Tier 1 — Always replace

| Replace | With |
|---|---|
| delve / delve into | explore, dig into, look at |
| landscape (metaphor) | field, space, industry, world |
| tapestry | (describe the actual complexity) |
| realm | area, field, domain |
| paradigm | model, approach, framework |
| embark | start, begin |
| beacon | (rewrite entirely) |
| testament to | shows, proves, demonstrates |
| robust | strong, reliable, solid |
| comprehensive | thorough, complete, full |
| cutting-edge | latest, newest, advanced |
| leverage (verb) | use |
| pivotal | important, key, critical |
| underscores | highlights, shows |
| meticulous / meticulously | careful, detailed, precise |
| seamless / seamlessly | smooth, easy, without friction |
| game-changer / game-changing | describe what specifically changed and why it matters |
| hit differently / hits different | (say what specifically changed, or cut) |
| utilize | use |
| watershed moment | turning point, shift (or describe what changed) |
| marking a pivotal moment | (state what happened) |
| the future looks bright | (cut — say something specific or nothing) |
| only time will tell | (cut — say something specific or nothing) |
| nestled | is located, sits, is in |
| vibrant | (describe what makes it active, or cut) |
| thriving | growing, active (or cite a number) |
| despite challenges… continues to thrive | (name the challenge and the response, or cut) |
| showcasing | showing, demonstrating (or cut the clause) |
| deep dive / dive into | look at, examine, explore |
| unpack / unpacking | explain, break down, walk through |
| bustling | busy, active (or cite what makes it busy) |
| intricate / intricacies | complex, detailed (or name the specific complexity) |
| complexities | (name the actual complexities, or use "problems" / "details") |
| ever-evolving | changing, growing (or describe how) |
| enduring | lasting, long-running (or cite how long) |
| daunting | hard, difficult, challenging |
| holistic / holistically | complete, full, whole (or describe what's included) |
| actionable | practical, useful, concrete |
| impactful | effective, significant (or describe the impact) |
| learnings | lessons, findings, takeaways |
| thought leader / thought leadership | expert, authority (or describe their actual contribution) |
| best practices | what works, proven methods, standard approach |
| at its core | (cut — just state the thing) |
| synergy / synergies | (describe the actual combined effect) |
| interplay | relationship, connection, interaction |
| in order to | to |
| due to the fact that | because |
| serves as | is |
| features (verb) | has, includes |
| boasts | has |
| presents (inflated) | is, shows, gives |
| commence | start, begin |
| ascertain | find out, determine, learn |
| endeavor | effort, attempt, try |
| keen (as intensifier) | interested, eager, enthusiastic (or cut — just state the interest) |
| symphony (metaphor) | (describe the actual coordination or combination) |
| embrace (metaphor) | adopt, accept, use, switch to |

#### Tier 2 — Flag when 2+ appear in the same paragraph

These words are legitimate on their own. When two or more show up together, the paragraph likely needs a rewrite.

| Replace | With |
|---|---|
| harness | use, take advantage of |
| navigate / navigating | work through, handle, deal with |
| foster | encourage, support, build |
| elevate | improve, raise, strengthen |
| unleash | release, enable, unlock |
| streamline | simplify, speed up |
| empower | enable, let, allow |
| bolster | support, strengthen, back up |
| spearhead | lead, drive, run |
| resonate / resonates with | connect with, appeal to, matter to |
| revolutionize | change, transform, reshape (or describe what changed) |
| facilitate / facilitates | enable, help, allow, run |
| underpin | support, form the basis of |
| nuanced | specific, subtle, detailed (or name the actual nuance) |
| crucial | important, key, necessary |
| multifaceted | (describe the actual facets, or cut) |
| ecosystem (metaphor) | system, community, network, market |
| myriad | many, numerous (or give a number) |
| plethora | many, a lot of (or give a number) |
| encompass | include, cover, span |
| catalyze | start, trigger, accelerate |
| reimagine | rethink, redesign, rebuild |
| galvanize | motivate, rally, push |
| augment | add to, expand, supplement |
| cultivate | build, develop, grow |
| illuminate | clarify, explain, show |
| elucidate | explain, clarify, spell out |
| juxtapose | compare, contrast, set side by side |
| paradigm-shifting | (describe what actually shifted) |
| transformative / transformation | (describe what changed and how) |
| cornerstone | foundation, basis, key part |
| paramount | most important, top priority |
| poised (to) | ready, set, about to |
| burgeoning | growing, emerging (or cite a number) |
| nascent | new, early-stage, emerging |
| quintessential | typical, classic, defining |
| overarching | main, central, broad |
| underpinning / underpinnings | basis, foundation, what supports |

#### Tier 3 — Flag only at high density

These are normal words. Only flag them when the text is saturated with them — a sign that AI filled space with vague praise instead of specifics.

| Word | What to do |
|---|---|
| significant / significantly | Replace some with specifics: numbers, comparisons, examples |
| innovative / innovation | Describe what's actually new |
| effective / effectively | Say how or cite a metric |
| dynamic / dynamics | Name the actual forces or changes |
| scalable / scalability | Describe what scales and to what |
| compelling | Say why it compels |
| unprecedented | Name the precedent it breaks (or cut) |
| exceptional / exceptionally | Cite what makes it an exception |
| remarkable / remarkably | Say what's worth remarking on |
| sophisticated | Describe the sophistication |
| instrumental | Say what role it played |
| world-class / state-of-the-art / best-in-class | Cite a benchmark or comparison |

### Template phrases (avoid)

These slot-fill constructions signal that a sentence was generated, not written. If a phrase has a blank where a noun or adjective could go and still sound the same, it's too generic.

- "a [adjective] step towards [adjective] AI infrastructure" → describe the specific capability, benchmark, or outcome
- "a [adjective] step forward for [noun]" → same rule: say what actually changed
- "Whether you're [X] or [Y]" → false-breadth construction. Pick the audience you're actually addressing, or cut. "Whether you're a startup founder or an enterprise architect" means nothing — it's just "everyone."
- "I recently had the pleasure of [verb]-ing" → review/social AI pattern. Just say what happened: "I talked to," "I read," "I attended."

### Transition phrases to remove or rewrite
- "Moreover" / "Furthermore" / "Additionally" → restructure so the connection is obvious, or use "and," "also," "on top of that"
- "In today's [X]" / "In an era where" → cut or state specific context
- "It's worth noting that" / "Notably" → just state the fact
- "Here's what's interesting" / "Here's what caught my eye" / "Here's what stood out" → reader-steering frames. Let the content signal its own importance. If you need a lead-in, make it specific: "The revenue number matters because..." not "Here's the interesting part."
- "In conclusion" / "In summary" / "To summarize" → your conclusion should be obvious
- "When it comes to" → just talk about the thing directly
- "At the end of the day" → cut
- "That said" / "That being said" → cut or use "but," "yet," or "however." Don't overuse any one of them.

### Structural issues
- **Uniform paragraph length**: Vary deliberately. Include some 1-2 sentence paragraphs and some longer ones. If every paragraph is roughly the same size, fix it.
- **Formulaic openings**: If the piece opens with broad context before getting to the point ("In the rapidly evolving world of..."), rewrite to lead with the news or the insight. Context can come second.
- **Suspiciously clean grammar**: Don't sand away all personality. Deliberate fragments, sentences starting with "And" or "But," comma splices for effect: if the natural voice uses them, keep them.

### Significance inflation
- Phrases like "marking a pivotal moment in the evolution of..." or "a watershed moment for the industry" inflate routine events into history-making ones. State what happened and let the reader judge significance.
- If the sentence still works after you delete the inflation clause, delete it.

### Copula avoidance
- AI text avoids "is" and "has" by substituting fancier verbs: "serves as," "features," "boasts," "presents," "represents." These sound like a press release.
- Default to "is" or "has" unless a more specific verb genuinely adds meaning.

### Synonym cycling
- AI rotates synonyms to avoid repeating a word: "developers… engineers… practitioners… builders" in the same paragraph. Human writers repeat the clearest word.
- If the same noun or verb appears three times in a paragraph and that's the right word, keep all three. Forced variation reads as thesaurus abuse.

### Vague attributions
- "Experts believe," "Studies show," "Research suggests," "Industry leaders agree" — without naming the expert, study, or leader. Either cite a specific source or drop the attribution and state the claim directly.

### Filler phrases
- Strip mechanical padding that adds words without meaning:
  - "It is important to note that" → (just state it)
  - "In terms of" → (rewrite)
  - "The reality is that" → (cut or just state the claim)
- Note: "In order to," "Due to the fact that," and "At the end of the day" are covered in the word/phrase table and transition sections above — don't duplicate rules.

### Generic conclusions
- "The future looks bright," "Only time will tell," "One thing is certain," "As we move forward" — these are filler disguised as conclusions. Cut them. If the piece needs a closing thought, make it specific to the argument.

### Chatbot artifacts
- "I hope this helps!", "Certainly!", "Absolutely!", "Great question!", "Feel free to reach out," "Let me know if you need anything else" — these are conversational tics from chat interfaces, not writing. Remove entirely.
- Also watch for: "In this article, we will explore…" or "Let's dive in!" — these are AI-generated meta-narration. Cut or rewrite with a direct opening.

### "Let's" constructions
- "Let's explore," "Let's take a look," "Let's break this down," "Let's examine" — AI uses "let's" as a false-collaborative opener to ease into a topic. It's filler that delays the actual point. Just start with the point. "Let's dive in" is covered above under chatbot artifacts, but the pattern is broader than that — flag any "let's + verb" that's functioning as a transition rather than a genuine invitation to act.

### Notability name-dropping
- AI text piles on prestigious citations to manufacture credibility: "cited in The New York Times, BBC, Financial Times, and The Hindu." If a source matters, use it with context: "In a 2024 NYT interview, she argued..." One specific reference beats four name-drops.

### Superficial -ing analyses
- Strings of present participles used as pseudo-analysis: "symbolizing the region's commitment to progress, reflecting decades of investment, and showcasing a new era of collaboration." These say nothing. Replace with specific facts or cut entirely.

### Promotional language
- AI defaults to tourism-brochure prose: "nestled within the breathtaking foothills," "a vibrant hub of innovation," "a thriving ecosystem." Replace with plain description: "is a town in the Gonder region," "has 12 startups." If you wouldn't say it in conversation, cut it.

### Formulaic challenges
- "Despite challenges, [subject] continues to thrive" or "While facing headwinds, the organization remains resilient." This is a non-statement. Name the actual challenge and the actual response, or cut the sentence.

### False ranges
- AI creates false breadth by pairing unrelated extremes: "from the Big Bang to dark matter," "from ancient civilizations to modern startups." These sound sweeping but say nothing. List the actual topics or pick the one that matters.

### Inline-header lists
- Bullet lists where each item starts with a bold header that repeats itself: "**Performance:** Performance improved by..." Strip the bold header and write the point directly. If the list items need headers, they should probably be paragraphs.

### Title case headings
- AI over-capitalizes headings: "Strategic Negotiations And Key Partnerships" instead of "Strategic negotiations and key partnerships." Use sentence case for subheadings. Title case only for the piece's main title, if at all.

### Cutoff disclaimers
- "While specific details are limited based on available information," "As of my last update," "I don't have access to real-time data." These are model limitations leaking into prose. Either find the information or remove the hedge. Never publish a sentence that admits the writer didn't look something up.

### Novelty inflation
- AI text treats established concepts as if the speaker invented or discovered them: "He introduced a term," "She coined the phrase," "a concept nobody's naming," "a failure mode nobody talks about." In reality, most ideas in a conversation are applications of existing concepts, not inventions.
- Two problems. First, it's factually risky: if the concept already has a Wikipedia page or conference talks from last year, claiming novelty makes the writer look uninformed. Second, it flatters the subject in a way that reads as promotional rather than analytical.
- The fix: describe what the person *did with* the concept, not that they discovered it. "Michel walked through how context poisoning works in practice" instead of "Michel introduced a term I hadn't heard before: context poisoning." If you're unsure whether something is novel, assume it isn't and frame accordingly.
- Related patterns to flag: "the failure mode nobody's naming," "a problem nobody talks about," "the insight everyone's missing," "what nobody tells you about." These are engagement-bait framings that claim scarcity of knowledge where none exists.

### Emotional flatline
- AI claims emotions as a structural crutch without conveying them through the writing: "What surprised me most," "I was fascinated to discover," "What struck me was," "I was excited to learn," "The most interesting part."
- Two problems. First, it's tell-don't-show: if the thing is genuinely surprising, the reader should feel that from the content, not from the writer announcing it. Second, these phrases are massively overused as list introductions and transitions. They're filler wearing an emotion costume.
- This pattern isn't always AI. It's also a sign of lazy human writing on autopilot. Flag it either way.
- The fix isn't "never say surprised." It's: if you claim an emotion, the writing around it should earn it. Otherwise cut the claim and present the thing directly.
- Related pattern: "hit differently" / "hits different." AI uses trendy colloquialisms as a shortcut to sound relatable without earning the emotional beat. If something genuinely affected you, describe how. Otherwise cut.

### False concession structure
- "While X is impressive, Y remains a challenge" or "Although X has made strides, Y is still an open question." AI uses this to sound balanced without actually weighing anything. Both halves are vague. Either make the concession specific (name what's impressive, name the actual challenge) or pick a side and argue it.

### Rhetorical question openers
- "But what does this mean for developers?" / "So why should you care?" / "What's next?" — AI uses rhetorical questions to stall before the actual point. If you know the answer, just say it. Rhetorical questions are earned by strong setup, not dropped as section transitions.

### Parenthetical hedging
- "(and, increasingly, Z)" / "(or, more precisely, Y)" / "(and perhaps more importantly, W)" — AI inserts parenthetical asides to sound nuanced without committing. If the aside matters, give it its own sentence. If it doesn't, cut it.

### Numbered list inflation
- "Three key takeaways" / "Five things to know" / "Here are the top seven" — AI defaults to numbered lists because they're structurally safe. Only use numbered lists when the content genuinely has that many discrete, parallel items. If you're padding to hit a number, the list shouldn't exist.

### Reasoning chain artifacts
- "Let me think step by step," "Breaking this down," "To approach this systematically," "Step 1:," "Here's my thought process," "First, let's consider," "Working through this logically" — these are artifacts of chain-of-thought reasoning leaking into published prose. The reader doesn't need to see the scaffolding. State the conclusion, then the evidence.
- Also watch for numbered reasoning steps that read like an internal monologue rather than an argument meant for an audience.

### Sycophantic tone
- "Great question!", "Excellent point!", "You're absolutely right!", "That's a really insightful observation" — these are conversational rewards from chat interfaces, not writing. Remove entirely.
- Distinct from chatbot artifacts: sycophancy specifically validates the reader/questioner rather than just performing helpfulness.

### Acknowledgment loops
- "You're asking about," "The question of whether," "To answer your question," "That's a great question. The..." — AI restates the prompt before answering. In writing, this is pure filler. The reader knows what they asked. Just answer.
- Related pattern: opening a section by summarizing what the previous section said. If the structure is clear, the reader doesn't need a recap.

### Confidence calibration phrases
- "It's worth noting that," "Interestingly," "Surprisingly," "Importantly," "Significantly," "Notably," "Certainly," "Undoubtedly," "Without a doubt" — AI uses these to signal how the reader should feel about a fact instead of letting the fact speak for itself.
- "Here's what's interesting," "Here's the interesting part," "Here are the parts I found interesting" — reader-steering cue that pre-interprets importance. Works when followed by genuinely surprising data; fails when it introduces a restatement of something obvious (which is the AI default).
- One "notably" in a 2,000-word piece is fine. Three in 500 words is AI-style emphasis stacking. Flag by density.

### Excessive structure
- Too many headers in short text: more than 3 headings in under 300 words is almost always AI trying to look organized. Merge sections or use prose transitions instead.
- Too many list items: 8+ bullet points in under 200 words means the content should be a paragraph, not a list.
- Formulaic section headers: "Overview," "Key Points," "Summary," "Conclusion," "Introduction" — these are default AI scaffolding. Use headers that tell the reader something specific about what follows.

### Rhythm and uniformity

These aren't individual word or phrase problems — they're patterns in how the text flows as a whole. AI text is metronomic; human text has varied rhythm.

**Structure is the #1 detection signal.** AI detection tools (including Pangram, which trains a classifier on 28M human documents) weight structural regularity higher than vocabulary. Consistent sentence construction, uniform pacing, and symmetrical phrasing patterns are harder to mask than swapping out a few flagged words. If you fix every word on the Tier 1 list but leave the rhythm untouched, the text still reads as AI-generated.

- **Sentence length uniformity**: If most sentences are 15–25 words, the text sounds robotic. Mix short punchy sentences (3–8 words) with longer flowing ones (20+). Fragments work. Questions break the monotony.
- **Paragraph length uniformity**: If every paragraph is 3–5 sentences and roughly the same size, vary deliberately. Some paragraphs should be one sentence. Some should be longer.
- **Vocabulary repetition vs. synonym cycling**: AI either repeats the same word mechanically or cycles through synonyms conspicuously. Human writers repeat when the word is right and vary when it's natural — there's no formula.
- **Read-aloud test**: If the text sounds like it could be read by a text-to-speech engine without sounding weird, it's probably too uniform. Human writing has rhythm that resists robotic delivery.
- **Missing first-person perspective**: Where appropriate, the writer should have opinions, preferences, and reactions. AI is relentlessly neutral. If the piece is supposed to have a voice, the absence of "I think," "in my experience," or a stated preference is itself an AI tell.
- **Over-polishing**: Aggressively editing out every irregularity can push human writing *toward* AI statistical profiles. Natural disfluency, idiosyncratic word choices, and uneven pacing are what keep text out of the "AI-generated" classification. Don't sand away all personality in pursuit of clean prose. This skill should make writing sound more human, not less — if you apply every rule at maximum strictness, you risk creating the very uniformity you're trying to avoid.

### When to rewrite from scratch vs. patch

If the text has 5+ flagged vocabulary hits across multiple categories, 3+ distinct pattern categories triggered, and uniform sentence/paragraph length, patching individual phrases won't fix it — the structure itself is AI-generated. Advise a full rewrite: state the core point in one sentence, then rebuild from there.

---

## Severity tiers

Not all AI-isms are equal. When doing a quick pass or triaging a large document, prioritize by tier:

### P0 — Credibility killers (fix immediately)
- Cutoff disclaimers ("As of my last update")
- Chatbot artifacts ("I hope this helps!", "Great question!")
- Vague attributions without sources ("Experts believe")
- Significance inflation on routine events

### P1 — Obvious AI smell (fix before publishing)
- Word-list violations (delve, leverage, harness, robust, etc.)
- Template phrases and slot-fill constructions
- "Let's" transition openers
- Synonym cycling within a paragraph
- Formulaic openings ("In the rapidly evolving world of...")
- Bold overuse
- Em dash frequency (above 1 per 1,000 words)

### P2 — Stylistic polish (fix when time allows)
- Generic conclusions ("The future looks bright")
- Compulsive rule of three
- Uniform paragraph length
- Copula avoidance (serves as, features, boasts)
- Transition phrases (Moreover, Furthermore, Additionally)

Use P0+P1 for quick passes. Full audit covers all three tiers.

---

## Self-reference escape hatch

When writing *about* AI writing patterns (blog posts, tutorials, skill documentation like this file), quoted examples are exempt from flagging. Text inside quotation marks, code blocks, or explicitly marked as illustrative ("for example, AI might write...") should not be rewritten. Only flag patterns that appear in the author's own prose, not in cited examples of bad writing.

---

## Context profiles

Pass an optional context hint to adjust rule strictness. If no context is specified, auto-detect from content cues (short + hashtags = social, code blocks = technical, salutation = email, default = blog).

### Profile definitions

**`linkedin`** — Short-form social. Punchy fragments, visual formatting matter.
**`blog`** — Default. Standard long-form prose. All rules apply at full strength.
**`technical-blog`** — Long-form with code, architecture, APIs. Technical terms get a pass.
**`investor-email`** — High-trust audience. Tighten everything; promotional language is the biggest risk.
**`docs`** — Documentation, READMEs, guides. Clarity over voice.
**`casual`** — Slack messages, internal notes, quick replies. Only catch the worst offenders.

### Tolerance matrix

Rules not listed in the table apply at full strength across all profiles.

| Rule | linkedin | blog | technical-blog | investor-email | docs | casual |
|------|----------|------|----------------|----------------|------|--------|
| Em dashes | relaxed (2/post OK) | strict | strict | strict | relaxed | skip |
| Bold overuse | relaxed (bold hooks OK) | strict | strict | strict | relaxed | skip |
| Emoji in headers | relaxed (1-2 end-of-line OK) | strict | strict | strict | skip | skip |
| Excessive bullets | skip (lists work on LinkedIn) | strict | relaxed (technical lists OK) | strict | skip (lists are docs) | skip |
| Hedging | strict | strict | relaxed ("may" is accurate in technical) | strict | relaxed | skip |
| Word table (full list) | strict | strict | **partial** (see below) | strict | relaxed | P0 only |
| Promotional language | relaxed (some sell is expected) | strict | strict | **extra strict** | strict | skip |
| Significance inflation | strict | strict | strict | **extra strict** | relaxed | skip |
| Copula avoidance | skip | strict | relaxed | strict | skip | skip |
| Uniform paragraph length | skip (short-form) | strict | strict | strict | relaxed | skip |
| Numbered list inflation | relaxed | strict | relaxed | strict | skip | skip |
| Rhetorical questions | relaxed (1 as hook OK) | strict | strict | strict | strict | skip |
| Transition phrases | skip (short-form) | strict | strict | strict | relaxed | skip |
| Generic conclusions | skip | strict | strict | **extra strict** | skip | skip |

**Technical-blog word table exceptions:** These terms have legitimate technical meaning and should not be flagged in technical context: `robust`, `comprehensive`, `seamless`, `ecosystem`, `leverage` (when discussing actual platform leverage/APIs), `facilitate`, `underpin`, `streamline`. Still flag: `delve`, `tapestry`, `beacon`, `embark`, `testament to`, `game-changer`, `harness`.

**"Extra strict"** means: flag even borderline instances. In investor emails, a single "thriving ecosystem" can undermine the whole message.

**"Skip"** means: don't audit this category for this profile. The rule doesn't apply or isn't worth the edit.

### Auto-detection cues

When no context is specified, infer from these signals:

| Signal | Inferred context |
|--------|-----------------|
| Under 300 words + hashtags or mentions | `linkedin` |
| Code blocks, API references, or technical architecture | `technical-blog` |
| Salutation ("Hi [name]", "Dear") + investor/fundraising language | `investor-email` |
| Step-by-step instructions, parameter docs, README structure | `docs` |
| No strong signals | `blog` (safest default — all rules apply) |

If auto-detection feels wrong, say which profile you're using and why. The user can override.

---

## Output format

### Rewrite mode (default)

Return your response in four sections:

**1. Issues found**
A bulleted list of every AI-ism identified, with the offending text quoted.

**2. Rewritten version**
The full rewritten content. Preserve the original structure, intent, and all specific technical details. Only change what the guidelines require.

**3. What changed**
A brief summary of the major edits made. Not every word, just the meaningful changes.

**4. Second-pass audit**
Re-read the rewritten version from section 2. Identify any remaining AI tells that survived the first pass — recycled transitions, lingering inflation, copula avoidance, filler phrases, or anything else from the categories above. Fix them, return the corrected text inline, and note what changed in this pass. If the rewrite is clean, say so.

### Detect mode

Return your response in two sections:

**1. Issues found**
A bulleted list of every AI-ism identified, with the offending text quoted. Group by severity (P0, P1, P2).

**2. Assessment**
For each flag, note whether it's a clear problem or a judgment call. Some AI-associated patterns are effective writing techniques — uniform paragraph length is a problem, but a well-placed "however" isn't. Call out which flags the writer should definitely fix vs. which ones are worth a second look but might be fine in context. If the text is clean, say so.

---

## Tone calibration

The goal is writing that sounds like a person wrote it. Direct. Specific. The writing should demonstrate confidence, not assert it.

Five principles for human-sounding rewrites:
1. **Vary sentence length** — mix short with long. Fragments are fine.
2. **Be concrete** — replace vague claims with numbers, names, dates, or examples.
3. **Have a voice** — where appropriate, use first person, state preferences, show reactions.
4. **Cut the neutrality** — humans have opinions. If the piece is supposed to take a position, take it.
5. **Earn your emphasis** — don't tell the reader something is interesting. Make it interesting.

If the original writing is already strong, say so and make only the necessary cuts. Don't over-edit for the sake of it.

The replacement table provides defaults, not mandates. If a flagged word is clearly the right choice in context, preserve it.

---

# Part 2: 中文学术降 AIGC

> 中文学术实证论文的 AI 痕迹消除器。不是"改同义词"，不是"打乱语序"，而是**系统性地重构中文 AI 文本的统计学特征**，让它回归到真实研究者写作的语言分布上。

## 适用场景

- 中文期刊投稿前的 AIGC 率自查与修改
- 学位论文（本/硕/博）应对知网 AMLC 检测
- 基金申请书、开题报告、研究报告去 AI 化
- 已完成英文投稿、需要中文摘要/中文版本的场景

## 核心理念

**三个错误的做法**（很多教程都在做但无效）：

1. ❌ **同义词替换**：把"关键"改成"核心"，把"重要"改成"显著"——检测器看的是 n-gram 统计分布，不是单词替换
2. ❌ **句式倒装**：简单把"由于 A，所以 B"改成"B，这是因为 A"——句法模板没变
3. ❌ **全文重写交给 AI**：换另一个 AI 改写，只是从一种 AI 痕迹换成另一种

**本 Skill 的正确路径**：针对**中文 AI 的五大结构性特征**做定点破坏，而非字词层面的表面修改。

## 中文 AI 文本的五大结构性特征

与英文 AI 不同，中文大模型的痕迹主要表现在：

1. **四字套话密度过高**：每 200 字出现 3+ 个"综上所述/不可否认/毋庸置疑/显而易见"之类
2. **虚词与关联词冗余**：过度使用"然而/因此/此外/而且/并且/以及/等等"
3. **主语回避与隐藏被动**：大量"本文认为/本研究发现/相关研究表明"式的模糊主语
4. **句长方差低**：句子长度集中在 20-35 字，缺乏人类写作的节奏跳跃
5. **结论绝对化**：倾向使用"证明了/确立了/充分说明/必然导致"等过强断言

本 Skill 针对上述 5 大特征，提供 **17 类细分诊断规则**（详见 `references/patterns.md`）。

## 五步闭环工作流

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 1. 定位扫描 │ →  │ 2. 诊断分类 │ →  │ 3. 差异化改写 │
└─────────────┘    └─────────────┘    └─────────────┘
                                             │
┌─────────────┐    ┌─────────────┐          │
│ 5. 二次复查 │ ←  │ 4. 五维自评 │ ←────────┘
└─────────────┘    └─────────────┘
```

### Step 1 · 定位扫描

接收用户提交的文本，按 `references/patterns.md` 的 17 类规则做全文扫描，**输出结构化问题清单**：

```markdown
## AI 痕迹定位报告

| 段落 | 原文片段 | 命中规则 | 严重度 |
|------|---------|---------|--------|
| ¶2   | "毋庸置疑，数字化转型..." | P01 四字套话 | 高 |
| ¶3   | "...此外，该研究还..."   | P04 显性连接词 | 中 |
| ¶5   | "本文认为该机制充分证明了..." | P12 绝对化断言 | 高 |
```

⚠️ **不要**此时就开始改写，先让用户/作者看到问题全貌。

### Step 2 · 诊断分类

按**段落功能**分类：

- **事实陈述段**（方法/数据/结果）→ 低调改写，保留学术精确度
- **论证段**（文献综述/讨论/机制分析）→ 重点改写，注入学者的"认识论谨慎"
- **过渡段**（章节衔接、段落承上启下）→ 大幅改写，消除机械连接词

参考 `references/academic-sections.md` 的**分章节策略表**决定每段的改写力度。

### Step 3 · 差异化改写

针对 Step 1 清单里的每一条，按以下四条原则逐一修复：

1. **还原研究者视角**：把"本研究发现"改成具体的"我们用 2015-2023 年 CFPS 面板数据发现"——加入**可验证的颗粒度**
2. **制造句长方差**：在一段 200 字内，强制包含至少 1 个 ≤15 字的短句 + 1 个 ≥50 字的长句
3. **消灭显性连接词**：去掉"此外/而且/因此/从而"这类段首连接词，改用**语义接力**（下一句主语呼应上一句宾语的关键概念）
4. **绝对→谨慎**：所有"证明/确立/必然/充分说明"降级为"为...提供了证据/与...的预期一致/可能意味着"

⚠️ **禁止事项**：
- 禁止为了"显得人类"而刻意制造错别字或不规范表达
- 禁止使用生僻词、文言化词汇强行降困惑度——这会损害学术性
- 禁止篡改数据、结论或研究主张——只改表达，不改实质

### Step 4 · 五维自评

对改写后的文本做**中文学术版 5 维评分**（每维 1-10 分，详见 `references/scoring.md`）：

| 维度 | 检查点 | 权重 |
|------|--------|------|
| **具体性** | 是否用具体数据/案例/作者替代了模糊表达 | 1.5× |
| **节奏性** | 句长方差是否 ≥ 150（50 字长句 + 15 字短句混排） | 1.2× |
| **谨慎性** | 绝对化断言是否已降级为条件化表述 | 1.3× |
| **隐衔接** | 段落之间是否消除了显性关联词（此外/因此等） | 1.0× |
| **研究者语气** | 是否出现"我们/本团队/我"等第一人称研究立场 | 1.0× |

**加权总分 < 35 → 返回 Step 3 再改一轮。加权总分 ≥ 42 → 通过。**

### Step 5 · 二次复查

用"冷读者"视角重新审视全文，执行三项终审：

1. **通顺性复查**：改写后的逻辑链是否依然连贯？有没有为了降 AIGC 伤到学术表达？
2. **事实性复查**：数据、引用、作者名、年份有没有被误改？
3. **风格一致性复查**：全文语气是否统一，有没有出现"改过的段落 vs 没改的段落"断层？

**输出终稿 + 改动摘要**（哪些段落改了多少、为什么改）。

## 参考资料

- `references/patterns.md` — 17 类中文 AI 痕迹模式库（每类含识别规则 + 典型样本）
- `references/examples.md` — 12 组原文/改写前后对比（覆盖实证论文七个主要章节）
- `references/academic-sections.md` — 按章节差异化的改写策略表
- `references/scoring.md` — 五维评分量表细则

## 重要声明

本技能的目标是**让人工写作和 AI 辅助写作的文本回归到真实研究者的语言分布**，而不是"帮 AI 生成内容骗过检测"。

- ✅ 适用：研究者自己写的初稿，被 AIGC 检测误判为高 AI 率
- ✅ 适用：AI 辅助起草 + 研究者人工修改定稿的混合场景
- ❌ 不适用：完全 AI 生成的论文，希望零改动通过检测
- ❌ 不适用：学术不端场景，如代写、抄袭的掩盖

**学术诚信优先于检测率**。任何改写都不应触及研究结论、数据真实性、引用准确性。

---

# Part 3: 标点符号审查 — 破折号统计

本文档需要**强制执行的标点符号规则**，适用于 Part 1（英文）和 Part 2（中文）的所有输出。

## 破折号零容忍规则

**规则**：文章/论文中**禁止出现任何中文破折号（——，即 U+2014 U+2014 连续两个 em dash）**。

**适用范围**：学术论文、期刊文章、正式报告、博客文章、技术文档等所有书面输出。

**例外**：不适用于以下场景——
- 代码注释中的破折号
- 引文中的原文破折号（但建议标注[原文如此]）
- 直接讨论标点符号用法的语言学文本

## 执行流程

在每次输出最终文本前，必须执行以下步骤：

### 步骤 1：统计破折号数量

用以下方式统计文本中的中文破折号：

```python
import re
text = "待检查的文本内容..."
dashes = re.findall(r'——', text)  # 匹配连续两个 U+2014
count = len(dashes)
```

或手动搜索 `——`（连续两个中文 em dash）。

### 步骤 2：报告结果

在输出中明确包含：

```
## 标点符号审查
- 中文破折号（——）出现次数：{count}
- 判定：{'❌ 未通过' if count > 0 else '✅ 通过'}
```

### 步骤 3：修复

如果发现破折号，按以下优先级替换：

1. **用逗号分隔**：如果破折号表示语气的停顿或转折，替换为逗号
2. **用句号分句**：如果破折号前后的内容相对独立，拆分为两个句子
3. **用冒号引出**：如果破折号表示解释或举例，替换为冒号
4. **用括号补充**：如果破折号表示插入语，替换为括号
5. **重构句子**：如果以上都不合适，调整语序消除对破折号的依赖

## 与其他规则的关系

- Part 1 中也有英文 em dash 规则（"Em dashes (— and --): Target: zero. Hard max: one per 1,000 words"），那是针对英文单破折号的规则
- 本文针对**中文双破折号 `——`**，这是中文学术写作中的常见 AI 痕迹
- 两者独立执行：英文文本检查 `—` 和 `--`，中文文本检查 `——`，中英混杂文本两者都检查
