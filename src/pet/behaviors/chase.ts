/**
 * 桌面宠物引擎 — 追逐/逃离模式状态机。
 * 实现宠物与鼠标光标之间的追逐交互：
 *   鼠标活跃移动 → 宠物逃离（flee with panic/burst/dodge）
 *   鼠标静止 → 宠物靠近（approach with weighted direction）
 *
 * 核心状态：
 *   - Panic（恐慌）：鼠标活跃时升高逃离紧迫度
 *   - Burst（冲刺）：短暂高速逃离
 *   - Bounce（弹墙）：撞到屏幕边缘后的反弹机动
 *   - Dodge（闪避）：鼠标靠近时的侧向闪避
 *   - Settled（定居）：鼠标静止时宠物靠近后不再移动
 */
import { invoke } from '@tauri-apps/api/core';
import { S } from '@/pet/state';
import {
  DISPLAY_SIZE, MARGIN, MOUSE_IDLE_THRESHOLD,
  CHASE_SPEED, CHASE_BURST_DIST, WALK_DIRS, DIR_VEC,
} from '@/pet/state';
import { changeState, updateSpriteDirection } from '@/pet/rendering';
import { effectiveEdgeDist, desktopCenter, wouldLeaveScreen } from '@/pet/rendering';
import type { WalkDir } from '@/pet/state';

/**
 * 边缘重定向：当速度矢量会导致离开屏幕时，
 * 依次尝试 X/Y 反射 → 朝向桌面中心的随机角度弹射。
 * @returns 重定向后的速度矢量（可能与输入相同如果不会越界）
 */
function redirectFromEdge(
  vx: number, vy: number, pcx: number, pcy: number,
  spreadAngle: number, speedMul: number,
): { x: number; y: number } {
  let nx = vx, ny = vy;
  if (wouldLeaveScreen(nx, 0, S.currentX, S.currentY)) nx = -nx;
  if (wouldLeaveScreen(0, ny, S.currentX, S.currentY)) ny = -ny;
  if (wouldLeaveScreen(nx, ny, S.currentX, S.currentY)) {
    const dc = desktopCenter();
    const toCX = dc.x - pcx, toCY = dc.y - pcy;
    const mag = Math.sqrt(toCX * toCX + toCY * toCY);
    if (mag > 1) {
      const ang = (Math.random() - 0.5) * spreadAngle;
      const cos = Math.cos(ang), sin = Math.sin(ang);
      const cx = toCX / mag, cy = toCY / mag;
      nx = (cx * cos - cy * sin) * CHASE_SPEED * speedMul;
      ny = (cx * sin + cy * cos) * CHASE_SPEED * speedMul;
    }
  }
  return { x: nx, y: ny };
}

/**
 * 追逐/逃离模式每帧执行函数 — 由 movement tick 调用。
 *
 * 状态机流程：
 *   1. Bounce 优先 — 弹墙中持续执行弹射机动，完结后补充 panic 帧
 *   2. Dodge 优先 — 侧向闪避中持续执行，完结后进入冷却
 *   3. 鼠标靠近且冷却结束 → 触发 dodge
 *   4. 鼠标静止 → 靠近鼠标（approach）
 *   5. 鼠标活跃 → 逃离鼠标（flee）
 *
 * 逃离模式综合权重因素：
 *   - 远离鼠标方向加权
 *   - 桌面中心趋向
 *   - 屏幕边缘回避
 *   - 角落紧急弹射
 *   - 随机抖动和 burst 加速
 */
