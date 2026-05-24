mod display;

use std::sync::Mutex;
use tauri::{Emitter, Manager, WebviewWindow};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri_plugin_dialog::DialogExt;

struct ContextMenu(Mutex<tauri::menu::Menu<tauri::Wry>>);

#[tauri::command]
fn move_pet(window: WebviewWindow, app: tauri::AppHandle, dx: f64, dy: f64) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let displays = display::get_all(&window);

    let req_x = pos.x as f64 + dx;
    let req_y = pos.y as f64 + dy;
    let (fx, fy) = display::clamp(req_x.round() as i32, req_y.round() as i32, &displays);

    window
        .set_position(tauri::PhysicalPosition::new(fx, fy))
        .map_err(|e| e.to_string())?;

    let blocked_x = (req_x - fx as f64).abs() >= 0.5;
    let blocked_y = (req_y - fy as f64).abs() >= 0.5;

    app.emit(
        "current-pos",
        serde_json::json!({
            "currentX": fx,
            "currentY": fy,
            "blockedX": blocked_x,
            "blockedY": blocked_y,
            "displays": displays
        }),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn move_pet_absolute(window: WebviewWindow, x: f64, y: f64) -> Result<(), String> {
    let displays = display::get_all(&window);
    let (nx, ny) = display::clamp(x.round() as i32, y.round() as i32, &displays);
    window
        .set_position(tauri::PhysicalPosition::new(nx, ny))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_position(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let displays = display::get_all(&window);
    app.emit(
        "window-position",
        serde_json::json!({
            "x": pos.x,
            "y": pos.y,
            "displays": displays
        }),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn show_context_menu(window: WebviewWindow, state: tauri::State<'_, ContextMenu>) -> Result<(), String> {
    let menu = state.0.lock().map_err(|e| e.to_string())?;
    window.popup_menu(&*menu).map_err(|e| e.to_string())
}

fn show_about_dialog(app: &tauri::AppHandle) {
    app.dialog()
        .message("墨矩工坊 · 桌面宠物\nMoJu Tech · Desktop Pet\n\nv0.0.1-beta\n\n© 2026 墨矩工坊 MoJu Tech")
        .title("关于")
        .show(|_| {});
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            move_pet,
            move_pet_absolute,
            get_position,
            show_context_menu,
        ])
        .setup(|app| {
            let about = MenuItemBuilder::with_id("about", "关于").build(app.handle())?;
            let exit = MenuItemBuilder::with_id("exit", "退出").build(app.handle())?;
            let menu = MenuBuilder::new(app.handle())
                .items(&[&about, &exit])
                .build()?;

            if let Some(window) = app.get_webview_window("main") {
                // Initial position
                let monitors = window.available_monitors().unwrap_or_default();
                if let Some(m) = monitors.first() {
                    let sz = m.size();
                    let pos = m.position();
                    let x = pos.x + sz.width as i32 - 178;
                    let y = pos.y + sz.height as i32 - display::PET_H;
                    window.set_position(tauri::PhysicalPosition::new(x, y)).ok();
                }

                // Native context menu handler
                let app_handle = app.handle().clone();
                window.on_menu_event(move |_window, event| {
                    match event.id().as_ref() {
                        "about" => show_about_dialog(&app_handle),
                        "exit" => app_handle.exit(0),
                        _ => {}
                    }
                });
            }

            app.manage(ContextMenu(Mutex::new(menu)));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
