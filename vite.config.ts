/**
 * Vite 构建配置 — 桌面宠物项目。
 *
 * 多入口构建设置：
 *   - main (index.html)     → 宠物窗口
 *   - settings (settings.html) → 设置面板
 *
 * 路径别名 @ 映射到 src/ 目录。
 * root 指向 src/，publicDir 指向 public/。
 */
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue()],
  root: resolve(__dirname, 'src'),
  publicDir: resolve(__dirname, 'public'),
  build: {
    outDir: resolve(__dirname, 'dist'),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/index.html'),
        settings: resolve(__dirname, 'src/settings.html'),
      },
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});
