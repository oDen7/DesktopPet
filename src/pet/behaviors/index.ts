/**
 * 桌面宠物引擎 — 行为子系统聚合导出。
 * 包含 AI 决策（brain）、逐帧移动（movement）
 * 和追逐/逃离状态机（chase）三类行为逻辑。
 */
export { getTimeModifier, recordHotspot, checkMouseReaction,
  smartBrain, scheduleBrain } from '@/pet/behaviors/brain';
export { executeChase } from '@/pet/behaviors/chase';
export { pickFollowDir, handleWallBounce, updateMovement, startTick } from '@/pet/behaviors/movement';
