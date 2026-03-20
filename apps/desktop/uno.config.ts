import presetUno from '@unocss/preset-uno'
import presetIcons from '@unocss/preset-icons'
import type { UserConfig } from '@unocss/core'

const config: UserConfig = {
  presets: [
    presetUno(),
    presetIcons({
      scale: 1.2,
    }),
  ],
  theme: {
    colors: {
      value: {
        high: '#22c55e',
        medium: '#f59e0b',
        low: '#d1d5db',
      },
    },
  },
}

export default config
