//! PowerToys-style file tools, each backed by Windows itself instead of a bundled library:
//! New+ templates, Text Extractor (OCR), shell file icons and the native Explorer context menu.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::Manager;

// ---------- New+ ----------

fn templates_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
	let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("Templates");
	std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	Ok(dir)
}

#[derive(serde::Serialize)]
pub struct Template {
	name: String,
	is_dir: bool,
}

/// New+: every file or folder in the templates folder becomes an entry in the "New" menu.
#[tauri::command(async)]
pub fn newplus_list(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
	let dir = templates_dir(&app)?;
	let mut items: Vec<Template> = std::fs::read_dir(&dir)
		.map_err(|e| e.to_string())?
		.flatten()
		.map(|e| Template {
			name: e.file_name().to_string_lossy().into_owned(),
			is_dir: e.file_type().is_ok_and(|t| t.is_dir()),
		})
		.collect();
	items.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
	Ok(serde_json::json!({ "dir": dir, "items": items }))
}

#[tauri::command(async)]
pub fn newplus_create(app: tauri::AppHandle, template: String, dest_dir: String) -> Result<String, String> {
	// A bare name only, so "..\x" cannot reach outside the templates folder.
	if Path::new(&template).file_name() != Some(std::ffi::OsStr::new(&template)) {
		return Err("Invalid template name".into());
	}
	let src = templates_dir(&app)?.join(&template);
	let dest = crate::unique_path(&Path::new(&dest_dir).join(&template));
	#[cfg(windows)]
	crate::shell::copy(&src.to_string_lossy(), &dest.to_string_lossy())?;
	#[cfg(not(windows))]
	std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;
	Ok(dest.to_string_lossy().into_owned())
}

// ---------- Text Extractor (OCR) ----------

const OCR_PS1: &str = include_str!("ocr.ps1");

/// Windows PowerShell 5.1 by full path (pwsh 7 lacks the WinRT projection; PATH may not have it).
fn powershell() -> std::process::Command {
	let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
	std::process::Command::new(format!(r"{root}\System32\WindowsPowerShell\v1.0\powershell.exe"))
}

/// OCR an image with the Windows OCR engine: no bundled model, about 0.7 s per image.
#[tauri::command(async)]
pub fn ocr_image(path: String) -> Result<String, String> {
	use base64::Engine;
	let script: Vec<u8> = OCR_PS1.encode_utf16().flat_map(u16::to_le_bytes).collect();
	let mut cmd = powershell();
	cmd.args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &base64::engine::general_purpose::STANDARD.encode(script)])
		.env("NX_OCR_PATH", &path);
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
	}
	let out = cmd.output().map_err(|e| e.to_string())?;
	match out.status.code() {
		Some(0) => Ok(String::from_utf8_lossy(&out.stdout).trim().to_string()),
		Some(2) => Err("No OCR language is installed (Settings > Time & language > Language)".into()),
		_ => Err(format!("OCR failed: {}", String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("unknown error"))),
	}
}

// ---------- Shell file icons ----------

/// Types whose icon lives in the file itself; everything else shares one icon per extension.
const OWN_ICON: &[&str] = &["exe", "ico", "lnk", "url", "cur", "ani", "msc", "appref-ms"];

/// Explorer's own 32 px icon for an item, as a data URL. Cached per type (or per file for exe/lnk...).
#[tauri::command(async)]
pub fn file_icon(path: String, is_dir: bool) -> Option<String> {
	static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
	let ext = Path::new(&path).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
	let per_file = !is_dir && OWN_ICON.contains(&ext.as_str());
	let key = if is_dir { "<dir>".to_string() } else if per_file { path.clone() } else { format!(".{ext}") };

	let cache = CACHE.get_or_init(Default::default);
	if let Some(hit) = cache.lock().unwrap().get(&key) {
		return hit.clone();
	}
	// Per-type lookups never touch the disk (SHGFI_USEFILEATTRIBUTES on a dummy name).
	let query = if per_file { path } else if is_dir { "folder".into() } else { format!("file.{ext}") };
	let url = shell_icon(&query, !per_file, is_dir).map(|bmp| {
		use base64::Engine;
		format!("data:image/bmp;base64,{}", base64::engine::general_purpose::STANDARD.encode(bmp))
	});
	cache.lock().unwrap().insert(key, url.clone());
	url
}

