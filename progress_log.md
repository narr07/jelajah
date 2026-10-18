# Log Pengerjaan: Jelajah (Windows File Explorer) (Tauri 2 + Nuxt 4 + Rust Backend + tgrep)

**Status Proyek**: Aktif & Berjalan  
**Arsitektur**: **Nuxt 4** murni sebagai Frontend UI • **Tauri 2 / Rust** sebagai Engine Backend Lokal  
**Package Manager**: Bun (`bun tauri:dev`)

---

## 1. Arsitektur & Pergeseran Backend ke Rust
Sesuai arahan, **seluruh logika komputasi berat, filesystem IO, hashing, dan proses eksternal dipindahkan 100% ke sisi Rust (Tauri Backend)**. Nuxt hanya bertugas sebagai renderer UI yang responsif dan interaktif.

| Komponen | Implementasi Lama (Node/Nitro) | Implementasi Baru (Native Rust) | Keuntungan |
|---|---|---|---|
| **Pencarian File & Teks** | Eksekusi CLI via Node child_process | **tgrep sidecar daemon** (`tgrep serve`) via Rust | Index selalu live & auto-update, pencarian < 10ms |
| **TreeSize Disk Analyzer** | Recursive serial `fs.readdir` (lambat di drive besar) | **Multi-threaded parallel `rayon` + `walkdir`** di Rust | Scan ratusan ribu file paralel lintas core CPU |
| **File Checksum / Hasher** | Node stream crypto | **Native streaming SHA-256, MD5, SHA-1, SHA-512** di Rust | Zero node overhead, instan untuk file berukuran GB |
| **Git Status & Badges** | Node execFile | **Native Rust command** (`git rev-parse` & `git status`) | Deteksi branch & dirty file otomatis tanpa freeze |
| **File Locksmith** | Tidak ada | **Native Rust Win32 Process Inspector** | Mengetahui proses mana yang sedang mengunci file |

---

## 2. Integrasi tgrep sebagai Native Sidecar
1. **Target Triple**: Terdeteksi host Windows msvc `x86_64-pc-windows-msvc`.
2. **Binary Sidecar**: Binary tgrep telah disalin ke:
   `src-tauri/binaries/tgrep-x86_64-pc-windows-msvc.exe`
3. **Pendaftaran Konfigurasi**:
   Didaftarkan pada `src-tauri/tauri.conf.json`:
   ```json
   "bundle": {
     "externalBin": ["binaries/tgrep"]
   }
   ```
4. **Lifecycle Server Daemon**:
   - Dikelola oleh struct `TgrepServer(Mutex<Option<CommandChild>>)` di Rust.
   - `tgrep serve <root> --watcher-queue-cap 65536` dijalankan otomatis saat startup.
   - Output/stderr di-drain agar channel tidak tersumbat.
   - Ditutup secara bersih saat `RunEvent::Exit`.
5. **Tauri Command**:
   - `search(app, root, query)`: Pencarian teks/literal instan output JSON.
   - `search_status(app, root)`: Status watcher/indexing.
   - Frontend: Menggunakan **debounce 200ms** dan indikator status *"Mengindeks…"* ketika background indexing sedang berlangsung.

---

## 3. Fitur TreeSize (JAM Software Style)
- **Engine**: Menggunakan `rayon::par_iter()` dan `walkdir::WalkDir` pada Rust backend, membagi pemindaian folder di setiap subdirektori secara simultan lintas core CPU.
- **Visualisasi Chart**: Menggunakan komponen resmi **Nuxt UI v4 `<UChart type="donut">`** yang ditenagai oleh `@tanstack/charts`.
- **Fitur Tampilan**:
  - Donut chart alokasi storage berdasarkan tipe file (Code, Images, Videos, Audio, Documents, Archives, Binaries).
  - Progress bar persentase ruang pada setiap subdirektori.
  - Tabel peringkat subdirektori terberat dengan tombol direct "Open".
  - Tabel 30 file terbesar (disk space hogs) dengan tombol "Quick Peek" dan "Reveal in Explorer".
  - Shortcut: `Ctrl+T` atau tombol TreeSize di Command Bar & Status Bar.

