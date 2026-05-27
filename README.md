# 墨矩工坊 · 桌面宠物 (MoJu Desktop Pet)

基于 Tauri v2 + Rust 的像素桌面宠物，支持多精灵切换、多显示器跨屏漫游、拖拽交互、鼠标追逐。

## 技术栈

- **Tauri v2** — Rust 后端，透明无边框置顶窗口
- **TypeScript** — 宠物引擎（AI 决策、移动物理、精灵渲染）
- **Vue 3 + TypeScript** — 设置面板
- **Vite** — 多页构建（宠物窗口 + 设置窗口）
- **Less** — CSS 预处理
- **CSS Steps** — 精灵图逐帧动画

## 快速开始

```bash
npm install
npm run dev       # 开发模式（前端热更新 + Tauri 窗口）
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
│   ├── index.html                     # 宠物窗口入口（128×128 无边框透明）
│   ├── pet.less                       # 精灵图逐帧动画样式
│   ├── settings.html                  # 设置窗口入口
│   ├── pet/                           # 宠物引擎（TypeScript 模块化架构）
│   │   ├── main.ts                    # 入口：Tauri 事件绑定 + 启动流程
│   │   ├── export.ts                  # 公共 API 统一导出（barrel）
│   │   ├── state/                     # 数据层
│   │   │   ├── types.ts               # 类型与接口定义
│   │   │   ├── state.ts               # 共享可变状态对象 S
│   │   │   ├── constants.ts           # 物理常量 + 方向数学工具
│   │   │   └── index.ts
│   │   ├── behaviors/                 # 行为层
│   │   │   ├── brain.ts               # AI 决策引擎（时段修正、热点记忆）
│   │   │   ├── chase.ts               # 追逐/逃离状态机
│   │   │   ├── movement.ts            # 每帧移动执行（跟随/AI 漫游/物理）
│   │   │   └── index.ts
│   │   ├── rendering/                 # 渲染层
│   │   │   ├── render.ts              # 视觉状态 → DOM（CSS 类名 + 精灵行）
│   │   │   ├── sprite.ts              # 精灵图加载与动画注入
│   │   │   ├── display.ts             # 多显示器几何计算
│   │   │   └── index.ts
│   │   └── input/                     # 交互层
│   │       ├── drag.ts                # 鼠标拖拽 + 右键菜单
│   │       └── index.ts
│   └── settings/                      # Vue 3 设置应用
│       ├── main.ts
│       ├── App.vue
│       ├── style.less
│       └── components/
│           ├── ToggleSetting.vue      # 开关组件
│           ├── SpeedControl.vue       # 速度滑块
│           ├── SpritePicker.vue       # 精灵选择 & 上传
│           └── SpriteEditor.vue       # 精灵裁剪预览
├── public/
│   └── sprites/                       # 预设精灵 PNG
├── src-tauri/                         # Rust 后端
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                    # 入口 + Windows 控制台屏蔽
│       ├── lib.rs                     # Tauri 命令 + 托盘菜单 + 窗口管理
│       ├── display.rs                 # 多显示器工作区检测 + 坐标钳制
│       └── settings.rs                # 设置持久化 + 精灵库管理 + Base64 编解码
└── dist/                              # 前端构建输出
```

## 交互

| 操作 | 行为 |
|------|------|
| 鼠标拖拽 | 抓起宠物自由移动，松手后自由落体 |
| 右键 | 原生菜单（跟随鼠标 / 追逐模式 / 设置 / 关于 / 退出） |
| AI 漫游 | 步长规划 + 正弦摆动 + 缓入缓出速度曲线，走-歇-走自然节奏 |
| 跟随鼠标 | 八方向平滑跟随，< 8px 停止避免抖动 |
| 追逐模式 | 宠物逃离鼠标 + 静止时靠近，含冲刺/弹墙/闪避多状态 |
| 多屏跨屏 | 显示器接缝自动检测，无缝穿越异形屏幕组合 |

## 宠物引擎架构

基于共享状态的模块化设计，所有子系统直接读写 `S` 对象：

```
main.ts (入口 + 事件绑定)
  ├── state/        S 共享状态 + 常量 + 类型
  ├── behaviors/    每帧 tick → 模式分发 → AI/跟随/追逐
  │   ├── brain     1.5~3s 定时决策，八方向权重投票
  │   ├── chase     鼠标活性检测 → 逃离/靠近状态机
  │   └── movement  requestAnimationFrame 主循环
  ├── rendering/    CSS 精灵图切换 + 多显示器布局
  │   ├── render    状态 → DOM（类名、行偏移、动画速度）
  │   ├── sprite    精灵图加载（预设 URL / 用户 base64）
  │   └── display   屏幕边界、舒适区、越界预测
  └── input/        拖拽位移计算 + 右键菜单触发
```

### 移动算法

- **八方向行走**：8 个单位向量（含 cos45° 对角线归一化），方向平滑过渡
- **边缘回避**：距屏幕边缘 < 200px 时二次方斥力转向，接缝方向穿越不受阻
- **正弦抖动**：双频叠加（0.1 + 0.23 Hz），模拟自然行走左右摇摆
- **速度曲线**：前 15% 缓入 → 中间匀速 → 后 15% 缓出
- **墙壁反弹**：被边缘阻挡后方向反射 + 角度噪声，墙角朝桌面中心弹射
- **时段修正**：深夜(23-6)降速 50% 偏好休息，清晨(6-9)降速 20%，白天全速

### 追逐/逃离状态机

```
鼠标活跃移动 → 逃离（flee）
  ├── Panic: 紧迫度提升，逃离速度加权 x2
  ├── Burst: 短距冲刺（2.5x 速度）
  ├── Bounce: 碰壁后弹射转向 + 随机抖动
  ├── Dodge: 鼠标 < 80px 时侧向闪避（冷却 60~90 帧）
  └── Corner: 角落紧急朝桌面中心弹射

鼠标静止 > 1.5s → 靠近（approach）
  ├── 距离权重：越近休息概率越高（< 50px → 70%）
  ├── Settled: < 120px 且稳定时不再移动
  └── 随机行走：朝鼠标方向慢速靠近
```

## 设置

通过右键菜单打开设置窗口，可调整：

- AI 自动漫游开关
- 跟随鼠标开关（与追逐模式互斥）
- 追逐模式开关（与跟随鼠标互斥）
- 移动速度（0.25x ~ 3x）
- 窗口置顶
- 宠物形象（预设 4 款 + 用户上传，支持裁剪预览）

## 安全策略

- **CSP**：`default-src 'self'`，限制脚本和样式来源
- **Asset Protocol**：作用域收窄为 `$APPDATA/**`，仅允许读取应用数据目录
- **路径遍历防护**：文件路径 canonicalize 后检查目录归属，拒绝 `..` 注入
- **上传校验**：PNG 魔术字节校验 + 文件名过滤 + 5MB 大小限制

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
5. 若非标准正方形精灵，在 `src/pet/rendering/sprite.ts` 的 `SPRITE_ADJUST` 中配置高度覆写值
