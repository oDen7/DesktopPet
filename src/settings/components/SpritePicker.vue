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

    <!-- 精灵图上传预览/编辑器模态框 — 仅在选中文件时显示 -->
    <SpriteEditor
      v-if="pending"
      :dataUrl="pending.dataUrl"
      :fileName="pending.fileName"
      @confirm="confirmUpload"
      @cancel="pending = null"
    />
  </div>
</template>

<!--
  精灵图选择器组件。

  功能：
    - 展示所有可用精灵图（预设 + 用户上传）的网格预览
    - 点击选中当前精灵，高亮显示蓝色边框
    - 上传新 PNG 精灵图 → 弹出 SpriteEditor 预览/验证
    - 删除用户上传的精灵图（预设不可删除）
    - 用户精灵图通过 Rust read_sprite_preview 读取 base64 缩略图

  Props:
    modelValue — 当前选中的精灵图 ID

  Events:
    update:modelValue — 选择精灵图时触发
-->

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SpriteEditor from './SpriteEditor.vue';

/// 精灵图信息 — 与 Rust 端 SpriteInfo 结构对应
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
/// 用户精灵图的 base64 缩略图缓存（避免重复读取）
const thumbDataUrls = reactive<Record<string, string>>({});
const fileInput = ref<HTMLInputElement>();
/// 待上传文件信息 — 非 null 时显示编辑器模态框
const pending = ref<{ dataUrl: string; fileName: string } | null>(null);

/// 获取精灵图缩略图 URL：预设用直接路径，用户用缓存的 base64
function spriteUrl(sprite: SpriteInfo): string {
  if (sprite.is_preset) return sprite.url;
  return thumbDataUrls[sprite.id] || '';
}

/// 通过 Rust 命令读取用户精灵图的 base64 缩略图并缓存
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

/// 触发隐藏的文件选择器
function triggerUpload() {
  fileInput.value?.click();
}

/// 文件选择后读取为 Data URL → 弹出编辑器
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

  // 重置输入框以允许重复选择同一文件
  input.value = '';
}

/// 确认上传 → 提取 base64 部分 → 调用 Rust save_sprite_b64
async function confirmUpload() {
  if (!pending.value) return;
  try {
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

/// 删除用户精灵图 — 调用 Rust delete_sprite 并从列表中移除
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
