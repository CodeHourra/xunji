import presetUno from '@unocss/preset-uno'
import presetIcons from '@unocss/preset-icons'
import type { UserConfig } from '@unocss/core'

const config: UserConfig = {
  presets: [
    presetUno(),
    presetIcons({
      scale: 1.2,
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
