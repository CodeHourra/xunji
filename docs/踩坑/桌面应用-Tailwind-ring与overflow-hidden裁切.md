# 踩坑：Tailwind / Uno `ring` 与 `overflow-hidden` 同用导致描边/选中态「缺一块」

## 问题 / 背景

在会话列表等卡片上，用 **`ring-2` + `ring-emerald-500`** 表示批量选中，同时根节点为圆角使用了 **`overflow-hidden`**。界面上表现为：**绿色描边不完整、选中底色像被前景挡掉**，尤其在卡片四角或边缘更明显。

---

## 现象

- 选中态高亮框 **断线、缺角**，或只有部分边可见。
- 误以为子组件（`n-tag`、按钮等）**z-index 盖住**了选中背景；子组件不透明背景确实会遮住 **半透明底色**，但与 **整圈描边被裁切** 是两类问题，需分开排查。

---

## 根因

Tailwind / Uno 的 **`ring`** 通过 **`box-shadow`** 画在元素 **边框盒外侧**（默认向外扩展）。  

同一元素若设置了 **`overflow: hidden`**（含 `overflow-hidden` 工具类），**超出 padding edge 的阴影会被裁掉**，因此 `ring` 在视觉上不完整。

```text
  [ overflow:hidden 的卡片 ]
  ┌────────────────────┐
  │  ring 画在盒外 →   │ ← 外侧阴影被裁掉
  └────────────────────┘
```

---

## 解决方法

任选其一（本项目会话卡片采用 **A**）：

**A. 使用内描边 `ring-inset`**（推荐，圆角卡片常保留 `overflow-hidden`）

- 选中类改为：`ring-2 ring-inset ring-emerald-500`，必要时略加强 `bg-*` 透明度以便辨认。

**B. 去掉根节点 `overflow-hidden`**

- 把裁切挪到 **不需要外扩 ring** 的内层子节点；需确认无子内容溢出圆角。

**C. 不用 `ring`，改用 `outline` 或实线 `border`**

- 需自行处理暗色模式与圆角一致性；`outline` 在部分浏览器下与 `border-radius` 表现仍要目检。

**相关代码位置：**

- `apps/desktop/src/components/SessionCard.vue` — 批量选中 `cardClass` 使用 `ring-inset`

---

## 后续如何避免

- 卡片根节点若 **固定带 `overflow-hidden`**，选中/聚焦态 **不要用默认外向 `ring`**，应 **`ring-inset`** 或 **边框方案**。
- Code Review 时：**`overflow-hidden` + `ring`** 同框出现 → 优先怀疑裁切。
- **暂无** 自动化检测；依赖规范与审查。

---

## 相关文档

- [桌面应用-分段控件-WebView原生button默认样式.md](./桌面应用-分段控件-WebView原生button默认样式.md) — 另涉及 `ring` 的 UI 语义（分段条扁平描边）
- [桌面应用-脚手架-UnoCSS-Tauri与存储层常见问题.md](./桌面应用-脚手架-UnoCSS-Tauri与存储层常见问题.md)
