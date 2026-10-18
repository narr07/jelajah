//! Optional elevated helper for the filename index: Everything-Service speed without a resident
//! service. A Scheduled Task ("run with highest privileges") starts `<exe> --build-index <file>`,
//! which reads the NTFS MFT, writes the snapshot and exits. Registering the task costs one UAC
//! prompt; running it later does not, so the app itself never needs to run as admin.
//
// ponytail: the task runs our exe elevated, so the exe must live where only admins can write
// (per-machine install in Program Files). With a per-user install, user-level malware could swap
// the exe and inherit admin. Task defaults also skip runs on battery; refresh() then times out
// and the caller falls back to the in-process walk.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

pub const HELPER_ARG: &str = "--build-index";

pub fn task_name(app_name: &str) -> String {
	format!("{app_name} Fast Index")
}

/// Helper mode: build the snapshot and exit before Tauri creates any window.
pub fn run_helper_if_requested() {
	let args: Vec<String> = std::env::args().collect();
	if args.get(1).map(String::as_str) != Some(HELPER_ARG) {
		return;
	}
	let code = match args.get(2) {
		Some(file) => match crate::name_index::build_to_file(Path::new(file)) {
			Ok(()) => 0,
			Err(e) => {
				eprintln!("[names] helper failed: {e}");
				1
			}
		},
		None => 2,
	};
	std::process::exit(code);
}

fn schtasks(args: &[&str]) -> bool {
	let mut c = Command::new("schtasks");
	c.args(args);
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
	}
	c.output().is_ok_and(|o| o.status.success())
}

pub fn is_enabled(task: &str) -> bool {
	schtasks(&["/query", "/tn", task])
}

fn mtime(f: &Path) -> Option<SystemTime> {
	std::fs::metadata(f).and_then(|m| m.modified()).ok()
}

/// Run the helper and wait for it to rewrite the snapshot. false = not enabled / did not finish.
pub fn refresh(task: &str, file: &Path) -> bool {
	let before = mtime(file);
	if !schtasks(&["/run", "/tn", task]) {
		return false;
	}
	let start = Instant::now();
	while start.elapsed() < Duration::from_secs(90) {
		std::thread::sleep(Duration::from_millis(500));
		let now = mtime(file);
		if now.is_some() && now != before {
			return true;
		}
	}
	false
}

/// Register (enable) or remove the task with one elevated schtasks call: a single UAC prompt.
#[cfg(windows)]
pub fn set_enabled(task: &str, file: &Path, enable: bool) -> Result<(), String> {
	let exe = std::env::current_exe().map_err(|e| e.to_string())?;
	let params = if enable {
		format!(
			r#"/create /f /tn "{task}" /sc onlogon /rl highest /tr "\"{}\" {HELPER_ARG} \"{}\"""#,
			exe.display(),
			file.display()
		)
	} else {
		format!(r#"/delete /f /tn "{task}""#)
	};
	run_elevated("schtasks.exe", &params)
}

#[cfg(not(windows))]
pub fn set_enabled(_task: &str, _file: &Path, _enable: bool) -> Result<(), String> {
	Err("Only available on Windows".into())
}

#[cfg(windows)]
fn run_elevated(file: &str, params: &str) -> Result<(), String> {
	use windows_sys::Win32::Foundation::CloseHandle;
	use windows_sys::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};
	use windows_sys::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};

	let w = |s: &str| -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() };
	let (verb, file, params) = (w("runas"), w(file), w(params));
	let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
	info.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
	info.fMask = SEE_MASK_NOCLOSEPROCESS;
	info.lpVerb = verb.as_ptr();
	info.lpFile = file.as_ptr();
	info.lpParameters = params.as_ptr();
	info.nShow = 0; // SW_HIDE
	if unsafe { ShellExecuteExW(&mut info) } == 0 {
		let e = std::io::Error::last_os_error();
		return Err(if e.raw_os_error() == Some(1223) { "Cancelled at the UAC prompt".into() } else { e.to_string() });
	}
	let mut code = 1u32;
	unsafe {
		WaitForSingleObject(info.hProcess, INFINITE);
		GetExitCodeProcess(info.hProcess, &mut code);
		CloseHandle(info.hProcess);
	}
	if code == 0 { Ok(()) } else { Err(format!("schtasks exited with code {code}")) }
}
