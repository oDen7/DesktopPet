/**
 * 桌面宠物引擎 — 逐帧移动循环。
 * 负责每帧的物理位移计算：自由落体、鼠标跟随、
 * AI 漫游执行、边缘转向、墙壁反弹和速度平滑。
 */
import { invoke } from '@tauri-apps/api/core';
import { S } from '@/pet/state';
import {
  DISPLAY_SIZE, GRAVITY, MARGIN, DIR_VEC,
  angleToWalkDir, dirToAngle, reflectDir,
} from '@/pet/state';
import { changeState, updateSpriteDirection } from '@/pet/rendering';
import { edgeIsSeam, wouldLeaveScreen } from '@/pet/rendering';
import { smartBrain } from '@/pet/behaviors/brain';
import { executeChase } from '@/pet/behaviors/chase';
import type { WalkDir } from '@/pet/state';

/**
 * 根据鼠标和宠物位置选择跟随方向。
 * 距离 < 6px 时返回 idle 避免抖动。
 *
 * @param px — 宠物窗口左上角 X
 * @param py — 宠物窗口左上角 Y
 * @param mx — 鼠标 X 坐标
 * @param my — 鼠标 Y 坐标
 * @returns 最近八方向行走方向或 idle
 */
export function pickFollowDir(px: number, py: number, mx: number, my: number): WalkDir | 'idle' {
  const dx = mx - px - DISPLAY_SIZE / 2, dy = my - py - DISPLAY_SIZE / 2;
  if (Math.abs(dx) < 6 && Math.abs(dy) < 6) return 'idle';
  return angleToWalkDir(Math.atan2(dy, dx) * 180 / Math.PI);
}

/**
 * 处理墙壁反弹 — 当 Rust 端报告移动被屏幕边缘阻挡时调用。
 * 根据阻挡方向反射行走方向并重置速度矢量，
 * 落体状态下的反弹直接终止落体转为 idle。
 *
 * @param blockedLeft/Right/Up/Down — 哪些边缘被阻挡
 */
export function handleWallBounce(blockedLeft: boolean, blockedRight: boolean, blockedUp: boolean, blockedDown: boolean): void {
  if (S.isDragging) return;
  // Chase mode uses its own state (chaseBounceDir etc.), not S.currentPlan
  if (S.chaseEnabled) return;
  if (S.isFalling) { S.isFalling = false; changeState('idle'); return; }
  const dir = S.currentPlan.direction;
  if (dir === 'idle') return;
  const newDir = reflectDir(dir, blockedLeft, blockedRight, blockedUp, blockedDown) as WalkDir;
  S.currentPlan.direction = newDir;
  S.lastDir = newDir;
  S.plannedSpeed = S.speed;
  const angle = dirToAngle(newDir) + (Math.random() - 0.5) * 0.5;
  S.currentVelocity.x = Math.cos(angle) * S.speed;
  S.currentVelocity.y = Math.sin(angle) * S.speed;
  S.currentPlan.totalSteps = 30 + Math.floor(Math.random() * 30);
  S.currentPlan.currentStep = 0;
  S.currentPlan.isPause = false;
}

/**
 * 每帧移动更新 — 由 requestAnimationFrame 驱动。
 *
 * 优先级顺序：
 *   1. 拖拽或菜单打开 → 跳过
 *   2. 自由落体 → 累加重力并下落
 *   3. 鼠标跟随模式 → 向光标平滑移动
 *   4. 追逐/逃离模式 → 委托 executeChase()
 *   5. 鼠标惊吓反应 → 按预设方向跳开
 *   6. AI 漫游 → 执行当前 Plan（walk / pause）
 *
 * AI 漫游时支持：
 *   - 帧跳过（低速时隔帧移动）
 *   - 行进间正弦抖动（wobble）模拟自然行走
 *   - 速度渐入/渐出（ease-in-out ramp）
 *   - 边缘软转向（soft steering away from edges）
 *   - 越界前反弹（预测 + 修正）
 */
