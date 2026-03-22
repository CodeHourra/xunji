import presetUno from '@unocss/preset-uno'
import presetIcons from '@unocss/preset-icons'
import type { UserConfig } from '@unocss/core'
// 显式内联 lucide 图标集，避免依赖 Node 端 @iconify 文件加载器（部分环境/Node 版本下
// loadNodeIcon 不可用，导致生产构建里 `i-lucide-*` 规则为空，打包后界面图标全部丢失）
import lucideIcons from '@iconify-json/lucide/icons.json'

/**
 * 业务中用到的全部 Lucide 图标类（与 src 下 `i-lucide-*` 保持一致）。
 * preset-icons 的规则为异步；Vite 生产构建若未显式 safelist，可能出现图标层未写入 CSS、界面无图标。
 */
const LUCIDE_ICON_CLASSES = [
  'i-lucide-alert-circle',
  'i-lucide-alert-triangle',
  'i-lucide-arrow-left',
  'i-lucide-arrow-right',
  'i-lucide-book-open',
  'i-lucide-bot',
  'i-lucide-braces',
  'i-lucide-brain',
  'i-lucide-calendar',
  'i-lucide-check-circle',
  'i-lucide-chevron-down',
  'i-lucide-chevron-right',
  'i-lucide-chevron-up',
  'i-lucide-circle',
  'i-lucide-clock',
  'i-lucide-cloud',
  'i-lucide-code',
  'i-lucide-compass',
  'i-lucide-download',
  'i-lucide-folder',
  'i-lucide-hard-drive',
  'i-lucide-hash',
  'i-lucide-info',
  'i-lucide-layers',
  'i-lucide-layout-grid',
  'i-lucide-library',
  'i-lucide-list',
  'i-lucide-loader-2',
  'i-lucide-message-circle',
  'i-lucide-message-square',
  'i-lucide-message-square-off',
  'i-lucide-messages-square',
  'i-lucide-moon',
  'i-lucide-mouse-pointer-click',
  'i-lucide-play',
  'i-lucide-refresh-cw',
  'i-lucide-rotate-ccw',
  'i-lucide-search',
  'i-lucide-server',
  'i-lucide-settings',
  'i-lucide-sparkles',
  'i-lucide-square',
  'i-lucide-sun',
  'i-lucide-tags',
  'i-lucide-terminal',
  'i-lucide-terminal-square',
  'i-lucide-user',
  'i-lucide-x',
  'i-lucide-x-circle',
  'i-lucide-zap',
] as const

const config: UserConfig = {
  safelist: [...LUCIDE_ICON_CLASSES],
  presets: [
    presetUno(),
    presetIcons({
      scale: 1.2,
      collections: {
        // IconifyJSON 与 InlineCollection 类型定义略有不一致，运行时已验证可正常生成 CSS
        lucide: lucideIcons as never,
      },
      extraProperties: {
        'display': 'inline-block',
        'vertical-align': 'middle',
      },
    }),
  ],
  theme: {
    colors: {
      brand: {
        50: '#ecfdf5',
        100: '#d1fae5',
        200: '#a7f3d0',
        300: '#6ee7b7',
        400: '#34d399',
        500: '#10b981',
        600: '#059669',
        700: '#047857',
        800: '#065f46',
        900: '#064e3b',
        // 950 用于深色模式高亮背景（Sidebar 选中态等）
        950: '#022c22',
      },
      value: {
        high: '#10b981',
        medium: '#f59e0b',
        low: '#94a3b8',
      },
    },
  },
}

export default config
