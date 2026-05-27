//! 桌面宠物 — Tauri 后端核心模块。
//!
//! ## 架构职责
//!
//! 本文件是 Rust 端的中枢，负责：
//!
//! * **命令注册** — 所有 `#[tauri::command]` 在此声明并注册到 Tauri invoke handler
//! * **窗口管理** — 宠物窗口的创建、移动、置顶控制
//! * **系统托盘** — 托盘图标 + 5 项右键菜单（跟随/追逐/设置/关于/退出）
//! * **事件推送** — 将位置回报和设置变更通过 Tauri event 推送到前端
//!
//! ## 注册的命令
//!
//! | 命令 | 功能 |
//! |---|---|
//! | `move_pet(dx, dy)` | 相对位移宠物窗口，自动 clamp 到有效显示器内 |
//! | `move_pet_absolute(x, y)` | 绝对定位宠物窗口（拖拽用） |
//! | `get_position` | 回报窗口当前物理位置和显示器列表 |
//! | `get_window_physical_pos` | 返回窗口物理坐标（拖拽基准计算用） |
//! | `show_context_menu` | 弹出原生右键菜单 |
//! | `get_settings` | 从磁盘加载设置并返回给前端 |
//! | `update_settings` | 更新设置（持久化 + 宠物窗口实时生效） |
//! | `list_sprites` | 列出所有可用精灵图（预设 + 用户上传） |
//! | `read_sprite_preview` | 读取用户精灵图 base64 缩略图 |
//! | `save_sprite_b64` | 保存用户上传的 base64 PNG 精灵图 |
//! | `delete_sprite` | 删除用户精灵图文件 |
//! | `get_cursor_pos` | 查询鼠标光标当前屏幕坐标 |
//!
//! ## 事件推送
//!
//! * `current-pos` — 每次 move_pet / move_pet_absolute 后推送窗口新位置
//! * `window-position` — 窗口初始化或放置后推送位置及显示器列表
//! * `settings-changed` — 设置变更后推送到所有窗口同步

mod display;
mod settings;

use settings::{delete_sprite, find_sprite, get_cursor_pos, list_sprites, load_settings, read_sprite_preview, save_sprite_b64, save_settings, PetSettings};
use std::sync::Mutex;
use tauri::{Emitter, Manager, WebviewWindow};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri_plugin_dialog::DialogExt;

/// 存储原生右键菜单实例，供 `show_context_menu` 命令调用。
struct ContextMenu(Mutex<tauri::menu::Menu<tauri::Wry>>);

/// 存储系统托盘图标实例，用于运行时更新托盘菜单。
struct TrayState(TrayIcon<tauri::Wry>);

/// 向所有窗口推送 settings-changed 事件。
///
/// payload 中包含完整的 PetSettings 字段和当前精灵图配置（sprite），
/// 宠物窗口和设置面板都会监听到此事件并同步状态。
fn emit_settings_changed(app: &tauri::AppHandle, s: &PetSettings) {
    let sprite = find_sprite(app, &s.sprite_variant);
    let _ = app.emit("settings-changed", serde_json::json!({
        "ai_enabled": s.ai_enabled,
        "follow_enabled": s.follow_enabled,
        "chase_enabled": s.chase_enabled,
        "speed_multiplier": s.speed_multiplier,
        "always_on_top": s.always_on_top,
        "sprite_variant": s.sprite_variant,
        "sprite": sprite,
    }));
}

/// 更新 ContextMenu 中指定 ID 的菜单项文本。
///
/// 用于在用户切换模式后实时更新菜单项显示的勾选状态
/// （如 "跟随鼠标 ✓" ↔ "跟随鼠标"）。
fn update_menu_text(state: &tauri::State<'_, ContextMenu>, id: &str, text: &str) {
    if let Ok(ref mut m) = state.0.lock() {
        if let Some(tauri::menu::MenuItemKind::MenuItem(ref item)) = m.get(id) {
            let _ = item.set_text(text);
        }
    }
}

/// 从磁盘重新加载设置并同步更新所有菜单项和托盘菜单。
///
/// 托盘菜单是一个 Native Snapshot — 菜单项文本修改后
/// 必须通过 `set_menu` 重新设置才能让托盘弹出菜单也显示新文本。
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

