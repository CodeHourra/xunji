---
name: superdesign-ui
description: Senior frontend designer workflow for HTML/SVG UI iterations, wireframes, components, logos, and design-system extraction from screenshots. Outputs to .superdesign/design_iterations and .superdesign/design_system. Triggers when the user asks for UI design, HTML mockups, Superdesign-style variants, wireframes, SVG icons/logos, component-only pages, or extracting a design system from images.
---

# Superdesign UI（资深前端设计工作流）

以资深前端设计师视角工作：先定设计风格与信息层次，再落位到像素级间距、字体与颜色。

## 总原则

- **默认并行 3 套变体**：除非用户明确要求「只出 1 版」，否则用 **3 个并行子代理** 各产出一版单屏 HTML（或约定下的 SVG），便于快速对比迭代。
- **迭代不覆盖旧文件**：更新/迭代时 **禁止就地编辑** 已有设计文件；始终 **新建** 带版本后缀的文件（见下文命名）。
- **子代理分工**：每个子代理只负责 **一个** 输出文件（单页 HTML 或单个 SVG），避免一个代理里堆多套方案。

## 输出路径与命名

### 目录约定（统一使用下划线目录名）

| 用途 | 路径 |
|------|------|
| HTML / SVG 设计稿迭代 | `.superdesign/design_iterations/` |
| 从截图提取的设计系统 JSON | `.superdesign/design_system/` |

（若仓库中尚无 `.superdesign`，创建该目录后再写入文件。）

### HTML 命名

- **新设计**：`{设计名称}_{n}.html`，`n` 在本次任务中唯一，例如 `table_1.html`、`table_2.html`。
- **基于已有文件迭代**：若当前基准为 `ui_1.html`，新版本为 `ui_1_1.html`、`ui_1_2.html`（在基准文件名后追加 `_序号`）。

### SVG（徽标 / 图标）

1. **先复制再改**：从已有 SVG 复制到 `.superdesign/design_iterations/`，按命名规则改名，再编辑副本。
2. **一代理一文件**：每个子代理只处理 **一个** SVG 副本，专注 SVG 语法与路径正确性。
3. 命名与 HTML 规则一致（如 `icon_1.svg`、`logo_base_1.svg` 或基于 `original_1_2.svg` 迭代）。

### 设计系统 JSON

- 默认文件名：`design-system.json`，放在 `.superdesign/design_system/`。
- 若已存在：新建 `design-system_{n}.json`（`n` 唯一，如 `design-system_1.json`）。

---

## 任务类型与工作方式

### 1. 常规「创建设计 / UI」

- 流程同下：**先 3 子代理并行**（除非用户只要 1 版）。
- 每个子代理：一个 **单屏**、自包含的 HTML 文件，写入 `.superdesign/design_iterations/`。
- 须遵守本文 **「UI 设计与实现指南」**。

### 2. 更新或迭代设计

- **不要编辑**旧 HTML；按 **「HTML 命名 → 迭代」** 规则新建文件。
- 默认仍 **3 子代理并行**。

### 3. 组件（component only）

- 与常规设计类似，每代理 **一个 HTML**，内仅渲染 **该组件 + 必要模拟数据**。
- **不要**加营销文案、完整页面壳子等多余元素；焦点只在组件本身。

### 4. 线框图（wireframe）

- **仅黑白线稿**，不用彩色；**不要**真实图片、`placehold.co` 等外链占位图；用 **CSS 块/边框** 做占位。
- 风格参考 Balsamiq：**极简、流程清晰**，避免装饰性样式注释。
- 技术栈仍可 Tailwind，但视觉保持线框感（灰阶、细线、简单矩形）。

### 5. 徽标 / 图标（SVG）

- 严格 **复制 → 重命名 → 再编辑**；子代理之间文件不混用。

### 6. 从图像提取设计系统

目标：输出 **可复用的 tokens 与规则**（供前端或 AI 一致还原视觉语言），**不包含**截图里的具体文案、Logo、图标图形。

