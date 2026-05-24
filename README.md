# 墨矩工坊 · 桌面宠物 (MoJu Desktop Pet)

基于 Tauri + Rust 的像素桌面宠物，一只会漫游、可拖拽、撞墙反弹的奶牛猫。

## 技术栈

- **Electron** — 透明无边框置顶窗口
- **CSS Steps** — 精灵图逐帧动画
- **IPC** — 主进程统一管理窗口位置与多屏边界

## 快速开始

```bash
npm install
npm start
```

## 项目结构

```
├── src/
│   └── main.js     # 主进程：透明窗口、多屏边界、碰撞检测
├── renderer/
│   ├── index.html  # 渲染层：状态机、AI 漫游、拖拽下落
│   ├── style.css   # CSS Steps 切帧动画、方向翻转
│   └── main-3.png  # 4×4 精灵图 (256×256 RGBA)
└── package.json
```

## 精灵图布局 (4×4)

| 行 | 状态 | 说明 |
|---|---|---|
| 1 | idle | 发呆 |
| 2 | walk | 侧面行走（左走翻转，右走原画） |
| 3 | drag | 被抓起 |
| 4 | fall | 下落 |

## 交互

- **鼠标按住拖拽** — 抓起宠物
- **松开** — 自由落体
- **AI 漫游** — 8 方向随机行走 + 边缘排斥算法，偏爱开阔区域
- **多屏支持** — 自动检测所有显示器工作区，宠物可跨屏漫游

## 精灵图生成 Prompt

替换 `[角色特征]` 后可直接用于 AI 图像生成（DALL-E、Midjourney 等），输出标准 4×4 网格精灵图。

### English

```
Professional 2D pixel art game sprite sheet, ultra-sharp 16-bit individual game assets. [Aspect Ratio: 1:1 perfectly square canvas]

Subject: [角色外观、衣服、颜色、体型，强调所有帧必须 100% 保持一致]

Layout: A clean, perfectly aligned 4x4 matrix grid consisting of exactly 16 frames organized strictly into 4 horizontal rows with logical action transitions:
- Row 1 (Idle/Static): 4 sequential frames, FULL FRONT VIEW. The character stands completely still, facing the camera, blinking its eyes peacefully, looking at the player. (No moving head, no turning around).
- Row 2 (Walk): 4 sequential frames, STRICT LEFT-SIDE PROFILE VIEW. The character turns to the side and executes a smooth 4-frame walking/stepping loop as it moves horizontally.
- Row 3 (Drag/Picked up): 4 sequential frames, FULL FRONT VIEW. The character is suspended in mid-air from its collar/scruff/top, body elongated vertically, with its arms and legs/paws flailing helplessly in a surprised expression. Strictly NO mouse cursors.
- Row 4 (Fall): 4 sequential frames, FULL FRONT VIEW. Directly continuing the dangling pose from Row 3. As it falls downwards helplessly, its limbs are raised upwards, its eyes are closed tightly, and vertical speed lines appear behind it to simulate intense gravity.

Alpha & Transparency Control for PixelLab Optimization:
The entire canvas must be isolated on a 100% solid, flat jet-black background (#000000). Every asset must have clean, razor-sharp single-pixel outlines. Strictly NO white padding, NO gray blurry anti-aliasing edges, NO background glow, and NO environment dust/debris artifacts. All negative space must be pure black to ensure seamless chroma-key transparency ripping.
```

### 中文

```
专业2D像素风游戏精灵图大图，超高清晰度16位机游戏资产。【画面比例 1:1 正方形画布】

角色主体：【角色外观、衣服、颜色、体型，强调所有帧必须 100% 保持一致】

网格布局：一个干净、完美对齐的4x4矩阵网格，严格包含16个帧，分为4个水平行，具备严格的动作因果过渡：
- 第一行（静止/交互）：4帧连续序列，完全纯正面视角。角色完全静止站立，面对镜头，平静地眨着眼睛，看着玩家。（头部不允许晃动，不允许转身）。
- 第二行（水平走路）：4帧连续序列，严格的纯左侧面视角。角色转向侧面，展示流畅的4帧横向迈步走路循环，用于水平移动。
- 第三行（被鼠标抓起）：4帧连续序列，完全纯正面视角。角色仿佛被捏住后颈皮或衣领悬空提起来，身体纵向拉长，露出惊讶的表情，四肢无助地向下垂落并挥舞挣扎。严禁出现任何鼠标指针或人类手指图案。
- 第四行（自由落体下落）：4帧连续序列，完全纯正面视角。动作必须是第三行被抓姿势的直接物理延续。在无助受重力下落时，其四肢因风阻向上扬起，紧闭双眼，身后出现垂直向上的速度线以模拟强烈的重力下落感。

PixelLab透明通道与像素优化：
整张画布必须处于100%纯黑背景（#000000）上。每个资产必须具有干净、锐利、没有模糊的单像素硬轮廓线。严禁任何白色填充、严禁灰色模糊的抗锯齿边缘（Anti-aliasing）、严禁背景发光或发散、严禁地面尘土、阴影或碎屑杂质。所有空白区域必须是绝对死黑，以确保在PixelLab中能完美一键抠图并生成透明通道。
```

### 使用步骤

1. 将 Prompt 中的 `[角色特征]` 替换为目标角色描述
2. 提交给 AI 图像生成工具，或使用 [PixelLab](https://www.pixellab.ai) → `Create` → `Animated object/character` 模式直接生成精灵图
3. 将生成图放入 PixelLab → `Remove background` 抠黑底 → `Pixel art correction` 去毛边
4. 导出 PNG，放入 `renderer/` 目录
5. 更新 `style.css` 中的 `background-image` 和尺寸参数
