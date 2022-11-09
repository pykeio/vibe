const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const vibe = require('@pyke/vibe');

vibe.setup(app);

app.whenReady().then(() => {
	const mainWindow = new BrowserWindow({
		width: 600,
		height: 400,
		center: true,
		backgroundColor: '#00000000',
		show: false,
		autoHideMenuBar: true,
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false
		}
	});

	// Force dark theme for demonstration purposes.
	vibe.forceTheme(mainWindow, 'dark');

	ipcMain.on('apply', (_, id) => {
		mainWindow.setBackgroundColor('#00000000');
		vibe.applyEffect(mainWindow, id);
	});
	ipcMain.on('clear', () => {
		mainWindow.setBackgroundColor('#212121');
		vibe.clearEffects(mainWindow);
	});

	mainWindow.webContents.once('dom-ready', () => {
		// Set the background color to solid #212121 after creating the window, because apparently unified-acrylic and
		// blurbehind need the window to be created with 0 alpha to work (or at least on Windows 11 22H2).
		mainWindow.setBackgroundColor('#212121');
		mainWindow.show();
	});

	mainWindow.setIcon(path.join(__dirname, 'icon.png'));
	mainWindow.loadFile('src/index.html');
});
