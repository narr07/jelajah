//! Everything-style filename index over every fixed drive.
//!
//! Names live in one flat lowercase byte arena (Unicode-lowercased, '\0'-separated), so a search
//! is one SIMD `memmem` pass. Per entry we keep only a name offset, a parent index and a flag
//! byte (9 bytes + the name), never a full path. Real letter case is fetched from Windows only
//! for the hits we display.
//!
//! Lifecycle: load the last snapshot from disk (ready in < 1 s), rebuild in the background
//! (NTFS MFT via the elevated helper in fast_index.rs or when we run elevated, low-priority
//! directory walk otherwise), save, swap, then follow every drive live with ReadDirectoryChangesW. Deletions are applied lazily when
//! a hit turns out to be gone from disk.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use memchr::memmem;
use rayon::prelude::*;

const NO_PARENT: u32 = u32::MAX;
const DIR: u8 = 1;
const DEAD: u8 = 2;
const MAGIC: &[u8; 8] = b"NXNAMES1";

#[derive(Default)]
pub struct NameIndex {
	lower: Vec<u8>,          // lowercase UTF-8 names, each followed by '\0'
	off: Vec<u32>,           // name start per entry, ascending
	parent: Vec<u32>,
	flags: Vec<u8>,
	dirs: HashMap<u64, u32>, // key(dir path) -> entry, for `within` and live inserts
}

/// The index plus a flag so the UI can fall back to tgrep until the first load/build finishes.
#[derive(Default)]
pub struct NameIndexState {
	pub index: RwLock<NameIndex>,
	pub ready: AtomicBool,
}

pub struct Walk {
	name: String,
	files: Vec<String>,
	children: Vec<Walk>,
}

pub enum Added {
	File(String),
	Dir(Walk),
}

/// Parallel directory walk (same work-stealing shape as the TreeSize scan). Names come out lowercase.
pub fn walk(path: &Path, name: String) -> Walk {
	let mut w = Walk { name, files: Vec::new(), children: Vec::new() };
	let mut subdirs = Vec::new();
	if let Ok(rd) = std::fs::read_dir(path) {
		for e in rd.flatten() {
			let Ok(ft) = e.file_type() else { continue };
			let n = e.file_name().to_string_lossy().to_lowercase();
			// Symlinks/junctions are indexed as leaves, never followed (no cycles).
			if ft.is_dir() { subdirs.push((e.path(), n)) } else { w.files.push(n) }
		}
	}
	w.children = subdirs.into_par_iter().map(|(p, n)| walk(&p, n)).collect();
	w
}

fn key(path: &str) -> u64 {
	let mut h = DefaultHasher::new();
	path.replace('/', "\\").trim_end_matches('\\').to_lowercase().hash(&mut h);
	h.finish()
}

impl NameIndex {
	pub fn len(&self) -> usize {
		self.off.len()
	}

	/// `name` must already be lowercase.
	fn push(&mut self, name: &str, parent: u32, flags: u8) -> u32 {
		let i = self.off.len() as u32;
		self.off.push(self.lower.len() as u32);
		self.lower.extend_from_slice(name.as_bytes());
		self.lower.push(0);
		self.parent.push(parent);
		self.flags.push(flags);
		i
	}

	fn truncate(&mut self, entries: usize) {
		if entries < self.off.len() {
			self.lower.truncate(self.off[entries] as usize);
			self.off.truncate(entries);
			self.parent.truncate(entries);
			self.flags.truncate(entries);
		}
	}

	/// Append a walked tree under `parent` (NO_PARENT for a root); `path` is the tree's full path.
	fn add(&mut self, w: Walk, parent: u32, path: &str) {
		let i = self.push(&w.name, parent, DIR);
		self.dirs.insert(key(path), i);
		for f in &w.files {
			self.push(f, i, 0);
		}
		for c in w.children {
			let child = format!("{}\\{}", path.trim_end_matches('\\'), c.name);
			self.add(c, i, &child);
		}
	}

	pub fn add_root(&mut self, w: Walk, path: &str) {
		self.add(w, NO_PARENT, path);
	}

