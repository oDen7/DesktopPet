/**
 * 桌面宠物引擎 — 拖拽交互。
 * 处理鼠标拖拽移动窗口和右键菜单两种用户交互。
 */
import { invoke } from '@tauri-apps/api/core';
import { S } from '@/pet/state';
import { changeState } from '@/pet/rendering';

/**
 * 绑定拖拽和右键菜单事件监听器。
 * 拖拽流程：mousedown 记录起始位置 → mousemove 实时移动窗口 →
 * mouseup 松手后若产生位移则进入自由落体，否则恢复 idle。
 */
export function setupDrag(): void {
  /** 右键 → 弹出 Rust 原生右键菜单，菜单显示期间暂停宠物移动 */
  S.petEl.addEventListener('contextmenu', async e => {
    e.preventDefault();
    S.menuOpen = true;
    await invoke('show_context_menu');
    S.menuOpen = false;
  });

  S.petEl.addEventListener('mousedown', e => {
    if (e.button !== 0) return;
    S.isDragging = true;
    S.hasMoved = false;
    S.isFalling = false;
    S.dragStartScreenX = e.screenX;
    S.dragStartScreenY = e.screenY;
    S.dragBaseReady = false;
    // 向 Rust 端查询窗口当前物理坐标作为拖拽基准
    invoke<{ x: number; y: number }>('get_window_physical_pos').then(p => {
      S.dragBaseX = p.x;
      S.dragBaseY = p.y;
      S.dragBaseReady = true;
    }).catch(() => {
      // Rust 端查询失败时回退使用 JS 端记录的位置
      S.dragBaseX = S.currentX;
      S.dragBaseY = S.currentY;
      S.dragBaseReady = true;
    });
    changeState('drag');
  });

  /**
   * 鼠标移动 → 计算物理像素偏移量（乘以 devicePixelRatio），
   * 通过 Rust 端 move_pet_absolute 设置窗口绝对位置。
   */
  document.addEventListener('mousemove', e => {
    if (!S.isDragging || !S.dragBaseReady) return;
    S.hasMoved = true;
    const dpr = window.devicePixelRatio || 1;
    const physDx = (e.screenX - S.dragStartScreenX) * dpr;
    const physDy = (e.screenY - S.dragStartScreenY) * dpr;
    invoke('move_pet_absolute', { x: S.dragBaseX + physDx, y: S.dragBaseY + physDy });
  });

  /**
   * 鼠标松开 → 拖拽有位移则进入自由落体（fall），
   * 无位移（单击）则恢复 idle。
   */
  document.addEventListener('mouseup', e => {
    if (!S.isDragging || e.button !== 0) return;
    S.isDragging = false;
    S.dragBaseReady = false;
    if (S.hasMoved) { S.isFalling = true; S.fallVelocity = 0; changeState('fall'); }
    else { changeState('idle'); }
  });
}
