mod fast_index;
mod name_index;
mod tools;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::time::{Duration, UNIX_EPOCH};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, RunEvent};
use tauri_plugin_shell::{process::CommandChild, ShellExt};

struct TgrepServer(Mutex<Option<CommandChild>>);

/// Last TreeSize scan, kept in memory so drilling into subfolders is instant.
struct TreeCache(Mutex<Option<TreeScan>>);

struct TreeScan {
	root: PathBuf,
	node: DirNode,
	top: Vec<(u64, PathBuf)>,
}

struct DirNode {
	name: String,
	size: u64,
	files: u64,
	dirs: u64,
	cats: [(u64, u64); 8], // (bytes, count) per CATEGORIES index
	children: Vec<DirNode>,
}

impl TreeScan {
	fn find(&self, p: &Path) -> Option<&DirNode> {
		let rel = p.strip_prefix(&self.root).ok()?;
		rel.components().try_fold(&self.node, |n, c| n.children.iter().find(|ch| c.as_os_str() == ch.name.as_str()))
	}
}

const TOP_KEEP: usize = 1000;

#[derive(Default)]
struct ScanCtx {
	files_seen: AtomicU64,
	top: Mutex<Vec<(u64, PathBuf)>>,
	top_min: AtomicU64,
}

impl ScanCtx {
	// Lock only for files bigger than the current top-N floor; floor rises fast so contention dies early.
	fn offer_top(&self, len: u64, path: impl FnOnce() -> PathBuf) {
		if len < self.top_min.load(Relaxed) { return; }
		let mut top = self.top.lock().unwrap();
		top.push((len, path()));
		if top.len() >= TOP_KEEP * 2 {
			top.sort_unstable_by(|a, b| b.0.cmp(&a.0));
			top.truncate(TOP_KEEP);
			self.top_min.store(top[TOP_KEEP - 1].0, Relaxed);
		}
	}

