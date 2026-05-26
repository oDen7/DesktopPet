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

<script setup lang="ts">
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

// 跟随和追逐互斥
function onFollowChange(val: boolean) { if (val) settings.chaseEnabled = false; }
function onChaseChange(val: boolean) { if (val) settings.followEnabled = false; }

const saving = ref(false);
const errorMsg = ref('');
const debugInfo = ref('');

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let lastSaved = ''; // 用已保存状态的 JSON 指纹防止 echo 循环
let unlistenSettings: (() => void) | null = null;
let unlistenFocus: (() => void) | null = null;

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

async function save() {
  saveTimer = null;
  const fp = stateFingerprint();
  if (fp === lastSaved) return; // 与上次保存一致，跳过（防止 event echo 循环）
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

function debouncedSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(save, 150);
}

onMounted(async () => {
  await loadSettings();

  // 监听来自菜单的设置变更事件
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

  // 窗口获焦时拉取最新设置（兜底）
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

watch(
  () => ({ ...settings }),
  () => { debouncedSave(); },
);
</script>