export function executeChase(): void {
  const pcx = S.currentX + DISPLAY_SIZE / 2, pcy = S.currentY + DISPLAY_SIZE / 2;
  const rdx = pcx - S.mouseX, rdy = pcy - S.mouseY;
  const dist = Math.sqrt(rdx * rdx + rdy * rdy);
  const e = effectiveEdgeDist(pcx, pcy);

  const mdx2 = S.mouseX - S.lastMouseX2, mdy2 = S.mouseY - S.lastMouseY2;
  const mouseMoved = Math.sqrt(mdx2 * mdx2 + mdy2 * mdy2) > 3;
  if (mouseMoved) {
    S.lastMouseMoveTime = Date.now();
    S.lastMouseX2 = S.mouseX; S.lastMouseY2 = S.mouseY;
    S.chaseIdleHold = 0;
    S.chaseWalkHold = 0;
    S.chaseSettled = false;
    if (S.chasePanicFrames <= 0) {
      S.chasePanicFrames = 30 + Math.floor(Math.random() * 30);
      S.chaseBurstFrames = 15;
    }
  }
  const mouseIdle = Date.now() - S.lastMouseMoveTime > MOUSE_IDLE_THRESHOLD;

  // Bounce state
  if (S.chaseBounceFrames > 0) {
    S.chaseBounceFrames--;
    const bx = S.chaseBounceDir.x, by = S.chaseBounceDir.y;
    updateSpriteDirection(bx, by);
    if (wouldLeaveScreen(bx, by, S.currentX, S.currentY)) {
      S.chaseBounceDir = redirectFromEdge(bx, by, pcx, pcy, Math.PI * 0.6, 1.5 + Math.random());
    }
    if (Math.random() < 0.4) {
      const jitter = (Math.random() - 0.5) * Math.PI * 0.4;
      const cos = Math.cos(jitter), sin = Math.sin(jitter);
      S.chaseBounceDir = {
        x: S.chaseBounceDir.x * cos - S.chaseBounceDir.y * sin,
        y: S.chaseBounceDir.x * sin + S.chaseBounceDir.y * cos,
      };
    }
    if (dist < 200 && S.mouseX !== -999) {
      const toMouseX = S.mouseX - pcx, toMouseY = S.mouseY - pcy;
      const mouseDot = (S.chaseBounceDir.x * toMouseX + S.chaseBounceDir.y * toMouseY);
      if (mouseDot > 0) {
        const deflect = (Math.random() > 0.5 ? 1 : -1) * Math.PI * 0.3;
        const cos = Math.cos(deflect), sin = Math.sin(deflect);
        S.chaseBounceDir = {
          x: S.chaseBounceDir.x * cos - S.chaseBounceDir.y * sin,
          y: S.chaseBounceDir.x * sin + S.chaseBounceDir.y * cos,
        };
      }
    }
    S.currentVelocity.x = S.chaseBounceDir.x; S.currentVelocity.y = S.chaseBounceDir.y;
    invoke('move_pet', { dx: S.chaseBounceDir.x, dy: S.chaseBounceDir.y });
    if (S.chaseBounceFrames <= 0) {
      S.chasePanicFrames = Math.max(S.chasePanicFrames, 20 + Math.floor(Math.random() * 20));
      S.chaseBurstFrames = Math.max(S.chaseBurstFrames, 10);
    }
    return;
  }

  // Dodge state
  if (S.chaseDodgeFrames > 0) {
    S.chaseDodgeFrames--;
    if (Math.random() < 0.3) {
      const jitter = (Math.random() - 0.5) * 0.5;
      const cos = Math.cos(jitter), sin = Math.sin(jitter);
      S.chaseDodgeDir = {
        x: S.chaseDodgeDir.x * cos - S.chaseDodgeDir.y * sin,
        y: S.chaseDodgeDir.x * sin + S.chaseDodgeDir.y * cos,
      };
    }
    updateSpriteDirection(S.chaseDodgeDir.x, S.chaseDodgeDir.y);
    S.currentVelocity.x = S.chaseDodgeDir.x; S.currentVelocity.y = S.chaseDodgeDir.y;
    invoke('move_pet', { dx: S.chaseDodgeDir.x, dy: S.chaseDodgeDir.y });
    if (S.chaseDodgeFrames <= 0) {
      S.chaseDodgeCooldown = 60 + Math.floor(Math.random() * 30);
      S.chasePanicFrames = Math.max(S.chasePanicFrames, 25);
      S.chaseBurstFrames = Math.max(S.chaseBurstFrames, 12);
    }
    return;
  }
  if (S.chaseDodgeCooldown > 0) S.chaseDodgeCooldown--;

  // Trigger dodge
  if (!mouseIdle && dist < 80 && S.chaseDodgeCooldown <= 0 && Math.random() < 0.4) {
    const mouseAngle = Math.atan2(rdy, rdx);
    const perpAngle = mouseAngle + (Math.random() > 0.5 ? 1 : -1) * Math.PI / 2;
    const dodgeAngle = perpAngle + (Math.random() - 0.5) * 0.6;
    const dodgeSpd = CHASE_SPEED * (3.0 + Math.random());
    S.chaseDodgeDir = { x: Math.cos(dodgeAngle) * dodgeSpd, y: Math.sin(dodgeAngle) * dodgeSpd };
    S.chaseDodgeFrames = 10 + Math.floor(Math.random() * 8);
    updateSpriteDirection(S.chaseDodgeDir.x, S.chaseDodgeDir.y);
    S.currentVelocity.x = S.chaseDodgeDir.x; S.currentVelocity.y = S.chaseDodgeDir.y;
    invoke('move_pet', { dx: S.chaseDodgeDir.x, dy: S.chaseDodgeDir.y });
    return;
  }

  // Mouse idle → approach
  if (mouseIdle) {
    const idleProb = dist < 50 ? 0.7 : dist < 150 ? 0.1 + 0.6 * (150 - dist) / 100 : 0.03;
    const approachFactor = Math.min(0.7, Math.max(0.1, (dist - 40) / 400));
    if (S.chaseSettled) {
      if (S.currentState !== 'idle') changeState('idle');
      if (S.chaseLastDist >= 0 && Math.abs(dist - S.chaseLastDist) < 30) {
        S.chaseLastDist = dist;
        return;
      }
      S.chaseSettled = false;
    }
    S.chaseLastDist = dist;
    if (S.chaseIdleHold > 0) { S.chaseIdleHold--; if (S.currentState !== 'idle') changeState('idle'); return; }
    if (S.chaseWalkHold > 0) { S.chaseWalkHold--; }

    if (Math.random() < idleProb) {
      S.chaseIdleHold = 5 + Math.floor(Math.random() * 10);
      if (dist < 120) S.chaseSettled = true;
      if (S.currentState !== 'idle') changeState('idle');
      return;
    }

    S.chaseWalkHold = 8 + Math.floor(Math.random() * Math.random() * 112);
    const toMouseX = S.mouseX - pcx, toMouseY = S.mouseY - pcy;
    const mouseDist = Math.sqrt(toMouseX * toMouseX + toMouseY * toMouseY);
    const mouseDirX = mouseDist > 1 ? toMouseX / mouseDist : 0;
    const mouseDirY = mouseDist > 1 ? toMouseY / mouseDist : 0;
    const soft = (d: number) => d >= MARGIN ? 1.0 : Math.max(Math.sqrt(d / MARGIN), 0.02);
    const weights = WALK_DIRS.map(dir => {
      let w = 1.0; const v = DIR_VEC[dir];
      if (dir.includes('left')) w *= soft(e.dl); if (dir.includes('right')) w *= soft(e.dr);
      if (dir.includes('up')) w *= soft(e.du); if (dir.includes('down')) w *= soft(e.dd);
      w *= 1.0 + Math.tanh((v[0] * mouseDirX + v[1] * mouseDirY) * 2) * 2.0;
      if (v[0] !== 0 && v[1] !== 0) w *= 0.85;
      w *= 0.8 + Math.random() * 0.4; return w;
    });
    const total = weights.reduce((a, b) => a + b, 0);
    let r = Math.random() * total, chosenDir: WalkDir = WALK_DIRS[0];
    for (let i = 0; i < WALK_DIRS.length; i++) { r -= weights[i]; if (r <= 0) { chosenDir = WALK_DIRS[i]; break; } }
    const bv = DIR_VEC[chosenDir], spd = CHASE_SPEED * approachFactor;
    let vx = bv[0] * spd, vy = bv[1] * spd;
    vx += Math.sin(Date.now() * 0.006) * 0.08 * (-bv[1]); vy += Math.sin(Date.now() * 0.006) * 0.08 * bv[0];
    updateSpriteDirection(vx, vy);
    S.currentVelocity.x = vx; S.currentVelocity.y = vy;
    invoke('move_pet', { dx: vx, dy: vy });
    return;
  }

  // Mouse active → flee
  const minEdge = Math.min(e.dl, e.dr, e.du, e.dd);
  const nearEdge = minEdge < MARGIN;
  const nearL = e.dl < MARGIN * 1.2, nearR = e.dr < MARGIN * 1.2;
  const nearU = e.du < MARGIN * 1.2, nearD = e.dd < MARGIN * 1.2;
  const inCorner = (nearL || nearR) && (nearU || nearD);

  if (inCorner && dist < 500) {
    const dc = desktopCenter();
    const toCX = dc.x - pcx, toCY = dc.y - pcy;
    const mag = Math.sqrt(toCX * toCX + toCY * toCY);
    if (mag > 1) {
      const ang = (Math.random() - 0.5) * Math.PI * 0.5;
      const cos = Math.cos(ang), sin = Math.sin(ang);
      const cx = toCX / mag, cy = toCY / mag;
      const burstSpd = CHASE_SPEED * (2.0 + Math.random());
      S.chaseBounceDir = { x: (cx * cos - cy * sin) * burstSpd, y: (cx * sin + cy * cos) * burstSpd };
      S.chaseBounceFrames = 8 + Math.floor(Math.random() * 8);
      S.chasePanicFrames = Math.max(S.chasePanicFrames, 30);
      updateSpriteDirection(S.chaseBounceDir.x, S.chaseBounceDir.y);
      S.currentVelocity.x = S.chaseBounceDir.x; S.currentVelocity.y = S.chaseBounceDir.y;
    invoke('move_pet', { dx: S.chaseBounceDir.x, dy: S.chaseBounceDir.y });
      return;
    }
  }

  const mouseDirX = dist > 1 ? rdx / dist : 0, mouseDirY = dist > 1 ? rdy / dist : 0;
  const urgency = Math.min(1, Math.max(0, (800 - dist) / 800));
  let mouseStrength = 2.0 + urgency * 3.0;
  if (S.chasePanicFrames > 0) { S.chasePanicFrames--; mouseStrength *= 2.0; }
  const dc = desktopCenter();
  const dToCX = dc.x - pcx, dToCY = dc.y - pcy;
  const dToC = Math.sqrt(dToCX * dToCX + dToCY * dToCY);
  const dirCX = dToC > 1 ? dToCX / dToC : 0, dirCY = dToC > 1 ? dToCY / dToC : 0;
  const centerStrength = nearEdge ? Math.pow(1.0 - minEdge / MARGIN, 2) * 2.0 + 0.5 : 0.15;
  const edgeU = 1.0 + urgency * 2.0;
  const soft = (d: number) => d >= MARGIN ? 1.0 : Math.max(0.5, Math.pow(d / MARGIN, 0.6)) ** edgeU;
  const weights = WALK_DIRS.map(dir => {
    let w = 1.0; const v = DIR_VEC[dir];
    if (dir.includes('left')) w *= soft(e.dl); if (dir.includes('right')) w *= soft(e.dr);
    if (dir.includes('up')) w *= soft(e.du); if (dir.includes('down')) w *= soft(e.dd);
    if (inCorner) {
      if (nearL && dir.includes('left')) w *= 0.3; if (nearR && dir.includes('right')) w *= 0.3;
      if (nearU && dir.includes('up')) w *= 0.3; if (nearD && dir.includes('down')) w *= 0.3;
    }
    w *= 1.0 + Math.tanh((v[0] * mouseDirX + v[1] * mouseDirY) * 2) * mouseStrength;
    w *= 1.0 + Math.tanh((v[0] * dirCX + v[1] * dirCY) * 2) * centerStrength;
    if (v[0] !== 0 && v[1] !== 0) w *= 0.85;
    w *= 0.7 + Math.random() * 0.6; return w;
  });
  const total = weights.reduce((a, b) => a + b, 0);
  let r = Math.random() * total, chosenDir: WalkDir = WALK_DIRS[0];
  for (let i = 0; i < WALK_DIRS.length; i++) { r -= weights[i]; if (r <= 0) { chosenDir = WALK_DIRS[i]; break; } }
  const bv = DIR_VEC[chosenDir];
  if ((dist < CHASE_BURST_DIST * 2 && S.chaseBurstFrames <= 0 && Math.random() < 0.25) || S.chasePanicFrames > 20) {
    S.chaseBurstFrames = 10 + Math.floor(Math.random() * 10);
  }
  let spdMul = 1.0 + urgency * 0.8;
  if (S.chaseBurstFrames > 0) { S.chaseBurstFrames--; spdMul = 2.5; }
  const spd = CHASE_SPEED * spdMul;
  let vx = bv[0] * spd, vy = bv[1] * spd;
  vx += Math.sin(Date.now() * 0.008) * 0.1 * (-bv[1]); vy += Math.sin(Date.now() * 0.008) * 0.1 * bv[0];
  if (wouldLeaveScreen(vx, vy, S.currentX, S.currentY)) {
    const r = redirectFromEdge(vx, vy, pcx, pcy, Math.PI * 0.5, 2.0);
    const bounceAng = (Math.random() - 0.5) * Math.PI * 0.5;
    const cos = Math.cos(bounceAng), sin = Math.sin(bounceAng);
    const rnx = r.x * cos - r.y * sin, rny = r.x * sin + r.y * cos;
    const bounceSpd = CHASE_SPEED * (1.5 + Math.random() * 1.0);
    const bMag = Math.sqrt(rnx * rnx + rny * rny);
    if (bMag > 0.01) {
      S.chaseBounceDir = { x: rnx / bMag * bounceSpd, y: rny / bMag * bounceSpd };
      S.chaseBounceFrames = 8 + Math.floor(Math.random() * 12);
      updateSpriteDirection(S.chaseBounceDir.x, S.chaseBounceDir.y);
      S.currentVelocity.x = S.chaseBounceDir.x; S.currentVelocity.y = S.chaseBounceDir.y;
    invoke('move_pet', { dx: S.chaseBounceDir.x, dy: S.chaseBounceDir.y });
      return;
    }
  }
  updateSpriteDirection(vx, vy);
  invoke('move_pet', { dx: vx, dy: vy });
}