---

## 4. Rangkaian Fitur PowerToys yang Telah Diadopsi
1. **Quick Peek (`ExplorerPeekModal.vue`)**:
   - Shortcut `Spacebar` pada file apa pun.
   - Preview instan source code (lengkap dengan line number, syntax, tombol copy), gambar, markdown, dan folder info.
   - Navigasi `ArrowUp` / `ArrowDown` langsung berpindah file preview tanpa harus menutup modal.
2. **PowerRename (`ExplorerPowerRenameModal.vue`)**:
   - Shortcut `Ctrl+M` atau menu konteks.
   - Batch rename regex, format casing (UPPERCASE, lowercase, Title Case, camelCase, kebab-case, snake_case), dan penomoran otomatis dengan preview diff live side-by-side.
3. **Paste as File (`Ctrl+Shift+V`)**:
   - Menempel teks atau gambar screenshot clipboard langsung menjadi berkas baru di direktori aktif (`Pasted_Snippet_...` atau `Pasted_Image_....png`).
4. **PowerToys Run (`ExplorerCommandPalette.vue`)**:
   - Shortcut `Ctrl+P` / `Ctrl+K`.
   - Command launcher bergaya Spotlight untuk berpindah cepat ke drive (C:, D:, E:), quick access folders, aksi explorer, dan pemicu re-index.
5. **Image Resizer & Converter (`ExplorerImageResizerModal.vue`)**:
   - Batch / single resize gambar dengan preset Small (854×480), Medium (1366×768), Large (1920×1080), Mobile (1080×1920), atau Custom W×H.
   - Konversi format WebP, JPEG, PNG dengan pengatur kualitas (lossless / compression).
6. **File Checksum / Hasher (`ExplorerHashModal.vue`)**:
   - Pemeriksa integritas berkas (MD5, SHA-1, SHA-256, SHA-512) dengan 1-klik copy.
   - Komparasi otomatis dengan hash clipboard: menampilkan badge hijau "VERIFIED MATCH" jika cocok atau peringatan merah jika mismatch.
7. **File Locksmith (`file_locksmith`)**:
   - Perintah native Rust untuk memeriksa apakah suatu file sedang dikunci/digunakan oleh proses lain.

---

## 5. Layout & Komponen Nuxt UI `<USplitter>`
- Mengadopsi komponen resmi **Nuxt UI v4 `<USplitter>`** (`@nuxt/ui`) untuk menggantikan pembagian layout manual.
- Mendukung:
  - Panel kiri (Sidebar Quick Access / Drives) yang resizable dan collapsible.
  - Panel utama (File List).
  - Panel sekunder (Dual-Pane Split View) saat mode dua folder diaktifkan (`F6`).
  - Panel kanan (Preview / Details pane).
  - Handle drag halus sesuai standar desain Windows 11 Fluent.

---

## 6. Integrasi Status Git
- **Status Bar**: Menampilkan nama branch aktif (` main`), indikator commit `↑ahead` / `↓behind`, dan total file termodifikasi.
- **Daftar File**: Badge warna dinamis di samping nama file:
  - `M` (Kuning): Modified
  - `A` (Hijau): Added / Staged
  - `U` / `?` (Biru): Untracked
  - `D` (Merah): Deleted

---

## Log 2026-10-18 — Nuxt murni frontend, TreeSize cepat, Explorer parity

Acuan: [PRD.md](PRD.md)

### Arsitektur
- `useExplorerApi.ts` ditulis ulang: semua lewat `invoke`, **fallback `$fetch` ke Nitro dihapus**. Error dari Rust dibungkus menjadi `Error` agar `e.message` di toast terisi.
- Image Resizer sebelumnya memanggil `/api/fs/preview` (Nitro), padahal endpoint itu tidak ada di build produksi. Sekarang memakai `get_file_preview(inline=true)`.
- Command berat (`list_directory`, `file_action`, `get_file_preview`, `fast_file_hash`, `get_git_status`, `file_locksmith`) sekarang `#[tauri::command(async)]`, jadi tidak lagi berjalan di main thread (sebelumnya UI bisa freeze).
- Folder `server/` (Nitro) sudah tidak dipakai. **Belum dihapus**, menunggu konfirmasi karena project ini belum di bawah git.
- `@tanstack/charts` dikeluarkan dari `package.json` (donut diganti CSS). `walkdir` dikeluarkan dari `Cargo.toml`.