export function updateMovement(): void {
  if (S.isDragging || S.menuOpen) return;

  if (S.isFalling) {
    S.fallVelocity += GRAVITY;
    invoke('move_pet', { dx: 0, dy: S.fallVelocity });
    return;
  }

  if (S.followEnabled && S.mouseX !== -999) {
    const tx = S.mouseX - DISPLAY_SIZE / 2, ty = S.mouseY - DISPLAY_SIZE / 2;
    const rdx = tx - S.currentX, rdy = ty - S.currentY;
    const dist = Math.sqrt(rdx * rdx + rdy * rdy);
    if (dist < 8) { changeState('idle'); return; }
    const followDx = rdx * Math.min(1, (S.speed + 0.5) / dist);
    const followDy = rdy * Math.min(1, (S.speed + 0.5) / dist);
    changeState(pickFollowDir(S.currentX, S.currentY, S.mouseX, S.mouseY));
    S.currentVelocity.x = followDx; S.currentVelocity.y = followDy;
    invoke('move_pet', { dx: followDx, dy: followDy });
    return;
  }

  if (S.chaseEnabled && S.mouseX !== -999) {
    executeChase();
    return;
  }

  // AI roam
  const skipMod = S.speed > 3 ? 1 : 2;
  S.frameSkip = (S.frameSkip + 1) % skipMod;
  if (!S.aiEnabled || (skipMod > 1 && S.frameSkip)) return;

  if (S.mouseReaction > 0) {
    S.mouseReaction--;
    updateSpriteDirection(S.mouseReactionDir.x, S.mouseReactionDir.y);
    if (Math.abs(S.mouseReactionDir.x) > 1 || Math.abs(S.mouseReactionDir.y) > 1) {
      invoke('move_pet', { dx: S.mouseReactionDir.x, dy: S.mouseReactionDir.y });
    }
    return;
  }

  if (S.currentPlan.currentStep >= S.currentPlan.totalSteps) { smartBrain(); return; }
  S.currentPlan.currentStep++;

  if (S.currentPlan.isPause) {
    if (S.currentState !== 'idle') changeState('idle');
    const sway = Math.sin(S.currentPlan.currentStep * 0.05) * 0.3;
    if (Math.abs(sway) > 0.15) invoke('move_pet', { dx: sway, dy: 0 });
    return;
  }

  const moveDir = S.currentPlan.direction;
  if (moveDir === 'idle') return;
  const bv = DIR_VEC[moveDir];
  const wobble = Math.sin(S.currentPlan.currentStep * 0.1) * 0.12
               + Math.sin(S.currentPlan.currentStep * 0.23 + 1.5) * 0.08;
  let vx = bv[0] + wobble * (-bv[1]);
  let vy = bv[1] + wobble * bv[0];

  const progress = S.currentPlan.currentStep / S.currentPlan.totalSteps;
  let spdFactor: number;
  if (progress < 0.15) spdFactor = progress / 0.15;
  else if (progress > 0.85) spdFactor = (1.0 - progress) / 0.15;
  else spdFactor = 1.0;
  const currentSpeed = S.plannedSpeed * Math.max(spdFactor, 0.1);

  const nx = S.currentX + vx, ny = S.currentY + vy;
  const ncx = nx + DISPLAY_SIZE / 2, ncy = ny + DISPLAY_SIZE / 2;
  let steerX = 0, steerY = 0;
  for (const d of S.displays) {
    if (ncx < d.x - 2 || ncx >= d.x + d.w + 2 || ncy < d.y - 2 || ncy >= d.y + d.h + 2) continue;
    const dl = ncx - d.x, dr = d.x + d.w - ncx;
    const du = ncy - d.y, dd = d.y + d.h - ncy;
    if (dl < MARGIN && !edgeIsSeam(d, 'left'))   steerX += Math.pow(1.0 - dl / MARGIN, 2) * 2.0;
    if (dr < MARGIN && !edgeIsSeam(d, 'right'))  steerX -= Math.pow(1.0 - dr / MARGIN, 2) * 2.0;
    if (du < MARGIN && !edgeIsSeam(d, 'top'))    steerY += Math.pow(1.0 - du / MARGIN, 2) * 2.0;
    if (dd < MARGIN && !edgeIsSeam(d, 'bottom')) steerY -= Math.pow(1.0 - dd / MARGIN, 2) * 2.0;
  }
  vx += steerX; vy += steerY;

  const vm = Math.sqrt(vx * vx + vy * vy);
  if (vm > 0.01) { vx = vx / vm * currentSpeed; vy = vy / vm * currentSpeed; }
  S.currentVelocity.x = vx; S.currentVelocity.y = vy;

  if (wouldLeaveScreen(vx, vy, S.currentX, S.currentY)) {
    handleWallBounce(vx < 0, vx > 0, vy < 0, vy > 0);
    return;
  }
  updateSpriteDirection(vx, vy);
  invoke('move_pet', { dx: vx, dy: vy });
}

let tickRunning = false;

/**
 * 启动 requestAnimationFrame 主循环。
 * 只启动一次（tickRunning 守卫），每帧调用 updateMovement()。
 */
export function startTick(): void {
  if (tickRunning) return;
  tickRunning = true;
  function tick() { updateMovement(); requestAnimationFrame(tick); }
  requestAnimationFrame(tick);
}
