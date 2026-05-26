use serde::{Deserialize, Serialize};
use tauri::Manager;

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

fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

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

pub fn save_settings(app: &tauri::AppHandle, settings: &PetSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

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

#[tauri::command]
pub fn get_cursor_pos() -> Result<serde_json::Value, String> {
    use enigo::Mouse;
    let enigo = enigo::Enigo::new(&enigo::Settings::default())
        .map_err(|e| e.to_string())?;
    let (x, y) = enigo.location().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "x": x, "y": y }))
}

// ---- Sprite Library ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteInfo {
    pub id: String,
    pub name: String,
    pub is_preset: bool,
    /// Raw PNG image width / height (e.g. 256, 512)
    pub w: u32,
    pub h: u32,
    /// URL for the webview: presets are /sprites/xxx.png, user sprites are file system paths
    pub url: String,
}

fn read_png_size(path: &std::path::Path) -> Option<(u32, u32)> {
    let data = std::fs::read(path).ok()?;
    // PNG signature: 8 bytes, then IHDR at byte 8
    if data.len() < 24 { return None; }
    if &data[0..8] != b"\x89PNG\r\n\x1a\n" { return None; }
    if &data[12..16] != b"IHDR" { return None; }
    let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((w, h))
}

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

pub fn get_preset_sprites(app: &tauri::AppHandle) -> Vec<SpriteInfo> {
    use tauri::Manager;
    let mut tried = Vec::new();

    // Collect candidate dirs:
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    // 1. Dev paths: live source directories (reflect real-time changes)
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("public").join("sprites"));
        candidates.push(cwd.join("dist").join("sprites"));
    }

    // 2. Dev paths: relative to CARGO_MANIFEST_DIR
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = std::path::PathBuf::from(&manifest);
        candidates.push(base.join("..").join("public").join("sprites"));
        candidates.push(base.join("..").join("dist").join("sprites"));
    }

    // 3. Bundle resources (production fallback: build-time copy)
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

    // Fallback: hardcoded presets (should rarely be hit now)
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

#[tauri::command]
pub fn list_sprites(app: tauri::AppHandle) -> Result<Vec<SpriteInfo>, String> {
    let mut sprites = get_preset_sprites(&app);

    // Scan user-uploaded sprites
    if let Ok(data_dir) = app.path().app_data_dir() {
        let user_dir = data_dir.join("sprites");
        sprites.append(&mut scan_sprites_dir(&user_dir, false, ""));
    }

    Ok(sprites)
}

#[tauri::command]
pub fn upload_sprite(app: tauri::AppHandle) -> Result<Option<SpriteInfo>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file = app
        .dialog()
        .file()
        .add_filter("PNG 图片", &["png"])
        .blocking_pick_file();

    let Some(file_path) = file else {
        return Ok(None); // user cancelled
    };

    let path = file_path.as_path().ok_or("invalid file path")?;

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let user_dir = data_dir.join("sprites");
    std::fs::create_dir_all(&user_dir).map_err(|e| e.to_string())?;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("uploaded.png");
    let dest = user_dir.join(filename);

    std::fs::copy(path, &dest).map_err(|e| e.to_string())?;

    let id = dest
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let (w, h) = read_png_size(&dest).unwrap_or((512, 512));
    Ok(Some(SpriteInfo {
        id: id.clone(),
        name: id,
        is_preset: false,
        w,
        h,
        url: dest.to_string_lossy().to_string(),
    }))
}

/// Save base64-encoded PNG data to the user sprites directory.
#[tauri::command]
pub fn save_sprite_b64(app: tauri::AppHandle, filename: String, b64: String) -> Result<SpriteInfo, String> {
    // Reject excessively large inputs (5 MB base64 ≈ 3.75 MB decoded)
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
        let b1 = DECODE.get(bytes.get(i + 1).copied().unwrap_or(b'A') as usize).copied().unwrap_or(-1);
        let b2 = DECODE.get(bytes.get(i + 2).copied().unwrap_or(b'A') as usize).copied().unwrap_or(-1);
        let b3 = DECODE.get(bytes.get(i + 3).copied().unwrap_or(b'A') as usize).copied().unwrap_or(-1);
        if b0 < 0 || b1 < 0 { break; }
        out.push(((b0 as u32) << 2 | (b1 as u32) >> 4) as u8);
        if b2 >= 0 {
            out.push(((b1 as u32) << 4 | (b2 as u32) >> 2) as u8);
        }
        if b3 >= 0 {
            out.push(((b2 as u32) << 6 | b3 as u32) as u8);
        }
        i += 4;
    }
    Ok(out)
}

/// Read a PNG file and return its base64-encoded content for canvas preview.
#[tauri::command]
pub fn read_sprite_preview(app: tauri::AppHandle, path: String) -> Result<String, String> {
    // Resolve both the requested path and app data dir to canonical form
    let resolved = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|_| "Invalid path".to_string())?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let resolved_data_dir = data_dir.canonicalize().map_err(|_| "Cannot resolve app data dir".to_string())?;
    if !resolved.starts_with(&resolved_data_dir) {
        return Err("Access denied".into());
    }
    let data = std::fs::read(&resolved).map_err(|e| e.to_string())?;
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Not a valid PNG file".into());
    }
    Ok(base64_encode(&data))
}

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

/// Copy the selected file into app data dir and register it as a user sprite.
#[tauri::command]
pub fn confirm_sprite_upload(app: tauri::AppHandle, path: String) -> Result<SpriteInfo, String> {
    let src = std::path::Path::new(&path);

    // Reject files larger than 10 MB
    let meta = src.metadata().map_err(|e| format!("Cannot read file: {}", e))?;
    if meta.len() > 10 * 1024 * 1024 {
        return Err("File too large (max 10 MB)".into());
    }

    // Verify PNG header before copying
    {
        use std::io::Read;
        let mut f = std::fs::File::open(src).map_err(|e| format!("Cannot open file: {}", e))?;
        let mut header = [0u8; 8];
        f.read_exact(&mut header).map_err(|_| "Cannot read file header".to_string())?;
        if &header != b"\x89PNG\r\n\x1a\n" {
            return Err("Not a valid PNG file".into());
        }
    }

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let user_dir = data_dir.join("sprites");
    std::fs::create_dir_all(&user_dir).map_err(|e| e.to_string())?;

    let filename = src.file_name().and_then(|s| s.to_str()).unwrap_or("uploaded.png");
    let dest = user_dir.join(filename);
    std::fs::copy(src, &dest).map_err(|e| e.to_string())?;

    let id = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
    let (w, h) = read_png_size(&dest).unwrap_or((512, 512));
    Ok(SpriteInfo {
        id: id.clone(),
        name: id,
        is_preset: false,
        w,
        h,
        url: dest.to_string_lossy().to_string(),
    })
}

/// Delete a user-uploaded sprite file. Preset sprites cannot be deleted.
#[tauri::command]
pub fn delete_sprite(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = data_dir.join("sprites").join(format!("{}.png", id));
    if !path.exists() {
        return Err("File not found".into());
    }
    // Safety check: only delete files inside the app's sprites directory
    let user_dir = data_dir.join("sprites");
    if !path.starts_with(&user_dir) {
        return Err("Invalid sprite path".into());
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}
