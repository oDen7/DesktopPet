<template>
  <div class="settings-container">
    <h1>桌面宠物设置</h1>

    <div class="settings-group">
      <ToggleSetting v-model="settings.aiEnabled" title="AI 自动漫游" desc="宠物在桌面上自动行走" />
      <ToggleSetting v-model="settings.followEnabled" title="跟随鼠标" desc="宠物跟随光标移动" />
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
import ToggleSetting from './components/ToggleSetting.vue';
import SpeedControl from './components/SpeedControl.vue';
import SpritePicker from './components/SpritePicker.vue';

const settings = reactive({
  aiEnabled: true,
  followEnabled: false,
  speedMultiplier: 1.0,
  alwaysOnTop: true,
  spriteVariant: 'blackcat',
});

const saving = ref(false);
const errorMsg = ref('');
const debugInfo = ref('');

let saveTimer: ReturnType<typeof setTimeout> | null = null;

async function save() {
  errorMsg.value = '';
  try {
    await invoke('update_settings', {
      settings: {
        ai_enabled: settings.aiEnabled,
        follow_enabled: settings.followEnabled,
        speed_multiplier: settings.speedMultiplier,
        always_on_top: settings.alwaysOnTop,
        sprite_variant: settings.spriteVariant,
      },
    });
    saving.value = true;
    setTimeout(() => { saving.value = false; }, 1500);
  } catch (e: any) {
    errorMsg.value = '保存失败: ' + (e?.toString?.() || e);
  }
}

function debouncedSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(save, 300);
}

onMounted(async () => {
  try {
    const s: any = await invoke('get_settings');
    settings.aiEnabled = s.ai_enabled;
    settings.followEnabled = s.follow_enabled;
    settings.speedMultiplier = s.speed_multiplier;
    settings.alwaysOnTop = s.always_on_top;
    settings.spriteVariant = s.sprite_variant;
    if (s._load_log) debugInfo.value = s._load_log;
  } catch (e: any) {
    errorMsg.value = '加载失败: ' + (e?.toString?.() || e);
  }
});

onUnmounted(() => {
  if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
});

watch(
  () => ({ ...settings }),
  () => debouncedSave(),
  {},
);
</script>
