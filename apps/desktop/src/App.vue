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
</style>
