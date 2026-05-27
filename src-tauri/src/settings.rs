//! 设置持久化与精灵图管理模块。
//!
//! ## 功能分组
//!
//! ### 设置持久化
//! * `PetSettings` — 用户配置结构体，JSON 序列化存储于 `$APPDATA/settings.json`
//! * `load_settings()` — 从磁盘读取设置，失败时返回默认值 + 诊断日志
//! * `save_settings()` — 写入设置到磁盘（pretty-printed JSON）
//!
//! ### 精灵图管理
//! * `list_sprites` — 列出所有可用精灵图（预设 + 用户上传）
//! * `get_preset_sprites()` — 扫描预设精灵目录（开发/生产多路径回退）
//! * `find_sprite()` — 根据 ID 查找单个精灵图配置
//! * `save_sprite_b64` — 解码 base64 PNG 并保存到用户精灵目录
//! * `read_sprite_preview` — 读取用户精灵图的 base64 缩略图（带路径安全检查）
//! * `delete_sprite` — 删除用户精灵图文件（禁止删除预设）
//!
//! ### 鼠标查询
//! * `get_cursor_pos` — 通过 enigo 库查询鼠标物理屏幕坐标
//!
//! ## 安全说明
//!
//! * `read_sprite_preview` 和 `delete_sprite` 均包含路径遍历检查：
//!   解析后的路径必须在 `$APPDATA/sprites/` 目录内
//! * `save_sprite_b64` 校验 PNG 文件头 + 5MB 输入上限
//! * 自实现 base64 编解码器，不依赖外部 crate

use serde::{Deserialize, Serialize};
use tauri::Manager;

// ─── 设置数据结构 ─────────────────────────────────────────

/// 用户配置 — JSON 序列化存储于 `$APPDATA/settings.json`。
///
/// 字段对应设置面板中的各个控件：
/// * `ai_enabled` — AI 自动漫游开关
/// * `follow_enabled` — 鼠标跟随模式（与 chase_enabled 互斥）
/// * `chase_enabled` — 追逐/逃离模式（与 follow_enabled 互斥）
/// * `speed_multiplier` — 移动速度倍率（0.25 ~ 3.0）
/// * `always_on_top` — 宠物窗口置顶
/// * `sprite_variant` — 当前选择的精灵图 ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetSettings {
    pub ai_enabled: bool,
    pub follow_enabled: bool,
    #[serde(default)]
    pub chase_enabled: bool,
    pub speed_multiplier: f64,
    pub always_on_top: bool,
    pub sprite_variant: String,
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            ai_enabled: true,
            follow_enabled: false,
            chase_enabled: false,
            speed_multiplier: 1.0,
            always_on_top: true,
            sprite_variant: "blackcat".to_string(),
        }
    }
}

/// 获取设置文件的完整路径。
fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

/// 从磁盘加载用户设置。
///
/// 返回值包含两个部分：
///   (PetSettings, String) — 设置对象 + 诊断日志字符串
///
/// 日志记录加载来源（文件路径）或失败原因，
/// 供设置面板调试信息区域显示。
pub fn load_settings(app: &tauri::AppHandle) -> (PetSettings, String) {
    let path = settings_path(app);
    let path_str = path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|e| e.clone());
    if let Ok(path) = path {
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(s) = serde_json::from_str::<PetSettings>(&data) {
                    return (s, format!("loaded from {}", path_str));
                }
                return (PetSettings::default(), format!("parse error: {}", path_str));
            }
            return (PetSettings::default(), format!("read error: {}", path_str));
        }
        return (PetSettings::default(), format!("file not found: {}", path_str));
    }
    (PetSettings::default(), format!("path error: {}", path_str))
}

