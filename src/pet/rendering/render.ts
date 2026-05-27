/**
 * 桌面宠物引擎 — 视觉状态渲染。
 * 将 S 中的逻辑状态映射为 DOM 元素的 CSS 类名和精灵图行位置。
 */
import { S } from '@/pet/state';
import { BASE_SPEED, angleToWalkDir } from '@/pet/state';

/**
 * 切换宠物的视觉状态。
 * 根据状态字符串设置 CSS 类名（控制精灵图动画序列）、
 * 精灵图行偏移（控制面向方向/动作姿态）和动画播放速度。
 * @param newState — 目标状态字符串（idle / drag / fall / walk-*）
 */
export function changeState(newState: string): void {
  S.currentState = newState;
  const isWalking = newState !== 'idle' && newState !== 'drag' && newState !== 'fall';
  if (isWalking) {
    // 行走速度越快动画帧间隔越短，上限 4x 基准速度
    const speedRatio = Math.min(S.speed / BASE_SPEED, 4);
    S.petEl.style.animationDuration = (0.6 / Math.sqrt(speedRatio)) + 's';
  } else {
    S.petEl.style.animationDuration = '';
  }
  // 精灵图行布局：第 0 行 = idle, 第 1 行 = 行走, 第 2 行 = 拖拽, 第 3 行 = 下落
  const row = S.petEl._rowY || [0, -S.frameH, -2 * S.frameH, -3 * S.frameH];
  if (newState.includes('left')) {
    S.petEl.className = 'pet-walk-left';
    S.petEl.style.backgroundPositionY = row[1] + 'px';
  } else if (newState.includes('right')) {
    S.petEl.className = 'pet-walk-right';
    S.petEl.style.backgroundPositionY = row[1] + 'px';
  } else if (newState === 'walk-up' || newState === 'walk-down') {
    // 上下行走复用右向精灵（无独立的上/下精灵序列）
    S.petEl.className = 'pet-walk-right';
    S.petEl.style.backgroundPositionY = row[1] + 'px';
  } else if (newState === 'drag') {
    S.petEl.className = 'pet-drag';
    S.petEl.style.backgroundPositionY = row[2] + 'px';
  } else if (newState === 'fall') {
    S.petEl.className = 'pet-fall';
    S.petEl.style.backgroundPositionY = row[3] + 'px';
  } else {
    S.petEl.className = 'pet-' + newState;
    S.petEl.style.backgroundPositionY = row[0] + 'px';
  }
}

/**
 * 根据速度向量更新精灵行走方向。
 * 内置 150ms 防抖，避免方向频繁切换导致精灵图闪烁。
 * @param vx — X 方向速度分量
 * @param vy — Y 方向速度分量
 */
export function updateSpriteDirection(vx: number, vy: number): void {
  if (Math.abs(vx) < 0.1 && Math.abs(vy) < 0.1) {
    if (S.currentState !== 'idle') changeState('idle');
    return;
  }
  const now = Date.now();
  if (now - S.lastSpriteChange < 150) return;
  const deg = Math.atan2(vy, vx) * 180 / Math.PI;
  const st = angleToWalkDir(deg);
  if (S.currentState !== st) { changeState(st); S.lastSpriteChange = now; }
}
