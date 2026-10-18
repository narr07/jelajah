# Jelajah

**Jelajah** (Indonesian for *explore*) is a fast, lightweight file explorer for Windows. It looks and behaves like `explorer.exe`, and adds the file tools from Microsoft PowerToys, a TreeSize-style disk analyzer and Everything-style instant search.

Built with [Tauri 2](https://tauri.app) and Rust for all file work; [Nuxt 4](https://nuxt.com) + [Nuxt UI](https://ui.nuxt.com) render the interface only.

## Features

**Explorer, the way Windows does it**
- Tabs, dual pane (F6), navigation pane, details/preview pane
- Delete goes to the Recycle Bin (Shift+Delete for permanent); copy/move use the Windows shell, so you get its native progress and conflict dialogs
- Windows' own file icons, the real Properties dialog (Alt+Enter) and the full Explorer context menu ("Show more options", Shift+F10)
- Familiar shortcuts: F2, F5, Ctrl+T/W, Alt+←/→/↑, Ctrl+Shift+N

**Instant search**
- Filename index over every fixed drive, searched in milliseconds (2.7M names in ~0.2–10 ms), kept live with `ReadDirectoryChangesW`
- Snapshot on disk, so search works ~0.1 s after launch
- Optional **Fast Index**: an elevated helper reads the NTFS MFT like Everything does (about 23 s for 2.7M names); the app itself never runs as admin
- File contents search via [tgrep](https://crates.io/crates/tgrep) (trigram index)

**PowerToys file tools**
- Peek (Space): code, images, Markdown, PDF, video, audio
- PowerRename (Ctrl+M), Image Resizer, File Locksmith, New+ templates
- Text Extractor: copy the text out of any image (Windows OCR)
- Command Palette (Ctrl+P), Paste as File (Ctrl+Shift+V)

**TreeSize**
- Full-depth parallel scan (700k files in ~9.5 s), size by file type, largest files
- Results stay in memory: drill into any subfolder instantly

See [PRD.md](PRD.md) for the full feature status and roadmap.

## Development

Requirements: Windows 10/11, [Rust](https://rustup.rs), [Bun](https://bun.sh).

```bash
bun install
bun tauri:dev        # run the app
bun tauri:build      # installer (per-machine, required by Fast Index)
```

The tgrep sidecar is expected at `src-tauri/binaries/tgrep-x86_64-pc-windows-msvc.exe`.

```bash
cd src-tauri
cargo test --lib                                                   # unit tests
cargo test --release --lib bench_name_index -- --ignored --nocapture  # index benchmark (run elevated for MFT)
```

## License

MIT. Started from the [Nuxtor](https://github.com/NicolaSpadari/nuxtor) template by Nicola Spadari.
