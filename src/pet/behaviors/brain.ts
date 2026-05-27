/**
 * 桌面宠物引擎 — AI 决策模块。
 * 负责制定移动计划：选方向、定步数、设速度，
 * 考虑时段修正、边缘回避、热点回访和方向连续性。
 */
import { S } from '@/pet/state';
import { DISPLAY_SIZE, MARGIN, MAX_HOTSPOTS, WALK_DIRS, DIR_VEC } from '@/pet/state';
import { changeState } from '@/pet/rendering';
import { effectiveEdgeDist, currentDisplayCenter, getComfortZone } from '@/pet/rendering';
import type { TimeModifier, WalkDir } from '@/pet/state';

/**
 * 根据当前时段返回速度和休息偏好修正值。
 * 深夜(23:00–6:00)：降速 50%，高休息偏好 0.6
 * 清晨(6:00–9:00)：降速 20%，中等休息偏好 0.45
 * 白天(9:00–23:00)：全速，低休息偏好 0.4
 */
export function getTimeModifier(): TimeModifier {
  const h = new Date().getHours();
  if (h >= 23 || h < 6) return { speedMul: 0.5, restBias: 0.6 };
  if (h < 9) return { speedMul: 0.8, restBias: 0.45 };
  return { speedMul: 1.0, restBias: 0.4 };
}

/**
 * 记录当前停留位置为 AI 热点。
 * 条件：(1) idle 期间位移 ≤ MARGIN，(2) 附近 100px 内无已有热点。
 * 热点上限 MAX_HOTSPOTS，超出时移除最早记录。
 */
export function recordHotspot(): void {
  if (S.idleStartX < 0) return;
  const dx = S.currentX - S.idleStartX, dy = S.currentY - S.idleStartY;
  if (dx * dx + dy * dy < MARGIN * MARGIN) {
    const dup = S.hotspots.some(h => Math.abs(h.x - S.currentX) < 100 && Math.abs(h.y - S.currentY) < 100);
    if (!dup) {
      S.hotspots.push({ x: S.currentX, y: S.currentY });
      if (S.hotspots.length > MAX_HOTSPOTS) S.hotspots.shift();
    }
  }
  S.idleStartX = -1;
}

/**
 * 检测鼠标快速移动是否触发宠物惊吓反应。
 * 触发条件：鼠标移速 > 20px/帧，距离宠物 < 200px，未在冷却中。
 * 反应分为两种：(80%)小额移动保持原位、(20%)朝远离鼠标方向跳开。
 */
export function checkMouseReaction(): void {
  if (S.isDragging || S.followEnabled || S.chaseEnabled || !S.aiEnabled) return;
  const mdx = S.mouseX - S.prevMouseX, mdy = S.mouseY - S.prevMouseY;
  const mSpeed = Math.sqrt(mdx * mdx + mdy * mdy);
  const pcx = S.currentX + DISPLAY_SIZE / 2, pcy = S.currentY + DISPLAY_SIZE / 2;
  const distToMouse = Math.sqrt((S.mouseX - pcx) ** 2 + (S.mouseY - pcy) ** 2);
  if (mSpeed > 20 && distToMouse < 200 && S.mouseReaction <= 0) {
    S.mouseReaction = 20 + Math.floor(Math.random() * 20);
    if (Math.random() < 0.8) {
      const angle = Math.atan2(S.mouseY - pcy, S.mouseX - pcx);
      S.mouseReactionDir = { x: Math.cos(angle) * 0.5, y: Math.sin(angle) * 0.5 };
    } else {
      const angle = Math.atan2(pcy - S.mouseY, pcx - S.mouseX);
      S.mouseReactionDir = { x: Math.cos(angle) * 1.5, y: Math.sin(angle) * 1.5 };
    }
  }
  S.prevMouseX = S.mouseX;
  S.prevMouseY = S.mouseY;
}

/**
 * 核心 AI 决策：为下一次移动选择方向和参数。
 *
 * 决策流程：
 *   1. 计算时段修正（speed/restBias）
 *   2. 如果上一轮是 pause 且离边缘远，有 restBias 概率继续休息
 *   3. 否则选择新方向：
 *      - 计算八个方向的权重，考虑边缘距离惩罚、目标方向偏好、
 *        上一方向连续性加成和随机抖动
 *      - 随机选择一个方向（权重高的方向被选中的概率大）
 *   4. 目标选择：25%概率回访热点，否则朝向当前屏幕中心或舒适区
 *   5. 调度下一次 AI 决策
 */
