<template>
  <div class="setting-item sprite-picker">
    <div class="setting-label">
      <span class="setting-title">宠物形象</span>
      <span class="setting-desc">选择精灵造型</span>
    </div>
    <div class="sprite-grid">
      <div
        v-for="sprite in sprites"
        :key="sprite.id"
        class="sprite-card"
        :class="{ active: modelValue === sprite.id }"
        @click="$emit('update:modelValue', sprite.id)"
      >
        <img :src="spriteUrl(sprite)" class="sprite-thumb" :alt="sprite.name" />
        <span class="sprite-name">{{ sprite.name }}</span>
        <span v-if="sprite.is_preset" class="sprite-badge">预设</span>
      </div>
      <div class="sprite-card upload-card" @click="uploadSprite">
        <div class="upload-icon">+</div>
        <span class="sprite-name">上传新形象</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

interface SpriteInfo {
  id: string;
  name: string;
  is_preset: boolean;
  w: number;
  h: number;
  url: string;
}

defineProps<{ modelValue: string }>();
defineEmits<{ 'update:modelValue': [value: string] }>();

const sprites = ref<SpriteInfo[]>([]);

function spriteUrl(sprite: SpriteInfo): string {
  if (sprite.is_preset) return sprite.url;
  return convertFileSrc(sprite.url);
}

onMounted(async () => {
  try {
    sprites.value = await invoke<SpriteInfo[]>('list_sprites');
  } catch (e) {
    console.error('Failed to list sprites:', e);
  }
});

async function uploadSprite() {
  try {
    const result = await invoke<SpriteInfo | null>('upload_sprite');
    if (result) {
      sprites.value.push(result);
    }
  } catch (e) {
    console.error('Failed to upload sprite:', e);
  }
}
</script>