#[cfg(windows)]
fn shell_icon(query: &str, by_type: bool, is_dir: bool) -> Option<Vec<u8>> {
	use windows_sys::Win32::UI::Shell::*;
	use windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon;

	let w: Vec<u16> = query.encode_utf16().chain(Some(0)).collect();
	let mut sfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
	let attrs = if is_dir { 0x10 } else { 0x80 }; // FILE_ATTRIBUTE_DIRECTORY / NORMAL
	let flags = SHGFI_ICON | SHGFI_LARGEICON | if by_type { SHGFI_USEFILEATTRIBUTES } else { 0 };
	let ok = unsafe { SHGetFileInfoW(w.as_ptr(), attrs, &mut sfi, size_of::<SHFILEINFOW>() as u32, flags) };
	if ok == 0 || sfi.hIcon.is_null() {
		return None;
	}
	let px = icon_pixels(sfi.hIcon);
	unsafe { DestroyIcon(sfi.hIcon) };
	px.map(|(w, h, bgra)| bmp(w, h, &bgra))
}

#[cfg(not(windows))]
fn shell_icon(_query: &str, _by_type: bool, _is_dir: bool) -> Option<Vec<u8>> {
	None
}

/// HICON -> top-down BGRA pixels with real transparency.
#[cfg(windows)]
fn icon_pixels(icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON) -> Option<(u32, u32, Vec<u8>)> {
	use windows_sys::Win32::Graphics::Gdi::*;
	use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

	unsafe {
		let mut ii: ICONINFO = std::mem::zeroed();
		if GetIconInfo(icon, &mut ii) == 0 {
			return None;
		}
		let free = |ii: &ICONINFO| {
			DeleteObject(ii.hbmColor);
			DeleteObject(ii.hbmMask);
		};
		if ii.hbmColor.is_null() {
			free(&ii); // monochrome icon: not worth handling
			return None;
		}
		let mut bm: BITMAP = std::mem::zeroed();
		GetObjectW(ii.hbmColor, size_of::<BITMAP>() as i32, (&raw mut bm).cast());
		let (w, h) = (bm.bmWidth, bm.bmHeight);
		let header = || {
			let mut bi: BITMAPINFO = std::mem::zeroed();
			bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
			bi.bmiHeader.biWidth = w;
			bi.bmiHeader.biHeight = -h; // top-down
			bi.bmiHeader.biPlanes = 1;
			bi.bmiHeader.biBitCount = 32;
			bi
		};
		let dc = CreateCompatibleDC(std::ptr::null_mut());
		let mut px = vec![0u8; (w * h * 4) as usize];
		let mut bi = header();
		GetDIBits(dc, ii.hbmColor, 0, h as u32, px.as_mut_ptr().cast(), &mut bi, DIB_RGB_COLORS);
		if px.chunks_exact(4).all(|p| p[3] == 0) {
			// Old-style icon without alpha: transparency comes from the AND mask.
			let mut mask = vec![0u8; px.len()];
			let mut bi = header();
			GetDIBits(dc, ii.hbmMask, 0, h as u32, mask.as_mut_ptr().cast(), &mut bi, DIB_RGB_COLORS);
			for (p, m) in px.chunks_exact_mut(4).zip(mask.chunks_exact(4)) {
				p[3] = if m[0] == 0 { 255 } else { 0 };
			}
		}
		DeleteDC(dc);
		free(&ii);
		Some((w as u32, h as u32, px))
	}
}

/// 32-bit BMP with a BITMAPV4 header so the alpha channel survives in the webview (no PNG encoder needed).
fn bmp(w: u32, h: u32, bgra: &[u8]) -> Vec<u8> {
	const HEADERS: u32 = 14 + 108;
	let mut b = Vec::with_capacity(HEADERS as usize + bgra.len());
	b.extend_from_slice(b"BM");
	b.extend((HEADERS + bgra.len() as u32).to_le_bytes());
	b.extend([0u8; 4]);
	b.extend(HEADERS.to_le_bytes());
	b.extend(108u32.to_le_bytes());
	b.extend((w as i32).to_le_bytes());
	b.extend((-(h as i32)).to_le_bytes()); // top-down
	b.extend(1u16.to_le_bytes());
	b.extend(32u16.to_le_bytes());
	b.extend(3u32.to_le_bytes()); // BI_BITFIELDS
	b.extend((bgra.len() as u32).to_le_bytes());
	b.extend([0u8; 16]); // resolution + palette counts
	for mask in [0x00FF_0000u32, 0x0000_FF00, 0x0000_00FF, 0xFF00_0000] {
		b.extend(mask.to_le_bytes());
	}
	b.extend(0x7352_4742u32.to_le_bytes()); // 'sRGB'
	b.extend([0u8; 48]); // endpoints + gamma, unused for sRGB
	b.extend_from_slice(bgra);
	b
}

// ---------- Native Explorer context menu ----------

