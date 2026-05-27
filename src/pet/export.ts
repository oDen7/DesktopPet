/**
 * 桌面宠物引擎 — 统一公共导出模块。
 * 外部使用者（如设置面板）通过此模块引用所有
 * 宠物引擎的类型、状态常量和行为函数。
 *
 * 模块分组：
 *   Types     — 所有 TypeScript 接口和类型别名
 *   State     — 可变的全局状态对象 S 和不可变常量
 *   Render    — CSS 精灵图渲染和方向控制
 *   Display   — 多显示器几何计算工具
 *   Sprite    — 精灵图加载和应用
 *   Brain     — AI 决策引擎和鼠标反应检测
 *   Movement  — 逐帧移动循环和跟随逻辑
 *   Chase     — 追逐/逃离模式状态机
 *   Drag      — 鼠标拖拽交互
 */

/// Types
export type {
  Vec2, Display, EdgeDist, ComfortZone, Hotspot,
  WalkDir, EdgeSide, Plan, SpriteConfig,
  PetSettings, PetElement, CurrentPosPayload, WindowPosPayload,
  TimeModifier, SpriteAdjust,
} from '@/pet/state';

export { S } from '@/pet/state';

export {
  DISPLAY_SIZE, BASE_SPEED, CHASE_SPEED, GRAVITY,
  CHASE_BURST_DIST, MARGIN, GAP, MOUSE_IDLE_THRESHOLD, MAX_HOTSPOTS,
  WALK_DIRS, DIR_VEC,
  angleToWalkDir, dirToAngle, reflectDir,
} from '@/pet/state';

/// Render
export { changeState, updateSpriteDirection } from '@/pet/rendering';

/// Display geometry
export { edgeIsSeam, effectiveEdgeDist, desktopCenter,
  currentDisplayCenter, getComfortZone, wouldLeaveScreen } from '@/pet/rendering';

/// Sprite
export { applySprite } from '@/pet/rendering';

/// Brain
export { getTimeModifier, recordHotspot, checkMouseReaction,
  smartBrain, scheduleBrain } from '@/pet/behaviors';

/// Movement
export { pickFollowDir, handleWallBounce, updateMovement, startTick } from '@/pet/behaviors';

/// Chase
export { executeChase } from '@/pet/behaviors';

/// Drag
export { setupDrag } from '@/pet/input';