### tgrep
- `run_tgrep` dan `build_tgrep_index` memakai **sidecar** (sebelumnya `~/.cargo/bin/tgrep.exe` atau PATH, bisa beda versi dengan daemon).

### TreeSize (bug + kecepatan)
- Bug: `max_depth(5)` membuat ukuran folder yang dalam tidak terhitung. Top-30 file dan kategori hanya dari root. Field `files_count` tidak cocok dengan `filesCount` di frontend (kolom selalu kosong).
- Engine baru `scan_dir`: rekursif, paralel di setiap level, scan penuh, kategori dan top-1000 file dari seluruh tree.
- Cache `TreeCache`: drill-down (klik nama folder) dan tombol **Up** langsung tampil dari memori. **Rescan** memaksa scan ulang.
- Progres file terhitung tampil saat loading (event `treesize-progress`).
- Benchmark `C:\Users\hp`: 701.800 file, 101.923 folder, 49 GB dalam 9,5 detik.

### Explorer parity
- Delete → **Recycle Bin** (`SHFileOperationW` + `FOF_ALLOWUNDO`), Shift+Delete → permanen dengan konfirmasi.
- Copy/Move lewat shell Windows. Bug lama: copy folder gagal (`fs::copy`), move antar-drive gagal (`fs::rename`).
- "New folder (2)" dst. File kosong baru tidak lagi menimpa file yang sudah ada.
- Shortcut: F2, F5/Ctrl+R, Del, Shift+Del, Alt+←/→/↑, Backspace, Ctrl+T (tab baru), Ctrl+W, Ctrl+Shift+N. TreeSize dipindah ke **Alt+T**.

### PowerToys
- **File Locksmith** ditulis ulang dengan Windows Restart Manager. Versi lama hanya mencocokkan teks command line proses, jadi hasilnya tidak akurat.
- **Peek / File Explorer add-ons**: preview PDF, video, audio, dan gambar lewat asset protocol (`protocol-asset` + `assetProtocol.scope`), tanpa base64 lewat IPC.

### Verifikasi
- `cargo check` bersih. `cargo test --lib` lulus (`treesize_scan_and_unique_names`).
- `vue-tsc`: tidak ada error baru. Error `possibly undefined` yang tersisa sudah ada sebelumnya.

### Belum
- Hapus `server/` dan halaman demo Nuxtor (`commands`, `file`, `notifications`, `os`, `store`, `webview`) beserta plugin yang tidak terpakai (perlu konfirmasi).
- New+, OCR, ikon/thumbnail shell asli, rename inline, Ctrl+L, index nama file.

---

## Log 2026-10-18 (lanjutan) — Pembersihan: aplikasi lebih kecil

Disetujui pengguna. Project belum di bawah git, jadi file yang dihapus tidak bisa dipulihkan dari repo.

### Dihapus
- `server/` (Nitro API lama: list, drives, git, hash, preview, quick-access, treesize, search)
- Halaman demo Nuxtor: `commands`, `file`, `notifications`, `os`, `store`, `webview`
- Pendukung demo: `composables/pages.ts`, `components/Site/Navbar.vue`, `components/Design/*`, `components/Layout/Tile.vue`, `layouts/default.vue`, `layouts/home.vue`, `modules/tauri.ts` (auto-import `useTauri*`)
- Plugin Tauri yang tidak dipakai explorer: `fs`, `notification`, `opener`, `os`, `store` (Rust + JS). `@tauri-apps/plugin-shell` (JS) juga dilepas karena sidecar dipanggil dari Rust. Crate `tauri-plugin-shell` tetap.
- `app.config.ts`: blok `app` dan `pageCategories` milik demo.

