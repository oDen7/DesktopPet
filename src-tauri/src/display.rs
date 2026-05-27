//! 多显示器几何管理模块。
//!
//! ## 功能
//!
//! * 通过 Win32 `EnumDisplayMonitors` / `GetMonitorInfoW` 枚举所有显示器工作区
//! * 5 秒缓存显示器列表以减少系统调用开销
//! * 提供 `clamp()` 函数将宠物窗口中心约束到有效显示器范围内
//!
//! ## 常量
//!
//! * `PET_W` / `PET_H` — 宠物窗口尺寸 128×128 像素
//!
//! ## 安全说明
//!
//! 本模块使用 `unsafe` 调用 Win32 API。所有 FFI 调用限制在
//! 显示器信息枚举范围内，不执行任何窗口句柄操作，
//! 符合项目的"禁止使用原生 FFI 操作窗口句柄"原则。

use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};

/// 宠物窗口宽度（像素），与前端 DISPLAY_SIZE 常量一致。
pub const PET_W: i32 = 128;
/// 宠物窗口高度（像素）。
pub const PET_H: i32 = 128;

/// 单个显示器的虚拟桌面工作区矩形。
///
/// 工作区坐标排除任务栏占用区域，
/// 序列化为 JSON 后供前端多显示器几何计算使用。
#[derive(Debug, Clone, Serialize)]
pub struct Display {
    /// 工作区左边界（排除任务栏）
    pub x: i32,
    /// 工作区上边界
    pub y: i32,
    /// 工作区宽度
    pub w: i32,
    /// 工作区高度
    pub h: i32,
}

/// 显示器列表缓存，TTL 为 5 秒。
/// 避免高频调用 `move_pet` 时重复枚举显示器。
static CACHE: Mutex<Option<(Vec<Display>, Instant)>> = Mutex::new(None);

/// 获取所有显示器工作区列表（带缓存）。
///
/// 若缓存未过期直接返回缓存数据，
/// 否则通过 Win32 API 重新枚举。
pub fn get_all() -> Vec<Display> {
    {
        let guard = CACHE.lock().unwrap();
        if let Some((ref displays, ref ts)) = *guard {
            if ts.elapsed() < std::time::Duration::from_secs(5) {
                return displays.clone();
            }
        }
    }
    let displays = enumerate();
    *CACHE.lock().unwrap() = Some((displays.clone(), Instant::now()));
    displays
}

/// Win32 显示器枚举入口。
///
/// 调用 `EnumDisplayMonitors` 遍历所有显示器，
/// 若枚举失败或返回空列表则回退为默认 1920×1032 单显示器。
fn enumerate() -> Vec<Display> {
    let mut displays = Vec::new();

    unsafe {
        let _ = EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(monitor_enum_proc),
            windows::Win32::Foundation::LPARAM(&mut displays as *mut Vec<Display> as isize),
        );
    }

    // 回退 — 无法枚举显示器时使用常见分辨率
    if displays.is_empty() {
        displays.push(Display {
            x: 0,
            y: 0,
            w: 1920,
            h: 1032,
        });
    }

    displays
}

/// Win32 `EnumDisplayMonitors` 回调函数。
///
/// 对每个显示器调用 `GetMonitorInfoW` 获取工作区矩形
/// （`rcWork`，不含任务栏），将其加入 Vec<Display>。
unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let displays = &mut *(lparam.0 as *mut Vec<Display>);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO).as_bool() {
        let rw = info.monitorInfo.rcWork;
        displays.push(Display {
            x: rw.left,
            y: rw.top,
            w: rw.right - rw.left,
            h: rw.bottom - rw.top,
        });
    }

    windows::Win32::Foundation::BOOL(1)
}

/// 判断给定窗口位置（中心点）是否落在任何显示器内。
///
/// 使用 2px 容差补偿 DPI 缩放导致的显示器间微小间隙。
fn is_on_screen(x: i32, y: i32, displays: &[Display]) -> bool {
    let cx = x + PET_W / 2;
    let cy = y + PET_H / 2;
    displays.iter().any(|d| cx >= d.x - 2 && cx < d.x + d.w + 2 && cy >= d.y - 2 && cy < d.y + d.h + 2)
}

/// 将宠物窗口位置 clamp 到最近的显示器内。
///
/// 算法：
///   1. 若当前中心点已在某显示器内 → 直接返回
///   2. 否则找到中心点 Manhattan 距离最近的显示器
///   3. 将中心点 clamp 到该显示器内，反算窗口左上角坐标
pub fn clamp(x: i32, y: i32, displays: &[Display]) -> (i32, i32) {
    if is_on_screen(x, y, displays) {
        return (x, y);
    }
    let mut best_dist = i32::MAX;
    let mut best = (x, y);
    for d in displays {
        let ccx = (x + PET_W / 2).clamp(d.x, d.x + d.w - 1);
        let ccy = (y + PET_H / 2).clamp(d.y, d.y + d.h - 1);
        let dist = (x + PET_W / 2 - ccx).abs() + (y + PET_H / 2 - ccy).abs();
        if dist < best_dist {
            best_dist = dist;
            best = (ccx - PET_W / 2, ccy - PET_H / 2);
        }
    }
    best
}
