---
name: academic-plotting
description: >
  论文图表生成工具。支持架构图/系统流程图（Gemini AI生成）和数据图表
  （matplotlib/seaborn 折线图/柱状图/散点图/热力图）。
  当用户说"画图"、"论文图表"、"Figure"、"architecture diagram"、"plotting"、
  "数据可视化"、"画个图"时触发。
tags: [Academic Writing, Visualization, Matplotlib, Seaborn, Plotting, Figures, Diagrams, NeurIPS, ICML, ICLR, LaTeX]
---

# 论文图表生成

两种工作流：

1. **架构图/系统图** — Gemini AI 图像生成
2. **数据图表** — matplotlib/seaborn

## 工作流选择

| 图类型 | 工具 | 原因 |
|--------|------|------|
| 架构图/系统流程图 | Gemini | 复杂空间布局，框+箭头+标签 |
| 工作流/管线/生命周期 | Gemini | 多步骤流程 |
| 柱状图/折线图/散点图 | matplotlib | 精确数值数据，可复现 |
| 热力图/混淆矩阵 | matplotlib/seaborn | 结构化网格数据 |
| 消融实验对比 | matplotlib | 分组柱状图/折线对比 |
| 训练曲线 | matplotlib | Loss/accuracy 随步骤/epochs 变化 |

**经验法则**：有数值坐标轴用 matplotlib，有框和箭头用 Gemini。

## 上下文分析与提取

从用户输入中自动识别：

| 输入类型 | 提取目标 |
|---------|---------|
| 论文全文/章节草稿 | 系统组件、关系、数据流 |
| 描述段落 | 关键实体、层次、连接 |
| 原始结果/数据表 | 指标、方法、对比结构 |
| CSV/JSON 数据 | 变量、趋势、分组维度 |

## Workflow 1: 架构图（Gemini）

### 视觉风格

四种风格可选，一篇论文的所有图保持一致：

**A. 简笔画** — 温暖、亲切，适合概述图。手绘线条质感，柔和色彩
**B. 现代极简** — 自信、权威，适合方法图。几何形状，大胆色块
**C. 技术插画** — 丰富图标，适合教程型论文。每个组件有含义图标
**D. 经典学术** — 安全，适合任何会议，灰度友好。左侧色条区分

### 色板

- **Ocean Dusk**（推荐）：`#264653` `#2A9D8F` `#E9C46A` `#F4A261` `#E76F51`
- **Okabe-Ito**（色盲安全，数据图必用）：`#E69F00` `#56B4E9` `#009E73` `#F0E442` `#0072B2` `#D55E00` `#CC79A7`

### Prompt 结构

每次生成包含6部分：
1. **FRAMING**（5行）：图的目标和氛围
2. **VISUAL STYLE**（20-30行）：完整风格定义块
3. **COLOR PALETTE**（10行）：精确 hex 色值
4. **LAYOUT**（50-150行）：每个组件、位置、分组
5. **CONNECTIONS**（30-80行）：每条箭头——起点、终点、样式
6. **CONSTRAINTS**（10行）：不要包含什么

### 关键规则

- 总是尝试 **3 次**，质量差异大
- 风格定义块是**必须的**，否则 Gemini 默认通用企业风格
- 每次精确写出每个标签，Gemini 可能拼错
- 保存生成脚本到 `figures/gen_fig_<name>.py`

详细参考见 `references/diagram-generation.md`

## Workflow 2: 数据图表（matplotlib/seaborn）

### 图类型自动选择

| 数据模式 | 最佳图类型 |
|---------|-----------|
| 随时间/步长趋势 | 折线图 |
| 多类别对比 | 分组柱状图 |
| 分布 | 提琴图/箱线图 |
| 相关性 | 散点图 |
| 网格数值 | 热力图 |
| 占比 | 堆叠柱状图（避免饼图） |
| 多方法单指标 | 水平柱状图（排行榜） |

### 出版级样式模板

```python
import matplotlib.pyplot as plt
plt.rcParams.update({
    "font.family": "serif", "font.size": 10,
    "axes.titlesize": 11, "axes.titleweight": "bold",
    "axes.labelsize": 10, "legend.fontsize": 8.5,
    "axes.spines.top": False, "axes.spines.right": False,
    "axes.grid": True, "grid.alpha": 0.15,
    "figure.dpi": 300, "savefig.dpi": 300,
})
COLORS = ["#264653", "#2A9D8F", "#E9C46A", "#F4A261", "#E76F51"]
OUR_COLOR = "#E76F51"
```

### 常用图表模式

**折线图（训练曲线）：**
```python
fig, ax = plt.subplots(figsize=(3.25, 2.5))
markers = ["o", "s", "^", "D", "v"]
for i, (method, (mean, std)) in enumerate(results.items()):
    color = OUR_COLOR if method == "Ours" else COLORS[i]
    ax.plot(steps, mean, label=method, color=color,
            marker=markers[i], markevery=max(1, len(steps)//8))
    ax.fill_between(steps, mean-std, mean+std, color=color, alpha=0.12)
ax.set_xlabel("Training Steps"); ax.set_ylabel("Accuracy (%)")
ax.legend(loc="lower right")
fig.savefig("figures/fig_training.pdf")
```

**分组柱状图（消融实验）：**
```python
fig, ax = plt.subplots(figsize=(6.75, 2.8))
x = np.arange(len(categories)); width = 0.7 / len(methods)
for i, (method, scores) in enumerate(methods.items()):
    color = OUR_COLOR if method == "Ours" else COLORS[i]
    offset = (i - len(methods)/2 + 0.5) * width
    ax.bar(x + offset, scores, width*0.9, label=method, color=color)
ax.set_xticks(x); ax.set_xticklabels(categories)
ax.set_ylabel("Score"); ax.legend()
fig.savefig("figures/fig_ablation.pdf")
```

**热力图：**
```python
import seaborn as sns
fig, ax = plt.subplots(figsize=(4, 3.5))
sns.heatmap(matrix, annot=True, fmt=".2f", cmap="YlOrRd",
            linewidths=1.5, linecolor="white")
fig.savefig("figures/fig_heatmap.pdf")
```

### 出版尺寸参考

| 会议 | 单栏 | 全宽 | 字体 |
|------|------|------|------|
| NeurIPS | 5.5 in | 5.5 in | Times |
| ICML | 3.25 in | 6.75 in | Times |
| ICLR | 5.5 in | 5.5 in | Times |
| ACL | 3.3 in | 6.8 in | Times |

**始终输出 PDF** 以保证矢量质量。

## 常见问题

| 问题 | 解决 |
|------|------|
| LaTeX 中字体不对 | 导出 PDF，设 `text.usetex=True` |
| 图太大超出列宽 | 检查会议宽限，用英寸 `figsize` |
| 打印颜色难区分 | 色盲安全色板 + 不同线型/标记 |
| Gemini 拼错标签 | 精确写出每个标签，加 "SPELL EXACTLY" 约束 |
| 图模糊 | 导出 PDF（矢量）或用 300+ DPI PNG |

## 文件命名

```
figures/
├── gen_fig_<name>.py       # 生成脚本（必须保存以便复现）
├── fig_<name>.pdf          # 最终矢量输出（供LaTeX使用）
└── fig_<name>.png          # 光栅输出（300 DPI）
```