### Diubah
- Capabilities diperkecil: `main.json` → `core:default`; `desktop.json` → tray + menu. Izin `shell:allow-execute` (`sh -c` apa saja) **dihapus**, sehingga permukaan serangan dari webview berkurang.
- `layouts/blank.vue` (dipakai halaman 404) tidak lagi memuat Navbar.

### Verifikasi
- `bun install`: 7 paket terhapus. `cargo check` bersih. `cargo test --lib` lulus.
- `bun run generate` sukses; `dist/` = 3,3 MB.
- `vue-tsc`: tidak ada error baru.

---

## Log 2026-10-18 (lanjutan) — Index nama file ala Everything

### Yang dibuat
- Modul baru `src-tauri/src/name_index.rs`:
  - Semua nama file di **semua fixed drive** (C:, D:, E:) disimpan dalam dua arena byte (nama asli + huruf kecil ASCII). Per entri hanya offset, induk, dan flag (9 byte), tanpa path utuh.
  - Pencarian: satu pass SIMD `memmem` (crate `memchr`, sudah ada di dependency tree). Beberapa kata = AND (`laporan 2026`). Hasil dibatasi ke folder aktif seperti Explorer.
  - Dibangun otomatis di background saat startup memakai setengah core CPU.
  - Update live per drive lewat `ReadDirectoryChangesW` (windows-sys, tanpa crate baru). File/folder baru langsung masuk. File yang dihapus disaring saat dicari (dicek ke disk, lalu ditandai mati).
- Command `search_names`. Frontend: pencarian nama memakai index ini. Selama index belum siap, atau folder tidak terindeks, otomatis kembali ke tgrep. Pencarian isi file dan regex tetap memakai tgrep.

### Benchmark (laptop pengguna, release)
| | Hasil |
|---|---|
| Entri | 2.760.059 (C:, D:, E:) |
| RAM arena | ~148 MB |
| Build pertama | 315 detik (setengah core, disk dingin) |
| Cari `png` / `readme` | 0,05 ms / 0,6 ms |
| Cari `laporan 2026` (13 hasil) | 8,6 ms |
| Cari yang tidak ada | 5,5 ms (scan seluruh arena) |

### Verifikasi
- `cargo test --lib`: 2 lulus (`search_insert_and_scope`, `treesize_scan_and_unique_names`).
- `vue-tsc`: tidak ada error baru.

### Catatan / batasan
- Index belum disimpan ke disk, jadi **setiap startup membangun ulang** (~5 menit kerja disk di latar).
- RAM ~148 MB untuk 2,76 juta nama, separuhnya untuk arena nama asli.
- Event yang terjadi selama build pertama atau saat buffer watcher overflow belum ditangkap.
- Belum ada wildcard (`*.pdf`), filter `ext:`/`size:`, dan case-insensitive untuk huruf non-ASCII.

---

## Log 2026-10-18 (lanjutan) — Index nama: snapshot ke disk, RAM lebih kecil, MFT

### Perubahan di `name_index.rs`
- **Snapshot ke disk** (`%LOCALAPPDATA%\<identifier>\names.idx`): saat startup index dimuat dari disk lalu langsung siap. Setelah itu dibangun ulang di latar, disimpan, dan ditukar. Penyimpanan atomik (tulis `.tmp` lalu rename).
- **Satu arena huruf kecil** (Unicode `to_lowercase`). Arena nama asli dihapus. Kapitalisasi asli untuk hasil yang ditampilkan diambil dari Windows (`GetFinalPathNameByHandleW`), yang sekaligus memeriksa apakah file masih ada. Bonus: pencarian kini case-insensitive untuk huruf non-ASCII (é, Ä, dst.).
- **MFT** (`FSCTL_ENUM_USN_DATA`): kalau aplikasi jalan sebagai admin, volume NTFS dibaca langsung dari MFT seperti Everything. Kalau tidak (atau volume ReFS/FAT), fallback ke walk.
- Walk fallback jalan di Windows **background mode** (`THREAD_MODE_BACKGROUND_BEGIN`): CPU dan IO prioritas rendah, setengah core.
- Aplikasi **tidak** dipaksa jalan sebagai admin. Kalau elevated, semua file yang dibuka dari aplikasi ikut jalan sebagai admin, dan drag & drop dari Explorer biasa diblokir (UIPI).

