/// <reference types="vite/client" />

/** Vite `?raw`：关于页更新日志等静态 Markdown 文本 */
declare module '*.md?raw' {
  const content: string
  export default content
}

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
