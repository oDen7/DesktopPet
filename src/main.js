const { app, BrowserWindow, screen, ipcMain } = require('electron');
const path = require('path');
const { PET_W, PET_H, displays, refresh, clamp } = require('./display');

let petWindow;

function createPetWindow() {
  refresh();

  petWindow = new BrowserWindow({
    width: PET_W,
    height: PET_H,
    transparent: true,
    frame: false,
    alwaysOnTop: true,
    resizable: false,
    hasShadow: false,
    skipTaskbar: true,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  });

  petWindow.loadFile(path.join(__dirname, '..', 'renderer', 'index.html'));

  const p = screen.getPrimaryDisplay().workArea;
  petWindow.setPosition(p.x + p.width - 178, p.y + p.height - PET_H);

  // Walking
  ipcMain.on('move-pet', (event, { dX, dY }) => {
    if (!petWindow) return;
    const [cx, cy] = petWindow.getPosition();
    const requestedX = cx + dX;
    const requestedY = cy + dY;
    const [fx, fy] = clamp(requestedX, requestedY);
    petWindow.setPosition(fx, fy);

    event.reply('current-pos', {
      currentX: fx, currentY: fy,
      blockedX: Math.abs(requestedX - fx) >= 0.5,
      blockedY: Math.abs(requestedY - fy) >= 0.5,
      displays
    });
  });

  // Drag
  ipcMain.on('move-pet-absolute', (_event, { x, y }) => {
    if (!petWindow) return;
    const [nx, ny] = clamp(x, y);
    petWindow.setPosition(nx, ny);
  });

  // Position query
  ipcMain.on('get-position', (event) => {
    if (!petWindow) return;
    const [x, y] = petWindow.getPosition();
    event.reply('window-position', { x, y, displays });
  });

  screen.on('display-metrics-changed', () => refresh());
}

app.whenReady().then(createPetWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
