/**
 * 桌面宠物引擎 — 类型定义。
 * 所有共享的 TypeScript 接口和类型别名集中于此文件，
 * 供 pet/state、pet/rendering、pet/behaviors 等子系统统一引用。
 */

/// 二维向量，用于位置、速度、方向等坐标计算
export interface Vec2 {
  x: number;
  y: number;
}

/// 单个显示器的虚拟桌面工作区矩形（不含任务栏）
export interface Display {
  x: number;
  y: number;
  w: number;
  h: number;
}

/// 宠物中心点到四个屏幕边缘的像素距离
export interface EdgeDist {
  dl: number;
  dr: number;
  du: number;
  dd: number;
}

/// 屏幕舒适区 — 从显示器边缘向内收缩 25% 的内部矩形区域
export interface ComfortZone {
  left: number;
  right: number;
  top: number;
  bottom: number;
  centerX: number;
  centerY: number;
}

/// AI 记忆的热点位置（宠物曾在此 idle），用于回访行为
export interface Hotspot {
  x: number;
  y: number;
}

/// 八方向行走动画枚举，与 CSS 类名一一对应
export type WalkDir =
  | 'walk-left'
  | 'walk-up-left'
  | 'walk-up'
  | 'walk-up-right'
  | 'walk-right'
  | 'walk-down-right'
  | 'walk-down'
  | 'walk-down-left';

/// 屏幕边缘标识
export type EdgeSide = 'left' | 'right' | 'top' | 'bottom';

/// AI 移动计划 — 指定方向和步数，由 brain 制定、movement 执行
export interface Plan {
  direction: WalkDir | 'idle';
  totalSteps: number;
  currentStep: number;
  isPause: boolean;
}

/// 精灵图配置 — 由 Rust 端精灵库查询后返回给前端
export interface SpriteConfig {
  id: string;
  is_preset: boolean;
  url: string;
  w: number;
  h: number;
}

/// 用户设置 — 与 Rust 端 PetSettings 结构对应
export interface PetSettings {
  ai_enabled: boolean;
  follow_enabled: boolean;
  chase_enabled: boolean;
  speed_multiplier: number;
  sprite?: SpriteConfig;
}

/// 宠物 DOM 元素的扩展类型，携带精灵图行号缓存
export interface PetElement extends HTMLElement {
  _rowY?: number[];
}

/// Tauri current-pos 事件载荷 — 每次移动后的位置回报
export interface CurrentPosPayload {
  currentX: number;
  currentY: number;
  displays?: Display[];
  blockedX?: boolean;
  blockedY?: boolean;
}

/// Tauri window-position 事件载荷 — 窗口放置/初始化后的位置回报
export interface WindowPosPayload {
  x: number;
  y: number;
  displays?: Display[];
}

/// 时段速度修正因子 — 深夜降速、清晨渐快、白天全速
export interface TimeModifier {
  speedMul: number;
  restBias: number;
}

/// 精灵图尺寸修正 — 用于非正方形精灵图（如 shiba 530px 高度）的背景尺寸调整
export interface SpriteAdjust {
  bsH?: number;
}
