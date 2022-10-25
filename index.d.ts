/**
 * Copyright (c) 2022 pyke.io (https://github.com/pykeio/vibe)
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import type { App, BrowserWindow } from 'electron';

type AcrylicEffect = 'mica' | 'acrylic' | 'unified-acrylic' | 'blurbehind';
type ColourableEffect = 'acrylic' | 'unified-acrylic' | 'blurbehind';

/**
 * Performs magic on the Electron app to get vibrancy effects to work.
 *
 * This should be run **before** the Electron app is ready.
 *
 * At the moment, this is a convenience function for `app.commandLine.appendSwitch('enable-transparent-visuals')`, but
 * future versions of Electron may require more hacks.
 *
 * @throws if `appendSwitch` encounters an error, or `app` is not a valid instance of `import('electron').App`
 */
export function setup(app: App): void;

/**
 * Forces a window's acrylic effects to have the specified theme. Note that this may also change the titlebar/caption
 * colour.
 */
export function forceTheme(window: BrowserWindow, theme: 'dark' | 'light'): void;

/**
 * Applies an acrylic effect to a window. `effect` must be one of `mica` or `acrylic`. `mica` is supported only in
 * Windows 11. `acrylic` is supported in Windows 10 builds later than build 17763, though performance may suffer on
 * pre-Windows 11 builds.
 *
 * To change the theme of an effect, use Electron's `nativeTheme` module to override the default system theme for your
 * app's windows, or for older versions of Electron, use `vibe.setDarkMode(win)` and `vibe.setLightMode(win)`.
 *
 * Ideally, this should be run **before** `show()`ing the window to avoid an awkward flash.
 *
 * @throws if `window` is not a valid instance of `import('electron').BrowserWindow`, or if this version of Windows
 * does not support the desired effect
 */
export function applyEffect(window: BrowserWindow, effect: Exclude<AcrylicEffect, ColourableEffect>): void;
export function applyEffect(window: BrowserWindow, effect: ColourableEffect, colour?: string): void;
export function applyEffect(window: BrowserWindow, effect: AcrylicEffect, colour?: string): void;

/**
 * Clears all effects.
 *
 * @throws if `window` is not a valid instance of `import('electron').BrowserWindow`
 */
export function clearEffects(window: BrowserWindow): void;

/**
 * Utilities for detecting the platform version.
 */
export namespace platform {
	/**
	 * Returns `true` if the current Windows version is equal to or greater than Windows 10 version 1803, **but less than
	 * Windows 11**. Windows 10 version 1809 is the first Windows version to support acrylic blurbehind.
	 */
	export function isWin10_1809(): boolean;

	/**
	 * Returns `true` if the current Windows version is equal to or greater than Windows 11.
	 * Windows 11 is the first Windows version to support the Mica effect.
	 */
	export function isWin11(): boolean;

	/**
	 * Returns `true` if the current Windows version is equal to or greater than Windows 11 build 22H2.
	 * Windows 11 build 22H2 is the first Windows version to support the (Fluent) Acrylic effect.
	 *
	 * Windows 11 builds earlier than 22H2 will use Windows Vista-style Aero DWM blur when applying the `acrylic`
	 * effect.
	 */
	export function isWin11_22H2(): boolean;
}
