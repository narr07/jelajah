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

/// Any GDI bitmap -> top-down 32-bit BGRA pixels (alpha as stored).
#[cfg(windows)]
fn dib_pixels(hbm: windows_sys::Win32::Graphics::Gdi::HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
	use windows_sys::Win32::Graphics::Gdi::*;
	unsafe {
		let mut bm: BITMAP = std::mem::zeroed();
		if GetObjectW(hbm, size_of::<BITMAP>() as i32, (&raw mut bm).cast()) == 0 {
			return None;
		}
		let (w, h) = (bm.bmWidth, bm.bmHeight.abs());
		let mut bi: BITMAPINFO = std::mem::zeroed();
		bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
		bi.bmiHeader.biWidth = w;
		bi.bmiHeader.biHeight = -h; // top-down
		bi.bmiHeader.biPlanes = 1;
		bi.bmiHeader.biBitCount = 32;
		let dc = CreateCompatibleDC(std::ptr::null_mut());
		let mut px = vec![0u8; (w * h * 4) as usize];
		let lines = GetDIBits(dc, hbm, 0, h as u32, px.as_mut_ptr().cast(), &mut bi, DIB_RGB_COLORS);
		DeleteDC(dc);
		(lines > 0).then_some((w as u32, h as u32, px))
	}
}

/// HICON -> top-down BGRA pixels with real transparency.
#[cfg(windows)]
fn icon_pixels(icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON) -> Option<(u32, u32, Vec<u8>)> {
	use windows_sys::Win32::Graphics::Gdi::DeleteObject;
	use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

	unsafe {
		let mut ii: ICONINFO = std::mem::zeroed();
		if GetIconInfo(icon, &mut ii) == 0 {
			return None;
		}
		// monochrome icons (no color bitmap) are not worth handling
		let mut out = if ii.hbmColor.is_null() { None } else { dib_pixels(ii.hbmColor) };
		if let Some((_, _, px)) = out.as_mut() {
			if px.chunks_exact(4).all(|p| p[3] == 0) {
				// Old-style icon without alpha: transparency comes from the AND mask.
				if let Some((_, _, mask)) = dib_pixels(ii.hbmMask) {
					for (p, m) in px.chunks_exact_mut(4).zip(mask.chunks_exact(4)) {
						p[3] = if m[0] == 0 { 255 } else { 0 };
					}
				}
			}
		}
		DeleteObject(ii.hbmColor);
		DeleteObject(ii.hbmMask);
		out
	}
}

// ---------- Shell thumbnails (Quick Look-style Peek) ----------

/// What Explorer shows in "Extra large icons": first page of Office/PDF docs, a video frame,
/// folder contents, or the 256 px icon for anything else. `size` is the longest side in px.
#[cfg(windows)]
#[tauri::command(async)]
pub fn file_thumbnail(path: String, size: u32) -> Option<String> {
	use windows::Win32::Foundation::SIZE;
	use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
	use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_RESIZETOFIT};
	use windows::core::HSTRING;

	let size = size.clamp(32, 1024) as i32;
	let hbm = unsafe {
		// Thumbnail handlers expect an STA; this runs on a pooled thread, so init/uninit per call.
		let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
		let r = SHCreateItemFromParsingName::<_, _, IShellItemImageFactory>(&HSTRING::from(path.as_str()), None)
			.and_then(|f| f.GetImage(SIZE { cx: size, cy: size }, SIIGBF_RESIZETOFIT));
		if com {
			CoUninitialize();
		}
		r.ok()?
	};
	let px = dib_pixels(hbm.0);
	unsafe { windows_sys::Win32::Graphics::Gdi::DeleteObject(hbm.0) };
	let (w, h, mut px) = px?;

	if px.chunks_exact(4).all(|p| p[3] == 0) {
		px.chunks_exact_mut(4).for_each(|p| p[3] = 255); // opaque thumbnail (photos, pages)
	} else if px.chunks_exact(4).all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]) {
		// Shell bitmaps with alpha are premultiplied; BMP in the browser is straight alpha.
		for p in px.chunks_exact_mut(4).filter(|p| p[3] != 0 && p[3] != 255) {
			let a = p[3] as u32;
			for c in &mut p[..3] {
				*c = ((*c as u32 * 255 + a / 2) / a) as u8;
			}
		}
	}
	use base64::Engine;
	Some(format!("data:image/bmp;base64,{}", base64::engine::general_purpose::STANDARD.encode(bmp(w, h, &px))))
}

