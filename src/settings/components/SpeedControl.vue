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

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{ modelValue: number }>();
defineEmits<{ 'update:modelValue': [value: number] }>();

const label = computed(() => {
  const v = props.modelValue;
  if (v <= 0.5) return '缓慢';
  if (v <= 0.75) return '较慢';
  if (v <= 1.25) return '正常';
  if (v <= 2) return '较快';
  return '飞速';
});
</script>
