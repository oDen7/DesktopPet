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

<script setup lang="ts">
defineProps<{ modelValue: boolean; title: string; desc: string }>();
defineEmits<{ 'update:modelValue': [value: boolean] }>();
</script>