/// 保存用户设置到磁盘。
///
/// 自动创建父目录（`$APPDATA/`），以 prettified JSON 格式写入。
pub fn save_settings(app: &tauri::AppHandle, settings: &PetSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

/// 根据精灵 ID 查找单个精灵图配置。
///
/// 先在预设列表中搜索，再到用户精灵目录搜索。
/// 用户精灵通过读取 PNG 文件头获取实际宽高。
pub fn find_sprite(app: &tauri::AppHandle, id: &str) -> Option<SpriteInfo> {
    for s in get_preset_sprites(app) {
        if s.id == id {
            return Some(s);
        }
    }
    if let Ok(data_dir) = app.path().app_data_dir() {
        let path = data_dir.join("sprites").join(format!("{}.png", id));
        if path.exists() {
            let (w, h) = read_png_size(&path).unwrap_or((512, 512));
            return Some(SpriteInfo {
                id: id.to_string(),
                name: id.to_string(),
                is_preset: false,
                w,
                h,
                url: path.to_string_lossy().to_string(),
            });
        }
    }
    None
}

/// 查询鼠标光标当前物理屏幕坐标。
///
/// 使用 enigo 库跨平台查询，前端每 50ms 轮询一次。
#[tauri::command]
pub fn get_cursor_pos() -> Result<serde_json::Value, String> {
    use enigo::Mouse;
    let enigo = enigo::Enigo::new(&enigo::Settings::default())
        .map_err(|e| e.to_string())?;
    let (x, y) = enigo.location().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "x": x, "y": y }))
}

// ─── 精灵图管理 ───────────────────────────────────────────

/// 精灵图元信息 — 由精灵库扫描/查询生成，序列化为 JSON 返回前端。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteInfo {
    /// 精灵图唯一 ID（文件名去掉 .png 扩展名）
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 是否为预设精灵（预设不可删除）
    pub is_preset: bool,
    /// PNG 图片原始宽度（像素），用于计算帧格尺寸
    pub w: u32,
    /// PNG 图片原始高度（像素）
    pub h: u32,
    /// 图片 URL：预设使用相对路径 `/sprites/xxx.png`，用户使用文件系统路径
    pub url: String,
}

/// 读取 PNG 文件的 IHDR 头获取图片宽高。
///
/// PNG 格式：8 字节签名 → 4 字节长度 → 4 字节 "IHDR" →
/// 4 字节宽度（Big Endian） → 4 字节高度（Big Endian）
fn read_png_size(path: &std::path::Path) -> Option<(u32, u32)> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 24 { return None; }
    if &data[0..8] != b"\x89PNG\r\n\x1a\n" { return None; }
    if &data[12..16] != b"IHDR" { return None; }
    let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((w, h))
}

/// 扫描指定目录下的所有 PNG 文件，返回 SpriteInfo 列表。
///
/// 按精灵 ID 字母序排序以保证前端展示顺序稳定。
fn scan_sprites_dir(dir: &std::path::Path, is_preset: bool, url_prefix: &str) -> Vec<SpriteInfo> {
    let mut sprites = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "png") {
                let id = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let (sheet_w, sheet_h) = read_png_size(&path).unwrap_or((512, 512));
                let url = if is_preset {
                    format!("{}{}.png", url_prefix, id)
                } else {
                    path.to_string_lossy().to_string()
                };
                sprites.push(SpriteInfo {
                    id: id.clone(),
                    name: id.clone(),
                    is_preset,
                    w: sheet_w,
                    h: sheet_h,
                    url,
                });
            }
        }
    }
    sprites.sort_by(|a, b| a.id.cmp(&b.id));
    sprites
}

