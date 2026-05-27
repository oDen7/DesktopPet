/**
 * 桌面宠物引擎 — 多显示器几何计算工具。
 * 处理显示器接缝检测、有效屏幕边距、舒适区、
 * 桌面中心和越界预测等虚拟桌面空间几何问题。
 */
import { S } from '@/pet/state';
import { DISPLAY_SIZE, GAP } from '@/pet/state';
import type { Display, EdgeDist, ComfortZone, EdgeSide } from '@/pet/state';

/**
 * 判断显示器指定边缘是否与另一显示器邻接（接缝检测）。
 * 若两显示器边缘间距在 GAP 容差范围内且有重叠区间，
 * 则该边缘视为"内部接缝"而非真正的桌面边界。
 *
 * @param display — 待检测的显示器
 * @param side — 待检测的边缘方向
 * @returns true 表示该边缘邻接另一显示器，宠物可穿越
 */
export function edgeIsSeam(display: Display, side: EdgeSide): boolean {
  for (const other of S.displays) {
    if (other === display) continue;
    const yOverlap = display.y < other.y + other.h && display.y + display.h > other.y;
    const xOverlap = display.x < other.x + other.w && display.x + display.w > other.x;
    switch (side) {
      case 'left':   if (Math.abs(other.x + other.w - display.x) <= GAP && yOverlap) return true; break;
      case 'right':  if (Math.abs(other.x - (display.x + display.w)) <= GAP && yOverlap) return true; break;
      case 'top':    if (Math.abs(other.y + other.h - display.y) <= GAP && xOverlap) return true; break;
      case 'bottom': if (Math.abs(other.y - (display.y + display.h)) <= GAP && xOverlap) return true; break;
    }
  }
  return false;
}

/**
 * 计算宠物中心点到最近有效屏幕边缘的距离。
 * 若边缘是内部接缝则跳过（距离视为 Infinity），
 * 仅统计真正的桌面边界距离。
 *
 * @param cx — 宠物中心 X 坐标
 * @param cy — 宠物中心 Y 坐标
 * @returns 四边距离对象
 */
export function effectiveEdgeDist(cx: number, cy: number): EdgeDist {
  let dl = Infinity, dr = Infinity, du = Infinity, dd = Infinity;
  for (const d of S.displays) {
    if (cx >= d.x - 2 && cx < d.x + d.w + 2 && cy >= d.y - 2 && cy < d.y + d.h + 2) {
      if (!edgeIsSeam(d, 'left'))   dl = Math.min(dl, cx - d.x);
      if (!edgeIsSeam(d, 'right'))  dr = Math.min(dr, d.x + d.w - cx);
      if (!edgeIsSeam(d, 'top'))    du = Math.min(du, cy - d.y);
      if (!edgeIsSeam(d, 'bottom')) dd = Math.min(dd, d.y + d.h - cy);
    }
  }
  return { dl, dr, du, dd };
}

/**
 * 计算整个虚拟桌面的几何中心（所有显示器包围盒的中点）。
 * 用于 AI 居中倾向和边缘逃跑目标。
 */
export function desktopCenter(): { x: number; y: number } {
  let x1 = Infinity, y1 = Infinity, x2 = -Infinity, y2 = -Infinity;
  for (const d of S.displays) {
    if (d.x < x1) x1 = d.x; if (d.y < y1) y1 = d.y;
    if (d.x + d.w > x2) x2 = d.x + d.w; if (d.y + d.h > y2) y2 = d.y + d.h;
  }
  return { x: (x1 + x2) / 2, y: (y1 + y2) / 2 };
}

/**
 * 返回宠物当前所在显示器的几何中心。
 * 若宠物不在任何显示器内则回退至 desktopCenter()。
 *
 * @param cx — 宠物中心 X 坐标
 * @param cy — 宠物中心 Y 坐标
 */
export function currentDisplayCenter(cx: number, cy: number): { x: number; y: number } {
  for (const d of S.displays) {
    if (cx >= d.x && cx < d.x + d.w && cy >= d.y && cy < d.y + d.h) {
      return { x: d.x + d.w / 2, y: d.y + d.h / 2 };
    }
  }
  return desktopCenter();
}

/**
 * 获取宠物当前所在显示器的舒适区（向内收缩 25% 的内部区域）。
 * 宠物在舒适区内时 AI 的屏幕中心趋向力减弱，行为更自由。
 *
 * @param cx — 宠物中心 X 坐标
 * @param cy — 宠物中心 Y 坐标
 * @returns 舒适区矩形，若不在任何显示器内则返回 null
 */
export function getComfortZone(cx: number, cy: number): ComfortZone | null {
  for (const d of S.displays) {
    if (cx >= d.x && cx < d.x + d.w && cy >= d.y && cy < d.y + d.h) {
      const inset = Math.min(d.w, d.h) * 0.25;
      return {
        left: d.x + inset, right: d.x + d.w - inset,
        top: d.y + inset, bottom: d.y + d.h - inset,
        centerX: d.x + d.w / 2, centerY: d.y + d.h / 2,
      };
    }
  }
  return null;
}

/**
 * 预测宠物按给定位移移动后是否会离开所有屏幕（越界检测）。
 * 用于 movement tick 中在发出移动命令前提前阻挡非法位移。
 *
 * @param dx — 预测的 X 方向位移
 * @param dy — 预测的 Y 方向位移
 * @param cx — 当前宠物窗口左上角 X
 * @param cy — 当前宠物窗口左上角 Y
 * @returns true 表示预测位置将完全离开所有显示器
 */
export function wouldLeaveScreen(dx: number, dy: number, cx: number, cy: number): boolean {
  const nx = cx + dx, ny = cy + dy;
  let x1 = Infinity, y1 = Infinity, x2 = -Infinity, y2 = -Infinity;
  for (const d of S.displays) {
    if (d.x < x1) x1 = d.x; if (d.y < y1) y1 = d.y;
    if (d.x + d.w > x2) x2 = d.x + d.w; if (d.y + d.h > y2) y2 = d.y + d.h;
  }
  if (dx < 0 && nx < x1) return true;
  if (dx > 0 && nx + DISPLAY_SIZE > x2) return true;
  if (dy < 0 && ny < y1) return true;
  if (dy > 0 && ny + DISPLAY_SIZE > y2) return true;
  const ccx = nx + DISPLAY_SIZE / 2, ccy = ny + DISPLAY_SIZE / 2;
  for (const d of S.displays) {
    if (ccx >= d.x - 2 && ccx < d.x + d.w + 2 && ccy >= d.y - 2 && ccy < d.y + d.h + 2) return false;
  }
  return true;
}
