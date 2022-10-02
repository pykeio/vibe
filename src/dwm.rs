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

#![allow(non_snake_case, clippy::upper_case_acronyms, non_camel_case_types)]

use std::ffi::c_void;

use windows_sys::Win32::{
	Foundation::{BOOL, FARPROC, HWND},
	Graphics::Dwm::{DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWINDOWATTRIBUTE},
	System::{
		LibraryLoader::{GetProcAddress, LoadLibraryA},
		SystemInformation::OSVERSIONINFOW
	},
	UI::Controls::MARGINS
};

use crate::VibeError;

type WINDOWCOMPOSITIONATTRIB = u32;

const DWMWA_USE_IMMERSIVE_DARK_MODE: DWMWINDOWATTRIBUTE = 20i32;
const DWMWA_MICA_EFFECT: DWMWINDOWATTRIBUTE = 1029i32;
const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = 38i32;

#[derive(PartialEq, Eq)]
#[repr(C)]
enum ACCENT_STATE {
	ACCENT_DISABLED = 0,
	ACCENT_ENABLE_ACRYLICBLURBEHIND = 4
}

#[repr(C)]
struct ACCENT_POLICY {
	AccentState: u32,
	AccentFlags: u32,
	GradientColor: u32,
	AnimationId: u32
}

#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
	Attrib: WINDOWCOMPOSITIONATTRIB,
	pvData: *mut c_void,
	cbData: usize
}

#[allow(unused)]
#[repr(C)]
enum DWM_SYSTEMBACKDROP_TYPE {
	DWMSBT_DISABLE = 1,
	DWMSBT_MAINWINDOW = 2,      // Mica
	DWMSBT_TRANSIENTWINDOW = 3, // Acrylic
	DWMSBT_TABBEDWINDOW = 4     // Tabbed Mica
}

fn get_function_impl(library: &str, function: &str) -> Option<FARPROC> {
	assert_eq!(library.chars().last(), Some('\0'));
	assert_eq!(function.chars().last(), Some('\0'));

	let module = unsafe { LoadLibraryA(library.as_ptr()) };
	if module == 0 {
		return None;
	}
	Some(unsafe { GetProcAddress(module, function.as_ptr()) })
}

macro_rules! get_function {
	($lib:expr, $func:ident) => {
		get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0'))
			.map(|f| unsafe { std::mem::transmute::<::windows_sys::Win32::Foundation::FARPROC, $func>(f) })
	};
}

fn get_windows_ver() -> Option<(u32, u32, u32)> {
	type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
	let rtl_get_version = get_function!("ntdll.dll", RtlGetVersion)?;
	unsafe {
		let mut vi = OSVERSIONINFOW {
			dwOSVersionInfoSize: 0,
			dwMajorVersion: 0,
			dwMinorVersion: 0,
			dwBuildNumber: 0,
			dwPlatformId: 0,
			szCSDVersion: [0; 128]
		};

		let status = (rtl_get_version)(&mut vi as _);
		if status >= 0 { Some((vi.dwMajorVersion, vi.dwMinorVersion, vi.dwBuildNumber)) } else { None }
	}
}

#[inline]
fn is_win10_swca() -> bool {
	let v = get_windows_ver().unwrap_or_default();
	v.2 >= 17763 && v.2 < 22000
}

#[inline]
fn is_win11() -> bool {
	let v = get_windows_ver().unwrap_or_default();
	v.2 >= 22000
}

#[inline]
fn is_win11_dwmsbt() -> bool {
	let v = get_windows_ver().unwrap_or_default();
	v.2 >= 22523
}

unsafe fn SetWindowCompositionAttribute(hwnd: HWND, accent_state: ACCENT_STATE, color: Option<(u8, u8, u8, u8)>) {
	type SetWindowCompositionAttribute = unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
	if let Some(set_window_composition_attribute) = get_function!("user32.dll", SetWindowCompositionAttribute) {
		let mut color = color.unwrap_or_default();

		let is_acrylic = accent_state == ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND;
		if is_acrylic && color.3 == 0 {
			// acrylic doesn't like to have 0 alpha
			color.3 = 1;
		}

		let mut policy = ACCENT_POLICY {
			AccentState: accent_state as _,
			AccentFlags: if is_acrylic { 0 } else { 2 },
			GradientColor: (color.0 as u32) | (color.1 as u32) << 8 | (color.2 as u32) << 16 | (color.3 as u32) << 24,
			AnimationId: 0
		};
		let mut data = WINDOWCOMPOSITIONATTRIBDATA {
			Attrib: 0x13,
			pvData: &mut policy as *mut _ as _,
			cbData: std::mem::size_of_val(&policy)
		};
		set_window_composition_attribute(hwnd, &mut data as *mut _ as _);
	}
}

unsafe fn fix_client_area(hwnd: HWND) {
	let margins = MARGINS {
		cxLeftWidth: -1,
		cxRightWidth: -1,
		cyBottomHeight: -1,
		cyTopHeight: -1
	};
	DwmExtendFrameIntoClientArea(hwnd, &margins);
}

pub fn force_dark_theme(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &1 as *const _ as _, 4);
		}
	} else if is_win10_swca() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE - 1, &1 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatformVersion("\"force_dark_theme()\" is only available on Windows 10 v1809+ or Windows 11"));
	}
	Ok(())
}

pub fn force_light_theme(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &0 as *const _ as _, 4);
		}
	} else if is_win10_swca() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE - 1, &0 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatformVersion("\"force_light_theme()\" is only available on Windows 10 v1809+ or Windows 11"));
	}
	Ok(())
}

pub fn apply_acrylic(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11_dwmsbt() {
		unsafe {
			fix_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW as *const _ as _, 4);
		}
	} else if is_win10_swca() || is_win11() {
		unsafe {
			SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND, Some((40, 40, 40, 0)));
		}
	} else {
		return Err(VibeError::UnsupportedPlatformVersion("\"apply_acrylic()\" is only available on Windows 10 v1809+ or Windows 11"));
	}
	Ok(())
}

pub fn clear_acrylic(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11_dwmsbt() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _, 4);
		}
	} else if is_win10_swca() || is_win11() {
		unsafe {
			SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None);
		}
	} else {
		return Err(VibeError::UnsupportedPlatformVersion("\"clear_acrylic()\" is only available on Windows 10 v1809+ or Windows 11"));
	}
	Ok(())
}

pub fn apply_mica(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11_dwmsbt() {
		unsafe {
			fix_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW as *const _ as _, 4);
		}
	} else if is_win11() {
		unsafe {
			fix_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatformVersion("\"apply_mica()\" is only available on Windows 11"));
	}
	Ok(())
}

pub fn clear_mica(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11_dwmsbt() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _, 4);
		}
	} else if is_win11() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &0 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatformVersion("\"clear_mica()\" is only available on Windows 11"));
	}
	Ok(())
}
