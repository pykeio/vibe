// Copyright (c) 2022 pyke.io (https://github.com/pykeio/vibe)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Mutex;

use lazy_static::lazy_static;
use neon::prelude::*;

#[derive(PartialEq, Eq)]
pub enum VibeState {
	Uninitialized,
	Initialized,
	#[cfg(target_os = "windows")]
	Acrylic,
	#[cfg(target_os = "windows")]
	UnifiedAcrylic,
	#[cfg(target_os = "windows")]
	Blurbehind,
	#[cfg(target_os = "windows")]
	Mica
}

pub enum VibeError {
	UnsupportedPlatform(&'static str),
	UnknownEffect(String),
	Uninitialized
}

impl ToString for VibeError {
	fn to_string(&self) -> String {
		match self {
			Self::UnsupportedPlatform(msg) => format!("Unsupported platform: {}", msg),
			Self::UnknownEffect(effect) => format!("Expected `effect` to be one of ['mica', 'acrylic', 'unified-acrylic', 'blurbehind']; got `{}`", effect),
			Self::Uninitialized => "`vibe` was not setup before calling `applyEffect`!".to_owned()
		}
	}
}

#[cfg(target_os = "windows")]
pub mod dwm_win32;

#[cfg(target_os = "windows")]
fn get_native_window_handle(cx: &mut FunctionContext) -> NeonResult<windows_sys::Win32::Foundation::HWND> {
	let browser_window = cx.argument::<JsObject>(0)?;
	let get_native_window_handle: Handle<JsFunction> = browser_window.get(cx, "getNativeWindowHandle")?;
	let native_window_handle: Handle<JsObject> = get_native_window_handle.call(cx, browser_window, [])?.downcast_or_throw(cx)?;
	let read_int32_le: Handle<JsFunction> = native_window_handle.get(cx, "readInt32LE")?;
	Ok(read_int32_le
		.call(cx, native_window_handle, [])?
		.downcast_or_throw::<JsNumber, FunctionContext>(cx)?
		.value(cx) as _)
}

#[cfg(target_os = "linux")]
fn get_native_window_handle(cx: &mut FunctionContext) -> NeonResult<u32> {
	let browser_window = cx.argument::<JsObject>(0)?;
	let get_native_window_handle: Handle<JsFunction> = browser_window.get(cx, "getNativeWindowHandle")?;
	let native_window_handle: Handle<JsObject> = get_native_window_handle.call(cx, browser_window, [])?.downcast_or_throw(cx)?;
	let read_uint32_le: Handle<JsFunction> = native_window_handle.get(cx, "readUInt32LE")?;
	Ok(read_uint32_le
		.call(cx, native_window_handle, [])?
		.downcast_or_throw::<JsNumber, FunctionContext>(cx)?
		.value(cx) as _)
}

lazy_static! {
	static ref VIBE_STATE: Mutex<VibeState> = Mutex::new(VibeState::Uninitialized);
}

pub fn setup(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	if *VIBE_STATE.lock().unwrap() != VibeState::Uninitialized {
		return Ok(cx.undefined());
	}

	let app = cx.argument::<JsObject>(0)?;
	let command_line: Handle<JsObject> = app.get(&mut cx, "commandLine")?;
	let append_switch: Handle<JsFunction> = command_line.get(&mut cx, "appendSwitch")?;
	let enable_transparent_visuals = cx.string("enable-transparent-visuals").as_value(&mut cx);
	append_switch.call(&mut cx, command_line, [enable_transparent_visuals])?;

	*VIBE_STATE.lock().unwrap() = VibeState::Initialized;

	Ok(cx.undefined())
}

pub fn apply_effect(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let handle = get_native_window_handle(&mut cx)?;
	let effect = cx.argument::<JsString>(1)?.value(&mut cx);
	let colour = cx.argument_opt(2);

	let mut state = VIBE_STATE.lock().unwrap();
	match *state {
		VibeState::Uninitialized => cx.throw_error(VibeError::Uninitialized.to_string())?,
		VibeState::Initialized => (),
		#[cfg(target_os = "windows")]
		VibeState::Mica => {
			let _ = dwm_win32::clear_mica(handle);
		}
		#[cfg(target_os = "windows")]
		VibeState::UnifiedAcrylic | VibeState::Blurbehind => {
			let _ = dwm_win32::clear_acrylic(handle, true);
		}
		#[cfg(target_os = "windows")]
		VibeState::Acrylic => {
			let _ = dwm_win32::clear_acrylic(handle, false);
		}
	};

	match effect.as_str() {
		#[cfg(target_os = "windows")]
		"acrylic" => match dwm_win32::apply_acrylic(
			handle,
			false,
			true,
			match colour {
				Some(t) => match csscolorparser::parse(&t.downcast_or_throw::<JsString, FunctionContext>(&mut cx)?.value(&mut cx)) {
					Ok(colour) => Some(colour.to_rgba8()),
					Err(_) => None
				},
				None => None
			}
		) {
			Ok(_) => {
				*state = VibeState::Acrylic;
				Ok(cx.undefined())
			}
			Err(e) => cx.throw_error(e.to_string())?
		},
		#[cfg(target_os = "windows")]
		"unified-acrylic" => match dwm_win32::apply_acrylic(
			handle,
			true,
			true,
			match colour {
				Some(t) => match csscolorparser::parse(&t.downcast_or_throw::<JsString, FunctionContext>(&mut cx)?.value(&mut cx)) {
					Ok(colour) => Some(colour.to_rgba8()),
					Err(_) => None
				},
				None => None
			}
		) {
			Ok(_) => {
				*state = VibeState::UnifiedAcrylic;
				Ok(cx.undefined())
			}
			Err(e) => cx.throw_error(e.to_string())?
		},
		#[cfg(target_os = "windows")]
		"blurbehind" => match dwm_win32::apply_acrylic(
			handle,
			true,
			false,
			match colour {
				Some(t) => match csscolorparser::parse(&t.downcast_or_throw::<JsString, FunctionContext>(&mut cx)?.value(&mut cx)) {
					Ok(colour) => Some(colour.to_rgba8()),
					Err(_) => None
				},
				None => None
			}
		) {
			Ok(_) => {
				*state = VibeState::Blurbehind;
				Ok(cx.undefined())
			}
			Err(e) => cx.throw_error(e.to_string())?
		},
		#[cfg(target_os = "windows")]
		"mica" => match dwm_win32::apply_mica(handle) {
			Ok(_) => {
				*state = VibeState::Mica;
				Ok(cx.undefined())
			}
			Err(e) => cx.throw_error(e.to_string())?
		},
		_ => cx.throw_type_error(VibeError::UnknownEffect(effect).to_string())
	}
}

pub fn clear_effects(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let handle = get_native_window_handle(&mut cx)?;

	let mut state = VIBE_STATE.lock().unwrap();
	match *state {
		VibeState::Uninitialized => cx.throw_error(VibeError::Uninitialized.to_string())?,
		VibeState::Initialized => (),
		#[cfg(target_os = "windows")]
		VibeState::Mica => {
			let _ = dwm_win32::clear_mica(handle);
		}
		#[cfg(target_os = "windows")]
		VibeState::UnifiedAcrylic | VibeState::Blurbehind => {
			let _ = dwm_win32::clear_acrylic(handle, true);
		}
		#[cfg(target_os = "windows")]
		VibeState::Acrylic => {
			let _ = dwm_win32::clear_acrylic(handle, false);
		}
	};

	*state = VibeState::Initialized;

	Ok(cx.undefined())
}

pub fn set_dark_mode(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let handle = get_native_window_handle(&mut cx)?;
	#[cfg(target_os = "windows")]
	let _ = dwm_win32::force_dark_theme(handle);
	Ok(cx.undefined())
}

pub fn set_light_mode(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let handle = get_native_window_handle(&mut cx)?;
	#[cfg(target_os = "windows")]
	let _ = dwm_win32::force_light_theme(handle);
	Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	let platform = cx.empty_object();

	#[cfg(target_os = "windows")]
	{
		let is_win_10_1809 = JsFunction::new(&mut cx, |mut cx: FunctionContext| Ok(cx.boolean(dwm_win32::is_win10_1809())))?;
		let is_win_11 = JsFunction::new(&mut cx, |mut cx: FunctionContext| Ok(cx.boolean(dwm_win32::is_win11())))?;
		let is_win_22h2 = JsFunction::new(&mut cx, |mut cx: FunctionContext| Ok(cx.boolean(dwm_win32::is_win11_22h2())))?;
		platform.set(&mut cx, "isWin10_1809", is_win_10_1809)?;
		platform.set(&mut cx, "isWin11", is_win_11)?;
		platform.set(&mut cx, "isWin11_22H2", is_win_22h2)?;
	}

	cx.export_value("platform", platform)?;

	cx.export_function("applyEffect", apply_effect)?;
	cx.export_function("clearEffects", clear_effects)?;
	cx.export_function("setDarkMode", set_dark_mode)?;
	cx.export_function("setLightMode", set_light_mode)?;
	cx.export_function("setup", setup)?;
	Ok(())
}
