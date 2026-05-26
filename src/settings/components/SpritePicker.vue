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
        <button
          v-if="!sprite.is_preset"
          class="sprite-delete"
          title="删除"
          @click.stop="removeSprite(sprite)"
        >×</button>
      </div>
      <div class="sprite-card upload-card" @click="triggerUpload">
        <div class="upload-icon">+</div>
        <span class="sprite-name">上传新形象</span>
      </div>
    </div>

    <input
      ref="fileInput"
      type="file"
      accept="image/png"
      style="display:none"
      @change="onFileSelected"
    />

    <SpriteEditor
      v-if="pending"
      :dataUrl="pending.dataUrl"
      :fileName="pending.fileName"
      @confirm="confirmUpload"
      @cancel="pending = null"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SpriteEditor from './SpriteEditor.vue';

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
const thumbDataUrls = reactive<Record<string, string>>({});
const fileInput = ref<HTMLInputElement>();
const pending = ref<{ dataUrl: string; fileName: string } | null>(null);

function spriteUrl(sprite: SpriteInfo): string {
  if (sprite.is_preset) return sprite.url;
  return thumbDataUrls[sprite.id] || '';
}

async function loadThumbnail(sprite: SpriteInfo) {
  if (sprite.is_preset) return;
  try {
    const b64 = await invoke<string>('read_sprite_preview', { path: sprite.url });
    thumbDataUrls[sprite.id] = `data:image/png;base64,${b64}`;
  } catch (e) {
    console.error('Failed to load thumbnail:', e);
  }
}

onMounted(async () => {
  try {
    sprites.value = await invoke<SpriteInfo[]>('list_sprites');
    for (const s of sprites.value) {
      await loadThumbnail(s);
    }
  } catch (e) {
    console.error('Failed to list sprites:', e);
  }
});

function triggerUpload() {
  fileInput.value?.click();
}

function onFileSelected(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = () => {
    pending.value = {
      dataUrl: reader.result as string,
      fileName: file.name.replace(/\.png$/i, ''),
    };
  };
  reader.readAsDataURL(file);

  // Reset so the same file can be re-selected
  input.value = '';
}

async function confirmUpload() {
  if (!pending.value) return;
  try {
    // Extract base64 part from data URL (skip "data:image/png;base64,")
    const b64 = pending.value.dataUrl.split(',')[1];
    const sprite = await invoke<SpriteInfo>('save_sprite_b64', {
      filename: pending.value.fileName,
      b64,
    });
    sprites.value.push(sprite);
    await loadThumbnail(sprite);
    pending.value = null;
  } catch (e) {
    console.error('Failed to save sprite:', e);
  }
}

async function removeSprite(sprite: SpriteInfo) {
  try {
    await invoke('delete_sprite', { id: sprite.id });
    sprites.value = sprites.value.filter(s => s.id !== sprite.id);
    delete thumbDataUrls[sprite.id];
  } catch (e) {
    console.error('Failed to delete sprite:', e);
  }
}
</script>

<style scoped>
.sprite-card {
  position: relative;
}
.sprite-delete {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 50%;
  background: rgba(220, 50, 50, 0.85);
  color: #fff;
  font-size: 12px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
}
.sprite-card:hover .sprite-delete {
  opacity: 1;
}
.sprite-delete:hover {
  background: rgba(200, 30, 30, 1);
}
</style>