	fn range(&self, i: usize) -> std::ops::Range<usize> {
		let end = self.off.get(i + 1).map_or(self.lower.len(), |&o| o as usize) - 1;
		self.off[i] as usize..end
	}

	fn name(&self, i: usize) -> &str {
		std::str::from_utf8(&self.lower[self.range(i)]).unwrap_or("")
	}

	pub fn is_dir(&self, i: usize) -> bool {
		self.flags[i] & DIR != 0
	}

	pub fn mark_dead(&mut self, i: usize) {
		self.flags[i] |= DEAD;
	}

	/// Lowercase full path; see `real_path` for display case.
	pub fn path_of(&self, mut i: usize) -> String {
		let mut parts = Vec::new();
		loop {
			parts.push(self.name(i));
			let p = self.parent[i];
			if p == NO_PARENT { break; }
			i = p as usize;
		}
		parts.reverse();
		let s = parts.join("\\");
		if s.ends_with(':') { s + "\\" } else { s }
	}

	fn is_under(&self, i: usize, ancestor: u32) -> bool {
		let mut p = self.parent[i];
		while p != NO_PARENT {
			if p == ancestor { return true; }
			p = self.parent[p as usize];
		}
		false
	}

	/// Entries whose name contains every whitespace-separated term, optionally under `within`.
	/// `None` means `within` is not indexed, so the caller should fall back.
	// ponytail: substring terms only; wildcards (*.pdf) and ext:/size: filters not yet.
	pub fn search(&self, query: &str, within: Option<&str>, limit: usize) -> Option<Vec<usize>> {
		let within = match within {
			Some(w) if !w.is_empty() => Some(*self.dirs.get(&key(w))?),
			_ => None,
		};
		let q = query.replace('\0', "").to_lowercase();
		let terms: Vec<&str> = q.split_whitespace().collect();
		let Some((first, rest)) = terms.split_first() else { return Some(Vec::new()) };

		let mut out = Vec::new();
		let mut last = usize::MAX;
		for pos in memmem::find_iter(&self.lower, first.as_bytes()) {
			let i = self.off.partition_point(|&o| o as usize <= pos) - 1;
			if i == last { continue; }
			last = i;
			if self.flags[i] & DEAD != 0 { continue; }
			let lc = &self.lower[self.range(i)];
			if !rest.iter().all(|t| memmem::find(lc, t.as_bytes()).is_some()) { continue; }
			if within.is_some_and(|w| !self.is_under(i, w)) { continue; }
			out.push(i);
			if out.len() >= limit { break; }
		}
		Some(out)
	}

	/// Live insert; `added` is prepared outside the lock because walking a moved-in folder can be slow.
	pub fn insert(&mut self, path: &Path, added: Added) {
		let Some(parent) = path.parent() else { return };
		let Some(&pi) = self.dirs.get(&key(&parent.to_string_lossy())) else { return };
		match added {
			Added::File(n) => { self.push(&n, pi, 0); }
			Added::Dir(w) => self.add(w, pi, &path.to_string_lossy()),
		}
	}

	fn save(&self, file: &Path) -> std::io::Result<()> {
		let tmp = file.with_extension("tmp");
		let mut w = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
		w.write_all(MAGIC)?;
		for n in [self.off.len(), self.lower.len(), self.dirs.len()] {
			w.write_all(&(n as u64).to_le_bytes())?;
		}
		w.write_all(&self.lower)?;
		for v in self.off.iter().chain(&self.parent) {
			w.write_all(&v.to_le_bytes())?;
		}
		w.write_all(&self.flags)?;
		for (k, v) in &self.dirs {
			w.write_all(&k.to_le_bytes())?;
			w.write_all(&v.to_le_bytes())?;
		}
		w.into_inner()?.sync_all()?;
		std::fs::rename(tmp, file)
	}

