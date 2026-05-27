/**
 * 桌面宠物引擎 — 状态子系统聚合导出。
 * 包含所有 TypeScript 类型定义、全局可变状态对象 S
 * 和不可变配置常量。
 */
export type {
  Vec2, Display, EdgeDist, ComfortZone, Hotspot,
  WalkDir, EdgeSide, Plan, SpriteConfig,
  PetSettings, PetElement, CurrentPosPayload, WindowPosPayload,
  TimeModifier, SpriteAdjust,
} from '@/pet/state/types';

export { S } from '@/pet/state/state';

export {
  DISPLAY_SIZE, BASE_SPEED, CHASE_SPEED, GRAVITY,
  CHASE_BURST_DIST, MARGIN, GAP, MOUSE_IDLE_THRESHOLD, MAX_HOTSPOTS,
  WALK_DIRS, DIR_VEC,
  angleToWalkDir, dirToAngle, reflectDir,
} from '@/pet/state/constants';
