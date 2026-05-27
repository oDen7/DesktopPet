# 墨矩工坊 · 桌面宠物 (MoJu Desktop Pet)

基于 Tauri v2 + Rust 的像素桌面宠物，支持多精灵切换、多显示器跨屏漫游、拖拽交互。

## 技术栈

- **Tauri v2** — Rust 后端，透明无边框置顶窗口
- **Vue 3 + TypeScript** — 设置面板
- **Vite** — 多页构建 (宠物窗口 + 设置窗口)
- **Less** — CSS 预处理
- **CSS Steps** — 精灵图逐帧动画

## 快速开始

```bash
npm install
npm run dev       # 开发模式 (前端热更新 + Tauri 窗口)
npm run build     # 构建生产版本
```

构建 MSI 安装包：

```bash
cd src-tauri
cargo tauri build
```

## 项目结构

```
├── package.json
├── vite.config.ts
├── tsconfig.json
├── src/
│   ├── index.html               # 宠物窗口 (vanilla JS + 路径算法)
│   ├── pet.less                 # 精灵图逐帧动画样式
│   ├── settings.html            # 设置窗口入口
│   └── settings/                # Vue 3 设置应用
│       ├── main.ts
│       ├── App.vue
│       ├── style.less
│       └── components/
│           ├── ToggleSetting.vue # 开关组件
│           ├── SpeedControl.vue  # 速度滑块
│           └── SpritePicker.vue  # 精灵选择 & 上传
├── public/
│   └── sprites/                 # 预设精灵 PNG (4×4 网格)
├── src-tauri/                   # Rust 后端
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── capabilities/
│   └── src/
│       ├── main.rs              # 入口
│       ├── lib.rs               # Tauri 命令 + 菜单 + 窗口管理
│       ├── display.rs           # 多显示器工作区检测 + 边界 clamp
│       └── settings.rs          # 设置持久化 + 精灵库扫描
└── dist/                        # 前端构建输出
```

## 交互

| 操作 | 行为 |
|------|------|
| 鼠标拖拽 | 抓起宠物，松手后自由落体 |
| 右键 | 原生菜单 (跟随鼠标 / 设置 / 关于 / 退出) |
| AI 漫游 | 步长规划 + 正弦摆动，走-歇-走节奏 |
| 多屏跨屏 | 自动检测所有显示器工作区，接缝处平滑穿越 |

## AI 路径算法

基于步长规划的速度向量系统：

- **智能规划器** — 每 1.5~3 秒产出一个计划 (方向 + 步数 + 是否暂停)
- **8 方向加权选择** — 边缘回避 + 桌面中心引力 + 反对角线偏置 + 随机抖动
- **正弦摆动** — 在选定方向叠加垂直正弦扰动，路径自然不僵硬
- **物理反弹** — 碰壁后离散方向反射 + 角度噪声，墙角独立处理
- **接缝检测** — 逐显示器逐边判断是否相邻屏幕，接缝方向不回避可穿过
- **多显示器** — 中心点重叠判定，支持 L 形等异形屏幕组合

## 设置

通过右键菜单打开设置窗口，可调整：

- AI 自动漫游开关
- 跟随鼠标开关
- 移动速度 (0.25x ~ 3x)
- 窗口置顶
- 宠物形象 (预设 + 用户上传)

## 精灵图生成 Prompt

用于 AI 图像生成（DALL-E、Midjourney、PixelLab 等），输出 1024×1024 标准 4×4 网格精灵图。替换 `Subject:` 后的角色描述即可适配不同宠物。

### English

