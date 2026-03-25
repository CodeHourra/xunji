# 踩坑：Tauri / WebView 下分段条用原生 `<button>` 未选中项像「实心灰块」

## 问题 / 背景

在桌面端用 **Tailwind / Uno 工具类** 做顶栏 Tab、会话列表状态筛选等 **分段控件**（segmented control）时，设计稿期望：**仅外层轨道有浅灰底，未选中项透明、仅文字偏灰；选中项白底 + 细 ring**。实际在 Tauri 窗口里未选中按钮却像 **单独一块灰/白实心 pill**，暗色模式下甚至出现 **未选中比选中更亮** 的颠倒层次。

---

## 现象

- **浅色**：未选中的「对话记录 / 知识库」或「全部会话 / 待分析 / 已分析」呈 **明显实心中灰背景**，与 `design/ui_demo.html` 里扁平分段不一致。
- **深色**：未选中项 **偏亮灰/白底**，选中项反而是深色块，视觉焦点反了。
- **复现**：`apps/desktop` 内使用 **原生 `<button>`** + Uno 类名、**未** 引入完整 CSS reset/preflight 时，在 **Tauri + WebKit/WKWebView** 中易复现；普通浏览器中表现可能较轻。

---

## 根因

1. **用户代理（UA）默认按钮外观**  
   WebView 对未覆盖样式的 `<button>` 会套用 **系统按钮皮肤**（背景色、立体边框等），优先级与「你以为只有文字色」的意图冲突。

2. **项目未使用会统一「抹平」按钮的全局 reset**  
   入口仅 `import 'virtual:uno.css'`，**preset-uno 不等价于** Tailwind 的完整 **preflight**（不会默认给所有 `button` 设 `background-color: transparent` 等）。因此不能假设「没写背景就是透明」。

3. **未显式声明未选中背景**  
   若只写 `text-slate-500` 而不写 **`bg-transparent`**，在部分环境下仍可能看到 UA 提供的底色。

4. **暗色 + 系统按钮**  
   `color-scheme` / 系统主题下，UA 按钮可能呈现 **浅色块**，与自研 `dark:bg-neutral-*` 的选中 pill 叠加后层次错乱。

```text
  设计意图                    WebView 实际（未处理时）
  ─────────                   ────────────────────────
  [ 轨道灰 ][透明][透明]  →    [ 轨道灰 ][灰块][灰块]
       ↑选中白+ring              ↑选中深 + 未选「亮块」
```

---

## 解决方法（本项目已采用）

1. **对分段条内按钮单独去外观**（避免影响 Naive UI 的 `n-button`）  
   在 `App.vue` 全局样式中增加仅作用于 **带类名** 的按钮，例如：

   - `button.segment-pill-btn { -webkit-appearance: none; appearance: none; margin: 0; font: inherit; }`

2. **分段按钮类名约定**  
   - 使用 `type="button"`，避免表单内被当成 submit。  
   - 挂 `segment-pill-btn`，并加 **`bg-transparent`**（未选中）、**`border-0`** 等。  
   - 选中态再用 `bg-white`、`ring-1 ring-slate-200/90`（与 `design/ui_demo.html` 一致时可去掉 `shadow-sm`）。

3. **暗色模式单独调色**  
   轨道与选中 pill 使用同一套语义（如轨道 `dark:bg-neutral-900/55`、选中 `dark:bg-neutral-800`），未选中 **保持透明 + 较低对比文字色**，避免「未选中更亮」。

**相关代码位置：**

- `apps/desktop/src/App.vue` — `button.segment-pill-btn`
- `apps/desktop/src/components/TopBar.vue` — 顶栏 Tab
- `apps/desktop/src/components/SessionToolbar.vue` — 会话状态分段筛选
- `apps/desktop/src/views/KnowledgeView.vue` — 知识库「列表 / 卡片」视图切换

---

## 后续如何避免

- **新增自定义分段条**：一律使用 **`segment-pill-btn` + 未选中 `bg-transparent`**，或改用 **Naive UI** 自带分段能力（如 `n-tabs` / `n-radio-group` 等），避免裸 `button` 依赖 UA。  
- **可选**：在 `main.ts` 引入 `@unocss/reset/tailwind.css`（或等价 preflight）前，先在页面内 **检查 `n-button` 是否仍正常**，再决定是否全站启用。  
- **暂无** 自动化测试；依赖 Code Review 与视觉验收。

---

## 相关文档

- 同目录：[桌面应用-脚手架-UnoCSS-Tauri与存储层常见问题.md](./桌面应用-脚手架-UnoCSS-Tauri与存储层常见问题.md)（Uno/Tauri 脚手架与其它问题）
- 设计对照：`design/ui_demo.html` 顶栏 Tab 分段注释（ring 代替 shadow-sm）
