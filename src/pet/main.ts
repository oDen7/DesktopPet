/**
 * 桌面宠物引擎 — 入口文件。
 * 负责 Tauri 事件监听、初始化和启动流程。
 *
 * 启动顺序：
 *   1. 获取窗口当前物理位置（get_position）
 *   2. 加载用户设置（get_settings）
 *   3. 加载精灵图（applySprite）
 *   4. 启动 AI 决策循环（scheduleBrain）
 */
import '@/pet.less';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { S } from '@/pet/state';
import { BASE_SPEED } from '@/pet/state';
import { changeState } from '@/pet/rendering';
import { checkMouseReaction, scheduleBrain } from '@/pet/behaviors';
import { applySprite } from '@/pet/rendering';
import { startTick, handleWallBounce } from '@/pet/behaviors';
import { setupDrag } from '@/pet/input';
import type { CurrentPosPayload, WindowPosPayload, PetSettings } from '@/pet/state';

/** 每 50ms 轮询一次鼠标光标位置，同时检测鼠标惊吓反应 */
setInterval(async () => {
  try {
    const p = await invoke<{ x: number; y: number }>('get_cursor_pos');
    S.mouseX = p.x; S.mouseY = p.y;
  } catch (e) { console.warn('Cursor poll failed:', e); }
  checkMouseReaction();
}, 50);

/**
 * Rust 端每帧移动后回报窗口新位置。
 * blockedX/Y 表示移动被屏幕边缘阻挡，此时触发墙壁反弹。
 */
listen<CurrentPosPayload>('current-pos', (e) => {
  S.currentX = e.payload.currentX; S.currentY = e.payload.currentY;
  if (e.payload.displays) S.displays = e.payload.displays;
  if (e.payload.blockedX || e.payload.blockedY) {
    handleWallBounce(
      !!e.payload.blockedX && S.currentVelocity.x < 0,
      !!e.payload.blockedX && S.currentVelocity.x > 0,
      !!e.payload.blockedY && S.currentVelocity.y < 0,
      !!e.payload.blockedY && S.currentVelocity.y > 0,
    );
  }
});

/** 窗口被拖拽或初始放置后回报位置及显示器列表 */
listen<WindowPosPayload>('window-position', (e) => {
  S.windowStartX = e.payload.x; S.windowStartY = e.payload.y;
  if (e.payload.displays) S.displays = e.payload.displays;
});

/**
 * 用户在设置面板修改配置后 Rust 端发送此事件。
 * 处理：更新模式开关 → 调整速度 → 切换对应的 AI 模式 → 加载新精灵图。
 */
listen<PetSettings>('settings-changed', (e) => {
  const s = e.payload;
  S.aiEnabled = s.ai_enabled; S.followEnabled = s.follow_enabled; S.chaseEnabled = s.chase_enabled;
  S.speed = BASE_SPEED * s.speed_multiplier;
  S.plannedSpeed = S.speed;
  const vm = Math.sqrt(S.currentVelocity.x * S.currentVelocity.x + S.currentVelocity.y * S.currentVelocity.y);
  if (vm > 0.01) {
    S.currentVelocity.x = S.currentVelocity.x / vm * S.speed;
    S.currentVelocity.y = S.currentVelocity.y / vm * S.speed;
  }
  // Chase mode doesn't use the brain timer — clear it to avoid idle loop
  S.frameSkip = 0;
  if (S.chaseEnabled) {
    if (S.brainTimer) { clearTimeout(S.brainTimer); S.brainTimer = null; }
  } else if (S.followEnabled) {
    if (S.brainTimer) { clearTimeout(S.brainTimer); S.brainTimer = null; }
    S.currentVelocity.x = 0; S.currentVelocity.y = 0;
    changeState('idle');
  } else if (S.aiEnabled) {
    S.currentPlan.currentStep = S.currentPlan.totalSteps;
    scheduleBrain();
  } else {
    if (S.brainTimer) { clearTimeout(S.brainTimer); S.brainTimer = null; }
    changeState('idle');
  }
  if (s.sprite) applySprite(s.sprite);
});

setupDrag();

startTick();

/**
 * 初始化流程：
 *   get_position → 等待 50ms 让 Rust 端完成位置回报 →
 *   若位置仍为零则回退使用 windowStartX/Y →
 *   获取设置 → 应用速度/模式/精灵 → 启动 AI 循环
 */
invoke('get_position').then(() => {
  return new Promise(resolve => setTimeout(resolve, 50));
}).then(() => {
  if (S.currentX === 0 && S.currentY === 0 && (S.windowStartX !== 0 || S.windowStartY !== 0)) {
    S.currentX = S.windowStartX;
    S.currentY = S.windowStartY;
  }
  return invoke<PetSettings>('get_settings');
}).then(s => {
  S.aiEnabled = s.ai_enabled; S.followEnabled = s.follow_enabled; S.chaseEnabled = s.chase_enabled;
  S.speed = BASE_SPEED * s.speed_multiplier;
  S.plannedSpeed = S.speed;
  if (s.sprite) applySprite(s.sprite);
  if (S.chaseEnabled || (S.aiEnabled && !S.followEnabled)) scheduleBrain();
});