	fn load(file: &Path) -> Option<Self> {
		let b = std::fs::read(file).ok()?;
		let mut at = 0usize;
		let mut take = |n: usize| -> Option<&[u8]> {
			let s = b.get(at..at + n)?;
			at += n;
			Some(s)
		};
		if take(8)? != MAGIC { return None; }
		let mut num = || Some(u64::from_le_bytes(take(8)?.try_into().ok()?) as usize);
		let (n, bytes, ndirs) = (num()?, num()?, num()?);
		let lower = take(bytes)?.to_vec();
		let u32s = |s: &[u8]| s.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect::<Vec<_>>();
		let off = u32s(take(n * 4)?);
		let parent = u32s(take(n * 4)?);
		let flags = take(n)?.to_vec();
		let dirs = take(ndirs * 12)?
			.chunks_exact(12)
			.map(|c| (u64::from_le_bytes(c[..8].try_into().unwrap()), u32::from_le_bytes(c[8..].try_into().unwrap())))
			.collect();
		Some(Self { lower, off, parent, flags, dirs })
	}
}

pub fn prepare(path: &Path) -> Option<Added> {
	let name = path.file_name()?.to_string_lossy().to_lowercase();
	let md = std::fs::symlink_metadata(path).ok()?;
	Some(if md.is_dir() { Added::Dir(walk(path, name)) } else { Added::File(name) })
}

/// Path with its real letter case, or `None` if it no longer exists.
#[cfg(windows)]
pub fn real_path(p: &str) -> Option<String> {
	use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
	use windows_sys::Win32::Storage::FileSystem::*;

	let w: Vec<u16> = format!(r"\\?\{p}").encode_utf16().chain(Some(0)).collect();
	let h = unsafe {
		CreateFileW(
			w.as_ptr(),
			0, // attributes only: works on files other processes hold open
			FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
			std::ptr::null(),
			OPEN_EXISTING,
			FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
			std::ptr::null_mut(),
		)
	};
	if h == INVALID_HANDLE_VALUE {
		// not found / path not found => gone; anything else (access denied...) => it exists
		return match std::io::Error::last_os_error().raw_os_error() {
			Some(2 | 3) => None,
			_ => Some(p.to_string()),
		};
	}
	let mut buf = vec![0u16; 1024];
	let mut n = unsafe { GetFinalPathNameByHandleW(h, buf.as_mut_ptr(), buf.len() as u32, 0) } as usize;
	if n > buf.len() {
		buf.resize(n, 0);
		n = unsafe { GetFinalPathNameByHandleW(h, buf.as_mut_ptr(), buf.len() as u32, 0) } as usize;
	}
	unsafe { CloseHandle(h) };
	if n == 0 || n > buf.len() {
		return Some(p.to_string());
	}
	let s = String::from_utf16_lossy(&buf[..n]);
	Some(s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s))
}

#[cfg(not(windows))]
pub fn real_path(p: &str) -> Option<String> {
	std::fs::symlink_metadata(p).ok().map(|_| p.to_string())
}

#[cfg(windows)]
pub fn fixed_drives() -> Vec<String> {
	use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
	const DRIVE_FIXED: u32 = 3;
	(b'A'..=b'Z')
		.map(|c| format!("{}:\\", c as char))
		.filter(|d| {
			let w: Vec<u16> = d.encode_utf16().chain(Some(0)).collect();
			unsafe { GetDriveTypeW(w.as_ptr()) == DRIVE_FIXED }
		})
		.collect()
}

#[cfg(not(windows))]
pub fn fixed_drives() -> Vec<String> {
	Vec::new()
}

