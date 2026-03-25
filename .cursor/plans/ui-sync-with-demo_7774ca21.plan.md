---
name: ui-sync-with-demo
overview: 完全按照 design/ui_demo.html 原型还原项目的顶部导航、左侧目录树、顶部搜索栏、会话列表以及知识库视图，消除现存的样式差异。
todos:
  - id: sync-session-toolbar
    content: 重构 SessionToolbar.vue 搜索与分段过滤器的布局和胶囊样式
    status: completed
  - id: sync-sidebar-tree
    content: 定制 Sidebar.vue 目录树源/主机/项目层级的视觉渲染样式，并在会话整理模式下处理多选框的交互样式
    status: completed
  - id: sync-session-card
    content: 将 SessionCard.vue 会话卡片重写为原型的上下垂直布局结构
    status: completed
  - id: sync-knowledge-view
    content: 还原 KnowledgeView.vue 的顶部操作栏布局和视图切换器样式
    status: completed
  - id: sync-topbar
    content: 核对 TopBar.vue 中切换按钮的最终呈现细节
    status: completed
isProject: false
---

# UI 样式深度还原计划

根据您提供的对比图与 `design/ui_demo.html` 原型，发现当前的实现与设计稿在组件布局和细节上有较大偏离。接下来的工作将严格对齐原型，执行以下重构：

## 1. 顶部搜索栏与过滤区域 (`SessionToolbar.vue`)

- **布局重构**：将搜索栏和过滤操作从现有的平铺结构改为上下两行布局。
  - 第一行：宽大的搜索框 + 次级“刷新”按钮。
  - 第二行：左侧为“分段选择器（Segmented Control）”样式（全部会话 / 待分析 / 已分析），包含带数字徽标的“待分析”状态；右侧为记录总数与 Tooltip 提示收纳。
- **视觉风格**：去除当前原生的 `n-button` 过滤组，采用原型中胶囊背景槽加悬浮滑块的无边框设计。

## 2. 侧边栏目录树 (`Sidebar.vue`)

- **节点自定义渲染**：使用 `n-tree` 的自定义渲染（`renderLabel`, `renderPrefix`, `renderSuffix`）精确控制每级节点的 DOM。
- **层级样式对齐**：
  - 一级（来源）：使用特定的品牌色图标（如蓝色 Cursor、橙色 Claude Code 等）配合加粗标题。
  - 二级（主机）：增加层级缩进，图标换为 `lucide-server`。
  - 三级（项目）：带 `lucide-corner-down-right` 拐角指引和不同状态的文件夹图标，鼠标悬停时节点增加圆角高亮。
  - 数量徽标：放置在最右侧，使用单色等宽字体对齐（`font-mono text-[10px]`）。
- **会话整理模式处理**：特别处理点击「会话整理」进入多选状态（`checkable=true`）时的交互样式，确保左侧的复选框能优雅融入自定义渲染的节点，且选择高亮时背景和样式保持整洁，与设计稿多选风格对齐。

## 3. 会话卡片列表 (`SessionCard.vue`)

- **回归垂直排版**：抛弃目前采用的横向左右分列设计，严格按照原型的上下两层布局重构。
  - **上层视线区**：先展示状态 Tag（如：待分析 / 高价值）与来源角标，下方为主标题和摘要文字；最右侧为 Hover 显现的操作按钮。
  - **下层元数据区**：用顶部细线分割（`border-t`），采用横向平铺展示所属项目文件夹（带浅灰底色块）、时间、对话数与大小数据。

## 4. 知识库 Tab 页 (`KnowledgeView.vue`)

- **操作栏还原**：顶部操作栏左侧为标题与记录数，右侧为并排的操作按钮组。
- **功能按钮整合**：将“全选当页”与“清除选择”逻辑整合为一个按钮，并在选中时变为文字按钮外观。
- **视图切换组件**：深度覆写 `n-radio-group` 的样式，使用和对话页同源的“大胶囊底槽”设计，配合白底阴影块实现列表/卡片的平滑切换效果。

## 5. 顶栏细节兜底 (`TopBar.vue`)

- 确保中间的“对话记录 / 知识库”切换器彻底还原 `bg-slate-100/80` 胶囊包裹的方案，消除由于历史残留导致的 UI 割裂。