#[cfg(not(windows))]
#[tauri::command(async)]
pub fn file_thumbnail(_path: String, _size: u32) -> Option<String> {
	None
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

// ---------- Preview handlers (Office, PDF... exactly like Explorer's preview pane) ----------
//
// The handler draws into a native child window that we lay over the Peek body. Everything runs
// on the main thread (sync commands): it owns the window and is the STA COM needs.
// ponytail: keyboard focus moves into the handler once you click inside it, so Space/Esc reach
// Peek again only after clicking outside; forwarding via IPreviewHandlerFrame would fix that.

#[cfg(windows)]
thread_local! {
	static PREVIEW: std::cell::RefCell<Option<(windows::Win32::UI::Shell::IPreviewHandler, windows::Win32::Foundation::HWND)>> =
		const { std::cell::RefCell::new(None) };
}

/// CSS px from the webview -> physical px in the window's client area.
#[cfg(windows)]
fn client_rect(window: &tauri::WebviewWindow, x: f64, y: f64, w: f64, h: f64) -> windows::Win32::Foundation::RECT {
	let s = window.scale_factor().unwrap_or(1.0);
	let (l, t) = ((x * s).round() as i32, (y * s).round() as i32);
	windows::Win32::Foundation::RECT { left: l, top: t, right: l + (w * s).round() as i32, bottom: t + (h * s).round() as i32 }
}

#[cfg(windows)]
fn preview_handler_clsid(path: &str) -> Option<windows::core::GUID> {
	use windows::Win32::System::Com::CLSIDFromString;
	use windows::Win32::UI::Shell::{ASSOCF_INIT_DEFAULTTOSTAR, ASSOCSTR_SHELLEXTENSION, AssocQueryStringW};
	use windows::core::{HSTRING, PCWSTR, PWSTR, w};
	let ext = format!(".{}", Path::new(path).extension()?.to_string_lossy());
	let mut buf = [0u16; 64];
	let mut n = buf.len() as u32;
	unsafe {
		// IID_IPreviewHandler: the shellex key every preview handler registers under.
		AssocQueryStringW(ASSOCF_INIT_DEFAULTTOSTAR, ASSOCSTR_SHELLEXTENSION, &HSTRING::from(ext), w!("{8895b1c6-b41f-4c1c-a562-0d564250836f}"), Some(PWSTR(buf.as_mut_ptr())), &mut n).ok().ok()?;
		CLSIDFromString(PCWSTR(buf.as_ptr())).ok()
	}
}

#[cfg(windows)]
fn preview_close_inner() {
	use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
	if let Some((handler, host)) = PREVIEW.with(|p| p.borrow_mut().take()) {
		unsafe {
			let _ = handler.Unload();
			let _ = DestroyWindow(host);
		}
	}
}

/// Show `path` with its registered preview handler inside the given webview rect (CSS px).
/// Ok(false) = no handler for this type (or it failed), so the UI falls back to a thumbnail.
#[cfg(windows)]
#[tauri::command]
pub fn preview_open(window: tauri::WebviewWindow, path: String, x: f64, y: f64, w: f64, h: f64) -> Result<bool, String> {
	use windows::Win32::Foundation::{HWND, RECT};
	use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CLSCTX_LOCAL_SERVER, CoCreateInstance, STGM_READ, STGM_SHARE_DENY_NONE};
	use windows::Win32::UI::Shell::PropertiesSystem::{IInitializeWithFile, IInitializeWithStream};
	use windows::Win32::UI::Shell::{IInitializeWithItem, IPreviewHandler, IShellItem, SHCreateItemFromParsingName, SHCreateStreamOnFileEx};
	use windows::Win32::UI::WindowsAndMessaging::*;
	use windows::core::{HSTRING, Interface, w};

	preview_close_inner();
	let Some(clsid) = preview_handler_clsid(&path) else { return Ok(false) };
	let parent = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
	let r = client_rect(&window, x, y, w, h);
	let file = HSTRING::from(path.as_str());

	let shown = unsafe {
		(|| -> windows::core::Result<(IPreviewHandler, HWND)> {
			// Out of process first (prevhost.exe, as Explorer does) so a crashing handler can't take us down.
			let handler: IPreviewHandler = CoCreateInstance(&clsid, None, CLSCTX_LOCAL_SERVER)
				.or_else(|_| CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER))?;
			if let Ok(init) = handler.cast::<IInitializeWithFile>() {
				init.Initialize(&file, STGM_READ.0)?;
			} else if let Ok(init) = handler.cast::<IInitializeWithStream>() {
				let stream = SHCreateStreamOnFileEx(&file, (STGM_READ | STGM_SHARE_DENY_NONE).0, 0, false, None)?;
				init.Initialize(&stream, STGM_READ.0)?;
			} else {
				let item: IShellItem = SHCreateItemFromParsingName(&file, None)?;
				handler.cast::<IInitializeWithItem>()?.Initialize(&item, STGM_READ.0)?;
			}
			let host = CreateWindowExW(
				WINDOW_EX_STYLE(0),
				w!("STATIC"),
				None,
				WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
				r.left,
				r.top,
				r.right - r.left,
				r.bottom - r.top,
				Some(parent),
				None,
				None,
				None,
			)?;
			let inner = RECT { left: 0, top: 0, right: r.right - r.left, bottom: r.bottom - r.top };
			let shown = handler.SetWindow(host, &inner).and_then(|_| handler.DoPreview());
			if let Err(e) = shown {
				let _ = DestroyWindow(host);
				return Err(e);
			}
			Ok((handler, host))
		})()
	};
	match shown {
		Ok(p) => {
			PREVIEW.with(|cell| *cell.borrow_mut() = Some(p));
			// Some handlers grab focus on DoPreview; hand it back so Space/Esc still close Peek.
			let _ = window.set_focus();
			Ok(true)
		}
		Err(e) => {
			eprintln!("[preview] {path}: {e}");
			Ok(false)
		}
	}
}

