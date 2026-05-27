/**
 * 桌面宠物引擎 — 精灵图加载器。
 * 负责加载预设或用户上传的精灵图文件，
 * 计算逐帧尺寸、写入 CSS 背景属性并注入 @keyframes 动画规则。
 */
import { invoke } from '@tauri-apps/api/core';
import { S } from '@/pet/state';
import { DISPLAY_SIZE } from '@/pet/state';
import { changeState } from '@/pet/rendering/render';
import type { SpriteConfig, SpriteAdjust } from '@/pet/state';

/**
 * 非正方形精灵图的尺寸修正表。
 * 键为精灵 ID，值为高度或宽度覆写值（像素）。
 * 例如 shiba 精灵图实际高度为 530px 而非 4 的整数倍标准高度。
 */
const SPRITE_ADJUST: Record<string, SpriteAdjust> = { 'shiba': { bsH: 530 } };

/**
 * 根据 SpriteConfig 加载精灵图并应用到宠物 DOM 元素。
 *
 * 步骤：
 *   1. 预设精灵通过 HTTP URL 加载，用户精灵通过 Tauri 命令读取 base64
 *   2. 计算精灵图缩放后的实际像素尺寸
 *   3. 写入 CSS background-image / background-size
 *   4. 注入动态 @keyframes sprite-x-anim（水平滚动精灵帧）
 *   5. 通过重置 CSS 类名 + reflow 强制重新计算动画
 *
 * @param sprite — Rust 端 find_sprite 返回的精灵图配置
 */
export async function applySprite(sprite: SpriteConfig): Promise<void> {
  let url: string;
  if (sprite.is_preset) {
    url = sprite.url;
  } else {
    try {
      const b64 = await invoke<string>('read_sprite_preview', { path: sprite.url });
      url = `data:image/png;base64,${b64}`;
    } catch (e) {
      console.error('Failed to load user sprite:', e);
      return;
    }
  }
  const adj = SPRITE_ADJUST[sprite.id] || {};
  const fw = sprite.w / 4;
  const scale = DISPLAY_SIZE / fw;
  const bsW = Math.round(sprite.w * scale);
  const bsH = adj.bsH || Math.round(sprite.h * scale);
  const fh = bsH / 4;
  S.frameH = Math.round(fh);

  S.petEl.style.width = DISPLAY_SIZE + 'px';
  S.petEl.style.height = DISPLAY_SIZE + 'px';
  S.petEl.style.backgroundImage = `url(${url})`;
  S.petEl.style.backgroundSize = `${bsW}px ${bsH}px`;
  S.petEl._rowY = [0, -fh, -2 * fh, -3 * fh];
  S.petEl.setAttribute('data-sprite', sprite.id);
  let styleEl = document.getElementById('dyn-keyframes');
  if (!styleEl) {
    styleEl = document.createElement('style');
    styleEl.id = 'dyn-keyframes';
    document.head.appendChild(styleEl);
  }
  styleEl.textContent = `@keyframes sprite-x-anim { from { background-position-x: 0px; } to { background-position-x: -${bsW}px; } }`;
  const st = S.currentState;
  S.petEl.className = '';
  void S.petEl.offsetWidth;
  changeState(st);
}