/// 获取所有预设精灵图列表。
///
/// ## 搜索路径优先级
///
/// 为支持开发（dev server）和生产（bundle）两种运行模式，
/// 按以下顺序尝试多个路径：
///
/// 1. `{cwd}/public/sprites` — 开发模式：正在使用的源码目录
/// 2. `{cwd}/dist/sprites` — 开发模式：构建输出目录
/// 3. `{CARGO_MANIFEST_DIR}/../public/sprites` — 备选开发路径
/// 4. `{CARGO_MANIFEST_DIR}/../dist/sprites` — 备选构建路径
/// 5. `{resource_dir}/public/sprites` — 生产模式：Tauri 打包资源目录
///
/// 首个包含至少一个 PNG 文件的目录即为有效路径。
/// 全部失败时使用硬编码回退列表。
pub fn get_preset_sprites(app: &tauri::AppHandle) -> Vec<SpriteInfo> {
    let mut tried = Vec::new();

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    // 开发路径：当前工作目录（dev server 实时反映文件变更）
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("public").join("sprites"));
        candidates.push(cwd.join("dist").join("sprites"));
    }

    // 开发路径：相对于 Cargo.toml 所在目录
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = std::path::PathBuf::from(&manifest);
        candidates.push(base.join("..").join("public").join("sprites"));
        candidates.push(base.join("..").join("dist").join("sprites"));
    }

    // 生产路径：Tauri bundle 资源目录
    if let Ok(res_dir) = app.path().resource_dir() {
        candidates.push(res_dir.join("public").join("sprites"));
    }

    for dir in &candidates {
        tried.push(dir.to_string_lossy().to_string());
        if dir.exists() {
            let sprites = scan_sprites_dir(dir, true, "/sprites/");
            if !sprites.is_empty() {
                return sprites;
            }
        }
    }

    // 全部失败 — 硬编码回退（确保基本功能可用）
    eprintln!("[desktop-pet] preset scan tried: {:?}, using fallback", tried);
    vec![
        SpriteInfo {
            id: "blackcat".into(), name: "黑猫".into(), is_preset: true,
            w: 256, h: 256, url: "/sprites/blackcat.png".into(),
        },
        SpriteInfo {
            id: "Cs55_R".into(), name: "Cs55_R".into(), is_preset: true,
            w: 512, h: 512, url: "/sprites/Cs55_R.png".into(),
        },
        SpriteInfo {
            id: "orangecat".into(), name: "橘猫".into(), is_preset: true,
            w: 256, h: 256, url: "/sprites/orangecat.png".into(),
        },
        SpriteInfo {
            id: "shiba".into(), name: "柴犬".into(), is_preset: true,
            w: 512, h: 530, url: "/sprites/shiba.png".into(),
        },
    ]
}

/// 列出所有可用精灵图（Tauri 命令）。
///
/// 合并预设精灵和用户上传精灵的完整列表。
#[tauri::command]
pub fn list_sprites(app: tauri::AppHandle) -> Result<Vec<SpriteInfo>, String> {
    let mut sprites = get_preset_sprites(&app);

    if let Ok(data_dir) = app.path().app_data_dir() {
        let user_dir = data_dir.join("sprites");
        sprites.append(&mut scan_sprites_dir(&user_dir, false, ""));
    }

    Ok(sprites)
}

/// 保存用户上传的 base64 PNG 精灵图（Tauri 命令）。
///
/// ## 安全检查
///
/// * 输入限制 5MB base64 字符串（约 3.75MB 解码后）
/// * 校验解码后数据为有效 PNG 文件（文件头 magic bytes）
/// * 文件名过滤 — 仅保留 ASCII 字母、数字、下划线、连字符
///
/// @param filename — 文件名（不含扩展名）
/// @param b64 — base64 编码的 PNG 图片数据
/// @returns 新创建的 SpriteInfo
#[tauri::command]
pub fn save_sprite_b64(app: tauri::AppHandle, filename: String, b64: String) -> Result<SpriteInfo, String> {
    if b64.len() > 5 * 1024 * 1024 {
        return Err("Image too large (max 5 MB)".into());
    }
    let data = base64_decode(&b64)?;
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Not a valid PNG file".into());
    }
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let user_dir = data_dir.join("sprites");
    std::fs::create_dir_all(&user_dir).map_err(|e| e.to_string())?;

    let safe_name = filename.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-', "_");
    let dest = user_dir.join(format!("{}.png", safe_name));
    std::fs::write(&dest, &data).map_err(|e| e.to_string())?;

    let id = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
    let (w, h) = read_png_size(&dest).unwrap_or((512, 512));
    Ok(SpriteInfo {
        id: id.clone(),
        name: id,
        is_preset: false,
        w, h,
        url: dest.to_string_lossy().to_string(),
    })
}