/// Whole-volume listing straight from the NTFS MFT via FSCTL_ENUM_USN_DATA (what Everything does).
/// Needs admin to open the volume; returns false (index untouched) so the caller can walk instead.
// ponytail: one name per MFT record, so extra hard-link names (pnpm/bun node_modules, WinSxS) are
// missing; parsing every $FILE_NAME attribute from the raw MFT would cover them.
#[cfg(windows)]
fn mft_into(ix: &mut NameIndex, root: &str) -> bool {
	use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE};
	use windows_sys::Win32::Storage::FileSystem::*;
	use windows_sys::Win32::System::IO::DeviceIoControl;
	use windows_sys::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0, USN_RECORD_V2};
	const ROOT_RECORD: u64 = 5; // MFT record of the volume root
	const ATTR_DIRECTORY: u32 = 0x10;
	const ERROR_HANDLE_EOF: i32 = 38;

	let vol: Vec<u16> = format!(r"\\.\{}", root.trim_end_matches('\\')).encode_utf16().chain(Some(0)).collect();
	let h = unsafe {
		CreateFileW(vol.as_ptr(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, std::ptr::null(), OPEN_EXISTING, 0, std::ptr::null_mut())
	};
	if h == INVALID_HANDLE_VALUE {
		// os error 5 = access denied = not elevated
		eprintln!("[names] {root}: MFT unavailable, cannot open volume ({}); walking", std::io::Error::last_os_error());
		return false;
	}

	let start = ix.len();
	let root_idx = ix.push(&root.trim_end_matches('\\').to_lowercase(), NO_PARENT, DIR);
	let mut dir_of_frn: HashMap<u64, u32> = HashMap::new();
	let mut parent_frn: Vec<u64> = Vec::new();
	let mut med = MFT_ENUM_DATA_V0 { StartFileReferenceNumber: 0, LowUsn: 0, HighUsn: i64::MAX };
	let mut buf = vec![0u64; 1 << 17]; // 1 MB, 8-byte aligned for the records

	let ok = loop {
		let mut got = 0u32;
		let r = unsafe {
			DeviceIoControl(
				h,
				FSCTL_ENUM_USN_DATA,
				(&raw const med).cast(),
				size_of::<MFT_ENUM_DATA_V0>() as u32,
				buf.as_mut_ptr().cast(),
				(buf.len() * 8) as u32,
				&mut got,
				std::ptr::null_mut(),
			)
		};
		if r == 0 {
			let e = std::io::Error::last_os_error();
			if e.raw_os_error() == Some(ERROR_HANDLE_EOF) {
				break true;
			}
			eprintln!("[names] {root}: MFT enumeration failed ({e}); walking");
			break false;
		}
		if got <= 8 {
			break true;
		}
		med.StartFileReferenceNumber = buf[0];
		let bytes = buf.as_ptr() as *const u8;
		let mut at = 8usize;
		let mut v2 = true;
		while at < got as usize {
			let rec = unsafe { &*(bytes.add(at) as *const USN_RECORD_V2) };
			if rec.MajorVersion != 2 {
				eprintln!("[names] {root}: USN record v{} (ReFS?); walking", rec.MajorVersion);
				v2 = false; // ReFS / 128-bit ids: let the walker handle this volume
				break;
			}
			let name = unsafe {
				std::slice::from_raw_parts(bytes.add(at + rec.FileNameOffset as usize) as *const u16, rec.FileNameLength as usize / 2)
			};
			let dir = rec.FileAttributes & ATTR_DIRECTORY != 0;
			let i = ix.push(&String::from_utf16_lossy(name).to_lowercase(), NO_PARENT, if dir { DIR } else { 0 });
			if dir {
				dir_of_frn.insert(rec.FileReferenceNumber, i);
			}
			parent_frn.push(rec.ParentFileReferenceNumber);
			at += rec.RecordLength as usize;
		}
		if !v2 {
			break false;
		}
	};
	unsafe { CloseHandle(h) };
	if !ok {
		ix.truncate(start);
		return false;
	}

	// Link parents by file reference number; orphans (NTFS metadata under $Extend...) are hidden.
	for (k, pf) in parent_frn.into_iter().enumerate() {
		let i = root_idx as usize + 1 + k;
		ix.parent[i] = if pf & 0xFFFF_FFFF_FFFF == ROOT_RECORD {
			root_idx
		} else if let Some(&p) = dir_of_frn.get(&pf) {
			p
		} else {
			ix.flags[i] |= DEAD;
			root_idx
		};
	}
	for i in root_idx as usize..ix.len() {
		if ix.flags[i] == DIR {
			let k = key(&ix.path_of(i));
			ix.dirs.insert(k, i as u32);
		}
	}
	true
}

