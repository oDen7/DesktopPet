<template>
  <div class="setting-item">
    <div class="setting-label">
      <span class="setting-title">{{ title }}</span>
      <span class="setting-desc">{{ desc }}</span>
    </div>
    <label class="toggle-switch">
      <input type="checkbox" :checked="modelValue" @change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)">
      <span class="toggle-slider"></span>
    </label>
  </div>
</template>

<!--
  可复用的切换开关组件。
  使用 v-model 双向绑定 boolean 值，通过 CSS 滑块动画展示开/关状态。

  Props:
    modelValue — 当前开关状态（true/false）
    title      — 设置项标题文本
    desc       — 设置项描述文本

  Events:
    update:modelValue — 开关切换时触发，传递新值
-->

<script setup lang="ts">
defineProps<{ modelValue: boolean; title: string; desc: string }>();
defineEmits<{ 'update:modelValue': [value: boolean] }>();
</script>