export function smartBrain(): void {
  if (S.isDragging || S.isFalling || !S.aiEnabled || S.followEnabled || S.chaseEnabled) return;
  // Skip if a non-pause plan is still in progress — timer fires before plan completion
  if (!S.currentPlan.isPause && S.currentPlan.currentStep < S.currentPlan.totalSteps) return;
  const tm = getTimeModifier();
  const cx = S.currentX + DISPLAY_SIZE / 2, cy = S.currentY + DISPLAY_SIZE / 2;
  const e = effectiveEdgeDist(cx, cy);
  const minEdge = Math.min(e.dl, e.dr, e.du, e.dd);
  const nearEdge = minEdge < MARGIN;

  const wasPaused = S.currentPlan.isPause;
  if (wasPaused) recordHotspot();
  if (!nearEdge && !wasPaused && Math.random() < tm.restBias) {
    S.currentPlan.direction = 'idle';
    S.currentPlan.totalSteps = 45 + Math.floor(Math.random() * 90);
    S.currentPlan.isPause = true;
    S.currentPlan.currentStep = 0;
    S.idleStartX = S.currentX; S.idleStartY = S.currentY;
    S.currentVelocity.x = 0; S.currentVelocity.y = 0;
    changeState('idle');
    scheduleBrain();
    return;
  }

  S.currentPlan.isPause = false;
  S.currentPlan.totalSteps = 60 + Math.floor(Math.random() * 120);
  S.currentPlan.currentStep = 0;

  let targetX: number, targetY: number, hasHotspotTarget = false;
  if (S.hotspots.length > 0 && Math.random() < 0.25) {
    const spot = S.hotspots[Math.floor(Math.random() * S.hotspots.length)];
    targetX = spot.x + DISPLAY_SIZE / 2;
    targetY = spot.y + DISPLAY_SIZE / 2;
    hasHotspotTarget = true;
  }

  const dc = hasHotspotTarget ? { x: targetX!, y: targetY! } : currentDisplayCenter(cx, cy);
  const toCX = dc.x - cx, toCY = dc.y - cy;
  const distToCenter = Math.sqrt(toCX * toCX + toCY * toCY);
  const dirCX = distToCenter > 1 ? toCX / distToCenter : 0;
  const dirCY = distToCenter > 1 ? toCY / distToCenter : 0;

  const comfortZone = getComfortZone(cx, cy);
  let inComfortZone = true;
  if (comfortZone) {
    inComfortZone = cx >= comfortZone.left && cx <= comfortZone.right &&
                   cy >= comfortZone.top && cy <= comfortZone.bottom;
  }

  const edgeProximity = 1.0 - Math.min(1, minEdge / MARGIN);
  const centerStrength = hasHotspotTarget ? 0.5
    : (inComfortZone ? 0.15 : 0.15 + edgeProximity * 0.7);

  /** 边缘距离惩罚函数：距离越近权重越低，< 50px 几乎不可达 */
  const edgePenalty = (d: number): number => {
    if (d >= MARGIN) return 1.0;
    if (d < 50) return 0.1;
    return Math.max(0.1, Math.pow(d / MARGIN, 1.5));
  };

  const weights = WALK_DIRS.map(dir => {
    let w = 1.0;
    const v = DIR_VEC[dir];
    if (v[0] < 0) w *= edgePenalty(e.dl);
    if (v[0] > 0) w *= edgePenalty(e.dr);
    if (v[1] < 0) w *= edgePenalty(e.du);
    if (v[1] > 0) w *= edgePenalty(e.dd);
    const dot = v[0] * dirCX + v[1] * dirCY;
    w *= 1.0 + Math.tanh(dot * 2) * centerStrength;
    if (v[0] !== 0 && v[1] !== 0) w *= 0.85;
    if (S.lastDir === dir) w *= 1.8;
    if (S.lastDir) {
      const lastV = DIR_VEC[S.lastDir];
      const similarity = v[0] * lastV[0] + v[1] * lastV[1];
      if (similarity > 0) w *= 1.0 + similarity * 0.3;
    }
    w *= 0.6 + Math.random() * 0.8;
    return w;
  });

  const total = weights.reduce((a, b) => a + b, 0);
  let r = Math.random() * total;
  let chosenDir: WalkDir = WALK_DIRS[0];
  for (let i = 0; i < WALK_DIRS.length; i++) {
    r -= weights[i];
    if (r <= 0) { chosenDir = WALK_DIRS[i]; break; }
  }
  S.currentPlan.direction = chosenDir;
  S.lastDir = chosenDir;
  S.plannedSpeed = S.speed * tm.speedMul * (0.8 + Math.random() * 0.4);
  scheduleBrain();
}

/**
 * 调度下一次 AI 决策（1.5–3 秒后随机触发）。
 * 每次决策完成后重新调度，形成持续决策循环。
 */
export function scheduleBrain(): void {
  if (S.brainTimer) clearTimeout(S.brainTimer);
  const delay = 1500 + Math.random() * 1500;
  S.brainTimer = setTimeout(() => { smartBrain(); scheduleBrain(); }, delay);
}