/// Explorer's real context menu ("Show more options") at the cursor, for one item.
/// A sync command on purpose: Tauri runs those on the main thread, which owns the window,
/// and TrackPopupMenuEx must run there.
// ponytail: "Send to" / "Open with" submenus are filled through IContextMenu2::HandleMenuMsg on
// WM_INITMENUPOPUP; without subclassing the Tauri window they open empty.
#[cfg(windows)]
#[tauri::command]
pub fn shell_context_menu(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
	use windows::Win32::Foundation::{HWND, POINT};
	use windows::Win32::System::Com::CoTaskMemFree;
	use windows::Win32::UI::Shell::Common::ITEMIDLIST;
	use windows::Win32::UI::Shell::{CMF_NORMAL, CMINVOKECOMMANDINFO, IContextMenu, IShellFolder, SHBindToParent, SHParseDisplayName};
	use windows::Win32::UI::WindowsAndMessaging::*;
	use windows::core::{HSTRING, PCSTR};

	let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
	unsafe {
		let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
		SHParseDisplayName(&HSTRING::from(path.as_str()), None, &mut pidl, 0, None).map_err(|e| e.to_string())?;
		let result = (|| -> windows::core::Result<()> {
			let mut child: *mut ITEMIDLIST = std::ptr::null_mut();
			let folder: IShellFolder = SHBindToParent(pidl, Some(&mut child))?;
			let cm: IContextMenu = folder.GetUIObjectOf(hwnd, &[child as *const ITEMIDLIST], None)?;
			let menu = CreatePopupMenu()?;
			cm.QueryContextMenu(menu, 0, 1, 0x7FFF, CMF_NORMAL).ok()?;
			let mut pt = POINT::default();
			GetCursorPos(&mut pt)?;
			let _ = SetForegroundWindow(hwnd);
			let cmd = TrackPopupMenuEx(menu, (TPM_RETURNCMD | TPM_RIGHTBUTTON).0, pt.x, pt.y, hwnd, None).0;
			if cmd > 0 {
				let info = CMINVOKECOMMANDINFO {
					cbSize: size_of::<CMINVOKECOMMANDINFO>() as u32,
					hwnd,
					lpVerb: PCSTR((cmd - 1) as usize as *const u8), // MAKEINTRESOURCE(offset)
					nShow: SW_SHOWNORMAL.0,
					..Default::default()
				};
				cm.InvokeCommand(&info)?;
			}
			DestroyMenu(menu)?;
			Ok(())
		})();
		CoTaskMemFree(Some(pidl as *const _));
		result.map_err(|e| e.to_string())
	}
}

/// The real Windows Properties dialog (General / Security / Details / Previous Versions tabs).
#[cfg(windows)]
#[tauri::command]
pub fn show_properties(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
	use windows_sys::Win32::UI::Shell::{SHOP_FILEPATH, SHObjectProperties};
	let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
	let w: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
	match unsafe { SHObjectProperties(hwnd, SHOP_FILEPATH as u32, w.as_ptr(), std::ptr::null()) } {
		0 => Err(std::io::Error::last_os_error().to_string()),
		_ => Ok(()),
	}
}

#[cfg(not(windows))]
#[tauri::command]
pub fn show_properties(_path: String) -> Result<(), String> {
	Err("Only available on Windows".into())
}

#[cfg(not(windows))]
#[tauri::command]
pub fn shell_context_menu(_path: String) -> Result<(), String> {
	Err("Only available on Windows".into())
}

#[cfg(all(test, windows))]
mod tests {
	use super::*;

	#[test]
	fn shell_icons_are_32px_bmps_with_alpha() {
		for (q, by_type, dir) in [("file.txt", true, false), ("folder", true, true)] {
			let b = shell_icon(q, by_type, dir).expect(q);
			assert_eq!(&b[..2], b"BM");
			let w = i32::from_le_bytes(b[18..22].try_into().unwrap());
			assert_eq!(b.len(), 122 + (w * w * 4) as usize);
			assert!(b[122..].chunks_exact(4).any(|p| p[3] != 0), "{q} has no opaque pixels");
		}
	}

	#[test]
	fn ocr_reads_text() {
		// Render text with GDI+ via PowerShell, then OCR it back.
		let img = std::env::temp_dir().join(format!("nx-ocr-{}.png", std::process::id()));
		let draw = format!(
			"Add-Type -AssemblyName System.Drawing; $b=New-Object Drawing.Bitmap 600,80; $g=[Drawing.Graphics]::FromImage($b); \
			$g.Clear([Drawing.Color]::White); $g.DrawString('Laporan Keuangan',(New-Object Drawing.Font('Segoe UI',28)),[Drawing.Brushes]::Black,10,10); \
			$b.Save('{}')",
			img.display()
		);
		let ok = powershell().args(["-NoProfile", "-Command", &draw]).status().unwrap();
		assert!(ok.success());
		let text = ocr_image(img.to_string_lossy().into_owned()).unwrap();
		std::fs::remove_file(&img).unwrap();
		assert!(text.contains("Laporan Keuangan"), "{text}");
	}
}
