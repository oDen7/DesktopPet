const { screen } = require('electron');

const PET_W = 128, PET_H = 128;
let displays = [];

function refresh() {
  displays = screen.getAllDisplays().map(d => ({
    x: d.workArea.x,
    y: d.workArea.y,
    w: d.workArea.width,
    h: d.workArea.height
  }));
  return displays;
}

function isOnScreen(x, y) {
  const cx = x + PET_W / 2;
  const cy = y + PET_H / 2;
  for (const d of displays) {
    if (cx >= d.x && cx < d.x + d.w && cy >= d.y && cy < d.y + d.h) {
      return true;
    }
  }
  return false;
}

function clamp(x, y) {
  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    const p = screen.getPrimaryDisplay().workArea;
    return [Math.round(p.x + p.width - 178), Math.round(p.y + p.height - PET_H)];
  }
  if (isOnScreen(x, y)) return [Math.round(x), Math.round(y)];

  let bestDist = Infinity, bestX = x, bestY = y;
  for (const d of displays) {
    const cx = Math.max(d.x, Math.min(x, d.x + d.w - PET_W));
    const cy = Math.max(d.y, Math.min(y, d.y + d.h - PET_H));
    const dist = Math.abs(x - cx) + Math.abs(y - cy);
    if (dist < bestDist) { bestDist = dist; bestX = cx; bestY = cy; }
  }
  return [Math.round(bestX), Math.round(bestY)];
}

module.exports = { PET_W, PET_H, displays, refresh, isOnScreen, clamp };
