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

use once_cell::sync::Lazy;
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
	($lib:expr, $func:ident, $type:ty) => {
		get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0')).map(|f| unsafe { std::mem::transmute::<FARPROC, $type>(f) })
	};
}

static WVER: Lazy<(u32, u32, u32)> = Lazy::new(|| {
	let RtlGetVersion = get_function!("ntdll.dll", RtlGetVersion, unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32).unwrap();
	let mut vi = OSVERSIONINFOW {
		dwOSVersionInfoSize: 0,
		dwMajorVersion: 0,
		dwMinorVersion: 0,
		dwBuildNumber: 0,
		dwPlatformId: 0,
		szCSDVersion: [0; 128]
	};
	unsafe { (RtlGetVersion)(&mut vi as _) };
	(vi.dwMajorVersion, vi.dwMinorVersion, vi.dwBuildNumber)
});

#[derive(PartialEq, Eq)]
#[repr(C)]
enum ACCENT_STATE {
	ACCENT_DISABLED = 0,
	ACCENT_ENABLE_BLURBEHIND = 3,
	ACCENT_ENABLE_ACRYLICBLURBEHIND = 4
}

#[repr(C)]
struct ACCENT_POLICY {
	AccentState: u32,
	AccentFlags: u32,
	GradientColour: u32,
	AnimationId: u32
}

#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
	Attrib: WINDOWCOMPOSITIONATTRIB,
	pvData: *mut c_void,
	cbData: usize
}

#[repr(C)]
enum DWM_SYSTEMBACKDROP_TYPE {
	DWMSBT_DISABLE = 1,
	DWMSBT_MAINWINDOW = 2,      // Mica
	DWMSBT_TRANSIENTWINDOW = 3  // Acrylic
}

#[inline]
pub fn is_win7() -> bool {
	WVER.0 > 6 || (WVER.0 == 6 && WVER.1 == 1)
}

#[inline]
pub fn is_win10_1809() -> bool {
	WVER.2 >= 17763 && WVER.2 < 22000
}

#[inline]
pub fn is_win11() -> bool {
	WVER.2 >= 22000
}

#[inline]
pub fn is_win11_22h2() -> bool {
	WVER.2 >= 22621
}

unsafe fn set_accent_policy(hwnd: HWND, accent_state: ACCENT_STATE, colour: Option<[u8; 4]>) {
	if let Some(SetWindowCompositionAttribute) =
		get_function!("user32.dll", SetWindowCompositionAttribute, unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL)
	{
		let mut colour = colour.unwrap_or_default();

		let is_acrylic = accent_state == ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND;
		if is_acrylic && colour[3] == 0 {
			// acrylic doesn't like to have 0 alpha
			colour[3] = 1;
		}

		let mut policy = ACCENT_POLICY {
			AccentState: accent_state as _,
			AccentFlags: if is_acrylic { 0 } else { 2 },
			GradientColour: (colour[0] as u32) | (colour[1] as u32) << 8 | (colour[2] as u32) << 16 | (colour[3] as u32) << 24,
			AnimationId: 0
		};
		let mut data = WINDOWCOMPOSITIONATTRIBDATA {
			Attrib: 0x13,
			pvData: &mut policy as *mut _ as _,
			cbData: std::mem::size_of_val(&policy)
		};
		SetWindowCompositionAttribute(hwnd, &mut data as *mut _ as _);
	}
}

unsafe fn extend_client_area(hwnd: HWND) {
	let margins = MARGINS {
		cxLeftWidth: -1,
		cxRightWidth: -1,
		cyBottomHeight: -1,
		cyTopHeight: -1
	};
	DwmExtendFrameIntoClientArea(hwnd, &margins);
}

unsafe fn reset_client_area(hwnd: HWND) {
	let margins = MARGINS {
		cxLeftWidth: 0,
		cxRightWidth: 0,
		cyBottomHeight: 0,
		cyTopHeight: 0
	};
	DwmExtendFrameIntoClientArea(hwnd, &margins);
}

pub fn force_dark_theme(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &1 as *const _ as _, 4);
		}
	} else if is_win10_1809() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE - 1, &1 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatform("\"force_dark_theme()\" is only available on Windows 10 v1809+ or Windows 11"));
	}
	Ok(())
}

pub fn force_light_theme(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &0 as *const _ as _, 4);
		}
	} else if is_win10_1809() {
		unsafe {
			DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE - 1, &0 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatform("\"force_light_theme()\" is only available on Windows 10 v1809+ or Windows 11"));
	}
	Ok(())
}

pub fn apply_acrylic(hwnd: HWND, unified: bool, acrylic_blurbehind: bool, colour: Option<[u8; 4]>) -> Result<(), VibeError> {
	if !unified && is_win11_22h2() {
		unsafe {
			extend_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW as *const _ as _, 4);
		}
	} else if is_win7() {
		unsafe {
			set_accent_policy(
				hwnd,
				if acrylic_blurbehind {
					ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND
				} else {
					ACCENT_STATE::ACCENT_ENABLE_BLURBEHIND
				},
				Some(colour.unwrap_or([40, 40, 40, 0]))
			);
		}
	} else {
		return Err(VibeError::UnsupportedPlatform("\"apply_acrylic()\" is only available on Windows 7+"));
	}
	Ok(())
}

pub fn clear_acrylic(hwnd: HWND, unified: bool) -> Result<(), VibeError> {
	if !unified && is_win11_22h2() {
		unsafe {
			reset_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _, 4);
		}
	} else if is_win7() {
		unsafe {
			set_accent_policy(hwnd, ACCENT_STATE::ACCENT_DISABLED, None);
		}
	} else {
		return Err(VibeError::UnsupportedPlatform("\"clear_acrylic()\" is only available on Windows 7+"));
	}
	Ok(())
}

pub fn apply_mica(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11_22h2() {
		unsafe {
			extend_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW as *const _ as _, 4);
		}
	} else if is_win11() {
		unsafe {
			extend_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatform("\"apply_mica()\" is only available on Windows 11"));
	}
	Ok(())
}

pub fn clear_mica(hwnd: HWND) -> Result<(), VibeError> {
	if is_win11_22h2() {
		unsafe {
			reset_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _, 4);
		}
	} else if is_win11() {
		unsafe {
			reset_client_area(hwnd);
			DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &0 as *const _ as _, 4);
		}
	} else {
		return Err(VibeError::UnsupportedPlatform("\"clear_mica()\" is only available on Windows 11"));
	}
	Ok(())
}
