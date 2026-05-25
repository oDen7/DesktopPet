use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetSettings {
    pub ai_enabled: bool,
    pub follow_enabled: bool,
    pub speed_multiplier: f64,
    pub always_on_top: bool,
    pub sprite_variant: String,
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            ai_enabled: true,
            follow_enabled: false,
            speed_multiplier: 1.0,
            always_on_top: true,
            sprite_variant: "Cs55_R".to_string(),
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
    for s in get_preset_sprites() {
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

pub fn get_preset_sprites() -> Vec<SpriteInfo> {
    let mut tried = Vec::new();

    // Collect candidate dirs:
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    // 1. public/sprites/ from cwd (project root in npm run dev)
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("public").join("sprites"));
        candidates.push(cwd.join("dist").join("sprites"));
    }
    // 2. CARGO_MANIFEST_DIR = src-tauri/, so ../../public/sprites/
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = std::path::PathBuf::from(&manifest);
        candidates.push(base.join("..").join("public").join("sprites"));
        candidates.push(base.join("..").join("dist").join("sprites"));
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

    // Fallback: hardcoded presets
    eprintln!("[desktop-pet] preset scan tried: {:?}, using fallback", tried);
    vec![
        SpriteInfo {
            id: "blackcat".into(), name: "黑猫".into(), is_preset: true,
            w: 256, h: 256,
            url: "/sprites/blackcat.png".into(),
        },
        SpriteInfo {
            id: "Cs55_R".into(), name: "Cs55_R".into(), is_preset: true,
            w: 512, h: 512,
            url: "/sprites/Cs55_R.png".into(),
        },
    ]
}

#[tauri::command]
pub fn list_sprites(app: tauri::AppHandle) -> Result<Vec<SpriteInfo>, String> {
    let mut sprites = get_preset_sprites();

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
