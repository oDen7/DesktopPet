use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};

pub const PET_W: i32 = 128;
pub const PET_H: i32 = 128;

#[derive(Debug, Clone, Serialize)]
pub struct Display {
    /// Working-area left (excludes taskbar)
    pub x: i32,
    /// Working-area top
    pub y: i32,
    /// Working-area width
    pub w: i32,
    /// Working-area height
    pub h: i32,
}

static CACHE: Mutex<Option<(Vec<Display>, Instant)>> = Mutex::new(None);

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

fn is_on_screen(x: i32, y: i32, displays: &[Display]) -> bool {
    let cx = x + PET_W / 2;
    let cy = y + PET_H / 2;
    // 2px tolerance for DPI monitor-offset gaps
    displays.iter().any(|d| cx >= d.x - 2 && cx < d.x + d.w + 2 && cy >= d.y - 2 && cy < d.y + d.h + 2)
}

pub fn clamp(x: i32, y: i32, displays: &[Display]) -> (i32, i32) {
    if is_on_screen(x, y, displays) {
        return (x, y);
    }
    // Clamp center to nearest display, then derive window position
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
