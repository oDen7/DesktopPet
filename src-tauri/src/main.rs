//! 桌面宠物应用入口点。
//! 屏蔽 Windows 控制台窗口（release 模式），
//! 将控制权交给 `desktop_pet_lib::run()` 启动 Tauri 框架。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    desktop_pet_lib::run()
}
