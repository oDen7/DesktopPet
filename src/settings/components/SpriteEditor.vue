<template>
  <div class="editor-overlay" @click.self="$emit('cancel')">
    <div class="editor-panel">
      <h2>精灵图编辑器 — 4×4 帧格参考</h2>
      <p class="editor-desc">请确认精灵图各帧与 4×4 网格对齐，避免动画异常</p>

      <div class="canvas-row">
        <div class="canvas-box">
          <canvas ref="userCanvas" :width="CANVAS_SIZE" :height="CANVAS_SIZE"></canvas>
          <span class="canvas-label">你的图片</span>
          <span class="canvas-info">{{ userInfo }}</span>
        </div>
        <div class="canvas-box">
          <canvas ref="templateCanvas" :width="CANVAS_SIZE" :height="CANVAS_SIZE"></canvas>
          <span class="canvas-label">参考模板 · blackcat</span>
          <span class="canvas-info">256 × 256 / cell: 64 × 64</span>
        </div>
      </div>

      <div class="editor-warn" v-if="warnMsg">{{ warnMsg }}</div>
      <div class="editor-ok" v-else>✓ 图片尺寸符合 4×4 帧格要求</div>

      <div class="editor-actions">
        <button class="btn-cancel" @click="$emit('cancel')">取消</button>
        <button class="btn-confirm" @click="$emit('confirm')">确认上传</button>
      </div>
    </div>
  </div>
</template>

<!--
  精灵图上传预览编辑器。
  在用户确认上传前，在 Canvas 上并排展示用户图片和参考模板，
  叠加 4×4 红色网格线帮助用户判断帧对齐。

  验证规则：
    - 宽高必须为 4 的整数倍 → 每行 4 帧才能均匀切分
    - 宽高应相等（正方形）→ 帧格为正方形动画效果最佳
    - 不满足条件时显示警告但允许上传

  Props:
    dataUrl  — 用户选中图片的 Data URL (data:image/png;base64,...)
    fileName — 文件名（不含扩展名），用于建议的精灵 ID

  Events:
    confirm — 用户点击"确认上传"
    cancel  — 用户点击"取消"或点击遮罩层
-->

<script setup lang="ts">
import { ref, onMounted } from 'vue';

const props = defineProps<{ dataUrl: string; fileName: string }>();
defineEmits<{ confirm: []; cancel: [] }>();

/// Canvas 渲染尺寸（CSS 像素）— 两个 canvas 使用相同尺寸
const CANVAS_SIZE = 280;
const userCanvas = ref<HTMLCanvasElement>();
const templateCanvas = ref<HTMLCanvasElement>();
const userInfo = ref('');
const warnMsg = ref('');

/**
 * 在 Canvas 上绘制 4×4 红色网格参考线。
 *
 * @param ctx      — Canvas 2D 上下文
 * @param imgW     — 原始图片宽度
 * @param imgH     — 原始图片高度
 * @param canvasW  — Canvas 元素宽度
 * @param canvasH  — Canvas 元素高度
 */
function draw4x4Grid(
  ctx: CanvasRenderingContext2D,
  imgW: number,
  imgH: number,
  canvasW: number,
  canvasH: number,
) {
  const scale = Math.min(canvasW / imgW, canvasH / imgH);
  const dw = imgW * scale;
  const dh = imgH * scale;
  const ox = (canvasW - dw) / 2;
  const oy = (canvasH - dh) / 2;

  // 4×4 网格线（红色实线）
  ctx.strokeStyle = 'rgba(220, 50, 50, 0.7)';
  ctx.lineWidth = 1.5;
  ctx.setLineDash([]);

  for (let i = 1; i < 4; i++) {
    const vx = ox + (dw / 4) * i;
    ctx.beginPath();
    ctx.moveTo(vx, oy);
    ctx.lineTo(vx, oy + dh);
    ctx.stroke();

    const hy = oy + (dh / 4) * i;
    ctx.beginPath();
    ctx.moveTo(ox, hy);
    ctx.lineTo(ox + dw, hy);
    ctx.stroke();
  }

  // 外边框（红色加粗）
  ctx.strokeStyle = 'rgba(220, 50, 50, 0.9)';
  ctx.lineWidth = 2;
  ctx.strokeRect(ox, oy, dw, dh);
}

