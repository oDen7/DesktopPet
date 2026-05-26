mod display;
mod settings;

use settings::{confirm_sprite_upload, delete_sprite, find_sprite, get_cursor_pos, list_sprites, load_settings, read_sprite_preview, save_sprite_b64, save_settings, upload_sprite, PetSettings};
use std::sync::Mutex;
use tauri::{Emitter, Manager, WebviewWindow};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri_plugin_dialog::DialogExt;

struct ContextMenu(Mutex<tauri::menu::Menu<tauri::Wry>>);
struct TrayState(TrayIcon<tauri::Wry>);

fn update_menu_text(state: &tauri::State<'_, ContextMenu>, id: &str, text: &str) {
    if let Ok(ref mut m) = state.0.lock() {
        if let Some(tauri::menu::MenuItemKind::MenuItem(ref item)) = m.get(id) {
            let _ = item.set_text(text);
        }
    }
}

fn update_all_menus(app: &tauri::AppHandle) {
    let (s, _) = load_settings(app);
    let ft = if s.follow_enabled { "跟随鼠标 ✓" } else { "跟随鼠标" };
    let ct = if s.chase_enabled { "追逐模式 ✓" } else { "追逐模式" };

    if let Some(st) = app.try_state::<ContextMenu>() {
        update_menu_text(&st, "follow", ft);
        update_menu_text(&st, "chase", ct);

        // Tray menu is a native snapshot — re-set to pick up updated item text
        if let Some(ts) = app.try_state::<TrayState>() {
            if let Ok(m) = st.0.lock() {
                let _ = ts.0.set_menu(Some(m.clone()));
            }
        }
    }
}

fn handle_menu_event(app: &tauri::AppHandle, id: &str) {
    match id {
        "follow" => {
            let (mut s, _) = load_settings(app);
            s.follow_enabled = !s.follow_enabled;
            if s.follow_enabled { s.chase_enabled = false; }
            let _ = save_settings(app, &s);
            update_all_menus(app);
            let sprite = find_sprite(app, &s.sprite_variant);
            let _ = app.emit("settings-changed", serde_json::json!({
                "ai_enabled": s.ai_enabled, "follow_enabled": s.follow_enabled,
                "chase_enabled": s.chase_enabled,
                "speed_multiplier": s.speed_multiplier, "always_on_top": s.always_on_top,
                "sprite_variant": s.sprite_variant, "sprite": sprite,
            }));
        }
        "chase" => {
            let (mut s, _) = load_settings(app);
            s.chase_enabled = !s.chase_enabled;
            if s.chase_enabled { s.follow_enabled = false; }
            let _ = save_settings(app, &s);
            update_all_menus(app);
            let sprite = find_sprite(app, &s.sprite_variant);
            let _ = app.emit("settings-changed", serde_json::json!({
                "ai_enabled": s.ai_enabled, "follow_enabled": s.follow_enabled,
                "chase_enabled": s.chase_enabled,
                "speed_multiplier": s.speed_multiplier, "always_on_top": s.always_on_top,
                "sprite_variant": s.sprite_variant, "sprite": sprite,
            }));
        }
        "settings" => {
            if let Some(w) = app.get_webview_window("settings") {
                let _ = w.show();
                let _ = w.set_focus();
            } else {
                let _ = tauri::WebviewWindowBuilder::new(
                    app, "settings", tauri::WebviewUrl::App("settings.html".into())
                ).title("设置").inner_size(660.0, 720.0).center().build();
            }
        }
        "about" => show_about_dialog(app),
        "exit" => app.exit(0),
        _ => {}
    }
}

#[tauri::command]
fn move_pet(window: WebviewWindow, app: tauri::AppHandle, dx: f64, dy: f64) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let displays = display::get_all();
    let req_x = pos.x as f64 + dx;
    let req_y = pos.y as f64 + dy;
    let (fx, fy) = display::clamp(req_x.round() as i32, req_y.round() as i32, &displays);
    window.set_position(tauri::PhysicalPosition::new(fx, fy)).map_err(|e| e.to_string())?;
    let blocked_x = (req_x - fx as f64).abs() >= 0.5;
    let blocked_y = (req_y - fy as f64).abs() >= 0.5;
    app.emit("current-pos", serde_json::json!({
        "currentX": fx, "currentY": fy, "blockedX": blocked_x, "blockedY": blocked_y, "displays": displays
    })).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn move_pet_absolute(window: WebviewWindow, app: tauri::AppHandle, x: f64, y: f64) -> Result<(), String> {
    let displays = display::get_all();
    let req_x = x.round() as i32;
    let req_y = y.round() as i32;
    let (nx, ny) = display::clamp(req_x, req_y, &displays);
    window.set_position(tauri::PhysicalPosition::new(nx, ny)).map_err(|e| e.to_string())?;
    let blocked_x = (req_x - nx).abs() >= 1;
    let blocked_y = (req_y - ny).abs() >= 1;
    app.emit("current-pos", serde_json::json!({
        "currentX": nx, "currentY": ny, "blockedX": blocked_x, "blockedY": blocked_y, "displays": displays
    })).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_position(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let displays = display::get_all();
    app.emit("window-position", serde_json::json!({ "x": pos.x, "y": pos.y, "displays": displays }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_window_physical_pos(window: WebviewWindow) -> Result<serde_json::Value, String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "x": pos.x, "y": pos.y }))
}

