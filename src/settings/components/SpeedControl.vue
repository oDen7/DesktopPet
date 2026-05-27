<template>
  <div class="setting-item">
    <div class="setting-label">
      <span class="setting-title">移动速度</span>
      <span class="setting-desc">{{ label }}</span>
    </div>
    <div class="speed-slider">
      <input
        type="range"
        min="0.25"
        max="3"
        step="0.25"
        :value="modelValue"
        @input="$emit('update:modelValue', parseFloat(($event.target as HTMLInputElement).value))"
      >
      <div class="speed-value">{{ modelValue }}x</div>
    </div>
  </div>
</template>

<!--
  移动速度控制组件 — 使用 HTML range 滑块实现 0.25x-3x 速度调节。
  当前值以人类可读的中文标签（缓慢/正常/飞速）显示。

  Props:
    modelValue — 当前速度倍率（0.25 到 3.0）

  Events:
    update:modelValue — 滑块拖拽时实时触发
-->

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{ modelValue: number }>();
defineEmits<{ 'update:modelValue': [value: number] }>();

/// 根据速度值计算中文标签文本
const label = computed(() => {
  const v = props.modelValue;
  if (v <= 0.5) return '缓慢';
  if (v <= 0.75) return '较慢';
  if (v <= 1.25) return '正常';
  if (v <= 2) return '较快';
  return '飞速';
});
</script>
