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
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum VibeState {
	Uninitialized,
	Initialized,
	Acrylic,
	Mica
}

#[derive(Error, Debug)]
pub enum VibeError {
	#[error("Unsupported platform version: {0}")]
	UnsupportedPlatformVersion(&'static str),
	#[error("Expected `effect` to be one of 'mica' or 'acrylic'; got `{0}`")]
	UnsupportedEffect(String),
	#[error("`vibe` was not setup before calling `applyEffect`!")]
	Uninitialized
}

pub mod dwm;

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
	let browser_window = cx.argument::<JsObject>(0)?;
	let get_native_window_handle: Handle<JsFunction> = browser_window.get(&mut cx, "getNativeWindowHandle")?;
	let native_window_handle: Handle<JsObject> = get_native_window_handle.call(&mut cx, browser_window, [])?.downcast_or_throw(&mut cx)?;
	let read_int32_le: Handle<JsFunction> = native_window_handle.get(&mut cx, "readInt32LE")?;
	let hwnd = read_int32_le
		.call(&mut cx, native_window_handle, [])?
		.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?
		.value(&mut cx) as windows_sys::Win32::Foundation::HWND;
	let effect = cx.argument::<JsString>(1)?.value(&mut cx);

	let mut state = VIBE_STATE.lock().unwrap();
	match *state {
		VibeState::Uninitialized => cx.throw_error(VibeError::Uninitialized.to_string())?,
		VibeState::Initialized => (),
		VibeState::Mica => {
			let _ = dwm::clear_mica(hwnd);
		}
		VibeState::Acrylic => {
			let _ = dwm::clear_acrylic(hwnd);
		}
	};

	match effect.as_str() {
		"acrylic" => match dwm::apply_acrylic(hwnd) {
			Ok(_) => {
				*state = VibeState::Acrylic;
				Ok(cx.undefined())
			}
			Err(e) => cx.throw_error(e.to_string())?
		},
		"mica" => match dwm::apply_mica(hwnd) {
			Ok(_) => {
				*state = VibeState::Mica;
				Ok(cx.undefined())
			}
			Err(e) => cx.throw_error(e.to_string())?
		},
		_ => cx.throw_type_error(VibeError::UnsupportedEffect(effect).to_string())
	}
}

pub fn clear_effects(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let browser_window = cx.argument::<JsObject>(0)?;
	let get_native_window_handle: Handle<JsFunction> = browser_window.get(&mut cx, "getNativeWindowHandle")?;
	let native_window_handle: Handle<JsObject> = get_native_window_handle.call(&mut cx, browser_window, [])?.downcast_or_throw(&mut cx)?;
	let read_int32_le: Handle<JsFunction> = native_window_handle.get(&mut cx, "readInt32LE")?;
	let hwnd = read_int32_le
		.call(&mut cx, native_window_handle, [])?
		.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?
		.value(&mut cx) as windows_sys::Win32::Foundation::HWND;

	let mut state = VIBE_STATE.lock().unwrap();
	match *state {
		VibeState::Uninitialized => cx.throw_error(VibeError::Uninitialized.to_string())?,
		VibeState::Initialized => (),
		VibeState::Mica => {
			let _ = dwm::clear_mica(hwnd);
		}
		VibeState::Acrylic => {
			let _ = dwm::clear_acrylic(hwnd);
		}
	};

	*state = VibeState::Initialized;

	Ok(cx.undefined())
}

pub fn set_dark_mode(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let browser_window = cx.argument::<JsObject>(0)?;
	let get_native_window_handle: Handle<JsFunction> = browser_window.get(&mut cx, "getNativeWindowHandle")?;
	let native_window_handle: Handle<JsObject> = get_native_window_handle.call(&mut cx, browser_window, [])?.downcast_or_throw(&mut cx)?;
	let read_int32_le: Handle<JsFunction> = native_window_handle.get(&mut cx, "readInt32LE")?;
	let hwnd = read_int32_le
		.call(&mut cx, native_window_handle, [])?
		.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?
		.value(&mut cx) as windows_sys::Win32::Foundation::HWND;

	let _ = dwm::force_dark_theme(hwnd);
	Ok(cx.undefined())
}

pub fn set_light_mode(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let browser_window = cx.argument::<JsObject>(0)?;
	let get_native_window_handle: Handle<JsFunction> = browser_window.get(&mut cx, "getNativeWindowHandle")?;
	let native_window_handle: Handle<JsObject> = get_native_window_handle.call(&mut cx, browser_window, [])?.downcast_or_throw(&mut cx)?;
	let read_int32_le: Handle<JsFunction> = native_window_handle.get(&mut cx, "readInt32LE")?;
	let hwnd = read_int32_le
		.call(&mut cx, native_window_handle, [])?
		.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx)?
		.value(&mut cx) as windows_sys::Win32::Foundation::HWND;

	let _ = dwm::force_light_theme(hwnd);
	Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("applyEffect", apply_effect)?;
	cx.export_function("clearEffects", clear_effects)?;
	cx.export_function("setDarkMode", set_dark_mode)?;
	cx.export_function("setLightMode", set_light_mode)?;
	cx.export_function("setup", setup)?;
	Ok(())
}
