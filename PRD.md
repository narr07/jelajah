# PRD — Jelajah

**Versi dokumen**: 1.0 · 2026-10-18
**Platform**: Windows 10/11 (x64) · Tauri 2 + Nuxt 4 + Rust

## 1. Tujuan

File manager Windows yang **cepat, ringan, dan kecil**, dengan tampilan dan perilaku sedekat mungkin dengan `C:\Windows\explorer.exe`, plus fitur-fitur PowerToys yang berhubungan dengan file, dan analisis disk ala TreeSize.

## 2. Prinsip arsitektur

| Aturan | Alasan |
|---|---|
| Nuxt = **UI saja** (`ssr: false`, `nuxt generate`). Tidak ada `server/` Nitro, tidak ada `$fetch`. | Satu jalur IPC. Build statis kecil. |
| Semua IO, hashing, scan, dan proses eksternal ada di **Rust** (`src-tauri/src/lib.rs`). | Kecepatan native, UI tidak ikut kerja berat. |
| Command berat harus `async` atau `#[tauri::command(async)]`. | Command sync berjalan di main thread dan membekukan jendela. |
| Operasi file memakai **API shell Windows** (`SHFileOperationW`, Restart Manager). | Perilaku sama dengan Explorer tanpa kode atau crate tambahan. |
| File media dikirim lewat **asset protocol** (`convertFileSrc`), bukan base64. | Tanpa salinan memori, gambar/video besar tetap ringan. |
| Dependency baru hanya kalau API OS atau crate yang sudah ada tidak cukup. | Binary dan bundle tetap kecil. |

## 3. Kesetaraan dengan explorer.exe

| Perilaku | Status |
|---|---|
| Delete → Recycle Bin (tanpa prompt), Shift+Delete → permanen (dengan konfirmasi) | ✅ |
| Copy/Move: dialog progres dan konflik bawaan Windows, bisa folder, bisa lintas drive, "X - Copy" | ✅ |
| Nama baru "New folder", "New folder (2)", … | ✅ |
| Shortcut F2, F5/Ctrl+R, Del, Shift+Del, Alt+←/→/↑, Backspace, Ctrl+T/W, Ctrl+Shift+N, Ctrl+A | ✅ |
| Ctrl+L / Alt+D fokus ke address bar, F3/Ctrl+F fokus ke search | ⏳ |
| Rename inline di list (bukan `prompt()`) | ⏳ |
| Ikon file asli dari shell (SHGetFileInfo) | ✅ (thumbnail gambar ⏳) |
| Context menu shell asli ("Show more options", Shift+F10) | ✅ (submenu Send to/Open with ⏳) |
| Dialog Properties asli Windows (Alt+Enter) | ✅ |
| Drag & drop ke/dari aplikasi lain | ⏳ |
| Tab, panel navigasi, preview pane, details pane | ✅ |

## 4. Modul PowerToys

Sumber: daftar `src/modules` di <https://github.com/microsoft/PowerToys> (dicek 2026-10-18). Prinsipnya: semua modul yang **berhubungan dengan file** diadopsi. Modul yang bekerja di level sistem (jendela, mouse, keyboard, layar) tidak, karena butuh hook global dan proses latar yang bertentangan dengan target ringan.

### Berhubungan dengan file

| Modul | Di Jelajah | Status |
|---|---|---|
| Peek | Quick Peek (Space): kode, gambar, markdown, PDF, video, audio | ✅ |
| File Explorer add-ons (`previewpane`) | Preview pane: gambar, PDF, video, audio, teks | ✅ (thumbnail di daftar file, STL/G-code/QOI ⏳) |
| PowerRename | Ctrl+M: regex, casing, penomoran, preview | ✅ |
| Image Resizer | Preset dan custom, WebP/JPEG/PNG | ✅ |
| File Locksmith | Restart Manager: nama proses + PID yang mengunci file | ✅ (folder ⏳) |
| New+ | Template file/folder di menu New | ✅ |
| Text Extractor (`PowerOCR`) | "Copy Text from Image" di context menu | ✅ (berbasis file; tangkap area layar ❌) |
| PowerToys Run (`launcher`) / Command Palette (`cmdpal`) | Ctrl+P / Ctrl+K di dalam Jelajah | ✅ |
| Advanced Paste | Paste as File (Ctrl+Shift+V) | ✅ sebagian (paste dengan AI ❌, butuh API key) |
| Registry Preview | Preview `.reg` terstruktur | ⏳ |
| Hosts File Editor | Edit `hosts` dengan elevasi | ⏳ (prioritas rendah) |
| Workspaces | Simpan/pulihkan set tab | ⏳ (prioritas rendah) |
| Always On Top | Toggle untuk jendela Jelajah | ⏳ (prioritas rendah) |