### Benchmark mode non-admin (walk), C: + D: + E:
| | Sebelum | Sesudah |
|---|---|---|
| RAM index | ~148 MB | **~96 MB** |
| Siap dipakai saat startup | setelah build (~5 menit) | **0,12 detik** (muat snapshot) |
| Build ulang di latar | 315 detik | 149 detik (prioritas rendah) |
| Snapshot di disk | — | 91 MB, simpan 0,25 detik |
| Cari `png` / `readme` | 0,05 / 0,6 ms | 0,03 / 0,18 ms |
| Cari `laporan 2026` / tidak ada | 8,6 / 5,5 ms | 4,6 / 4,7 ms |

### Verifikasi
- `cargo test --lib`: 2 lulus. Tes index sekarang juga menguji pencarian non-ASCII, pemulihan kapitalisasi lewat Windows, dan round-trip snapshot.
- Benchmark mode MFT menunggu dijalankan pengguna di terminal admin.

### Benchmark mode MFT (admin), C: + D: + E:
| | Walk (non-admin) | MFT (admin) |
|---|---|---|
| C: | 56–120 detik | **12,2 detik** |
| D: | ~1 detik | 0,04 detik |
| E: | 28–32 detik | **10,3 detik** |
| Total build | 89–149 detik | **22,6 detik** |
| Entri | 2.760.482 | 2.687.978 |
| RAM / snapshot | ~96 MB / 91 MB | ~92 MB / 87 MB |
| Muat snapshot | 0,1–0,2 detik | 0,18 detik |
| Cari | 0,1–12 ms | 0,2–10 ms |

- Diagnosa ditambahkan: kalau MFT gagal, log `[names]` menyebutkan alasannya (mis. `os error 5` = bukan admin).
- Selisih jumlah entri di E: (~76 ribu lebih sedikit lewat MFT) kemungkinan besar karena hard link. Enumerasi USN hanya memberi satu nama per file, jadi nama tambahan dari hard link (mis. `node_modules` hasil pnpm/bun) tidak ikut. Lokasi utama file tetap ditemukan. Sudah dicatat di PRD sebagai pekerjaan lanjutan.

---

## Log 2026-10-18 (lanjutan) — Fast Index (helper admin) + fitur PowerToys berikutnya

### Fast Index: MFT tanpa aplikasi harus jalan sebagai admin (`src-tauri/src/fast_index.rs`)
- Mode helper: `<exe> --build-index <file>` membangun index (MFT), menyimpan snapshot, lalu keluar tanpa membuka jendela.
- Scheduled Task "highest privileges" (`<productName> Fast Index`, trigger saat logon) didaftarkan atau dihapus lewat Command Palette → **Toggle Fast Index**. Hanya satu prompt UAC.
- Saat startup: snapshot dimuat (instan), task dijalankan (`schtasks /run`, tanpa UAC), lalu snapshot baru dimuat ulang. Kalau task tidak ada atau gagal dalam 90 detik, index dibangun ulang di dalam aplikasi (walk).
- Siklus hidup index dipecah: `load_snapshot`, `rebuild`, `build_to_file`.
- **Syarat keamanan rilis**: install per-machine (Program Files), karena task menjalankan exe sebagai admin.

