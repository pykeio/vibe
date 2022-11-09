<div align=center>
    <h2>💫 <code>@pyke/vibe</code> 💫</h2>
    <h3>native windows acrylic effects for electron</h3>
    <hr />
    <img src="https://parcel.pyke.io/v2/cdn/spaces/vibe/vibe-demo.gif" width=650>
    <br /><br />
</div>

`vibe` is a library for acrylic/vibrancy effects for Electron on Windows 10/11. Any Electron version compatible with N-API v6 (Electron v11+) is supported.

## Requirements
A recent version of [Rust](https://rust-lang.org/) (>=1.56.1) is required. You can install it via [rustup](https://rustup.rs/).

If you don't plan on doing Rust development, you may choose to install the minimal profile in `rustup` to install a lighter Rust toolchain.

For end users, the Acrylic effect is supported in Windows 10 builds later than build 17763 (though performance may suffer on builds earlier than Windows 11 22000), and the Mica effect is supported in Windows 11 only. `vibe` uses an undocumented API for enabling Mica on early builds of Windows 11 (specifically <22523) that is *not heavily tested* and *may not work at all*.

## Usage
> **Note**:
> If you'd like to use `vibe` with Discord on Windows, you'll need to install an additional Rust target: `rustup target add i686-pc-windows-msvc`, then build `vibe` with `npm run build:windows-i686`. You can then use the resulting `index.node` file like you'd use `@pyke/vibe`.

There are 3 important points you must keep in mind when using `vibe`:
- **`vibe` must do some trickery on the Electron `app` object before Electron loads in order for effects to work**, so don't forget to run `vibe.setup(app)` **before** `app.whenReady()`.
- **Keep the default frame**. Windows gets fussy about frames when you attempt to use acrylic effects. `titleBarStyle` must always be set to `default` and `frame` must always be set to `true`. While there [is a way to have titlebar-less framed Mica windows](https://github.com/pykeio/millennium/commit/0964cb3), it does not work with Electron, and would unfortunately require changes in Electron's internals.
- **Both `html` and `body` need to be transparent in CSS**. It's a common mistake to only make either `html` or `body` have `background: transparent`, but *both* of them need to be transparent. Additionally, you must set the Electron window's `backgroundColor` to `#00000000` to trick Electron into making a framed transparent window. **Do not set `transparent` to `true`**, as this will disable the frame and effects will break.

```js
const { app, BrowserWindow, nativeTheme } = require('electron');
const vibe = require('@pyke/vibe');

// Very important - let vibe perform its magic before the app is ready
vibe.setup(app);

app.whenReady().then(() => {
    const mainWindow = new BrowserWindow({
        ...,

        // This part is very important!
        backgroundColor: '#00000000',

        // Recommendation: Wait to show the window to avoid an ugly flash of non-acrylic-ized content.
        show: false,
        // Recommendation: Hide the menu bar, as the colour of the bar will be solid and will look janky.
        autoHideMenuBar: true
    });

    // Apply effects! 💫
    // This should be run before the window is ready to be shown.
    vibe.applyEffect(mainWindow, 'acrylic');

    // To disable effects, run `clearEffects`.
    // The background colour of the window will be black, so you should reset the window's background colour here and/or send a message to the renderer to update the CSS.
    vibe.clearEffects(mainWindow);
    mainWindow.setBackgroundColor('#ffffff');
});
```

The `acrylic` effect for Windows 10 and below can also have a 'tint' applied to it:
```js
vibe.applyEffect(mainWindow, 'acrylic', '#AA80FF40');
```

**NOTE**: The Windows 11 22H2 'Fluent' Acrylic effect cannot be tinted and will simply follow the window/system theme (see below). You can use `vibe.platform.isWin11_22H2()` to detect if the system is Windows 11 22H2 or greater and style your app appropriately.

Additionally, you can use Electron's `nativeTheme` module to force the theme of the acrylic effects:
```js
const { nativeTheme } = require('electron');
nativeTheme.themeSource = 'dark';
```

or, for older versions of Electron:
```js
vibe.forceTheme(mainWindow, 'dark');
vibe.forceTheme(mainWindow, 'light');
```

**Need help?** Visit the [`#💬｜vibe-support`](https://discord.com/channels/1029216970027049072/1030139823136190495) channel in the pyke Discord server:

<a href="https://discord.gg/BAkXJ6VjCz"><img src="https://invidget.switchblade.xyz/BAkXJ6VjCz"></a>

## Thanks to:
- [**Tauri**](https://github.com/tauri-apps)'s [`window-vibrancy`](https://github.com/tauri-apps/window-vibrancy) package, which vibe borrows some code from.
- [**@alexmercerind**](https://github.com/alexmercerind) for discovering the `DwmExtendFrameIntoClientArea` hack
- [**@sylveon**](https://github.com/sylveon) for finding a workaround to `transparent: true`
- [**@GregVido**](https://github.com/GregVido) for discovering the `enable-transparent-visuals` hack
- [**Twitter**](https://twemoji.twitter.com/) for providing the `vibe` 'icon' used in the demo 💫