fn build_fresh() -> NameIndex {
	let mut ix = NameIndex::default();
	// Fallback walk: half the cores in Windows background mode (low CPU + IO priority).
	let threads = (std::thread::available_parallelism().map_or(2, |n| n.get()) / 2).max(1);
	let pool = rayon::ThreadPoolBuilder::new()
		.num_threads(threads)
		.start_handler(|_| {
			#[cfg(windows)]
			unsafe {
				use windows_sys::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN};
				SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN);
			}
		})
		.build()
		.unwrap();

	for root in fixed_drives() {
		let t = Instant::now();
		#[cfg(windows)]
		if mft_into(&mut ix, &root) {
			eprintln!("[names] {root} via MFT: {} entries in {:?}", ix.len(), t.elapsed());
			continue;
		}
		let w = pool.install(|| walk(Path::new(&root), root.trim_end_matches('\\').to_lowercase()));
		ix.add_root(w, &root);
		eprintln!("[names] {root} via walk: {} entries in {:?}", ix.len(), t.elapsed());
	}
	ix
}

/// Load the last snapshot so search works right away. false = no usable snapshot.
pub fn load_snapshot(state: &NameIndexState, cache: Option<&Path>) -> bool {
	let Some(ix) = cache.and_then(NameIndex::load) else { return false };
	*state.index.write().unwrap() = ix;
	state.ready.store(true, Ordering::Release);
	true
}

/// Rebuild in this process (MFT when elevated, walk otherwise), save, swap in.
pub fn rebuild(state: &NameIndexState, cache: Option<&Path>) {
	let fresh = build_fresh();
	if let Some(file) = cache {
		if let Err(e) = save_to(&fresh, file) {
			eprintln!("[names] could not save index: {e}");
		}
	}
	// ponytail: watcher events that hit the old index during the rebuild are dropped by this swap.
	*state.index.write().unwrap() = fresh;
	state.ready.store(true, Ordering::Release);
}

/// Elevated helper entry point (see fast_index.rs): build and write the snapshot only.
pub fn build_to_file(file: &Path) -> std::io::Result<()> {
	save_to(&build_fresh(), file)
}

fn save_to(ix: &NameIndex, file: &Path) -> std::io::Result<()> {
	if let Some(dir) = file.parent() {
		std::fs::create_dir_all(dir)?;
	}
	ix.save(file)
}