### PowerToys / Explorer (`src-tauri/src/tools.rs`)
- **New+**: isi folder `%LOCALAPPDATA%\<identifier>\Templates` muncul di menu **New**, plus "Open templates folder". Nama template divalidasi (tidak bisa keluar folder). Nama hasil mengikuti Explorer ("(2)").
- **Text Extractor (OCR)**: context menu gambar → "Copy Text from Image". Memakai engine OCR bawaan Windows lewat Windows PowerShell 5.1 (`src/ocr.ps1`, dikirim lewat `-EncodedCommand`; path gambar dikirim sebagai env var, bukan disambung ke kode). Sekitar 0,7 detik per gambar, tanpa model atau crate tambahan.
- **Ikon asli Windows** (`SHGetFileInfoW`): komponen `ExplorerFileIcon.vue`, dimuat hanya saat terlihat, cache per tipe (per file untuk exe/lnk/ico/url). Ikon dikodekan sebagai BMP 32-bit + alpha (tanpa encoder PNG).
- **Context menu asli** ("Show more options", Shift+F10): `IContextMenu` lewat crate `windows` 0.62 (sudah ada di tree via Tauri). Batasan: submenu "Send to"/"Open with" masih kosong.
- **Dialog Properties asli** (Alt+Enter dan menu Properties): `SHObjectProperties`. Sebelumnya hanya `alert()`.

### Bug yang ikut diperbaiki
- `Ctrl+Shift+V` (Paste as File) tidak pernah berfungsi: dengan Shift, `e.key` adalah `'V'`, bukan `'v'`.
- `Ctrl+P/K`, `Ctrl+M`, `Ctrl+Shift+V` ditangani dua kali (FileList + global): palette terbuka lalu langsung tertutup, dan Paste as File berpotensi membuat dua file. Duplikat di FileList dihapus.

### Verifikasi
- `cargo test --lib`: 4 lulus (TreeSize, name index, ikon shell 32 px + alpha, OCR end-to-end).
- `vue-tsc`: tidak ada error baru.

---

## Log 2026-10-18 (lanjutan) — Nama Jelajah, ikon, repo baru

- Nama aplikasi: **Jelajah** (`com.narr07.jelajah`, versi 0.1.0), repo <https://github.com/narr07/jelajah>. Nama diganti di `tauri.conf.json`, `Cargo.toml` (lib `jelajah_lib`), `package.json`, `nuxt.config.ts`, dan workflow rilis.
- Installer NSIS di-set **per-machine** (syarat keamanan Fast Index).
- Ikon baru: map folder hijau-teal dengan jarum kompas (`app-icon.svg`). Semua ukuran dibuat dengan `bun tauri icon app-icon.svg`.
- README baru. Dokumen dan gambar template dihapus (`README.zh-CN.md`, `preview.md`, `public/*`). Bundle frontend turun dari 3,3 MB ke 1,5 MB.
- `.claude/` (ter-commit oleh template) dikeluarkan dari repo dan masuk `.gitignore`.
- Riwayat git dimulai bersih (riwayat template dihapus). Kredit template tetap ada di LICENSE dan README. Binary tgrep 1.0.8 ikut di-commit.
- PRD: tabel PowerToys diperbarui dari daftar `src/modules` terbaru (termasuk AltWindowCycle, GrabAndMove, PowerDisplay sebagai modul level sistem).

---

## Log 2026-10-18 (lanjutan) — Peek seperti Quick Look macOS

### Bug: Space tidak menutup Peek
- Penyebab: Space ditangani di tiga tempat. Peek menutup dirinya, lalu event yang sama naik ke handler global di `pages/index.vue`, yang melihat Peek sudah tertutup dan membukanya lagi. FileList juga punya handler Space sendiri.
- Perbaikan: Space hanya ditangani handler global sebagai **toggle** (buka/tutup, seperti Quick Look). Auto-repeat saat tombol ditahan diabaikan. Handler Space di Peek dan FileList dihapus.

### Quick Look: semua file langsung terlihat
- Command baru `file_thumbnail` (`tools.rs`): `IShellItemImageFactory`, mesin thumbnail yang sama dengan Explorer ("Extra large icons"). Menghasilkan halaman pertama PDF/Office, frame video, isi folder, atau ikon 256 px untuk tipe lain. Ukuran mengikuti DPI layar (maks. 1024 px).
- Peek memakai thumbnail ini untuk semua item yang belum punya preview lebih kaya (dokumen Office, arsip, exe, folder, dll.).
- Alpha *premultiplied* dari shell dikonversi ke straight alpha untuk BMP. Dicek visual: halaman pertama PDF dan thumbnail folder warnanya tepat.
- Pembacaan piksel bitmap (`dib_pixels`) dipakai bersama oleh ikon dan thumbnail.

