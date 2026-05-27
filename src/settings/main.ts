/**
 * 桌面宠物 — 设置面板入口文件。
 * 创建 Vue 3 应用实例，挂载 App 根组件到 #app 元素，
 * 并加载全局样式。
 */
import { createApp } from 'vue';
import App from './App.vue';
import './style.less';

createApp(App).mount('#app');