/// Follow the Peek body when the window or modal resizes.
#[cfg(windows)]
#[tauri::command]
pub fn preview_move(window: tauri::WebviewWindow, x: f64, y: f64, w: f64, h: f64) {
	use windows::Win32::Foundation::RECT;
	use windows::Win32::UI::WindowsAndMessaging::MoveWindow;
	let r = client_rect(&window, x, y, w, h);
	PREVIEW.with(|p| {
		if let Some((handler, host)) = p.borrow().as_ref() {
			unsafe {
				let _ = MoveWindow(*host, r.left, r.top, r.right - r.left, r.bottom - r.top, true);
				let _ = handler.SetRect(&RECT { left: 0, top: 0, right: r.right - r.left, bottom: r.bottom - r.top });
			}
		}
	});
}

#[cfg(windows)]
#[tauri::command]
pub fn preview_close() {
	preview_close_inner();
}

#[cfg(not(windows))]
#[tauri::command]
pub fn preview_open(_path: String, _x: f64, _y: f64, _w: f64, _h: f64) -> Result<bool, String> {
	Ok(false)
}
#[cfg(not(windows))]
#[tauri::command]
pub fn preview_move(_x: f64, _y: f64, _w: f64, _h: f64) {}
#[cfg(not(windows))]
#[tauri::command]
pub fn preview_close() {}

// ---------- Open with ----------

#[derive(serde::Serialize)]
pub struct OpenWithApp {
	name: String,
	icon: Option<String>,
}

/// The apps Windows recommends for this file type, as in Explorer's "Open with" submenu.
#[cfg(windows)]
fn assoc_handlers(path: &str) -> Vec<windows::Win32::UI::Shell::IAssocHandler> {
	use windows::Win32::UI::Shell::{ASSOC_FILTER_RECOMMENDED, IAssocHandler, SHAssocEnumHandlers};
	use windows::core::HSTRING;
	let Some(ext) = Path::new(path).extension() else { return Vec::new() };
	let mut out = Vec::new();
	unsafe {
		if let Ok(e) = SHAssocEnumHandlers(&HSTRING::from(format!(".{}", ext.to_string_lossy())), ASSOC_FILTER_RECOMMENDED) {
			loop {
				let mut one: [Option<IAssocHandler>; 1] = [None];
				let mut got = 0u32;
				if e.Next(&mut one, Some(&mut got)).is_err() || got == 0 {
					break;
				}
				out.extend(one[0].take());
			}
		}
	}
	out
}

#[cfg(windows)]
fn take_pwstr(p: windows::core::PWSTR) -> String {
	let s = unsafe { p.to_string() }.unwrap_or_default();
	unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(p.0 as *const _)) };
	s
}