### Bug: tgrep yatim setelah aplikasi dimatikan paksa
- `tauri dev` me-restart aplikasi dengan kill paksa, sehingga `RunEvent::Exit` tidak jalan. Akibatnya `tgrep.exe` tertinggal, mengunci file, dan build berikutnya gagal "Access is denied". Hal sama terjadi kalau aplikasi crash.
- Perbaikan: sidecar dimasukkan ke Job Object `KILL_ON_JOB_CLOSE` (`die_with_us`), sehingga Windows ikut mematikannya begitu Jelajah mati dengan cara apa pun.

### Verifikasi
- `cargo test --lib`: 5 lulus (+ `thumbnails_for_any_file`).
- `vue-tsc`: tidak ada error baru.

---

## Log 2026-10-18 (lanjutan) — tgrep: index lebih ringan, log lebih bersih

- Pesan `tgrep: skipping ... (os error 33)` bukan error. tgrep melewati file yang sedang dikunci aplikasi lain (mis. `.lock` milik Codex). Pesan "skipping" kini tidak lagi diteruskan ke terminal; error lain tetap tampil.
- `tgrep serve` sekarang memakai `--exclude` untuk `AppData`, `node_modules`, `.cargo`, `.rustup`, `.npm`, `.bun`, `.nuget`, `.gradle` (konstanta `TGREP_EXCLUDE` di `lib.rs`). Ini meniru default Windows Search. Folder-folder itu besar, terus berubah, dan hampir tidak pernah dicari isinya. Pencarian **nama** file (name index) tetap mencakup semuanya.
- Diuji: `--exclude` bekerja berdasarkan nama folder di level mana pun.

---

## Log 2026-10-18 (lanjutan) — Ikon baru, scrollbar dark mode

- **Ikon baru** (`app-icon.svg`): tile *squircle* bergradasi zamrud ke teal, dengan mata angin bersegi (efek 3D, cahaya dari kiri atas, ujung utara amber) dan dial tipis. Menggantikan desain map folder yang terlalu mirip folder Windows. Dicek di 256 px dan 32 px, lalu semua ukuran dibuat ulang dengan `bun tauri icon app-icon.svg`.
- **Scrollbar terang di dark mode**: class `no-scrollbar` dipakai di bar tab dan breadcrumb tapi tidak pernah didefinisikan, jadi scrollbar bawaan (terang) muncul di sebelah tombol dark mode. Class itu sekarang didefinisikan di `main.css`.
- `color-scheme` sekarang mengikuti tema (`html.dark` → `dark`), sehingga scrollbar dan kontrol bawaan lain ikut gelap di dark mode.
- tgrep setelah `--exclude`: index isi file **60.328 file, 809 MB** (sebelumnya 2,67 GB termasuk `AppData`).

---

## Log 2026-10-18 (lanjutan) — Office preview, Open with, terminal, tombol mouse

- **Open in Terminal tidak jalan**: `powershell.exe` dijalankan tanpa console sendiri, sehingga saat dev ia menumpang di console `tauri dev` dan tidak terlihat. Path juga disambung ke `-Command` (folder dengan `'` bisa merusak perintah). Sekarang seperti Explorer: buka **Windows Terminal** (`wt -d <folder>`); kalau tidak ada, jendela PowerShell baru (`CREATE_NEW_CONSOLE`) dengan folder sebagai working directory.
- **Tombol back/forward mouse** (XButton1/2) → Back/Forward, seperti explorer.exe. Navigasi history bawaan webview dibatalkan.
- **Preview Office di Peek** (`preview_open/move/close`): memakai *preview handler* Windows (`IPreviewHandler`), komponen yang sama dengan preview pane Explorer. Handler dijalankan out-of-process (prevhost.exe) dan menggambar ke child window native di atas area Peek, mengikuti resize. Di PC pengguna tersedia handler untuk xlsx, docx, pptx, dan pdf. Urutan fallback: preview handler → thumbnail Explorer → ikon. Fokus dikembalikan ke webview setelah handler tampil, supaya Space/Esc tetap menutup Peek.
- **Open with** di context menu: daftar aplikasi yang direkomendasikan Windows (`SHAssocEnumHandlers`) lengkap dengan ikon, plus "Choose another app" (dialog asli `SHOpenWithDialog`). Ini juga menggantikan submenu "Open with" yang kosong di "Show more options".

