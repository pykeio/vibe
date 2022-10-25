const { app, BrowserWindow } = require('electron');
const path = require('path');
const vibe = require('@pyke/vibe');

vibe.setup(app);

app.whenReady().then(() => {
	const mainWindow = new BrowserWindow({
		width: 450,
		height: 450,
		center: true,
		backgroundColor: '#00000000',
		show: false,
		autoHideMenuBar: true
	});

	// Force dark theme for demonstration purposes.
	vibe.forceTheme(mainWindow, 'dark');

	vibe.applyEffect(mainWindow, 'acrylic');

	mainWindow.setIcon(path.join(__dirname, '1f4ab.png'));

	mainWindow.webContents.once('dom-ready', () => {
		mainWindow.show();

		setTimeout(() => {
			vibe.applyEffect(mainWindow, 'mica');
			setTimeout(() => {
				vibe.clearEffects(mainWindow);
				mainWindow.setBackgroundColor('#ffffff');
			}, 5000);
		}, 5000);
	});

	mainWindow.loadFile('src/index.html');
});