1. 分析：色板、排版、间距、布局（网格/卡片/容器）、组件形态、圆角/阴影等模式。
2. 若为 **App 界面** 语义：在 JSON 或说明中增加 **1px 描边模拟手机边框** 的规则，便于还原设备框。
3. 写入 `.superdesign/design_system/design-system.json` 或 `design-system_{n}.json`。
4. **禁止**把截图中的具体文字、品牌图形写入 JSON；只抽象 **原则、结构、样式变量**。

---

## UI 设计与实现指南

### 设计风格（与极简技术约束对齐）

- 在 **优雅极简** 与 **可用性** 之间平衡；**比例留白**、**清晰层次**、**细腻圆角**、**轻量微交互**（仅用 CSS 可实现的 hover/focus 等）。
- **响应式**：单 HTML 需在移动 / 平板 / 桌面可用；若以 App 为主则移动优先。
- 色彩：以 **UI 技术规范中的「最小调色板」** 为准；若用户明确要求「品牌渐变」，仅使用 **极低对比、中性向** 的柔和渐变，避免花哨多色渐变。

### 技术规范

1. **图片**：HTML 内 **不要** 使用真实图片资源；用 **CSS** 做占位几何。**不要**使用 `placehold.co` 等外链占位（环境可能无法渲染）。
2. **样式**：通过 **CDN 引入 Tailwind CSS**；自定义补充样式放在 Tailwind 之后，必要时对关键标记使用 `!important`，并避免污染全局（可用一层 wrapper class）。
3. **不要**绘制系统状态栏（时间、信号、电池等）。
4. **文本颜色**：仅 **黑或白**（及与黑白配套的灰阶若需层次，仍属中性灰，不用彩色字）。
5. **间距**：**4 或 8 点网格** — 所有 margin/padding/gap/尺寸优先为 **4、8、16、24、32…** 的倍数，避免 `5px`、`13px` 等随意值。
6. **分组**：组内更紧（如 4–8），组间更疏（16–24）。
7. **排版节奏**：字号与行高对齐网格（如正文 16px / 行高 24px）。
8. **触控**：可点击区域尽量 **≥ 48×48px**（或等价可点区域）。

### 色彩

- **60-30-10**：约 60% 背景（白/浅灰）、30% 表面（白/中灰）、10% 强调（如炭黑或极淡中性强调色）。
- **强调色**：一种 subtle 色调即可；交互控件谨慎使用。
- 对比度：正文与背景建议满足 **WCAG ≥ 4.5:1**（在黑白灰体系下较易满足）。

### 排版层次

- 至少三级：**H1（页内主标题）、H2（区块标题）、正文**。
- 级别之间字号差距建议 **≥ 6–8px**（避免只差 2px）。
- 标题偏粗/中粗，正文常规字重；少用斜体/全大写堆砌。
- 行高：正文约 **0.8×–1.5×** 字号区间内的可读节奏；标题上下 margin 与行高成比例（如约 1.2× 行高）。

---

## 并行执行（给主代理的操作说明）

当用户需要新设计或迭代且未要求单版本时：

1. 规划 3 个 **有差异** 的方向（如信息密度、卡片布局、导航结构之一项变化），避免三份几乎相同。
2. 使用 **3 个并行子代理**，各生成 **一个** 符合命名规则的文件于 `.superdesign/design_iterations/`。
3. 主代理汇总路径列表与简短差异说明，便于用户挑选下一版迭代基准文件。

---

## 自检清单（输出前）

- [ ] 路径是否为 `.superdesign/design_iterations/` 或 `.superdesign/design_system/`？
- [ ] 迭代是否 **新建文件** 而非覆盖？
- [ ] 线框/技术约束下是否 **无外链图片**？
- [ ] 间距是否为 **4/8 点体系**？
- [ ] 文本是否仅为 **黑/白/中性灰**？
- [ ] 设计系统 JSON 是否 **无具体文案与 Logo 内容**，仅抽象规则与 token？