### Verifikasi
- `cargo test --lib`: 7 lulus (+ daftar Open with untuk `.txt` berisi Notepad, deteksi preview handler xlsx/docx/pptx/pdf).
- `vue-tsc`: tidak ada error baru.
- Belum diuji di aplikasi yang berjalan: tampilan preview handler, posisi child window, dan fokus keyboard.

---

## Log 2026-10-18 (lanjutan) — Build rilis + website

### Build rilis 0.1.0 (`bun tauri build`)
- `src-tauri/target/release/jelajah.exe`: 5,8 MB
- `bundle/nsis/Jelajah_0.1.0_x64-setup.exe`: **3,9 MB** (per-machine)
- `bundle/msi/Jelajah_0.1.0_x64_en-US.msi`: 5,5 MB
- Disk E: sempat tinggal 175 MB. `target/debug` dan `target/check` (cache build, total ~19 GB bersama release) dihapus dulu supaya build muat. Cache ini bisa dibuat ulang.

### Website (`website/`)
- Landing page satu halaman berbasis **Nuxt UI SaaS template** (MIT, nuxt-ui-templates/saas). Nuxt Content, og-image, auth, pricing, blog, dan docs dibuang; yang dipakai hanya `@nuxt/ui`.
- Tema mengikuti ikon: primary `emerald`, secondary `amber`, latar dark mode teal gelap (`#06110f`). Logo dan favicon memakai ikon aplikasi.
- Isi: hero + mockup jendela Jelajah (HTML, bukan screenshot), angka performa, tiga bagian fitur (Search, Explorer parity, TreeSize) dengan ilustrasi, grid 9 alat PowerToys, **Docs** (3 langkah mulai + 14 shortcut keyboard), dan **Download** (CTA ke GitHub Releases).
- Statis: `cd website && bun run generate` → `.output/public` (1,3 MB). Untuk GitHub Pages di `/jelajah/`: `NUXT_APP_BASE_URL=/jelajah/`. Path aset lewat `useAsset()` supaya base URL dihormati.
- Root `nuxt.config.ts` mengabaikan `website/**`.
- Dicek visual dengan Edge headless, mode terang dan gelap.
- Catatan: tombol Download menuju `releases/latest`, yang belum ada sampai sebuah GitHub Release dibuat.

### Rilis dan deploy (2026-10-18)
- **GitHub Release v0.1.0**: <https://github.com/narr07/jelajah/releases/tag/v0.1.0> (setup.exe 3,9 MB + MSI 5,5 MB). Installer belum ditandatangani, jadi SmartScreen akan memberi peringatan.
- **Website live**: <https://narr07.github.io/jelajah/>, deploy otomatis lewat `.github/workflows/website.yml` setiap ada perubahan di `website/`.
- Deploy pertama gagal karena `website/` belum punya `tsconfig.json`, sehingga build memakai tsconfig aplikasi utama yang merujuk ke `.nuxt` yang tidak ada di CI. Diperbaiki dengan `website/tsconfig.json` sendiri.
- Catatan uji lokal: Git Bash mengubah `NUXT_APP_BASE_URL=/jelajah/` menjadi `C:/Program Files/Git/jelajah/` (konversi path MSYS). Untuk uji lokal pakai `MSYS_NO_PATHCONV=1` atau PowerShell.
- Pembersihan: installer disalin ke `%USERPROFILE%\Downloads\Jelajah-0.1.0\`, lalu `src-tauri/target`, `node_modules`, `.nuxt`, dan `.output` (root dan website) dihapus. Build ulang: `bun install` lalu `bun tauri build`.
