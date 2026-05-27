/**
 * 桌面宠物引擎 — 不可变配置常量与方向数学工具。
 * 纯数据 + 纯函数，不依赖 S 状态对象。
 */
import type { WalkDir } from '@/pet/state/types';

/** 宠物窗口尺寸（正方形），与精灵图单帧尺寸一致 */
export const DISPLAY_SIZE = 128;
/** 基准速度（像素/帧），speed_multiplier=1.0 时的实际速度 */
export const BASE_SPEED = 1.5;
/** 追逐/逃离模式的移动速度（BASE_SPEED × 3） */
export const CHASE_SPEED = BASE_SPEED * 3;
/** 自由落体重力加速度（像素/帧²） */
export const GRAVITY = 0.8;
/** 追逐-冲刺触发距离阈值（像素） */
export const CHASE_BURST_DIST = 150;
/** 屏幕边缘回避距离（像素），宠物中心距边缘小于此值开始转向 */
export const MARGIN = 200;
/** 显示器接缝检测容差（像素），两屏间隔小于此值视为无缝拼接 */
export const GAP = 50;
/** 鼠标静止判定阈值（毫秒），追逐模式下超过此时长鼠标视为静止 */
export const MOUSE_IDLE_THRESHOLD = 1500;
/** AI 热点记忆最大数量，先进先出 */
export const MAX_HOTSPOTS = 6;

/** 八方向行走顺序（0° 起逆时针），用于权重轮询 */
export const WALK_DIRS = [
  'walk-left', 'walk-up-left', 'walk-up', 'walk-up-right',
  'walk-right', 'walk-down-right', 'walk-down', 'walk-down-left',
] as const;

/**
 * 八方向单位向量表。
 * 对角线方向使用 cos(45°) ≈ 0.7071，使对角线移动速度与水平/垂直一致。
 */
export const DIR_VEC: Record<WalkDir, [number, number]> = {
  'walk-left': [-1, 0],
  'walk-right': [1, 0],
  'walk-up': [0, -1],
  'walk-down': [0, 1],
  'walk-up-left': [-0.7071, -0.7071],
  'walk-up-right': [0.7071, -0.7071],
  'walk-down-left': [-0.7071, 0.7071],
  'walk-down-right': [0.7071, 0.7071],
};

/** X 轴方向反射表（左→右、右→左），仅包含在 X 方向上有变化的键 */
const REFLECT_X: Partial<Record<WalkDir, WalkDir>> = {
  'walk-left': 'walk-right', 'walk-right': 'walk-left',
  'walk-up-left': 'walk-up-right', 'walk-up-right': 'walk-up-left',
  'walk-down-left': 'walk-down-right', 'walk-down-right': 'walk-down-left',
};

/** Y 轴方向反射表（上→下、下→上），仅包含在 Y 方向上有变化的键 */
const REFLECT_Y: Partial<Record<WalkDir, WalkDir>> = {
  'walk-up': 'walk-down', 'walk-down': 'walk-up',
  'walk-up-left': 'walk-down-left', 'walk-up-right': 'walk-down-right',
  'walk-down-left': 'walk-up-left', 'walk-down-right': 'walk-up-right',
};

/**
 * 将 atan2 返回的角度（度）映射到最近的八方向行走动画。
 * @param deg — atan2(dy, dx) * 180 / PI 计算的角度值
 * @returns 对应的 WalkDir 枚举值
 */
export function angleToWalkDir(deg: number): WalkDir {
  if (deg > 157.5 || deg <= -157.5) return 'walk-left';
  if (deg > 112.5) return 'walk-up-left';
  if (deg > 67.5) return 'walk-up';
  if (deg > 22.5) return 'walk-up-right';
  if (deg > -22.5) return 'walk-right';
  if (deg > -67.5) return 'walk-down-right';
  if (deg > -112.5) return 'walk-down';
  return 'walk-down-left';
}

/**
 * 将行走方向转换为弧度角。
 * walk-right = 0rad, walk-left = πrad, walk-up = π/2rad
 * @param dir — 行走方向
 * @returns 弧度角度值
 */
export function dirToAngle(dir: WalkDir): number {
  switch (dir) {
    case 'walk-left': return Math.PI;
    case 'walk-up-left': return Math.PI * 0.75;
    case 'walk-up': return Math.PI * 0.5;
    case 'walk-up-right': return Math.PI * 0.25;
    case 'walk-right': return 0;
    case 'walk-down-right': return -Math.PI * 0.25;
    case 'walk-down': return -Math.PI * 0.5;
    case 'walk-down-left': return -Math.PI * 0.75;
    /* istanbul ignore next */
    default: return ((_: never) => 0)(dir);
  }
}

/**
 * 在反射表中查找方向映射，找不到时保持原方向。
 * idle 方向始终原样返回。
 */
function tryReflect(map: Partial<Record<WalkDir, WalkDir>>, dir: WalkDir | 'idle'): WalkDir | 'idle' {
  if (dir === 'idle') return 'idle';
  return map[dir] || dir;
}

/**
 * 根据阻塞方向反射行走方向。
 * 用于墙壁反弹和屏幕边缘回避。
 * @param dir — 当前行走方向
 * @param bl/br/bu/bd — 是否被左/右/上/下边缘阻挡
 * @returns 反射后的行走方向
 */
export function reflectDir(dir: WalkDir | 'idle', bl: boolean, br: boolean, bu: boolean, bd: boolean): WalkDir | 'idle' {
  let result = dir;
  if (bl || br) result = tryReflect(REFLECT_X, result);
  if (bu || bd) result = tryReflect(REFLECT_Y, result);
  return result;
}