/// Follow one drive with ReadDirectoryChangesW (the API Explorer itself uses). Blocks forever.
// ponytail: events during a rebuild and on buffer overflow are lost until the next start; the
// arena only grows during a session (temp-file churn adds ~25 B per name); restart compacts.
#[cfg(windows)]
pub fn watch(root: &str, state: &NameIndexState) {
	use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
	use windows_sys::Win32::Storage::FileSystem::*;

	let wide: Vec<u16> = root.encode_utf16().chain(Some(0)).collect();
	let h = unsafe {
		CreateFileW(
			wide.as_ptr(),
			FILE_LIST_DIRECTORY,
			FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
			std::ptr::null(),
			OPEN_EXISTING,
			FILE_FLAG_BACKUP_SEMANTICS,
			std::ptr::null_mut(),
		)
	};
	if h == INVALID_HANDLE_VALUE { return; }

	let mut buf = vec![0u32; 16 * 1024]; // 64 KB; u32 keeps records DWORD-aligned
	loop {
		let mut n = 0u32;
		let ok = unsafe {
			ReadDirectoryChangesW(
				h,
				buf.as_mut_ptr().cast(),
				(buf.len() * 4) as u32,
				1,
				FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME,
				&mut n,
				std::ptr::null_mut(),
				None,
			)
		};
		if ok == 0 { return; }
		if n == 0 { continue; } // overflow, see ponytail note

		let mut added = Vec::new();
		let mut p = buf.as_ptr() as *const u8;
		loop {
			let info = unsafe { &*(p as *const FILE_NOTIFY_INFORMATION) };
			if info.Action == FILE_ACTION_ADDED || info.Action == FILE_ACTION_RENAMED_NEW_NAME {
				let rel = unsafe { std::slice::from_raw_parts(info.FileName.as_ptr(), info.FileNameLength as usize / 2) };
				added.push(std::path::PathBuf::from(format!("{root}{}", String::from_utf16_lossy(rel))));
			}
			if info.NextEntryOffset == 0 { break; }
			p = unsafe { p.add(info.NextEntryOffset as usize) };
		}

		let prepared: Vec<_> = added.into_iter().filter_map(|a| Some((prepare(&a)?, a))).collect();
		if !prepared.is_empty() {
			let mut ix = state.index.write().unwrap();
			for (item, path) in prepared {
				ix.insert(&path, item);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn search_insert_scope_case_and_snapshot() {
		let root = std::env::temp_dir().join(format!("jelajah-ni-{}", std::process::id()));
		let _ = std::fs::remove_dir_all(&root);
		std::fs::create_dir_all(root.join("Docs/Work")).unwrap();
		std::fs::write(root.join("Docs/Work/Laporan Keuangan 2026.xlsx"), b"").unwrap();
		std::fs::write(root.join("Docs/laporan-lama.pdf"), b"").unwrap();
		std::fs::write(root.join("ÉTÉ Foto.PNG"), b"").unwrap();

		let rs = root.to_string_lossy().into_owned();
		let lrs = rs.to_lowercase();
		let mut ix = NameIndex::default();
		ix.add_root(walk(&root, lrs.clone()), &rs);
		let paths = |v: Vec<usize>, ix: &NameIndex| -> Vec<String> { v.into_iter().map(|i| ix.path_of(i)).collect() };

		// case-insensitive (incl. non-ASCII), multi-term AND
		assert_eq!(ix.search("LAPORAN", None, 10).unwrap().len(), 2);
		assert_eq!(ix.search("été", None, 10).unwrap().len(), 1);
		let hit = paths(ix.search("laporan 2026", None, 10).unwrap(), &ix);
		assert_eq!(hit, vec![format!("{lrs}\\docs\\work\\laporan keuangan 2026.xlsx")]);
		// display case comes back from Windows
		assert_eq!(real_path(&hit[0]).unwrap(), format!("{rs}\\Docs\\Work\\Laporan Keuangan 2026.xlsx"));
		assert!(real_path(&format!("{lrs}\\gone.txt")).is_none());
		// scoped to a folder; unknown folder -> None (caller falls back)
		assert_eq!(ix.search("laporan", Some(&format!("{rs}\\Docs\\Work")), 10).unwrap().len(), 1);
		assert!(ix.search("laporan", Some("Z:\\nope"), 10).is_none());

		// live insert of a new folder with content, then lazy delete
		std::fs::create_dir_all(root.join("New/Deep")).unwrap();
		std::fs::write(root.join("New/Deep/target.txt"), b"").unwrap();
		let p = root.join("New");
		ix.insert(&p, prepare(&p).unwrap());
		let hit = ix.search("target", None, 10).unwrap();
		assert_eq!(ix.path_of(hit[0]), format!("{lrs}\\new\\deep\\target.txt"));
		ix.mark_dead(hit[0]);
		assert!(ix.search("target", None, 10).unwrap().is_empty());

		// snapshot round-trip
		let file = root.join("names.idx");
		ix.save(&file).unwrap();
		let back = NameIndex::load(&file).unwrap();
		assert_eq!(back.len(), ix.len());
		assert_eq!(back.search("laporan", Some(&format!("{rs}\\Docs")), 10).unwrap().len(), 2);
		assert!(back.search("target", None, 10).unwrap().is_empty());

		std::fs::remove_dir_all(&root).unwrap();
	}

	/// cargo test --release --lib bench_name_index -- --ignored --nocapture   (run elevated to test MFT)
	#[test]
	#[ignore]
	fn bench_name_index() {
		let t = Instant::now();
		let ix = build_fresh();
		let mb = (ix.lower.len() + ix.off.len() * 9 + ix.dirs.len() * 24) as f64 / 1e6;
		println!("{} entries, ~{mb:.0} MB, built in {:?}", ix.len(), t.elapsed());
		let file = std::env::temp_dir().join("jelajah-bench-names.idx");
		let t = Instant::now();
		ix.save(&file).unwrap();
		println!("save {:?} ({} MB on disk)", t.elapsed(), std::fs::metadata(&file).unwrap().len() / 1_000_000);
		let t = Instant::now();
		let ix = NameIndex::load(&file).unwrap();
		println!("load {:?}", t.elapsed());
		for q in ["readme", "png", "laporan 2026", "zzzqqq"] {
			let t = Instant::now();
			let n = ix.search(q, None, 200).unwrap().len();
			println!("search {q:?}: {n} hits in {:?}", t.elapsed());
		}
		let _ = std::fs::remove_file(file);
	}
}