// ─── Base64 编解码器 ──────────────────────────────────────

/// 自实现 base64 解码器（不依赖 base64 crate）。
///
/// 使用编译期预计算的查找表（DECODE array），
/// 正确处理 padding（=）和截断输入。
fn base64_decode(b64: &str) -> Result<Vec<u8>, String> {
    const DECODE: [i8; 128] = {
        let mut t = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < chars.len() {
            t[chars[i] as usize] = i as i8;
            i += 1;
        }
        t
    };
    let mut out = Vec::with_capacity(b64.len() * 3 / 4);
    let bytes = b64.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' { break; }
        let b0 = DECODE.get(bytes[i] as usize).copied().unwrap_or(-1);
        let b1 = if i + 1 < bytes.len() && bytes[i + 1] != b'=' {
            DECODE.get(bytes[i + 1] as usize).copied().unwrap_or(-1)
        } else { -1 };
        if b0 < 0 || b1 < 0 { break; }
        out.push(((b0 as u32) << 2 | (b1 as u32) >> 4) as u8);
        let b2 = if i + 2 < bytes.len() && bytes[i + 2] != b'=' {
            DECODE.get(bytes[i + 2] as usize).copied().unwrap_or(-1)
        } else { -1 };
        if b2 >= 0 {
            out.push(((b1 as u32) << 4 | (b2 as u32) >> 2) as u8);
        }
        let b3 = if i + 3 < bytes.len() && bytes[i + 3] != b'=' {
            DECODE.get(bytes[i + 3] as usize).copied().unwrap_or(-1)
        } else { -1 };
        if b3 >= 0 {
            out.push(((b2 as u32) << 6 | b3 as u32) as u8);
        }
        i += 4;
    }
    Ok(out)
}

/// 读取用户精灵图并返回其 base64 编码内容（Tauri 命令）。
///
/// ## 安全检查
///
/// * 路径解析 — 将输入路径 canonicalize 为绝对路径
/// * 目录限制 — 禁止读取 `$APPDATA/sprites/` 之外的任何文件
/// * 文件类型 — 仅允许 PNG 文件
#[tauri::command]
pub fn read_sprite_preview(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let resolved = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|_| "Invalid path".to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let resolved_sprites_dir = data_dir.join("sprites")
        .canonicalize()
        .map_err(|_| "Cannot resolve sprites dir".to_string())?;
    if !resolved.starts_with(&resolved_sprites_dir) {
        return Err("Access denied".into());
    }
    let data = std::fs::read(&resolved).map_err(|e| e.to_string())?;
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Not a valid PNG file".into());
    }
    Ok(base64_encode(&data))
}

/// 自实现 base64 编码器（不依赖 base64 crate）。
///
/// 标准 base64 编码，自动处理 padding（1-2 个 = 号）。
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// 删除用户上传的精灵图文件（Tauri 命令）。
///
/// ## 安全检查
///
/// * 仅允许删除 `$APPDATA/sprites/` 内的文件
/// * 预设精灵图不存在于此目录，天然受保护
#[tauri::command]
pub fn delete_sprite(app: tauri::AppHandle, id: String) -> Result<(), String> {
    // Reject path separators and parent-directory traversal in the id
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("Invalid sprite id".into());
    }
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = data_dir.join("sprites").join(format!("{}.png", id));
    if !path.exists() {
        return Err("File not found".into());
    }
    // Canonicalize both paths before checking containment — Path::starts_with
    // is purely lexical and does not resolve ".." components.
    let resolved = path.canonicalize().map_err(|_| "Invalid path".to_string())?;
    let user_dir = data_dir.join("sprites");
    let resolved_user_dir = user_dir.canonicalize().map_err(|_| "Cannot resolve sprites dir".to_string())?;
    if !resolved.starts_with(&resolved_user_dir) {
        return Err("Access denied".into());
    }
    std::fs::remove_file(&resolved).map_err(|e| e.to_string())
}
