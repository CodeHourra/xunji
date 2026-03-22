export * from './types'
export * from './cardTypes'
// 注意：勿在此导出 ./constants（依赖 Node 的 path/os）。浏览器端 import '@xunji/shared'
// 会整包解析，会导致 Vite 白屏。Node 侧请使用：import '...' from '@xunji/shared/constants'
