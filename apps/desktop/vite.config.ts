import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import UnoCSS from '@unocss/vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [vue(), UnoCSS()],
  // 强制 vue 单例，防止 bun/pnpm 混合安装导致多份 Vue 实例
  resolve: {
    dedupe: ['vue'],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