/// 统一的菜单事件处理入口。
///
/// 托盘菜单和右键菜单共享此处理函数：
/// * "follow" — 切换跟随鼠标模式（与追逐互斥）
/// * "chase" — 切换追逐模式（与跟随互斥）
/// * "settings" — 打开或聚焦设置窗口
/// * "about" — 显示关于对话框
/// * "exit" — 退出应用
fn handle_menu_event(app: &tauri::AppHandle, id: &str) {
    match id {
        "follow" => {
            let (mut s, _) = load_settings(app);
            s.follow_enabled = !s.follow_enabled;
            if s.follow_enabled { s.chase_enabled = false; }
            let _ = save_settings(app, &s);
            update_all_menus(app);
            emit_settings_changed(app, &s);
        }
        "chase" => {
            let (mut s, _) = load_settings(app);
            s.chase_enabled = !s.chase_enabled;
            if s.chase_enabled { s.follow_enabled = false; }
            let _ = save_settings(app, &s);
            update_all_menus(app);
            emit_settings_changed(app, &s);
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

/// 相对位移宠物窗口。
///
/// 将请求位移量 clamp 到有效显示器范围内后设置窗口位置。
/// 通过信号量 blocked_x/blocked_y 告知前端位移被屏幕边缘阻挡，
/// 前端据此触发墙壁反弹动画。
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

/// 绝对定位宠物窗口（拖拽时使用）。
///
/// 与 move_pet 的区别：接受目标绝对坐标而非相对位移。
/// 同样经过 clamp 处理确保不超出显示器范围。
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

/// 回报宠物窗口当前物理位置及显示器列表。
///
/// 前端初始化时调用，结果通过 window-position 事件异步推送。
#[tauri::command]
fn get_position(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let displays = display::get_all();
    app.emit("window-position", serde_json::json!({ "x": pos.x, "y": pos.y, "displays": displays }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 查询窗口当前物理坐标（同步返回 JSON）。
///
/// 拖拽开始时前端调用此命令获取窗口的物理像素基准坐标。
/// 前端随后用 `dragBaseX + (screenX - dragStartScreenX) * dpr`
/// 计算目标物理位移量。
#[tauri::command]
fn get_window_physical_pos(window: WebviewWindow) -> Result<serde_json::Value, String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "x": pos.x, "y": pos.y }))
}

/// 在宠物窗口上弹出原生右键菜单。
///
/// 前端在 contextmenu 事件中调用，弹出前设置 menuOpen 标记
/// 以暂停宠物移动，popup_menu 是阻塞调用，返回后清除标记。
#[tauri::command]
fn show_context_menu(window: WebviewWindow, state: tauri::State<'_, ContextMenu>) -> Result<(), String> {
    let menu = state.0.lock().map_err(|e| e.to_string())?;
    window.popup_menu(&*menu).map_err(|e| e.to_string())
}

/// 读取设置文件并返回给前端。
///
/// 返回的 JSON 中包含 `_load_log` 字段用于设置面板调试信息显示。
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

/// 保存设置并同步到宠物窗口。
///
/// 执行顺序：
///   1. 更新宠物窗口置顶状态（set_always_on_top）
///   2. 持久化写入 settings.json
///   3. 更新托盘/右键菜单项文本
///   4. 推送 settings-changed 事件通知所有窗口
#[tauri::command]
fn update_settings(app: tauri::AppHandle, settings: PetSettings) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(settings.always_on_top).map_err(|e| e.to_string())?;
    }
    save_settings(&app, &settings)?;
    update_all_menus(&app);
    emit_settings_changed(&app, &settings);
    Ok(())
}

/// 显示"关于"对话框 — 包含产品名称、版本号和版权信息。
fn show_about_dialog(app: &tauri::AppHandle) {
    app.dialog()
        .message("墨矩工坊 · 桌面宠物\nMoJu Tech · Desktop Pet\n\nv0.2.0-beta\n\n© 2026 墨矩工坊 MoJu Tech")
        .title("关于")
        .show(|_| {});
}

/// Tauri 应用入口 — 注册插件、命令、初始化窗口和托盘。
///
/// 启动流程：
///   1. 加载 dialog 插件（供关于对话框使用）
///   2. 注册所有 Tauri commands
///   3. 在 setup 阶段：
///      a. 从磁盘加载设置
///      b. 构建 5 项菜单（跟随/追逐/设置/关于/退出）
///      c. 将宠物窗口定位到主显示器右下角
///      d. 创建系统托盘图标并绑定菜单事件
///      e. 将菜单和托盘状态存储为 Tauri managed state
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            move_pet, move_pet_absolute, get_position, get_window_physical_pos,
            show_context_menu, get_settings, update_settings,
            list_sprites, read_sprite_preview, save_sprite_b64, delete_sprite,
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

            // 系统托盘图标 — 统一处理托盘菜单和右键菜单事件
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
