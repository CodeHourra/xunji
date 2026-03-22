import presetUno from '@unocss/preset-uno'
import presetIcons from '@unocss/preset-icons'
import type { UserConfig } from '@unocss/core'
// 显式内联 lucide 图标集，避免依赖 Node 端 @iconify 文件加载器（部分环境/Node 版本下
// loadNodeIcon 不可用，导致生产构建里 `i-lucide-*` 规则为空，打包后界面图标全部丢失）
import lucideIcons from '@iconify-json/lucide/icons.json'

const config: UserConfig = {
  presets: [
    presetUno(),
    presetIcons({
      scale: 1.2,
      collections: {
        /**
         * 必须传入「返回 IconifyJSON 的函数」，不能直接塞 JSON 对象：
         * @iconify/utils 对 object 会走 getCustomIcon（按 key 取单个 SVG 字符串），
         * 对 function 且返回值含 `icons` 才会走 searchForIcon，才能识别整套 lucide。
         */
        lucide: () => lucideIcons,
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