### Level sistem: sengaja tidak diadopsi

FancyZones, Keyboard Manager, Mouse Utilities, Mouse Without Borders, ZoomIt, Color Picker, Screen Ruler (`MeasureTool`), Quick Accent (`poweraccent`), Shortcut Guide, Awake, Crop And Lock, Light Switch, Environment Variables, Command Not Found, Alt Window Cycle, Grab And Move, PowerDisplay.

## 5. Pencarian (tgrep)

- `tgrep serve <home>` berjalan sebagai **sidecar** sejak startup dan dimatikan saat keluar.
- Semua pemanggilan (`run_tgrep`, `search`, `search_status`, `build_tgrep_index`) memakai binary sidecar yang sama.
- Pencarian isi file memakai index tgrep. Pencarian nama memakai `tgrep --files -g`, yang juga menemukan file biner.
- ✅ **Index nama sendiri (gaya Everything)** di `src-tauri/src/name_index.rs`: semua fixed drive diindeks otomatis saat startup (setengah core CPU), lalu diperbarui live lewat `ReadDirectoryChangesW`. Pencarian nama memakai index ini; tgrep hanya dipakai sebagai cadangan selama index dibangun dan untuk pencarian isi file / regex.
- ✅ Snapshot index di disk (startup siap dalam ~0,1 detik), arena huruf kecil saja (~96 MB untuk 2,76 juta nama), case-insensitive Unicode, dan baca MFT NTFS saat aplikasi jalan sebagai admin.
- ⏳ Mode MFT hanya menyimpan satu nama per file: nama tambahan dari hard link (mis. `node_modules` pnpm/bun) belum ikut. Perlu parsing atribut `$FILE_NAME` dari MFT mentah.
- ✅ **Fast Index**: Scheduled Task "highest privileges" menjalankan `<exe> --build-index <file>` (MFT, ~23 detik untuk 2,7 juta nama) lalu keluar. Didaftarkan sekali lewat Command Palette (satu prompt UAC). Aplikasi utama tetap jalan tanpa admin.
- ⚠️ Syarat rilis: install **per-machine** (Program Files). Task menjalankan exe sebagai admin, jadi exe tidak boleh bisa ditulis user biasa.
- ⏳ USN journal untuk menangkap perubahan saat aplikasi tertutup; setting task agar tetap jalan saat baterai.
- ⏳ Wildcard (`*.pdf`), filter `ext:`/`size:`, dan kompaksi arena untuk uptime panjang.
- ⏳ Indikator "Mengindeks…" berdasarkan output `search_status`.

## 6. TreeSize

- Scan **penuh tanpa batas kedalaman**, paralel di setiap level folder (rayon work-stealing). Symlink dan junction dilewati.
- Hasil di-cache di memori Rust, jadi drill-down ke subfolder dan tombol Up di dalam hasil scan langsung tampil tanpa scan ulang.
- Progres `treesize-progress` (jumlah file) dikirim setiap 150 ms.
- Donut kategori memakai CSS `conic-gradient`, tanpa library chart.
- Benchmark: `C:\Users\hp`, 701.800 file / 101.923 folder / 49 GB → **9,5 detik** (release).
- ⏳ Ukuran terpakai di disk (allocated size), export CSV, tampilan tree yang bisa di-expand, baca MFT NTFS (gaya WizTree, perlu admin).

## 7. Target non-fungsional

| Metrik | Target |
|---|---|
| Buka folder 10.000 item | < 100 ms sampai tampil |
| UI tidak pernah freeze saat scan, hash, atau search | wajib |
| Ukuran installer | < 15 MB |
| RAM idle | < 150 MB |

## 8. Cara verifikasi

```bash
cd src-tauri
cargo test --lib                                        # tes scan TreeSize + penamaan unik
cargo test --release --lib bench_scan -- --ignored --nocapture   # benchmark (TREESIZE_DIR=C:\ opsional)
```
