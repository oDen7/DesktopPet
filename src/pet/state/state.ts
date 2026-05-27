/**
 * 桌面宠物引擎 — 共享可变状态。
 * 所有子系统（brain、movement、chase、drag、rendering）
 * 均直接读写此对象，不经过 getter/setter 封装。
 */
import type { PetElement, Display, Vec2, WalkDir, Plan, Hotspot } from '@/pet/state/types';

export const S = {
  /** 宠物 DOM 元素，携带精灵图行号缓存 */
  petEl: document.getElementById('pet')! as PetElement,
  /** 单帧精灵图高度（像素），由精灵加载时更新 */
  frameH: 128,

  /** 窗口左上角在虚拟桌面中的 X 坐标 */
  currentX: 0,
  /** 窗口左上角在虚拟桌面中的 Y 坐标 */
  currentY: 0,
  /** 所有显示器的虚拟坐标矩形数组，由 Rust 端回报 */
  displays: [{ x: 0, y: 0, w: 1920, h: 1032 }] as Display[],
  /** 当前帧实际移动速度向量 */
  currentVelocity: { x: 0, y: 0 } as Vec2,
  /** 当前有效速度（像素/帧），由 settings.speed_multiplier * BASE_SPEED 决定 */
  speed: 1.5,
  /** AI 计划中指定的目标速度 */
  plannedSpeed: 1.5,

  /** 是否启用 AI 漫游模式 */
  aiEnabled: true,
  /** 是否启用鼠标跟随模式 */
  followEnabled: false,
  /** 是否启用追逐/逃离模式 */
  chaseEnabled: false,

  /** AI 移动计划：方向、总帧数、已执行帧数、是否为暂停 */
  currentPlan: { direction: 'idle', totalSteps: 0, currentStep: 0, isPause: false } as Plan,
  /** 上一个移动方向，用于 AI 连续性偏好 */
  lastDir: null as WalkDir | null,
  /** AI 决策定时器句柄，用于取消重新调度 */
  brainTimer: null as ReturnType<typeof setTimeout> | null,
  /** 帧跳过计数器，高速时每帧都移动，低速时隔帧移动 */
  frameSkip: 0,

  /** AI 记忆的热点位置（最多 MAX_HOTSPOTS 个） */
  hotspots: [] as Hotspot[],
  /** 开始本次 idle 时的 X 坐标，-1 表示未记录 */
  idleStartX: -1,
  /** 开始本次 idle 时的 Y 坐标 */
  idleStartY: -1,

  /** 鼠标光标在虚拟桌面中的 X 坐标，-999 表示未获取 */
  mouseX: -999,
  /** 鼠标光标在虚拟桌面中的 Y 坐标 */
  mouseY: -999,
  /** 上一帧鼠标 X，用于计算鼠标移动速度 */
  prevMouseX: -999,
  /** 上一帧鼠标 Y */
  prevMouseY: -999,
  /** 惊吓反应剩余帧数，>0 时执行惊吓跳开 */
  mouseReaction: 0,
  /** 惊吓反应的方向向量（低速靠近 = 0.5px/帧，快速跳开 = 1.5px/帧） */
  mouseReactionDir: { x: 0, y: 0 } as Vec2,

  /** 追逐-冲刺剩余帧数 */
  chaseBurstFrames: 0,
  /** 追逐-弹墙剩余帧数 */
  chaseBounceFrames: 0,
  /** 追逐-弹墙方向向量 */
  chaseBounceDir: { x: 0, y: 0 } as Vec2,
  /** 追逐-闪避剩余帧数 */
  chaseDodgeFrames: 0,
  /** 追逐-闪避方向向量 */
  chaseDodgeDir: { x: 0, y: 0 } as Vec2,
  /** 闪避冷却帧数，防止连续闪避 */
  chaseDodgeCooldown: 0,
  /** 鼠标最后一次移动的时间戳（ms），用于判断鼠标静止 */
  lastMouseMoveTime: Date.now(),
  /** 上一帧鼠标 X（追逐专用，独立于 prevMouseX） */
  lastMouseX2: -999,
  /** 上一帧鼠标 Y（追逐专用） */
  lastMouseY2: -999,
  /** 追逐-恐慌状态剩余帧数，提升逃离速度 */
  chasePanicFrames: 0,
  /** 鼠标静止时随机 idle 保持帧数 */
  chaseIdleHold: 0,
  /** 鼠标静止时随机行走保持帧数 */
  chaseWalkHold: 0,
  /** 鼠标静止时是否已进入 settled 状态（靠近后不再移动） */
  chaseSettled: false,
  /** 上一帧到鼠标的距离，用于检测 settled 状态变化 */
  chaseLastDist: -1,

  /** 是否正在拖拽移动窗口 */
  isDragging: false,
  /** 本次拖拽是否产生了位移（用于 mouseup 时判断 fall vs idle） */
  hasMoved: false,
  /** 是否正在自由落体（拖拽松手后） */
  isFalling: false,
  /** 落体速度（像素/帧²），每帧累加 GRAVITY */
  fallVelocity: 0,
  /** 拖拽开始时的鼠标屏幕 X 坐标 */
  dragStartScreenX: 0,
  /** 拖拽开始时的鼠标屏幕 Y 坐标 */
  dragStartScreenY: 0,
  /** 拖拽开始时窗口物理 X 坐标（由 Rust get_window_physical_pos 回报） */
  dragBaseX: 0,
  /** 拖拽开始时窗口物理 Y 坐标 */
  dragBaseY: 0,
  /** dragBaseX/Y 是否已获取完成 */
  dragBaseReady: false,
  /** 窗口被放置/移动后的 X 坐标 */
  windowStartX: 0,
  /** 窗口被放置/移动后的 Y 坐标 */
  windowStartY: 0,
  /** 右键菜单是否正在显示（暂停移动） */
  menuOpen: false,

  /** 当前视觉状态字符串（idle / drag / fall / walk-*） */
  currentState: 'idle',
  /** 上次精灵方向切换的时间戳（ms），用于 150ms 防抖 */
  lastSpriteChange: 0,
};
