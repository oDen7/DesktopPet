/**
 * 桌面宠物引擎 — 渲染子系统聚合导出。
 * 包含 CSS 精灵动画渲染、精灵图加载
 * 和多显示器几何计算三类功能。
 */
export { changeState, updateSpriteDirection } from '@/pet/rendering/render';
export { applySprite } from '@/pet/rendering/sprite';
export { edgeIsSeam, effectiveEdgeDist, desktopCenter,
  currentDisplayCenter, getComfortZone, wouldLeaveScreen } from '@/pet/rendering/display';
