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
├── CLAUDE.md                    # AI 开发原则
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

Alpha & Transparency Control:
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
4. 导出 PNG，放入 `public/sprites/` 目录（应用会自动扫描注册）
5. 若精灵图尺寸非标准 4×4 均分，在 `src/index.html` 的 `SPRITE_ADJUST` 中配置 `{ w, h }` 或 `{ rowH: [...] }`
