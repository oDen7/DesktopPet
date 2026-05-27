/**
 * Vite 客户端类型声明与环境变量类型补充。
 * - 引用 Vite 客户端类型（import.meta.env 等）
 * - 声明 .vue 单文件组件模块类型，使 TypeScript 能正确识别 .vue 导入
 */
/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<object, object, unknown>;
  export default component;
}