#[tauri::command]
fn show_context_menu(window: WebviewWindow, state: tauri::State<'_, ContextMenu>) -> Result<(), String> {
    let menu = state.0.lock().map_err(|e| e.to_string())?;
    window.popup_menu(&*menu).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let (disk, log) = load_settings(&app);
    let sprite = find_sprite(&app, &disk.sprite_variant);
    Ok(serde_json::json!({
        "ai_enabled": disk.ai_enabled,
        "follow_enabled": disk.follow_enabled,
        "chase_enabled": disk.chase_enabled,
        "speed_multiplier": disk.speed_multiplier,
        "always_on_top": disk.always_on_top,
        "sprite_variant": disk.sprite_variant,
        "sprite": sprite,
        "_load_log": log,
    }))
}

#[tauri::command]
fn update_settings(app: tauri::AppHandle, settings: PetSettings) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(settings.always_on_top).map_err(|e| e.to_string())?;
    }
    save_settings(&app, &settings)?;
    update_all_menus(&app);
    let sprite = find_sprite(&app, &settings.sprite_variant);
    app.emit("settings-changed", serde_json::json!({
        "ai_enabled": settings.ai_enabled,
        "follow_enabled": settings.follow_enabled,
        "chase_enabled": settings.chase_enabled,
        "speed_multiplier": settings.speed_multiplier,
        "always_on_top": settings.always_on_top,
        "sprite_variant": settings.sprite_variant,
        "sprite": sprite,
    })).map_err(|e| e.to_string())?;
    Ok(())
}

fn show_about_dialog(app: &tauri::AppHandle) {
    app.dialog()
        .message("墨矩工坊 · 桌面宠物\nMoJu Tech · Desktop Pet\n\nv0.2.0-beta\n\n© 2026 墨矩工坊 MoJu Tech")
        .title("关于")
        .show(|_| {});
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            move_pet, move_pet_absolute, get_position, get_window_physical_pos,
            show_context_menu, get_settings, update_settings,
            list_sprites, upload_sprite, confirm_sprite_upload, read_sprite_preview, save_sprite_b64, delete_sprite,
            get_cursor_pos,
        ])
        .setup(|app| {
            let (s, _) = load_settings(app.handle());
            let follow_txt = if s.follow_enabled { "跟随鼠标 ✓" } else { "跟随鼠标" };
            let chase_txt = if s.chase_enabled { "追逐模式 ✓" } else { "追逐模式" };
            let follow_item = MenuItemBuilder::with_id("follow", follow_txt).build(app.handle())?;
            let chase_item = MenuItemBuilder::with_id("chase", chase_txt).build(app.handle())?;
            let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app.handle())?;
            let about = MenuItemBuilder::with_id("about", "关于").build(app.handle())?;
            let exit = MenuItemBuilder::with_id("exit", "退出").build(app.handle())?;
            let menu = MenuBuilder::new(app.handle())
                .items(&[&follow_item, &chase_item, &settings_item, &about, &exit])
                .build()?;

            if let Some(window) = app.get_webview_window("main") {
                let displays = display::get_all();
                if let Some(d) = displays.first() {
                    let x = d.x + d.w - display::PET_W - 50;
                    let y = d.y + d.h - display::PET_H - 50;
                    window.set_position(tauri::PhysicalPosition::new(x, y)).ok();
                }
            }

            // System tray icon — handles ALL menu events (tray + context menu) globally
            let tray_icon = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("墨矩工坊 · 桌面宠物")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    let id = event.id().as_ref().to_string();
                    handle_menu_event(app, &id);
                })
                .build(app)?;

            app.manage(ContextMenu(Mutex::new(menu)));
            app.manage(TrayState(tray_icon));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
