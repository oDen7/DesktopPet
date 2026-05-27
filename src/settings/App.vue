<template>
  <div class="settings-container">
    <h1>桌面宠物设置</h1>

    <div class="settings-group">
      <ToggleSetting v-model="settings.aiEnabled" title="AI 自动漫游" desc="宠物在桌面上自动行走" />
      <ToggleSetting v-model="settings.followEnabled" title="跟随鼠标" desc="宠物跟随光标移动" @update:modelValue="onFollowChange" />
      <ToggleSetting v-model="settings.chaseEnabled" title="追逐模式" desc="宠物躲避鼠标光标" @update:modelValue="onChaseChange" />
      <SpeedControl v-model="settings.speedMultiplier" />
      <ToggleSetting v-model="settings.alwaysOnTop" title="窗口置顶" desc="宠物始终显示在最前" />
      <SpritePicker v-model="settings.spriteVariant" />
    </div>

    <div class="actions">
      <span v-if="saving" class="save-hint">已保存</span>
      <span v-if="errorMsg" class="save-err">{{ errorMsg }}</span>
      <span class="debug-path">{{ debugInfo }}</span>
    </div>
  </div>
</template>

<!--
  设置面板 — 主组件。

  功能：
    - 提供 AI 漫游 / 鼠标跟随 / 追逐模式开关
    - 移动速度滑块 (0.25x-3x)
    - 窗口置顶开关
    - 精灵图选择器（预设 + 用户上传）
    - 所有修改通过防抖写入 Rust 端持久化

  通信机制：
    - 读：invoke('get_settings') 从 Rust 加载当前设置
    - 写：Vue watch 驱动 150ms 防抖 → invoke('update_settings')
    - 外部变更：Rust 端通过 settings-changed 事件推送
    - 窗口获焦：重新拉取设置（兜底同步）
-->

<script setup lang="ts">
/**
 * 响应式设置对象 — 与 Rust 端 PetSettings 结构对应。
 * 每个字段通过 v-model 绑定到对应子组件。
 */
import { reactive, ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import ToggleSetting from './components/ToggleSetting.vue';
import SpeedControl from './components/SpeedControl.vue';
import SpritePicker from './components/SpritePicker.vue';

const settings = reactive({
  aiEnabled: true,
  followEnabled: false,
  chaseEnabled: false,
  speedMultiplier: 1.0,
  alwaysOnTop: true,
  spriteVariant: 'blackcat',
});

/**
 * 跟随和追逐模式互斥 — 开启一个时自动关闭另一个。
 */
function onFollowChange(val: boolean) { if (val) settings.chaseEnabled = false; }
function onChaseChange(val: boolean) { if (val) settings.followEnabled = false; }

const saving = ref(false);
const errorMsg = ref('');
const debugInfo = ref('');

let saveTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * 上次成功写入 Rust 端的设置状态指纹。
 * 用于防止 echo 循环 — 如果 Rust 端回报的变更
 * 与当前保存的状态一致，则跳过重复保存。
 */
let lastSaved = '';

let unlistenSettings: (() => void) | null = null;
let unlistenFocus: (() => void) | null = null;

/// 生成当前设置状态的 JSON 指纹
function stateFingerprint() {
  return JSON.stringify({
    ai_enabled: settings.aiEnabled,
    follow_enabled: settings.followEnabled,
    chase_enabled: settings.chaseEnabled,
    speed_multiplier: settings.speedMultiplier,
    always_on_top: settings.alwaysOnTop,
    sprite_variant: settings.spriteVariant,
  });
}

/// 从 Rust 端加载当前设置
async function loadSettings() {
  try {
    const s: any = await invoke('get_settings');
    settings.aiEnabled = s.ai_enabled;
    settings.followEnabled = s.follow_enabled;
    settings.chaseEnabled = s.chase_enabled;
    settings.speedMultiplier = s.speed_multiplier;
    settings.alwaysOnTop = s.always_on_top;
    settings.spriteVariant = s.sprite_variant;
    if (s._load_log) debugInfo.value = s._load_log;
    lastSaved = stateFingerprint();
  } catch (e: any) {
    errorMsg.value = '加载失败: ' + (e?.toString?.() || e);
  }
}

/// 保存当前设置到 Rust 端（JSON 持久化 + 宠物窗口实时生效）
async function save() {
  saveTimer = null;
  const fp = stateFingerprint();
  if (fp === lastSaved) return;
  errorMsg.value = '';
  try {
    await invoke('update_settings', { settings: JSON.parse(fp) });
    lastSaved = fp;
    saving.value = true;
    setTimeout(() => { saving.value = false; }, 1500);
  } catch (e: any) {
    errorMsg.value = '保存失败: ' + (e?.toString?.() || e);
  }
}

/// 150ms 防抖保存 — 避免连续拖拽滑块时频繁写入
function debouncedSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(save, 150);
}

onMounted(async () => {
  await loadSettings();

  // 监听来自 Rust 端（托盘菜单）的设置变更事件
  unlistenSettings = await listen('settings-changed', (event: any) => {
    if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
    const s = event.payload;
    settings.aiEnabled = s.ai_enabled;
    settings.followEnabled = s.follow_enabled;
    settings.chaseEnabled = s.chase_enabled;
    settings.speedMultiplier = s.speed_multiplier;
    settings.alwaysOnTop = s.always_on_top;
    settings.spriteVariant = s.sprite_variant;
    lastSaved = stateFingerprint();
  });

  // 窗口获焦时重新拉取设置（兜底同步）
  unlistenFocus = await getCurrentWindow().listen('tauri://focus', () => {
    if (saveTimer) return;
    loadSettings();
  });
});

onUnmounted(() => {
  if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; save(); }
  if (unlistenSettings) { unlistenSettings(); unlistenSettings = null; }
  if (unlistenFocus) { unlistenFocus(); unlistenFocus = null; }
});

/// 深度监听设置变更 → 自动触发防抖保存
watch(
  () => ({ ...settings }),
  () => { debouncedSave(); },
);
</script>
