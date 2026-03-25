<script setup lang="ts">
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  darkTheme,
  zhCN,
  dateZhCN,
} from 'naive-ui'
import AppLayout from './components/AppLayout.vue'
import { useUiStore } from './stores/ui'

const ui = useUiStore()
</script>

<template>
  <!-- NConfigProvider 为 Naive UI 组件提供全局主题 + 中文 locale -->
  <n-config-provider :theme="ui.darkMode ? darkTheme : undefined" :locale="zhCN" :date-locale="dateZhCN">
    <!-- useMessage / useDialog（导出确认、Toast 等）依赖以下 Provider -->
    <n-message-provider>
      <n-dialog-provider>
        <AppLayout />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', Helvetica, Arial, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  --brand-500: #10b981;
  --brand-600: #059669;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  margin: 0;
}

/* Webkit 滚动条 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.25);
  border-radius: 999px;
}
.dark ::-webkit-scrollbar-thumb {
  background-color: rgba(75, 85, 99, 0.35);
}

/**
 * 分段条内的原生 <button>：Tauri/WebKit 会套用系统按钮外观（灰底、凸起），
 * 未选中项看起来像「实心灰块」，浅色/深色下都会破坏与 ui_demo 一致的扁平分段样式。
 * 仅对带 .segment-pill-btn 的按钮去外观，避免影响 Naive UI 的 n-button。
 */
button.segment-pill-btn {
  -webkit-appearance: none;
  appearance: none;
  margin: 0;
  font: inherit;
}
</style>