/**
 * 将图片加载到 Canvas 并叠加网格线。
 *
 * @param src      — 图片源（URL 或 Data URL）
 * @param canvas   — 目标 Canvas 元素
 * @param showInfo — 是否显示尺寸信息和验证结果
 */
function loadImageToCanvas(
  src: string,
  canvas: HTMLCanvasElement | undefined,
  showInfo: boolean,
) {
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const img = new Image();
  img.onload = () => {
    const scale = Math.min(CANVAS_SIZE / img.width, CANVAS_SIZE / img.height);
    const dw = img.width * scale;
    const dh = img.height * scale;
    const ox = (CANVAS_SIZE - dw) / 2;
    const oy = (CANVAS_SIZE - dh) / 2;

    ctx.clearRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);
    ctx.drawImage(img, ox, oy, dw, dh);
    draw4x4Grid(ctx, img.width, img.height, CANVAS_SIZE, CANVAS_SIZE);

    if (showInfo) {
      const cellW = img.width / 4;
      const cellH = img.height / 4;
      userInfo.value = `${img.width} × ${img.height} / cell: ${cellW} × ${cellH}`;

      if (img.width % 4 !== 0 || img.height % 4 !== 0) {
        warnMsg.value = `⚠ 图片尺寸 ${img.width}×${img.height} 不是 4 的整数倍，动画帧可能偏移。建议使用 256×256 或 512×512 等 4 的倍数尺寸。`;
      } else if (img.width !== img.height) {
        warnMsg.value = `⚠ 图片宽高不等 (${img.width}×${img.height})，帧格不是正方形，动画效果可能受影响。`;
      } else {
        warnMsg.value = '';
      }
    }
  };
  img.onerror = () => {
    warnMsg.value = '⚠ 图片加载失败，请检查文件是否有效。';
  };
  img.src = src;
}

onMounted(() => {
  // 加载用户图片（Data URL，不需要经过 Rust）
  loadImageToCanvas(props.dataUrl, userCanvas.value, true);

  // 加载参考模板（预设 blackcat 精灵图）
  loadImageToCanvas('/sprites/blackcat.png', templateCanvas.value, false);
});
</script>

<style scoped>
.editor-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.editor-panel {
  background: #fff;
  border-radius: 12px;
  padding: 24px 28px;
  max-width: 640px;
  width: 90vw;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}
.editor-panel h2 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
}
.editor-desc {
  font-size: 12px;
  color: #999;
  margin-bottom: 16px;
}
.canvas-row {
  display: flex;
  gap: 20px;
  justify-content: center;
}
.canvas-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.canvas-box canvas {
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  background: #fafafa;
}
.canvas-label {
  font-size: 12px;
  font-weight: 500;
  color: #555;
}
.canvas-info {
  font-size: 11px;
  color: #999;
}
.editor-warn {
  margin-top: 14px;
  padding: 8px 12px;
  background: #fff8e1;
  border: 1px solid #ffe082;
  border-radius: 6px;
  font-size: 11px;
  color: #e65100;
  line-height: 1.5;
}
.editor-ok {
  margin-top: 14px;
  padding: 8px 12px;
  background: #e8f5e9;
  border: 1px solid #a5d6a7;
  border-radius: 6px;
  font-size: 11px;
  color: #2e7d32;
}
.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 16px;
}
.btn-cancel {
  padding: 7px 20px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: #fff;
  color: #666;
  font-size: 13px;
  cursor: pointer;
}
.btn-cancel:hover {
  background: #f5f5f5;
}
.btn-confirm {
  padding: 7px 20px;
  border: none;
  border-radius: 6px;
  background: #4a90d9;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}
.btn-confirm:hover {
  background: #3a7bc8;
}
</style>