```
A professional 2D pixel art game sprite sheet of a single character, styled exactly like an official 16-bit retro video game asset. The final image must be generated on a 1024x1024 perfectly square canvas.

Subject: A cute, chubby Golden Retriever puppy with thick, vibrant golden-yellow fluffy fur and a soft cream-colored chest. It features classic soft floppy ears, a feathered fluffy tail, and big, circular dark brown eyes with a friendly, joyful, and goofy expression. The puppy's size, shape, coat color, and body proportions must remain 100% mathematically consistent and identical across all 16 frames.

Layout configuration: The canvas must be organized into a perfectly clean, straight 4x4 matrix grid consisting of exactly 16 frames, divided into 4 horizontal rows. Each row must contain 4 sequential animation frames aligned perfectly horizontally and vertically:
- Row 1 (Top Row): 4 frames, FULL FRONT VIEW. The Golden Retriever puppy stands completely still, facing the camera, blinking its eyes peacefully with a happy, smiling expression. (No body rotation, no turning around).
- Row 2 (Second Row): 4 frames, STRICT LEFT-SIDE PROFILE VIEW. A smooth 4-frame walking cycle showing the puppy walking horizontally to the left, with its fluffy feathered tail wagging happily.
- Row 3 (Third Row): 4 frames, FULL FRONT VIEW. The golden puppy is suspended in mid-air from its scruff, body elongated vertically, with all four paws dangling downwards helplessly in a surprised yet funny, derpy expression. Strictly NO human hands, NO fingers, and NO mouse cursors allowed in any frame.
- Row 4 (Bottom Row): 4 frames, FULL FRONT VIEW. Directly continuing the dangling pose from Row 3, showing the puppy free-falling downwards with its eyes tightly closed, its paws raised upwards due to wind resistance, and vertical single-pixel speed lines around it to simulate intense gravity.

Technical Standards for PixelLab processing:
The entire background of the canvas must be a 100% solid, flat, jet-black color (#000000) with absolutely NO gradients, NO shadows, and NO background glow. Every single frame must have razor-sharp, crisp pixel outlines with zero blurry anti-aliasing edges. All negative space must be pure dead black to allow clean transparency removal in post-processing.
```

### 中文

```
专业 2D 像素风游戏精灵图，单角色，严格仿官方 16-bit 复古游戏资产风格。最终图像须生成为 1024×1024 完美正方形画布。

角色主体：一只可爱胖乎乎的金毛幼犬，拥有浓密鲜艳的金黄色蓬松毛发和柔和的奶油色胸部。经典柔软的垂耳，羽毛般蓬松的尾巴，圆圆的大眼睛呈深棕色，表情友好、快乐、带点傻气。幼犬的大小、体型、毛色和身体比例在所有 16 帧中必须保持 100% 数学一致且完全相同。

网格布局：画布须组织为一个干净笔直的 4×4 矩阵网格，严格包含 16 帧，分为 4 个水平行，每行 4 帧水平垂直完美对齐：
- 第一行（顶部）：4 帧，完全正面视角。金毛幼犬完全静止站立，面对镜头，平静眨眼，带着快乐微笑的表情。（不允许身体旋转或转身）。
- 第二行：4 帧，严格的纯左侧面视角。4 帧流畅的走路循环，幼犬水平向左行走，蓬松的羽毛尾巴快乐摇摆。
- 第三行：4 帧，完全正面视角。金毛幼犬被捏住后颈悬在半空中，身体纵向拉长，四肢无助地向下垂落，露出惊讶又有趣的傻傻表情。严格禁止任何人手、手指、鼠标指针出现在任意帧中。
- 第四行（底部）：4 帧，完全正面视角。直接延续第三行的悬空姿势，幼犬自由落体向下坠落，双眼紧闭，四肢因风阻向上扬起，周围有垂直单像素速度线以模拟强烈重力感。

PixelLab 后处理技术标准：
整张画布背景必须为 100% 纯黑（#000000），绝对没有任何渐变、阴影和背景发光。每一帧必须具有锐利清晰的像素轮廓，零模糊抗锯齿边缘。所有空白区域必须是纯死黑，以确保后处理中能干净去除背景生成透明通道。
```

### 使用步骤

1. 将 Prompt 中 `Subject:` 后的角色描述替换为目标角色
2. 提交给 AI 图像生成工具，或使用 [PixelLab](https://www.pixellab.ai) → `Create` → `Animated object/character` 模式直接生成精灵图
3. 将生成图放入 PixelLab → `Remove background` 抠黑底 → `Pixel art correction` 去毛边
4. 导出 PNG，放入 `public/sprites/` 目录（应用会自动扫描注册）
5. 若精灵图尺寸非标准 4×4 均分，在 `src/index.html` 的 `SPRITE_ADJUST` 中配置 `{ w, h }` 或 `{ rowH: [...] }`
