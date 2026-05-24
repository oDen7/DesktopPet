use serde::Serialize;

pub const PET_W: i32 = 128;
pub const PET_H: i32 = 128;

#[derive(Debug, Clone, Serialize)]
pub struct Display {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub fn get_all(window: &tauri::WebviewWindow) -> Vec<Display> {
    window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let sz = m.size();
            let pos = m.position();
            Display {
                x: pos.x,
                y: pos.y,
                w: sz.width as i32,
                h: sz.height as i32,
            }
        })
        .collect()
}

pub fn is_on_screen(x: i32, y: i32, displays: &[Display]) -> bool {
    let cx = x + PET_W / 2;
    let cy = y + PET_H / 2;
    displays
        .iter()
        .any(|d| cx >= d.x && cx < d.x + d.w && cy >= d.y && cy < d.y + d.h)
}

pub fn clamp(x: i32, y: i32, displays: &[Display]) -> (i32, i32) {
    if is_on_screen(x, y, displays) {
        return (x, y);
    }
    let mut best_dist = i32::MAX;
    let mut best = (x, y);
    for d in displays {
        let cx = x.clamp(d.x, d.x + d.w - PET_W);
        let cy = y.clamp(d.y, d.y + d.h - PET_H);
        let dist = (x - cx).abs() + (y - cy).abs();
        if dist < best_dist {
            best_dist = dist;
            best = (cx, cy);
        }
    }
    best
}