#[cfg(windows)]
#[tauri::command(async)]
pub fn open_with_list(path: String) -> Vec<OpenWithApp> {
	use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
	use windows_sys::Win32::UI::Shell::ExtractIconExW;
	use windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon;

	let com = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok() };
	let apps = assoc_handlers(&path)
		.into_iter()
		.filter_map(|h| unsafe {
			let name = take_pwstr(h.GetUIName().ok()?);
			let mut icon_path = windows::core::PWSTR::null();
			let mut index = 0;
			let icon = h.GetIconLocation(&mut icon_path, &mut index).ok().and_then(|_| {
				let file: Vec<u16> = take_pwstr(icon_path).encode_utf16().chain(Some(0)).collect();
				let mut hicon = std::ptr::null_mut();
				if ExtractIconExW(file.as_ptr(), index, &mut hicon, std::ptr::null_mut(), 1) == 0 || hicon.is_null() {
					return None; // e.g. Store apps: the icon is a package resource; the UI shows a generic one
				}
				let px = icon_pixels(hicon);
				DestroyIcon(hicon);
				px.map(|(w, h, bgra)| {
					use base64::Engine;
					format!("data:image/bmp;base64,{}", base64::engine::general_purpose::STANDARD.encode(bmp(w, h, &bgra)))
				})
			});
			Some(OpenWithApp { name, icon })
		})
		.collect();
	if com {
		unsafe { CoUninitialize() };
	}
	apps
}

/// Open the file with the recommended app called `app` (as listed by open_with_list).
#[cfg(windows)]
#[tauri::command]
pub fn open_with_invoke(path: String, app: String) -> Result<(), String> {
	use windows::Win32::System::Com::IDataObject;
	use windows::Win32::UI::Shell::{BHID_DataObject, IShellItem, SHCreateItemFromParsingName};
	use windows::core::HSTRING;
	let handler = assoc_handlers(&path)
		.into_iter()
		.find(|h| unsafe { h.GetUIName().map(take_pwstr).is_ok_and(|n| n == app) })
		.ok_or("App not found")?;
	unsafe {
		let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path.as_str()), None).map_err(|e| e.to_string())?;
		let data: IDataObject = item.BindToHandler(None, &BHID_DataObject).map_err(|e| e.to_string())?;
		handler.Invoke(&data).map_err(|e| e.to_string())
	}
}

/// Windows' own "How do you want to open this file?" picker ("Choose another app").
#[cfg(windows)]
#[tauri::command]
pub fn open_with_dialog(window: tauri::WebviewWindow, path: String) -> Result<(), String> {
	use windows::Win32::Foundation::HWND;
	use windows::Win32::UI::Shell::{OAIF_ALLOW_REGISTRATION, OAIF_EXEC, OAIF_REGISTER_EXT, OPENASINFO, SHOpenWithDialog};
	use windows::core::{HSTRING, PCWSTR};
	let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
	let file = HSTRING::from(path.as_str());
	let info = OPENASINFO { pcszFile: PCWSTR(file.as_ptr()), pcszClass: PCWSTR::null(), oaifInFlags: OAIF_ALLOW_REGISTRATION | OAIF_REGISTER_EXT | OAIF_EXEC };
	unsafe { SHOpenWithDialog(Some(hwnd), &info) }.map_err(|e| e.to_string())
}

#[cfg(not(windows))]
#[tauri::command(async)]
pub fn open_with_list(_path: String) -> Vec<OpenWithApp> {
	Vec::new()
}
#[cfg(not(windows))]
#[tauri::command]
pub fn open_with_invoke(_path: String, _app: String) -> Result<(), String> {
	Err("Only available on Windows".into())
}
#[cfg(not(windows))]
#[tauri::command]
pub fn open_with_dialog(_path: String) -> Result<(), String> {
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
	fn open_with_lists_notepad_for_txt() {
		let apps = open_with_list(r"C:\any\file.txt".into());
		let names: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
		assert!(names.iter().any(|n| n.to_lowercase().contains("notepad")), "{names:?}");
	}

	#[test]
	fn preview_handlers_found_for_office_and_pdf() {
		// Machine-dependent: prints what this PC has rather than failing on a box without Office.
		for f in [r"C:\x.xlsx", r"C:\x.docx", r"C:\x.pptx", r"C:\x.pdf"] {
			println!("{f}: {:?}", preview_handler_clsid(f));
		}
		assert!(preview_handler_clsid(r"C:\x.nohandler-ext").is_none());
	}

	#[test]
	fn thumbnails_for_any_file() {
		// No thumbnail handler for .rs: the shell falls back to the big file-type icon.
		let me = format!(r"{}\src\tools.rs", env!("CARGO_MANIFEST_DIR"));
		let url = file_thumbnail(me, 256).expect("thumbnail");
		assert!(url.starts_with("data:image/bmp;base64,"));
		assert!(file_thumbnail(r"C:\definitely\missing.bin".into(), 256).is_none());
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