	/// Largest files first, capped at TOP_KEEP.
	fn into_top(self) -> Vec<(u64, PathBuf)> {
		let mut top = self.top.into_inner().unwrap();
		top.sort_unstable_by(|a, b| b.0.cmp(&a.0));
		top.truncate(TOP_KEEP);
		top
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Hit {
	pub path: String,
	pub line: u64,
	pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubfolderStats {
	pub name: String,
	pub path: String,
	pub size: u64,
	pub percent: f64,
	pub files_count: u64,
	pub dirs_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryStats {
	pub category: String,
	pub size: u64,
	pub count: u64,
	pub percent: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TopFile {
	pub name: String,
	pub path: String,
	pub size: u64,
	pub ext: String,
}

#[cfg(desktop)]
use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileItem {
	pub name: String,
	pub path: String,
	pub is_dir: bool,
	pub size: u64,
	pub modified: u64,
	pub created: u64,
	pub readonly: bool,
	pub hidden: bool,
	pub extension: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DriveItem {
	pub letter: String,
	pub path: String,
	pub label: String,
	pub total_bytes: Option<u64>,
	pub free_bytes: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuickAccessItem {
	pub id: String,
	pub name: String,
	pub path: String,
	pub icon: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchHit {
	pub path: String,
	pub name: String,
	pub is_dir: bool,
	pub line_number: Option<usize>,
	pub line_content: Option<String>,
	pub match_start: Option<usize>,
	pub match_end: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilePreview {
	pub is_dir: bool,
	pub is_image: bool,
	pub is_binary: bool,
	pub size: u64,
	pub modified: u64,
	pub created: u64,
	pub preview: Option<String>,
}

/// "" -> home, "C:" -> "C:\"
fn resolve_dir(path: String) -> String {
	if path.is_empty() {
		std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string())
	} else if path.ends_with(':') {
		format!("{}\\", path)
	} else {
		path
	}
}

#[tauri::command(async)]
fn list_directory(path: String, show_hidden: bool) -> Result<serde_json::Value, String> {
	let target_path = resolve_dir(path);

	let dir = Path::new(&target_path);
	if !dir.exists() {
		return Err(format!("Directory does not exist: {}", target_path));
	}

	let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
	let mut items = Vec::new();

	for entry in read_dir.flatten() {
		let file_name = entry.file_name().to_string_lossy().to_string();
		let is_hidden = file_name.starts_with('.');
		if !show_hidden && is_hidden {
			continue;
		}

		let full_path = entry.path().to_string_lossy().to_string();
		let metadata = entry.metadata().ok();

		let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
		let size = if is_dir { 0 } else { metadata.as_ref().map(|m| m.len()).unwrap_or(0) };

		let modified = metadata.as_ref().and_then(|m| m.modified().ok())
			.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
			.map(|d| d.as_millis() as u64)
			.unwrap_or(0);

		let created = metadata.as_ref().and_then(|m| m.created().ok())
			.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
			.map(|d| d.as_millis() as u64)
			.unwrap_or(0);

		let ext = if is_dir {
			String::new()
		} else {
			Path::new(&file_name).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase()
		};

		items.push(FileItem {
			name: file_name,
			path: full_path,
			is_dir,
			size,
			modified,
			created,
			readonly: metadata.as_ref().map(|m| m.permissions().readonly()).unwrap_or(false),
			hidden: is_hidden,
			extension: ext,
		});
	}

	items.sort_by(|a, b| {
		if a.is_dir != b.is_dir {
			return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
		}
		a.name.to_lowercase().cmp(&b.name.to_lowercase())
	});

	Ok(serde_json::json!({
		"current_path": target_path,
		"items": items
	}))
}

#[tauri::command]
fn get_drives() -> serde_json::Value {
	let mut drives = Vec::new();

	#[cfg(target_os = "windows")]
	{
		for c in b'A'..=b'Z' {
			let drive_path = format!("{}:\\", c as char);
			if Path::new(&drive_path).exists() {
				let letter = format!("{}:", c as char);
				let label = if c == b'C' {
					format!("Local Disk ({})", letter)
				} else {
					format!("Volume ({})", letter)
				};

				drives.push(DriveItem {
					letter,
					path: drive_path,
					label,
					total_bytes: None,
					free_bytes: None,
				});
			}
		}
	}

	#[cfg(not(target_os = "windows"))]
	{
		drives.push(DriveItem {
			letter: "/".to_string(),
			path: "/".to_string(),
			label: "Root File System".to_string(),
			total_bytes: None,
			free_bytes: None,
		});
	}

	serde_json::json!({ "drives": drives })
}

#[tauri::command]
fn get_quick_access() -> serde_json::Value {
	let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| "C:\\".to_string());
	let home_path = PathBuf::from(&home);

	let candidates = vec![
		("home", "Home", home.clone(), "lucide:home"),
		("desktop", "Desktop", home_path.join("Desktop").to_string_lossy().to_string(), "lucide:monitor"),
		("downloads", "Downloads", home_path.join("Downloads").to_string_lossy().to_string(), "lucide:download"),
		("documents", "Documents", home_path.join("Documents").to_string_lossy().to_string(), "lucide:file-text"),
		("pictures", "Pictures", home_path.join("Pictures").to_string_lossy().to_string(), "lucide:image"),
		("music", "Music", home_path.join("Music").to_string_lossy().to_string(), "lucide:music"),
		("videos", "Videos", home_path.join("Videos").to_string_lossy().to_string(), "lucide:video"),
	];

	let mut items = Vec::new();
	for (id, name, path_str, icon) in candidates {
		if Path::new(&path_str).exists() {
			items.push(QuickAccessItem {
				id: id.to_string(),
				name: name.to_string(),
				path: path_str,
				icon: icon.to_string(),
			});
		}
	}

	serde_json::json!({ "items": items })
}

#[tauri::command]
async fn run_tgrep(
	app: tauri::AppHandle,
	query: String,
	search_path: String,
	content_search: bool,
	case_sensitive: bool,
	regex: bool,
) -> Result<serde_json::Value, String> {
	let mut args: Vec<String> = Vec::new();
	if content_search {
		if !case_sensitive {
			args.push("-i".into());
		}
		args.push(if regex { "-P" } else { "-F" }.into());
		args.extend(["--json".into(), "--".into(), query, search_path]);
	} else {
		args.push("--files".into());
		if !case_sensitive {
			args.push("--glob-case-insensitive".into());
		}
		if regex {
			args.extend(["-e".into(), query]);
		} else {
			args.extend(["-g".into(), format!("*{}*", query)]);
		}
		args.push(search_path);
	}

	// Sidecar: same binary as the `tgrep serve` daemon, so it hits the live index.
	let output = app.shell().sidecar("tgrep").map_err(|e| e.to_string())?
		.args(args)
		.output().await.map_err(|e| e.to_string())?;

	let stdout_str = String::from_utf8_lossy(&output.stdout);
	let mut hits = Vec::new();

	if content_search {
		for line in stdout_str.lines() {
			if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
				if parsed.get("type").and_then(|v| v.as_str()) == Some("match") {
					if let Some(data) = parsed.get("data") {
						let hit_path = data.get("path").and_then(|p| p.get("text")).and_then(|t| t.as_str()).unwrap_or("");
						let line_num = data.get("line_number").and_then(|n| n.as_u64()).map(|n| n as usize);
						let line_content = data.get("lines").and_then(|l| l.get("text")).and_then(|t| t.as_str()).unwrap_or("").trim().to_string();

						let submatch = data.get("submatches").and_then(|s| s.as_array()).and_then(|arr| arr.first());
						let match_start = submatch.and_then(|s| s.get("start")).and_then(|n| n.as_u64()).map(|n| n as usize);
						let match_end = submatch.and_then(|s| s.get("end")).and_then(|n| n.as_u64()).map(|n| n as usize);

						let name = Path::new(hit_path).file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();

						hits.push(SearchHit {
							path: hit_path.to_string(),
							name,
							is_dir: false,
							line_number: line_num,
							line_content: Some(line_content),
							match_start,
							match_end,
						});

						if hits.len() >= 200 {
							break;
						}
					}
				}
			}
		}
	} else {
		for line in stdout_str.lines() {
			let trimmed = line.trim();
			if trimmed.is_empty() || trimmed.starts_with("warning:") || trimmed.starts_with("note:") {
				continue;
			}
			let name = Path::new(trimmed).file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
			hits.push(SearchHit {
				path: trimmed.to_string(),
				name,
				is_dir: false,
				line_number: None,
				line_content: None,
				match_start: None,
				match_end: None,
			});

			if hits.len() >= 200 {
				break;
			}
		}
	}

	let count = hits.len();
	Ok(serde_json::json!({ "hits": hits, "count": count }))
}

#[tauri::command]
async fn build_tgrep_index(app: tauri::AppHandle, path: String) -> Result<serde_json::Value, String> {
	let out = app.shell().sidecar("tgrep").map_err(|e| e.to_string())?
		.args(["index", &path])
		.output().await.map_err(|e| e.to_string())?;
	let success = out.status.success();
	let msg = String::from_utf8_lossy(if success { &out.stdout } else { &out.stderr }).to_string();
	Ok(serde_json::json!({ "success": success, "message": msg }))
}

/// Explorer naming: "New folder", "New folder (2)", "New folder (3)"...
fn unique_path(p: &Path) -> PathBuf {
	if !p.exists() {
		return p.to_path_buf();
	}
	let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
	let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
	(2..).map(|n| p.with_file_name(format!("{stem} ({n}){ext}"))).find(|c| !c.exists()).unwrap()
}

fn remove_path(path: &str) -> Result<(), String> {
	let p = Path::new(path);
	if p.is_dir() { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) }.map_err(|e| e.to_string())
}

/// SHFileOperationW: the same engine explorer.exe uses for copy/move/delete.
#[cfg(windows)]
mod shell {
	use windows_sys::Win32::UI::Shell::*;

	fn op(func: u32, from: &str, to: Option<&str>, flags: u32) -> Result<(), String> {
		// The API wants backslashes and double-NUL-terminated path lists.
		let wide = |s: &str| -> Vec<u16> { s.replace('/', "\\").encode_utf16().chain([0, 0]).collect() };
		let from_w = wide(from);
		let to_w = to.map(wide);
		let mut fo: SHFILEOPSTRUCTW = unsafe { std::mem::zeroed() };
		fo.wFunc = func;
		fo.pFrom = from_w.as_ptr();
		fo.pTo = to_w.as_ref().map_or(std::ptr::null(), |v| v.as_ptr());
		fo.fFlags = flags as u16;
		let rc = unsafe { SHFileOperationW(&mut fo) };
		if rc != 0 {
			return Err(format!("Shell file operation failed (code {rc:#x})"));
		}
		if fo.fAnyOperationsAborted != 0 {
			return Err("Operation cancelled".into());
		}
		Ok(())
	}

	pub fn recycle(path: &str) -> Result<(), String> {
		op(FO_DELETE, path, None, FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI)
	}

	pub fn copy(from: &str, to: &str) -> Result<(), String> {
		op(FO_COPY, from, Some(to), FOF_ALLOWUNDO | FOF_RENAMEONCOLLISION | FOF_NOCONFIRMMKDIR)
	}

	pub fn mv(from: &str, to: &str) -> Result<(), String> {
		op(FO_MOVE, from, Some(to), FOF_ALLOWUNDO | FOF_NOCONFIRMMKDIR)
	}
}

#[tauri::command(async)]
fn file_action(
	action: String,
	target_path: String,
	name: Option<String>,
	new_path: Option<String>,
	destination_path: Option<String>,
	content: Option<String>,
	content_base64: Option<String>,
) -> Result<serde_json::Value, String> {
	match action.as_str() {
		"open" => {
			#[cfg(target_os = "windows")]
			{
				Command::new("explorer").arg(&target_path).spawn().map_err(|e| e.to_string())?;
			}
			#[cfg(not(target_os = "windows"))]
			{
				Command::new("open").arg(&target_path).spawn().map_err(|e| e.to_string())?;
			}
			Ok(serde_json::json!({ "success": true }))
		}
		"openTerminal" => {
			#[cfg(target_os = "windows")]
			{
				let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
				let ps_path = format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", sys_root);
				let exe = if std::path::Path::new(&ps_path).exists() { ps_path } else { "powershell".to_string() };
				Command::new(exe)
					.args(["-NoExit", "-Command", &format!("Set-Location -LiteralPath '{}'", target_path)])
					.spawn()
					.map_err(|e| e.to_string())?;
			}
			#[cfg(not(target_os = "windows"))]
			{
				Command::new("open").args(["-a", "Terminal", &target_path]).spawn().map_err(|e| e.to_string())?;
			}
			Ok(serde_json::json!({ "success": true }))
		}
		"createFolder" => {
			let folder_name = name.unwrap_or_else(|| "New folder".to_string());
			let target = unique_path(&Path::new(&target_path).join(&folder_name));
			std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
			Ok(serde_json::json!({ "success": true, "createdPath": target.to_string_lossy() }))
		}
		"createFile" => {
			let file_name = name.unwrap_or_else(|| "New Text Document.txt".to_string());
			let target = Path::new(&target_path).join(&file_name);
			if let Some(b64) = content_base64 {
				use base64::Engine;
				let decoded = base64::engine::general_purpose::STANDARD.decode(b64.as_bytes()).map_err(|e| e.to_string())?;
				std::fs::write(&target, &decoded).map_err(|e| e.to_string())?;
			} else if let Some(txt) = content {
				std::fs::write(&target, txt.as_bytes()).map_err(|e| e.to_string())?;
			} else {
				let target = unique_path(&target); // blank "New" file never clobbers an existing one
				std::fs::write(&target, b"").map_err(|e| e.to_string())?;
				return Ok(serde_json::json!({ "success": true, "createdPath": target.to_string_lossy() }));
			}
			Ok(serde_json::json!({ "success": true, "createdPath": target.to_string_lossy() }))
		}
		"rename" => {
			let new_p = new_path.ok_or_else(|| "Missing newPath".to_string())?;
			std::fs::rename(&target_path, &new_p).map_err(|e| e.to_string())?;
			Ok(serde_json::json!({ "success": true, "newPath": new_p }))
		}
		// Explorer parity: Delete -> Recycle Bin, Shift+Delete -> permanent.
		"delete" => {
			#[cfg(windows)]
			shell::recycle(&target_path)?;
			#[cfg(not(windows))]
			remove_path(&target_path)?;
			Ok(serde_json::json!({ "success": true }))
		}
		"deletePermanent" => {
			remove_path(&target_path)?;
			Ok(serde_json::json!({ "success": true }))
		}
		// Copy/move go through the Windows shell: native progress + conflict dialogs, folders,
		// cross-drive moves, "X - Copy" on collision, and Ctrl+Z in explorer.exe.
		"copy" => {
			let dest = destination_path.ok_or_else(|| "Missing destinationPath".to_string())?;
			#[cfg(windows)]
			shell::copy(&target_path, &dest)?;
			#[cfg(not(windows))]
			std::fs::copy(&target_path, &dest).map_err(|e| e.to_string())?;
			Ok(serde_json::json!({ "success": true }))
		}
		"move" => {
			let dest = destination_path.ok_or_else(|| "Missing destinationPath".to_string())?;
			#[cfg(windows)]
			shell::mv(&target_path, &dest)?;
			#[cfg(not(windows))]
			std::fs::rename(&target_path, &dest).map_err(|e| e.to_string())?;
			Ok(serde_json::json!({ "success": true }))
		}
		_ => Err(format!("Unsupported action: {}", action)),
	}
}

#[tauri::command(async)]
fn get_file_preview(path: String, inline: Option<bool>) -> Result<serde_json::Value, String> {
	let p = Path::new(&path);
	let metadata = std::fs::metadata(p).map_err(|e| e.to_string())?;

	let is_dir = metadata.is_dir();
	let size = metadata.len();
	let modified = metadata.modified().ok()
		.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
		.map(|d| d.as_millis() as u64)
		.unwrap_or(0);

	let created = metadata.created().ok()
		.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
		.map(|d| d.as_millis() as u64)
		.unwrap_or(0);

	if is_dir {
		return Ok(serde_json::json!({
			"is_dir": true,
			"is_image": false,
			"is_binary": false,
			"size": size,
			"modified": modified,
			"created": created,
			"preview": null
		}));
	}

	let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
	let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" | "avif");
	let media = match ext.as_str() {
		_ if is_image => Some("image"),
		"pdf" => Some("pdf"),
		"mp4" | "webm" | "mov" | "m4v" | "mkv" => Some("video"),
		"mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" => Some("audio"),
		_ => None,
	};

	// Media streams to the webview through the asset protocol (convertFileSrc) instead of base64 over IPC.
	if media.is_some() && !inline.unwrap_or(false) {
		return Ok(serde_json::json!({
			"is_dir": false,
			"is_image": is_image,
			"is_binary": true,
			"media": media,
			"size": size,
			"modified": modified,
			"created": created,
			"preview": null
		}));
	}

	if is_image {
		if let Ok(bytes) = std::fs::read(p) {
			use base64::Engine;
			let encoded = base64::prelude::BASE64_STANDARD.encode(&bytes);
			let mime = if ext == "svg" { "image/svg+xml" } else { &format!("image/{}", ext) };
			return Ok(serde_json::json!({
				"is_dir": false,
				"is_image": true,
				"is_binary": false,
				"size": size,
				"modified": modified,
				"created": created,
				"preview": format!("data:{};base64,{}", mime, encoded)
			}));
		}
	}

	// Try reading up to 64KB as text
	let max_bytes = 64 * 1024;
	let mut file = std::fs::File::open(p).map_err(|e| e.to_string())?;
	use std::io::Read;
	let mut buffer = vec![0; std::cmp::min(size as usize, max_bytes)];
	let bytes_read = file.read(&mut buffer).unwrap_or(0);
	buffer.truncate(bytes_read);

	let is_binary = buffer.contains(&0);
	let preview_text = if is_binary {
		None
	} else {
		String::from_utf8(buffer).ok()
	};

	Ok(serde_json::json!({
		"is_dir": false,
		"is_image": false,
		"is_binary": is_binary,
		"size": size,
		"modified": modified,
		"created": created,
		"preview": preview_text
	}))
}

const CATEGORIES: [&str; 8] = [
	"Code & Scripts", "Images & Media", "Videos", "Audio", "Documents", "Archives", "Binaries & Executables", "Other",
];

fn get_file_category(ext: &str) -> usize {
	match ext.to_lowercase().as_str() {
		"js" | "ts" | "vue" | "py" | "rs" | "go" | "json" | "yaml" | "yml" | "toml" | "html" | "css" | "scss" | "c" | "cpp" | "h" | "cs" | "java" | "php" | "sh" | "ps1" | "bat" => 0,
		"png" | "jpg" | "jpeg" | "svg" | "webp" | "gif" | "bmp" | "ico" | "tiff" | "avif" => 1,
		"mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => 2,
		"mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => 3,
		"pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "csv" | "rtf" => 4,
		"zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => 5,
		"exe" | "dll" | "msi" | "bin" | "apk" => 6,
		_ => 7,
	}
}

/// Full-depth parallel scan. Rayon splits work at every directory level (work-stealing),
/// so one giant subfolder no longer pins a single core. On Windows `DirEntry::metadata`
/// comes from FindNextFileW data, so there is no extra syscall per file.
// ponytail: logical size, not allocated size; reading the NTFS MFT directly (WizTree-style) is the next 10x but needs admin.
fn scan_dir(path: &Path, name: String, ctx: &ScanCtx) -> DirNode {
	let mut node = DirNode { name, size: 0, files: 0, dirs: 0, cats: [(0, 0); 8], children: Vec::new() };
	let mut subdirs = Vec::new();
	if let Ok(rd) = std::fs::read_dir(path) {
		for e in rd.flatten() {
			let Ok(ft) = e.file_type() else { continue };
			if ft.is_symlink() { continue; } // also junctions: avoids cycles and double counting
			if ft.is_dir() { subdirs.push(e.path()); continue; }
			let Ok(meta) = e.metadata() else { continue };
			let len = meta.len();
			node.size += len;
			node.files += 1;
			let fname = e.file_name();
			let ext = Path::new(&fname).extension().and_then(|s| s.to_str()).unwrap_or("");
			let c = &mut node.cats[get_file_category(ext)];
			c.0 += len;
			c.1 += 1;
			ctx.offer_top(len, || e.path());
		}
		ctx.files_seen.fetch_add(node.files, Relaxed);
	}

	node.children = subdirs.into_par_iter().map(|p| {
		let n = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
		scan_dir(&p, n, ctx)
	}).collect();

	for ch in &node.children {
		node.size += ch.size;
		node.files += ch.files;
		node.dirs += 1 + ch.dirs;
		for (a, b) in node.cats.iter_mut().zip(ch.cats) {
			a.0 += b.0;
			a.1 += b.1;
		}
	}
	node.children.sort_unstable_by(|a, b| b.size.cmp(&a.size));
	node
}

fn summarize(path: &Path, node: &DirNode, top: &[(u64, PathBuf)]) -> serde_json::Value {
	let total = node.size;
	let pct = |s: u64| if total > 0 { ((s as f64 / total as f64) * 1000.0).round() / 10.0 } else { 0.0 };

	// ponytail: cap at 500 rows (sorted by size) so WinSxS-sized dirs don't flood the table; paginate if needed.
	let subfolders: Vec<SubfolderStats> = node.children.iter().take(500).map(|c| SubfolderStats {
		name: c.name.clone(),
		path: path.join(&c.name).to_string_lossy().into_owned(),
		size: c.size,
		percent: pct(c.size),
		files_count: c.files,
		dirs_count: c.dirs,
	}).collect();

	let mut by_category: Vec<CategoryStats> = CATEGORIES.iter().zip(node.cats)
		.filter(|(_, (_, count))| *count > 0)
		.map(|(cat, (size, count))| CategoryStats { category: cat.to_string(), size, count, percent: pct(size) })
		.collect();
	by_category.sort_by(|a, b| b.size.cmp(&a.size));

	// ponytail: drill-down reuses the root's global top-1000; a subfolder whose big files all rank below that shows fewer than 30.
	let top_files: Vec<TopFile> = top.iter().filter(|(_, p)| p.starts_with(path)).take(30).map(|(size, p)| TopFile {
		name: p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
		path: p.to_string_lossy().into_owned(),
		size: *size,
		ext: p.extension().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default(),
	}).collect();

	serde_json::json!({
		"targetPath": path.to_string_lossy(),
		"totalSize": total,
		"totalFiles": node.files,
		"totalFolders": node.dirs,
		"subfolders": subfolders,
		"byCategory": by_category,
		"topLargestFiles": top_files
	})
}

fn treesize_blocking(app: &tauri::AppHandle, path: String, rescan: bool) -> Result<serde_json::Value, String> {
	let target = PathBuf::from(resolve_dir(path));
	if !target.is_dir() {
		return Err(format!("Path does not exist: {}", target.display()));
	}

	let cache = app.state::<TreeCache>();
	if !rescan {
		if let Some(scan) = cache.0.lock().unwrap().as_ref() {
			if let Some(node) = scan.find(&target) {
				return Ok(summarize(&target, node, &scan.top));
			}
		}
	}

	let ctx = ScanCtx::default();
	let done = AtomicBool::new(false);
	let node = std::thread::scope(|s| {
		s.spawn(|| while !done.load(Relaxed) {
			let _ = app.emit("treesize-progress", ctx.files_seen.load(Relaxed));
			std::thread::sleep(Duration::from_millis(150));
		});
		let n = scan_dir(&target, String::new(), &ctx);
		done.store(true, Relaxed);
		n
	});

	let top = ctx.into_top();
	let result = summarize(&target, &node, &top);
	*cache.0.lock().unwrap() = Some(TreeScan { root: target, node, top });
	Ok(result)
}

/// TreeSize: first call scans, later calls for any folder inside that scan answer from memory.
#[tauri::command]
async fn fast_treesize(app: tauri::AppHandle, path: String, rescan: Option<bool>) -> Result<serde_json::Value, String> {
	tauri::async_runtime::spawn_blocking(move || treesize_blocking(&app, path, rescan.unwrap_or(false)))
		.await
		.map_err(|e| e.to_string())?
}

/// Everything-style filename search over all fixed drives, scoped to `within` like Explorer.
/// `None` = index still building or folder not indexed; the UI then falls back to tgrep.
#[tauri::command(async)]
fn search_names(app: tauri::AppHandle, query: String, within: Option<String>) -> Option<Vec<SearchHit>> {
	const LIMIT: usize = 200;
	let state = app.state::<Arc<name_index::NameIndexState>>();
	if !state.ready.load(Relaxed) {
		return None;
	}
	let candidates: Vec<(usize, String, bool)> = {
		let ix = state.index.read().unwrap();
		ix.search(&query, within.as_deref(), LIMIT * 2)?
			.into_iter()
			.map(|i| (i, ix.path_of(i), ix.is_dir(i)))
			.collect()
	};

	// The index is lowercase: ask Windows for the real case, which also tells us if the hit is
	// gone (deletions are applied lazily here).
	let (mut hits, mut dead, mut seen) = (Vec::new(), Vec::new(), HashSet::new());
	for (i, lower, is_dir) in candidates {
		if hits.len() >= LIMIT {
			break;
		}
		let Some(path) = name_index::real_path(&lower) else {
			dead.push(i);
			continue;
		};
		if seen.insert(lower) {
			let name = Path::new(&path).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
			hits.push(SearchHit { path, name, is_dir, line_number: None, line_content: None, match_start: None, match_end: None });
		}
	}
	if !dead.is_empty() {
		let mut ix = state.index.write().unwrap();
		dead.into_iter().for_each(|i| ix.mark_dead(i));
	}
	Some(hits)
}

fn names_cache(app: &tauri::AppHandle) -> Option<PathBuf> {
	app.path().app_local_data_dir().ok().map(|d| d.join("names.idx"))
}

fn fast_task(app: &tauri::AppHandle) -> String {
	fast_index::task_name(&app.package_info().name)
}

#[tauri::command(async)]
fn fast_index_status(app: tauri::AppHandle) -> bool {
	fast_index::is_enabled(&fast_task(&app))
}

/// Turn the elevated MFT helper on/off (one UAC prompt). Enabling refreshes the index right away.
#[tauri::command(async)]
fn set_fast_index(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
	let cache = names_cache(&app).ok_or("No app data folder")?;
	let task = fast_task(&app);
	fast_index::set_enabled(&task, &cache, enable)?;
	if enable {
		let names = app.state::<Arc<name_index::NameIndexState>>().inner().clone();
		std::thread::spawn(move || {
			if fast_index::refresh(&task, &cache) {
				name_index::load_snapshot(&names, Some(&cache));
			}
		});
	}
	Ok(())
}

#[tauri::command]
async fn search(app: tauri::AppHandle, root: String, query: String) -> Result<Vec<Hit>, String> {
	let out = app.shell().sidecar("tgrep").map_err(|e| e.to_string())?
		.args(["-F", "-S", "-m", "50", &query, &root, "--json"])
		.output().await.map_err(|e| e.to_string())?;

	if out.status.code() == Some(2) {
		return Err(String::from_utf8_lossy(&out.stderr).into());
	}

	let mut hits = Vec::new();
	for line in String::from_utf8_lossy(&out.stdout).lines() {
		let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
		if v["type"] != "match" { continue; }
		let d = &v["data"];
		hits.push(Hit {
			path: d["path"]["text"].as_str().unwrap_or_default().into(),
			line: d["line_number"].as_u64().unwrap_or(0),
			text: d["lines"]["text"].as_str().unwrap_or_default().trim_end().into(),
		});
	}
	Ok(hits)
}

#[tauri::command(async)]
fn fast_file_hash(path: String) -> Result<serde_json::Value, String> {
	use sha2::{Digest, Sha256, Sha512};
	use md5::Md5;
	use sha1::Sha1;
	use std::io::Read;

	let p = Path::new(&path);
	let meta = std::fs::metadata(p).map_err(|e| e.to_string())?;
	if meta.is_dir() {
		return Err("Cannot hash a directory".to_string());
	}

	let mut file = std::fs::File::open(p).map_err(|e| e.to_string())?;
	let mut md5_hasher = Md5::new();
	let mut sha1_hasher = Sha1::new();
	let mut sha256_hasher = Sha256::new();
	let mut sha512_hasher = Sha512::new();

	let mut buffer = [0u8; 64 * 1024];
	loop {
		let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
		if n == 0 { break; }
		let chunk = &buffer[..n];
		md5_hasher.update(chunk);
		sha1_hasher.update(chunk);
		sha256_hasher.update(chunk);
		sha512_hasher.update(chunk);
	}

	let modified = meta.modified().ok()
		.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
		.map(|d| d.as_millis() as u64)
		.unwrap_or(0);

	Ok(serde_json::json!({
		"name": p.file_name().and_then(|s| s.to_str()).unwrap_or(""),
		"path": path,
		"size": meta.len(),
		"modified": modified,
		"md5": format!("{:x}", md5_hasher.finalize()),
		"sha1": format!("{:x}", sha1_hasher.finalize()),
		"sha256": format!("{:x}", sha256_hasher.finalize()),
		"sha512": format!("{:x}", sha512_hasher.finalize()),
	}))
}

#[tauri::command(async)]
fn get_git_status(path: String) -> serde_json::Value {
	#[cfg(target_os = "windows")]
	const CREATE_NO_WINDOW: u32 = 0x08000000;

	let mut rev_parse = Command::new("git");
	rev_parse.args(["rev-parse", "--show-toplevel"]).current_dir(&path);
	#[cfg(target_os = "windows")]
	{
		use std::os::windows::process::CommandExt;
		rev_parse.creation_flags(CREATE_NO_WINDOW);
	}

	let repo_root = match rev_parse.output() {
		Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
		_ => return serde_json::json!({ "isRepo": false }),
	};

	let mut status_cmd = Command::new("git");
	status_cmd.args(["status", "--porcelain=v1", "-b"]).current_dir(&path);
	#[cfg(target_os = "windows")]
	{
		use std::os::windows::process::CommandExt;
		status_cmd.creation_flags(CREATE_NO_WINDOW);
	}

	let status_out = match status_cmd.output() {
		Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
		_ => return serde_json::json!({ "isRepo": true, "repoRoot": repo_root, "branch": "HEAD", "fileStatuses": {} }),
	};

	let mut branch = "HEAD".to_string();
	let mut ahead = 0u32;
	let mut behind = 0u32;
	let mut file_statuses = HashMap::new();

	for line in status_out.lines() {
		let trimmed = line.trim();
		if trimmed.is_empty() { continue; }

		if line.starts_with("## ") {
			let branch_str = &line[3..];
			let parts: Vec<&str> = branch_str.split("...").collect();
			branch = parts[0].trim().to_string();
			if let Some(pos) = branch_str.find("ahead ") {
				let rest = &branch_str[pos + 6..];
				if let Some(end) = rest.find(|c: char| !c.is_numeric()) {
					ahead = rest[..end].parse().unwrap_or(0);
				}
			}
			if let Some(pos) = branch_str.find("behind ") {
				let rest = &branch_str[pos + 7..];
				if let Some(end) = rest.find(|c: char| !c.is_numeric()) {
					behind = rest[..end].parse().unwrap_or(0);
				}
			}
		} else if line.len() >= 3 {
			let code = line[..2].trim().to_string();
			let file_part = line[3..].trim();
			let full_file_path = Path::new(&repo_root).join(file_part).to_string_lossy().to_string();
			let base_name = Path::new(file_part).file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
			file_statuses.insert(full_file_path, code.clone());
			file_statuses.insert(base_name, code);
		}
	}

	serde_json::json!({
		"isRepo": true,
		"repoRoot": repo_root,
		"branch": branch,
		"ahead": ahead,
		"behind": behind,
		"fileStatuses": file_statuses
	})
}

/// Processes holding a file open, via the Windows Restart Manager (same API installers use).
// ponytail: files only; folders need NtQuerySystemInformation handle enumeration like PowerToys does.
#[cfg(windows)]
fn locking_processes(path: &str) -> Vec<String> {
	use windows_sys::Win32::System::RestartManager::*;
	let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
	let files = [wide.as_ptr()];
	let mut session = 0u32;
	let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
	let mut out = Vec::new();
	unsafe {
		if RmStartSession(&mut session, 0, key.as_mut_ptr()) != 0 {
			return out;
		}
		if RmRegisterResources(session, 1, files.as_ptr(), 0, std::ptr::null(), 0, std::ptr::null()) == 0 {
			let (mut needed, mut count, mut reasons) = (0u32, 0u32, 0u32);
			RmGetList(session, &mut needed, &mut count, std::ptr::null_mut(), &mut reasons);
			if needed > 0 {
				let mut infos: Vec<RM_PROCESS_INFO> = vec![std::mem::zeroed(); needed as usize];
				count = needed;
				if RmGetList(session, &mut needed, &mut count, infos.as_mut_ptr(), &mut reasons) == 0 {
					for i in &infos[..count as usize] {
						let end = i.strAppName.iter().position(|&c| c == 0).unwrap_or(i.strAppName.len());
						out.push(format!("{} (PID {})", String::from_utf16_lossy(&i.strAppName[..end]), i.Process.dwProcessId));
					}
				}
			}
		}
		RmEndSession(session);
	}
	out
}

#[cfg(not(windows))]
fn locking_processes(_path: &str) -> Vec<String> {
	Vec::new()
}

#[tauri::command(async)]
fn file_locksmith(path: String) -> serde_json::Value {
	let processes = if Path::new(&path).is_file() { locking_processes(&path) } else { Vec::new() };
	serde_json::json!({
		"isLocked": !processes.is_empty(),
		"path": path,
		"processes": processes
	})
}

#[tauri::command]
async fn search_status(app: tauri::AppHandle, root: String) -> Result<String, String> {
	let out = app.shell().sidecar("tgrep").map_err(|e| e.to_string())?
		.args(["status", &root]).output().await.map_err(|e| e.to_string())?;
	Ok(String::from_utf8_lossy(&out.stdout).into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	fast_index::run_helper_if_requested(); // `--build-index`: elevated helper, exits before any window
	let app = tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.manage(TreeCache(Mutex::new(None)))
		.invoke_handler(tauri::generate_handler![
			list_directory,
			get_drives,
			get_quick_access,
			run_tgrep,
			build_tgrep_index,
			file_action,
			get_file_preview,
			fast_treesize,
			fast_file_hash,
			get_git_status,
			file_locksmith,
			search,
			search_status,
			search_names,
			fast_index_status,
			set_fast_index,
			tools::newplus_list,
			tools::newplus_create,
			tools::ocr_image,
			tools::file_icon,
			tools::shell_context_menu,
			tools::show_properties
		])
		.setup(|app| {
			// Filename index: build in the background, then follow every drive live.
			let names = Arc::new(name_index::NameIndexState::default());
			app.manage(names.clone());
			let cache = names_cache(app.handle());
			let task = fast_task(app.handle());
			std::thread::spawn(move || {
				let cache = cache.as_deref();
				name_index::load_snapshot(&names, cache); // search works immediately
				// Elevated MFT helper if the user enabled it, else rebuild here (walk, low priority).
				if cache.is_some_and(|c| fast_index::refresh(&task, c)) {
					name_index::load_snapshot(&names, cache);
				} else {
					name_index::rebuild(&names, cache);
				}
				#[cfg(windows)]
				for root in name_index::fixed_drives() {
					let s = names.clone();
					std::thread::spawn(move || name_index::watch(&root, &s));
				}
			});

			let initial_root = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
			if let Ok(sidecar_cmd) = app.shell().sidecar("tgrep") {
				if let Ok((mut rx, child)) = sidecar_cmd
					.args(["serve", &initial_root, "--watcher-queue-cap", "65536"])
					.spawn()
				{
					tauri::async_runtime::spawn(async move {
						while let Some(ev) = rx.recv().await {
							if let tauri_plugin_shell::process::CommandEvent::Stderr(l) = ev {
								eprintln!("[tgrep] {}", String::from_utf8_lossy(&l));
							}
						}
					});

					app.manage(TgrepServer(Mutex::new(Some(child))));
				} else {
					app.manage(TgrepServer(Mutex::new(None)));
				}
			} else {
				app.manage(TgrepServer(Mutex::new(None)));
			}

			#[cfg(desktop)]
			{
				let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
				let menu = Menu::with_items(app, &[&quit])?;

				TrayIconBuilder::new()
					.icon(app.default_window_icon().expect("missing default window icon").clone())
					.menu(&menu)
					.show_menu_on_left_click(true)
					.on_menu_event(|app_handle, event| {
						if event.id.as_ref() == "quit" {
							app_handle.exit(0);
						}
					})
					.build(app)?;
			}

			Ok(())
		})
		.build(tauri::generate_context!())
		.expect("error while building tauri application");

	app.run(|handle, event| {
		if let RunEvent::Exit = event {
			if let Some(state) = handle.try_state::<TgrepServer>() {
				if let Ok(mut lock) = state.0.lock() {
					if let Some(child) = lock.take() {
						let _ = child.kill();
					}
				}
			}
		}
	});
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn treesize_scan_and_unique_names() {
		let root = std::env::temp_dir().join(format!("jelajah-ts-{}", std::process::id()));
		let _ = std::fs::remove_dir_all(&root);
		std::fs::create_dir_all(root.join("a/b/c/d/e/f")).unwrap(); // deeper than the old max_depth(5)
		std::fs::write(root.join("top.txt"), [0u8; 10]).unwrap();
		std::fs::write(root.join("a/pic.png"), [0u8; 100]).unwrap();
		std::fs::write(root.join("a/b/c/d/e/f/deep.mp4"), [0u8; 1000]).unwrap();

		let ctx = ScanCtx::default();
		let node = scan_dir(&root, String::new(), &ctx);
		assert_eq!(node.size, 1110);
		assert_eq!(node.files, 3);
		assert_eq!(node.dirs, 6);
		assert_eq!(node.cats[2], (1000, 1)); // Videos

		let scan = TreeScan { root: root.clone(), top: ctx.into_top(), node };
		assert_eq!(scan.find(&root.join("a/b")).unwrap().size, 1000);
		let v = summarize(&root.join("a"), scan.find(&root.join("a")).unwrap(), &scan.top);
		assert_eq!(v["topLargestFiles"][0]["name"], "deep.mp4");
		assert_eq!(v["subfolders"][0]["filesCount"], 1);

		assert_eq!(unique_path(&root.join("top.txt")), root.join("top (2).txt"));
		assert_eq!(unique_path(&root.join("nope")), root.join("nope"));
		std::fs::remove_dir_all(&root).unwrap();
	}

	/// cargo test --lib bench_scan -- --ignored --nocapture   (TREESIZE_DIR=C:\ to try a drive)
	#[test]
	#[ignore]
	fn bench_scan() {
		let dir = std::env::var("TREESIZE_DIR").unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap());
		let t = std::time::Instant::now();
		let ctx = ScanCtx::default();
		let n = scan_dir(Path::new(&dir), String::new(), &ctx);
		println!("{dir}: {} files, {} dirs, {} GB in {:?}", n.files, n.dirs, n.size / (1 << 30), t.elapsed());
	}
}
