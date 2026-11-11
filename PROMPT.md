# MASTER PROMPT — NevIxplore (Rust file manager for Windows)

> **How to use this document.** Paste this entire file into an AI coding agent (plan mode
> first, then execution). The agent must produce a **plan** from it, get approval, and then
> implement. Everything below is normative: where a decision is marked **[LOCKED]**, do not
> re-litigate it; where it is marked **[DECIDE]**, make the choice and record it in
> `docs/adr/` with a one-paragraph rationale.

---

## 0. ROLE, SCOPE AND HARD CONSTRAINTS

You are a senior Rust engineer and product-minded developer. You will build **NevIxplore**, a
fast, beautiful, Windows-only file manager — a spiritual successor to Windows Explorer that
fixes its four worst sins: *slow on huge directories, bad search, no preview, and pushed
ads/OneDrive nagging.*

### 0.1 Non-negotiable constraints

1. **Windows 10 (21H2+) and Windows 11 only.** x86_64. ARM64 is out of scope for v1.0 — but
   the code must not *block* an ARM64 build.
2. **Nothing is installed on the developer's machine.** No Rust toolchain, no Node, no
   WebView2 bootstrapper downloaded locally. **All compilation, linting, testing and packaging
   happen in GitHub Actions.** The agent produces source code, not build artifacts. If a build
   is needed to verify, it must be verified *in CI*, not locally.
3. **The repository is `https://github.com/neverjoskiy/nevixplore`.** It already exists and is
   empty (`main`, 0 bytes). All work is committed there.
4. **Do not write code comments.** No `//` explanatory comments. `///` doc comments on public
   API items are allowed and encouraged (that is documentation, not commentary). Rationale:
   the code must be self-describing; comments that explain *what* rot, comments that explain
   *why* belong in `docs/adr/`.
5. **Documentation, commit messages, and all user-facing strings are English.** The product's
   UI language is English. Localization scaffolding may exist, but English is the only shipped
   locale for v1.0.
6. **The user does not write code.** They review plans and PRs. Therefore every prompt-facing
   artifact (this doc, `docs/`, CI) must be complete enough that an agent can work from it
   without asking questions.

### 0.2 The four product pillars (in priority order)

| # | Pillar | What it means concretely |
|---|---|---|
| P1 | **Fast** | Opening a 200 000-entry directory must render the first screen in **< 250 ms**, and scroll at 60 fps. Achieved by: virtualized list, incremental `ReadDirectoryChangesW`, lazy metadata, no full-directory scan on open. |
| P2 | **Beautiful** | A calm, minimal, Fluent-inspired design that feels native on Win11 and intentional on Win10. Dark/light/system theme. Zero gratuitous animation -- motion is only used to convey state (progress, drop target). |
| P3 | **More useful than Explorer** | Tabs, split view, fuzzy jump, instant filename search, rich metadata panel, previews, batch rename, folder diff. |
| P4 | **Extensible** | A Lua plugin API (mlua, sandboxed) and a custom context-menu system, shipped from v0.2. |

**[LOCKED] Tech stack: Tauri 2.x** (`tauri = "2"`, `tauri-build`, `@tauri-apps/cli` v2).
Rust backend, TypeScript + React + Vite frontend. Rationale: Tauri gives a native window,
native menus, native file dialogs, native drag-and-drop, tiny binaries (~10 MB), and a CSS
engine for the "beautiful" pillar -- while keeping heavy I/O in Rust threads where the "fast"
pillar lives. WebView2 ships with Windows 10/11.

> **Note on Tauri 3:** `tauri-v3.0.0-alpha.*` exists (Nov 2026). Do **not** use alpha software
> for this project. Pin to the latest stable **2.x** line. If, at implementation time, 3.0 is
> stable, record an ADR and migrate -- but never ship an alpha.

---

## 1. PRODUCT VISION

### 1.1 One sentence
NevIxplore is a Windows file manager that opens any directory instantly, finds any file by name
as you type, shows you the metadata you actually need, and gets out of your way.

### 1.2 Who it is for
The user themselves first. Secondarily: developers, archivists, and anyone who manages large
media or code collections on Windows and has given up on Explorer's speed.

### 1.3 Explicit anti-goals (things we will NOT do)

- No cloud sync, no account, no telemetry, no "sign in".
- No OneDrive promotion, no "recommended" section, no ads of any kind.
- No AI features in v1.0. The plugin API can grow this later; a hard-coded AI integration cannot.
- No cross-platform promise. Linux/macOS builds are not supported.
- No animation for animation's sake. Every transition must earn its milliseconds.
- No rewriting of user files without explicit confirmation. Destructive operations are
  always confirmed, always logged, always undoable where the OS permits.
- No kernel drivers, no shell extensions, no injection into `explorer.exe`.
- No bundling of proprietary codecs. Previews of HEVC/ProRes depend on OS codecs.

### 1.4 What "done" looks like for v1.0
A user can install NevIxplore, browse their whole disk with tabs and split view, find any file
by typing part of its name, preview images/PDFs/text/Markdown, inspect rich metadata, batch
rename files, diff two folders, open an archive without extracting it, and empty their trash
on a schedule. It never shows an ad. It never asks them to sign in.

---

## 2. FUNCTIONAL REQUIREMENTS

Priority: **M** = must (blocks release), **S** = should, **C** = could (cut first).

### 2.1 M0 - Walking skeleton

| ID | Req | Pri |
|---|---|---|
| M0-1 | Repo builds in CI on Windows: cargo check, cargo clippy -D warnings, cargo test, npm run build, tauri build. | M |
| M0-2 | App window opens with a title bar, an address bar, and a file list pane. | M |
| M0-3 | App navigates to a directory passed on the command line and to one chosen via a native folder picker. | M |
| M0-4 | File list renders entries (name, icon placeholder, size, modified date) from a directory read. | M |
| M0-5 | Dark/light/system theme resolves correctly and persists across restarts. | M |
| M0-6 | App icon, product name, version metadata and publisher are set in the installer. | M |
| M0-7 | CI produces a portable .exe and an NSIS installer on every push to main, and attaches them to a GitHub Release on a v* tag. | M |

### 2.2 M1 - Browsing that is actually fast (the core value)

| ID | Req | Pri |
|---|---|---|
| M1-1 | **Virtualized list.** Only rows in (or near) the viewport exist in the DOM. A 500 000-entry directory scrolls at 60 fps with no layout thrash. | M |
| M1-2 | **Non-blocking open.** Opening a directory returns control to the UI within one frame; entries stream in and the list fills progressively. A spinner appears only in the status bar, never over the content. | M |
| M1-3 | **Incremental change tracking.** The directory is watched via ReadDirectoryChangesW (notify, debounced). Created/deleted/renamed entries update the list without a full re-read. | M |
| M1-4 | **Lazy metadata.** Name, size, kind and icon come from directory enumeration. Expensive metadata (EXIF, duration, line counts, hashes) is fetched on demand, off the UI thread, per-column toggle. | M |
| M1-5 | **Column sorting** by name, size, modified, type, and any lazy metadata column. Stable sort, remembers direction per column. | M |
| M1-6 | **Column resizing and reordering**, per-view, persisted in settings. | S |
| M1-7 | **Multiple views:** Details (table), List (compact), Grid (large icons). View choice is per-location. | S |
| M1-8 | **Path autocomplete** in the address bar: typing "\Dow" suggests "Downloads". Sourced from the on-disk index when available, else live reads. | S |
| M1-9 | **Breadcrumb** address bar with per-segment dropdown (siblings at each level). | S |
| M1-10 | **Navigation history:** back, forward, up, "recent locations". Per-window, in-memory, **not** persisted. | M |
| M1-11 | **Keyboard-first navigation:** arrows, Home/End, PageUp/PageDown, Enter open, Backspace up, Alt+Left/Right, Ctrl+L address bar, F2 rename, Delete/Shift+Delete, Ctrl+C/V/X, Ctrl+A, Ctrl+Shift+N, F5 refresh. | M |
| M1-12 | **Hidden/system files are shown by default** with a distinct "hidden" badge. A toolbar toggle hides them. | M |
| M1-13 | **Long paths** (> 260 chars) work everywhere: the extended-length prefix is applied internally for all Win32 calls that need it. | M |
| M1-14 | **Files in use:** locked files list normally; open/rename/delete produce a clear, actionable error ("file is used by <process>" where obtainable). | S |
| M1-15 | **Elevation:** operations requiring admin (e.g. writing to C:\Windows) surface a clear message and offer "restart as administrator". No silent UAC prompts. | S |

### 2.3 M2 - Search and navigation

| ID | Req | Pri |
|---|---|---|
| M2-1 | **Instant filename search** (fuzzy, subsequence, ranked). Typing repor finds reports, Q3_report.docx, ReportTemplate.xls. Debounced ~40 ms. The match is **highlighted in place** in the row (the matched characters are underlined/bolded), so it is obvious *why* a row matched. | M |
| M2-2 | **Search scope** dropdown: This folder / This folder + subfolders / Indexed locations. Default: This folder + subfolders. | M |
| M2-3 | **Background index** in %LOCALAPPDATA%\NevIxplore\index\ (SQLite). Indexes **file names only**, never file contents. Incremental, resumable, cancellable. | M |
| M2-4 | **Index is opt-in per root.** User adds a root; app shows size, entry count, last build, and rebuild. Default roots: Desktop, Documents, Downloads. | M |
| M2-5 | **Content search (grep)** scoped to the **current folder only**, with hard limits: max file size (default 8 MiB), max files (default 50 000), confirmation dialog when limits would be exceeded. Binary files skipped unless opted in. | M |
| M2-6 | **Dual highlighting is required.** A filename hit highlights the *name* in the file list. A content hit highlights **every matching line inside the file**: the file row gets a "content match" badge with the number of hits, the row expands into a snippet block listing the matching lines with line numbers, and the matched substring inside each snippet is highlighted. Opening the snippet's preview (or the file) shows the same lines highlighted in context. | M |
| M2-7 | **Snippet blocks are virtualized** like the list: a file with 5 000 hits does not render 5 000 nodes; it renders the first 50 with a "show all" affordance. | M |
| M2-8 | **Search result grouping:** files-first, then folders, then content hits, each group collapsible, with a count in the group header. Sorting within a group is selectable (relevance / name / mtime). | S |
| M2-9 | **Jump-to-path:** Ctrl+P palette; typing a path fragment and pressing Enter navigates there. | S |
| M2-10 | **Everything-style filters:** ext:, size:>10mb, modified:<7d, path:. | C |
| M2-11 | **Regex toggle** for content search (off by default); when on, invalid patterns are reported inline without leaving the search box. Case sensitivity and whole-word toggles for both modes. | S |

### 2.4 M3 - Files and operations

| ID | Req | Pri |
|---|---|---|
| M3-1 | **Open** files with the default app (ShellExecuteW), folders in a new tab, archives inside NevIxplore. | M |
| M3-2 | **Rename** (inline, F2), **delete to Recycle Bin** (Delete), **permanent delete** (Shift+Delete, always confirmed). | M |
| M3-3 | **Copy / move / cut-paste** with a progress dialog: current file, total, speed, ETA, cancel, per-conflict choice (overwrite / skip / keep both / apply to all). | M |
| M3-4 | **Drag and drop:** inside the app (move/copy with modifier), from Explorer into the app, from the app into Explorer. Drop target highlighted; cursor reflects the operation. | M |
| M3-5 | **Multi-select:** click, Ctrl+click, Shift+click, marquee in Grid view, Ctrl+A. | M |
| M3-6 | **New folder** (Ctrl+Shift+N), **new file** (from template list), inline name editing. | M |
| M3-7 | **Properties dialog:** metadata-rich panel (see M4). | M |
| M3-8 | **Custom context menu** (right-click): declarative command list built in Rust with icons, separators, submenus, enable/disable state. The OS context menu is not used (slow, inconsistent). | M |
| M3-9 | **Batch rename:** pattern (prefix/suffix/find-replace/regex/numbering/case), live old-to-new preview, validation (no duplicates, no reserved names, no illegal characters). | S |
| M3-10 | **Batch metadata:** set modified/created date on a selection. | C |
| M3-11 | **Undo:** single-level undo for the last file operation (move/copy/rename/delete-to-bin) where reversible. Not offered for permanent delete. | S |

### 2.5 M4 - Metadata and preview

| ID | Req | Pri |
|---|---|---|
| M4-1 | **Metadata panel** (right side, toggleable): name, type, size, **exact** byte count, modified/created/accessed, attributes, owner, path, and a **checksum** (Blake3, on demand). | M |
| M4-2 | **Image metadata:** dimensions, color space, bit depth, EXIF (camera, lens, exposure, ISO, GPS), EXIF thumbnail. | M |
| M4-3 | **Audio metadata:** title/artist/album/track/year/genre/duration/bitrate/sample rate, plus embedded cover art. | M |
| M4-4 | **Video metadata:** container, duration, resolution, frame rate, codec (OS shell properties where possible). | S |
| M4-5 | **Document metadata:** page/slide count for PDF/DOCX/PPTX, author, title, creation date. | S |
| M4-6 | **Text metadata:** line count, character count, encoding (BOM detection), line-ending style, longest line. Lazily computed; files above a configurable size (default 5 MiB) are skipped with a note. | S |
| M4-7 | **Preview pane** (toggleable), rendered off the UI thread, **off by default for directories above a configurable entry count**. Supported: images (jpg/png/gif/webp/bmp/svg/tiff), video (first frame + play-in-place), PDF (first page), text/code (syntax-highlighted, first N lines), Markdown (rendered), HTML (sandboxed iframe, no network), audio (player). Can be **turned off entirely** in settings. | M |
| M4-8 | Preview rendering must be **cancellable** and **bounded** (max pixels decoded, max file size, timeout) so a 200 MB TIFF cannot hang the app. | M |

### 2.6 M5 - Archives

| ID | Req | Pri |
|---|---|---|
| M5-1 | **Read** ZIP, 7z, TAR, GZ, TAR.GZ, TAR.XZ, TAR.ZST, RAR (read-only) **in place**: an archive opens as a virtual folder in a new tab. | M |
| M5-2 | **Extract** selected entries, with progress and conflict handling identical to file copy. | M |
| M5-3 | **Create** ZIP (deflate) and 7z (LZMA2) from a selection, with optional password. | S |
| M5-4 | **Archive corruption** is reported clearly ("7z: unexpected end of data at offset 0x1A3F00"), never as a hang. | M |
| M5-5 | **Nested archives** open recursively, depth-limited to 8. | C |

### 2.7 M6 - Disk and folder insight

| ID | Req | Pri |
|---|---|---|
| M6-1 | **Folder diff:** compare two directories and classify each entry as identical / modified / only-left / only-right, by size and mtime, with optional content hash. | M |
| M6-2 | **Git-aware diff:** if a folder contains a .git, annotate each entry with M/A/D/U/! read via git2, and offer a "show only changed" filter. | S |
| M6-3 | **Disk usage view:** treemap (and/or sunburst) of a chosen root, sized by actual size, colored by depth, click-to-navigate. Computed in a worker with progress. Skips reparse points by default. | S |
| M6-4 | **Duplicate finder:** group by (size, then Blake3), show groups, allow bulk selection of all-but-one. | C |

### 2.8 M7 - Trash, terminal, extensibility

| ID | Req | Pri |
|---|---|---|
| M7-1 | **Trash view:** virtual folder showing Recycle Bin contents with original location and deletion date; restore to original or chosen folder; empty on a schedule (never/1d/7d/30d). | S |
| M7-2 | **Integrated terminal:** a pane running the user's shell (pwsh/powershell/cmd/wsl), opened at the current directory, with "new terminal here". **Off by default**; spawning a shell is always explicit. | C |
| M7-3 | **Plugin API (Lua):** plugins are Lua 5.4 scripts run by **mlua** (vendored, via the `lua54` feature) in a **sandboxed** interpreter. Loaded from %APPDATA%\NevIxplore\plugins\*.lua. Shipped with v0.2+; the API is frozen at v0.3. | S |
| M7-4 | **Plugin manifest** (`nevixplore-plugin.toml`): name, version, entrypoint, min_host_version, permissions, contributed menu items / columns / previewers. | S |
| M7-5 | **Plugin sandbox (mandatory):** no `os.execute`, `io`, `package.loadlib`, `require` of host modules, `dofile`, `loadfile` unless the plugin declared a permission and the user approved it at install. The sandbox is installed by removing globals and installing a read-only metatable on `_G`; wall-clock and instruction-count limits per call (default 50 ms / 10 M instructions) abort a runaway plugin. | S |
| M7-6 | **Plugin host API** (the *only* way in): `nv.log(level, msg)`, `nv.fs.list(path, opts)`, `nv.fs.stat(path)`, `nv.entry.icon(path, size)`, `nv.ui.notify(msg)`, `nv.ui.menu.register({id, label, icon, when})`, `nv.ui.column.register({id, label, compute(entry) -> any})`, `nv.ui.preview.register({exts, render(path) -> image|html|text})`. All host calls are queued to a worker thread; a plugin can never block the UI thread. | S |
| M7-7 | **Plugin lifecycle:** enable/disable per plugin, version compatibility check against the host API version, and a visible error banner in the status bar (never a modal) when a plugin throws. A broken plugin cannot prevent startup. | S |
| M7-8 | **Plugin example shipped in-repo** (`examples/plugins/hello.lua`): registers a context-menu item that reports the selected entries' total size, plus a column that shows the file's first line. | S |
| M7-9 | **Third-party plugin install** is a file copy into the plugins dir with a user confirmation listing exactly what the plugin is allowed to do. No plugin is ever loaded from the network automatically. | S |

**[LOCKED] Plugin runtime = Lua via mlua.** Rejected alternatives, for the record: WASM/wasmtime (heavier host API surface, awkward for UI callbacks, no user ecosystem), and a Rust dylib (unsandboxed, unsound, Windows DLL hell). Lua is small, embeddable, universally understood for plugin scripting, and mlua with the `lua54` feature vendors the interpreter — so the build needs no system Lua.

### 2.9 Non-functional requirements (apply to every milestone)

| ID | Requirement |
|---|---|
| NFR-1 | **Performance budget:** cold start to interactive window **< 1.2 s**; first screen of a 200 k-entry directory **< 250 ms**; steady-state memory **< 180 MB** with 100 k entries open; no UI stall **> 50 ms** while browsing. |
| NFR-2 | **No data loss.** Every destructive operation is reversible or preceded by an explicit confirmation naming count and total size. |
| NFR-3 | **No silent failure.** Every error is a typed error with a user-readable message and a "details" expander with the OS code and path. |
| NFR-4 | **Crash safety.** A panic in any worker thread must not kill the UI. Panics are caught at the thread boundary, logged, and surfaced in the status bar. |
| NFR-5 | **Security.** No network access except the optional update check. No shell interpolation of user paths. All paths pass as argv arrays, never a concatenated string. Archive extraction rejects path traversal and absolute paths. |
| NFR-6 | **Privacy.** No telemetry, no analytics. The only outbound request is a user-initiated "check for updates". |
| NFR-7 | **Accessibility.** Full keyboard operation; visible focus ring; prefers-reduced-motion disables non-essential motion; WCAG AA contrast in both themes. |
| NFR-8 | **Observability.** Structured JSON logs to %LOCALAPPDATA%\NevIxplore\logs\, daily rotation, 7-day retention, "reveal log folder" in settings. |

---

## 3. UX AND DESIGN SYSTEM

### 3.1 Visual direction
**Calm, minimal, Fluent-inspired, but our own.** Not a Windows 11 clone. Design tokens:

- **Layout:** 56 px title bar (custom chrome on Win11, standard on Win10 where custom chrome looks foreign), 40 px toolbar, 36 px tab strip, then a split of sidebar (200 px, collapsible) / file list / optional metadata panel (320 px).
- **Density:** comfortable (row height 32 px), compact (24 px) toggle.
- **Type:** Segoe UI Variable on Win11, Segoe UI on Win10. Monospace: Cascadia Code.
- **Radii:** 6 px on controls, 8 px on panels, 0 on list rows.
- **Elevation:** one shadow level only, on floating panels (dialogs, palette).
- **Motion:** 120 ms ease-out for state changes only (hover, selection, drop target). No entrance/exit animations. No parallax. No blur except the Win11 Mica backdrop, which is free and native.

### 3.2 Themes
Three modes: **Dark / Light / System**. All colors are CSS custom properties in src/styles/tokens.css, generated from a single Rust-side theme table so the OS accent color (Win11) can be injected. Contrast-checked in both modes.

### 3.3 Iconography
Icons come from the OS shell (IShellItemImageFactory) for files, so they match Explorer. Folder icons are tinted with the theme accent. All app-chrome icons are inline SVG (Lucide subset), 16/20 px, currentColor.

### 3.4 Entry states (every row must express all of these)
default, hover, selected, focused, cut (ghosted), hidden (badge), system (badge), symlink (arrow), offline (cloud badge), git-modified (badge), error (badge + tooltip).

### 3.5 Interaction details that matter
- Selection is **never** lost when the list refreshes; it is keyed by a stable entry id (volume serial + file id, falling back to full path).
- Clicking a directory once selects; double-click opens; Enter opens.
- The address bar is editable in place (click the breadcrumb, it becomes a text input).
- The status bar shows: N items, M selected, total size and, during operations, a progress bar with a cancel button.
- Every destructive confirm names the count and the total size: "Delete 1 284 items (32.7 GB)?" and never "Are you sure?".

### 3.6 Windows 10 vs 11
Feature-detect at runtime (windows-version crate). Mica/backdrop and rounded corners only on 11 (build >= 22000). On 10, fall back to a solid window background. Custom title bar only where DwmSetWindowAttribute supports it; otherwise use the standard frame.

---

## 4. ARCHITECTURE

### 4.1 Workspace layout

```
nevixplore/
├─ Cargo.toml                     # workspace
├─ package.json                   # frontend (Vite + React + TS)
├─ src/                           # frontend
│  ├─ main.tsx
│  ├─ app/                        # shell, routing, providers
│  ├─ views/                      # Browser, Search, Diff, DiskMap, Trash, Settings
│  ├─ components/                 # FileList, TabStrip, AddressBar, MetadataPanel, Preview
│  ├─ hooks/                      # useEntries, useSelection, useInvoke
│  ├─ state/                      # zustand stores
│  └─ styles/                     # tokens.css, base.css
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ build.rs
│  └─ src/
│     ├─ main.rs                  # entry, plugin registration, panic hook
│     ├─ lib.rs
│     ├─ commands/                # #[tauri::command] thin wrappers only
│     ├─ plugins/                 # mlua host: sandbox, loader, host API bindings
│     ├─ vfs/                     # virtual file system (local, archive, trash)
│     ├─ scan/                    # directory enumeration, streaming, watcher
│     ├─ meta/                    # metadata extractors per type
│     ├─ search/                  # index (SQLite), fuzzy, grep
│     ├─ ops/                     # copy/move/delete/rename, progress, undo
│     ├─ archive/                 # read/extract/create
│     ├─ git/                     # git status via git2
│     ├─ prefs/                   # settings load/save, paths
│     └─ error.rs                 # typed error -> user message mapping
├─ crates/
│  ├─ nv-entry/                   # Entry, EntryId, FileKind, attribute types
│  ├─ nv-vfs/                     # Vfs trait, LocalFs, ArchiveFs, TrashFs
│  ├─ nv-scan/                    # streaming directory reader (no Tauri deps)
│  ├─ nv-meta/                    # metadata extractors (no Tauri deps)
│  ├─ nv-fmt/                     # byte size, duration, date formatting
│  └─ nv-host-api/                # Lua host API contract + sandbox (no Tauri deps)
├─ .github/workflows/
│  ├─ ci.yml
│  ├─ release.yml
│  └─ nightly.yml
└─ docs/
   ├─ adr/
   ├─ ARCHITECTURE.md
   ├─ BUILDING.md
   └─ PLUGIN-API.md
```

**Rationale for the crates/ split:** nv-scan, nv-meta and nv-vfs have no Tauri dependency, so they can be unit-tested and benchmarked in plain cargo test (which is what CI runs). Tauri commands are thin adapters. This is what keeps the fast path testable.

### 4.2 Layering rules

```
UI (React)  ->  commands (Tauri)  ->  services (scan/meta/ops)  ->  OS (Win32 / std)
                     ^                      ^
              typed DTOs (serde)      plain Rust types (no serde)
```

- Commands accept and return **serde DTOs only**; never leak Rust types with lifetimes.
- Services never import tauri. If a service needs to notify the UI, it takes a callback.
- The OS layer is the only place unsafe is allowed, and only for Win32 FFI. Every unsafe block must be inside a module with deny(unsafe_op_in_unsafe_fn) and a safety comment (a safety comment is documentation, which the no-comments rule permits).

### 4.3 State and persistence
**Persisted** (in %APPDATA%\NevIxplore\settings.toml): theme, accent override, density, visible columns and their order, sidebar width, panel visibility, preview enabled, index roots, trash policy, max content-search size, terminal choice.
**Not persisted** (explicitly, per user instruction): window position/size, last location, tabs, history, scroll position, expanded tree nodes, sort state. The app always opens at the user's home directory.

### 4.4 Threading model
- **UI thread (main):** Tauri event loop. Never blocks. No file I/O.
- **Scan workers:** rayon pool plus one dedicated thread per active directory watcher.
- **Meta workers:** a bounded pool (num_cpus), per-column task queues, cancellable.
- **Search workers:** index builder (background, low priority), query executor (bounded).
- **Comms:** tokio::sync::mpsc for streams of entries into the UI; Tauri emit for high-level events (entry-updated, op-progress). Backpressure: drop intermediate entry batches if the UI is behind, but never drop a terminal event.

### 4.5 Error model

```rust
pub enum AppError {
    Io { path: PathBuf, source: std::io::Error },
    Permission { path: PathBuf },
    NotFound { path: PathBuf },
    NotADirectory { path: PathBuf },
    PathTooLong { path: PathBuf },
    Archive(ArchiveError),
    Git(git2::Error),
    Cancelled,
    Internal(String),
}
```

Every variant maps to (a) a short user message, (b) a longer explanation, (c) an OS code if any. Serialize produces { kind, message, detail, path, code } for the UI.

### 4.6 Data flow for "open a directory"

```
User navigates
  -> command list_dir(path, opts)
    -> nv_scan::open(path) returns a stream handle immediately
    -> first chunk (up to 512 entries) returned to the UI synchronously if available
    -> remaining chunks streamed via emit("entries", chunk)
    -> each chunk is deduped by EntryId and merged into the store
    -> metadata enrichment for the visible window only (requested by the UI)
```


### 4.7 Plugin host architecture (Lua)

```
Lua script (untrusted, sandboxed)
    |
    |  only reachable through the host table "nv"
    v
plugins/host.rs  -- clamps, timeouts, instruction budget, panic boundary (lua is FFI: a panic in a callback must not unwind into C)
    |
    |  enqueues a typed request (Vec<HostCall>)
    v
plugins/worker.rs (single thread, mpsc)
    |
    +-- reads:  nv.fs.*   -> services (scan/meta)
    +-- writes: nv.ui.*   -> Tauri emit
    +-- pure:   nv.log    -> tracing
```

Rules:

- A Lua call never touches the Tauri app handle. It produces a **HostCall** value; the Rust worker executes it and emits the result back to the plugin through a continuation.
- Every host call is **cost-limited**: a per-call instruction budget plus a per-invocation wall clock (default 50 ms). Exceeding either aborts the plugin invocation and raises a catchable Lua error (so `pcall` in the plugin still works) rather than killing the app.
- Lua panics are caught at the `lua.load().exec()` boundary (`mlua::Error::CallbackError` already converts panics; keep it that way and never `unwrap()` inside a callback).
- The `nv` global is installed per plugin, not shared. One plugin cannot see another's state.
- `os`, `io`, `debug`, `package`, `dofile`, `loadfile`, `loadstring`, `require`, `os.exit`, `collectgarbage` are **removed** from the plugin's globals. `pcall`, `xpcall`, `select`, `string`, `table`, `math`, `utf8` remain.

---

## 5. TECHNICAL DECISIONS (ADR index)

The agent must write a real ADR per row in `docs/adr/NNNN-title.md` (Context / Decision / Consequences / Alternatives rejected). Rows marked [LOCKED] are already decided — write the ADR to record the reasoning, do not reopen them.

| ID | Decision | Status |
|---|---|---|
| ADR-1 | Desktop shell: **Tauri 2.x** (latest stable 2.x; never a 3.0 alpha). Frontend: React 18 + TypeScript + Vite. | [LOCKED] |
| ADR-2 | Frontend list rendering: **virtualized** (custom windowing over a flat array, or `@tanstack/react-virtual`). Must survive 500 k rows. | [LOCKED] |
| ADR-3 | Directory enumeration: `std::fs::read_dir` on a worker, chunked into batches of 512, streamed to the UI. Benchmark against `walkdir` before committing to either. | [DECIDE] |
| ADR-4 | Change tracking: **notify** (ReadDirectoryChangesW on Windows) with debounce, per watched directory. Fallback to a full re-read on overflow. | [DECIDE] |
| ADR-5 | Name index: **rusqlite** (bundled SQLite), FTS5 for prefix/substring, plain B-tree for exact. Store names and paths only. | [DECIDE] |
| ADR-6 | Content search: single-pass streaming grep over files in the current folder, hard size/count caps, results as snippets. No external `rg.exe` dependency. | [DECIDE] |
| ADR-7 | Archive reading: `sevenz-rust` (7z/rar read), `zip` (zip), `tar`+`flate2`+`xz2`+`zstd`. Archives exposed through the `Vfs` trait as a read-only virtual filesystem. | [DECIDE] |
| ADR-8 | Metadata: `kamadak-exif` (images), `lofty` (audio tags), `pdf-extract` + `docx`-style unzip+XML parse for page counts. OS shell properties are used as a fallback where a crate is missing. | [DECIDE] |
| ADR-9 | Fuzzy matching: in-house subsequence scorer (case-folded, boundary-weighted, length-penalized). No new dependency. | [DECIDE] |
| ADR-10 | Git diff: **git2** (libgit2). Read-only. Never write to the user's repo. | [DECIDE] |
| ADR-11 | Plugins: **mlua 0.12 with the `lua54` feature** (vendored Lua 5.4). Sandboxed interpreter, host API table, instruction + wall-clock budget. | [LOCKED] |
| ADR-12 | Installer: **Inno Setup** (single .exe, optional desktop shortcut, per-user install by default, no admin required). Portable build via `tauri build --no-bundle` + a zip. | [DECIDE] |
| ADR-13 | Error handling: one `AppError` enum in `src-tauri/src/error.rs`, `thiserror` + a manual `Serialize` impl that yields { kind, message, detail, path, code }. Never `unwrap()` in a command. | [LOCKED] |
| ADR-14 | No `unsafe` outside the Win32 FFI layer. Any `unsafe` block needs a `// SAFETY:` comment (a safety comment is documentation, permitted by §0.1.4). | [LOCKED] |
| ADR-15 | Previews: render to a bounded RGBA buffer in a worker, hand an encoded PNG/JPEG to the WebView as a data URL or a temp file. Never decode in the WebView. | [DECIDE] |

### 5.1 Verified dependency versions (as of 2026-11-11 — re-check before use)

| Crate | Version | Used for |
|---|---|---|
| tauri | 2.12.5 | shell, window, tray, commands |
| tauri-plugin-dialog | 2.8.4 | native file/folder pickers |
| tauri-plugin-fs | 2.7.1 | scoped fs access (still bypassed for performance paths) |
| tauri-plugin-opener | 2.7.3 | open with default app |
| tauri-plugin-shell | 2.6.0 | terminal pane |
| rusqlite | 0.42.0 | index (bundled feature) |
| notify | 8.2.2 | directory watching |
| git2 | 0.21.0 | git status for diff |
| rayon | 1.12.0 | parallel scanning |
| tokio | 1.54.1 | async commands, streams |
| serde | 1.0.229 | DTOs |
| windows | 0.62.2 | Win32 FFI (ShellExecuteW, SHFileOperationW, IShellItemImageFactory) |
| sevenz-rust | 0.6.1 | 7z/RAR reading |
| zip | 9.5.1 | zip read/write |
| kamadak-exif | 0.6.2 | image EXIF |
| lofty | 0.25.5 | audio tags |
| pdf-extract | 0.12.1 | PDF text/page count |
| image | 0.25.10 | image decode/thumbnail |
| mlua (lua54) | 0.12.2 | plugin host |

Rules for dependencies: **no version range other than a caret**, no `git` dependencies, no dependency that pulls in a C toolchain the CI does not already have, and every addition justified in the PR description with the license. Any crate that is unmaintained (> 18 months since release) requires an ADR.

---

## 6. IMPLEMENTATION PLAN (do these in order, one PR each)

Every PR is **independently mergeable**, **keeps main green**, and **ends with the Definition of Done in §6.2**. Do not batch PRs. Do not leave a PR with a failing CI.

| PR | Title | Scope | Acceptance |
|---|---|---|---|
| PR-0 | `chore: repo scaffold, CI, and a window that opens` | Workspace Cargo.toml, `src-tauri` with a Tauri 2 app, `src/` Vite+React shell, `.github/workflows/ci.yml` (fmt, clippy -D warnings, test, npm build, tauri build), `release.yml` (v* tag), `nightly.yml`, `rust-toolchain.toml`, `.gitignore`, `rustfmt.toml`, `clippy.toml`, `Cargo.lock` committed, `README.md`, `LICENSE` (MIT), `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`. | CI green on `main`; a `v0.0.1` tag produces a portable .exe attached to a GitHub Release. |
| PR-1 | `feat(scan): streaming directory enumeration core` | `crates/nv-entry` (Entry, EntryId, FileKind, attrs), `crates/nv-scan` (chunked reader, `\?\` prefix handling, error isolation per entry), `crates/nv-fmt`. Unit tests only. | `cargo test -p nv-scan` green in CI; a test enumerates a directory with a 300-char path. |
| PR-2 | `feat(vfs): Vfs trait + LocalFs` | `crates/nv-vfs`: `trait Vfs { fn read_dir(..) -> Stream<Entry>; fn stat(..); fn open(..) }`, LocalFs impl, archive impl stubbed. | Trait tests pass with a fake in-memory Vfs. |
| PR-3 | `feat(ui): window chrome, address bar, virtualized file list` | React shell, tokens.css, dark/light/system, virtualized list wired to a **mock** command, selection, keyboard nav, context menu skeleton. | 500 000 mock rows scroll at 60 fps in the CI-built binary (manual check recorded in the PR). |
| PR-4 | `feat(commands): list_dir + streaming into the UI` | `list_dir` command, tokio mpsc → `emit`, first-chunk latency budget, error → `AppError` → user message. | Opening `C:\Windows\System32` shows entries in < 250 ms; cancellation on navigate-away works. |
| PR-5 | `feat(scan): watcher with ReadDirectoryChangesW` | notify-based watcher, debounce, per-directory task, overflow → full re-read. | Create/rename/delete a file from Explorer; the list updates without F5. |
| PR-6 | `feat(ops): copy, move, delete, rename with progress` | SHFileOperationW for delete-to-bin; custom worker for copy/move with progress channel, conflict policy, cancel. | 10 GB copy streams progress, cancel stops within 200 ms, no partial temp left behind. |
| PR-7 | `feat(search): filename index + fuzzy search` | rusqlite schema, FTS5 index, incremental builder, cancellable; subsequence scorer in `nv-scan` or a new `nv-fuzzy`. | Index a 200 k-file tree; typing 3 chars returns results in < 50 ms with highlight ranges. |
| PR-8 | `feat(search): content grep with line snippets + dual highlighting` | Streaming grep, size/count caps, snippet extraction with line numbers, highlighted spans returned as **ranges** (not HTML) so the UI does the highlighting. | Search for `TODO` in a folder of 5 000 files; each hit shows its lines; opening the file shows them highlighted. |
| PR-9 | `feat(meta): metadata panel + per-type extractors` | `crates/nv-meta`: exif, lofty, pdf page count, text stats; lazy per-column compute with cancellation. | EXIF of a photo, tags of an mp3, and line count of a .rs file all appear; a 5 MiB+ text file is skipped with a note. |
| PR-10 | `feat(preview): bounded preview pipeline` | Worker decode → RGBA → PNG, size caps, timeout, cancel; WebView renders data URL; **off by default for large folders**, master toggle. | A 200 MB TIFF does not hang; disabling preview stops all decoding. |
| PR-11 | `feat(archive): read + extract via Vfs` | sevenz-rust / zip / tar backends, `ArchiveFs` implementing Vfs, traversal rejection, corruption messages. | Opening a .7z shows its tree; extracting 1 000 entries streams progress; `../../evil` is rejected. |
| PR-12 | `feat(git): folder diff with git2` | Two-pane diff, size+mtime compare, optional hash, .git annotation with M/A/D/U. | Diffing a repo against its working tree shows the right badges; no writes to the repo. |
| PR-13 | `feat(plugins): mlua host, sandbox, API v0` | `crates/nv-host-api`, `plugins/host.rs`, `plugins/worker.rs`, manifest loader, the shipped example, settings UI. | `examples/plugins/hello.lua` loads, adds a menu item, adds a column; a plugin calling `os.execute` fails loudly and does not crash the app. |
| PR-14 | `feat(release): installer, nightly artifacts, docs` | Inno Setup script, version metadata, `docs/` (ARCHITECTURE, BUILDING, PLUGIN-API, adr/), release notes template. | `v0.1.0` release has an installer and a portable zip; docs render on GitHub. |

**[LOCKED] Never combine a feature PR with a refactor PR.** If a refactor is required, it goes first and alone.

### 6.1 Sequencing rules
- PR-0 is a hard gate. Nothing else starts until CI is green on `main`.
- PR-1..PR-2 are pure-Rust crates with unit tests — they must be finished before any UI work depends on them.
- PR-3 ships a **mock** data source so the UI can be reviewed before the backend exists. The mock is deleted in PR-4.
- Performance budgets (§2.9 NFR-1) are verified at the end of PR-3, PR-4, PR-7 and PR-10, and recorded as a comment in the PR with numbers.

### 6.2 Definition of Done (every PR)
1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run build` all pass **in CI**.
2. No `unwrap()`, `expect()`, `panic!()` in non-test code outside a documented panic boundary. `unsafe` only in FFI modules with `// SAFETY:`.
3. No new dependency without a version pin and a license note in the PR body.
4. New behavior has a test. Pure logic in `crates/` gets unit tests; command handlers get at least one integration test against a temp dir.
5. User-facing strings are in English, in one `strings.ts` / `i18n/en.ts` file — never inline.
6. `docs/` updated if the PR changes an ADR-level decision.
7. The PR body states: what changed, why, how it was verified, and the numbers for any performance claim.
8. No commit message without a Conventional Commits prefix (`feat:`, `fix:`, `chore:`, `refactor:`, `docs:`, `test:`, `perf:`, `ci:`).

---

## 7. CODE STANDARDS

### 7.1 Rust
- Edition **2024**, rustfmt default profile with `max_width = 100`, `imports_granularity = "Crate"`.
- Clippy: `-D warnings`, plus `clippy::pedantic` promoted to warnings on `crates/` only (the Tauri command layer is exempt to keep the diff small).
- **No `unwrap()`/`expect()` outside tests.** `let Some(x) = ... else { return Err(...) } `.
- Errors: `thiserror` for library crates, a single `AppError` at the boundary.
- `unsafe`: only in `win32/`, and only with a `// SAFETY:` comment. `deny(unsafe_op_in_unsafe_fn)` at the workspace level.
- Async: `tokio::task::spawn_blocking` for anything that touches the disk. Never `async` on a blocking API.
- No `#[allow(...)]` without a reason in the same line's scope (e.g. `#[allow(clippy::too_many_lines)] // UI helper, split in #NN`).
- Tests: `#[cfg(test)] mod tests` in the same file for unit tests, `tests/` for integration. Use `tempfile` for filesystem fixtures. Tests must pass in a **clean CI container** with no network.

### 7.2 TypeScript / React
- `strict: true`, `noUncheckedIndexedAccess: true`, `exactOptionalPropertyTypes: true`.
- ESLint + `typescript-eslint` recommended-strict; `npm run lint` is part of CI.
- No `any`. `unknown` + a type guard, or a zod schema for anything crossing the IPC boundary.
- **IPC DTOs are generated from Rust** with `ts-rs` (or hand-written and reviewed together). The Rust type is the source of truth.
- React: function components only, hooks, no class components, no `useEffect` for data fetching that could be an event handler.
- The virtualized list must not re-render on every streamed chunk: entries are merged in a store (zustand) and the list subscribes to a slice.
- Accessibility: the list is `role="grid"` with `aria-selected`, `aria-rowcount`; keyboard nav is complete without a mouse.

### 7.3 Git
- Branch names: `feat/<slug>`, `fix/<slug>`, `chore/<slug>`.
- **Do not push directly to `main`.** Open a PR. (If branch protection blocks self-merge, ask for a bypass or leave the PR open — never force-push.)
- Conventional Commits, imperative mood, no trailing period, body wrapped at 72.
- One logical change per commit. Do not mix a rename with a logic change.

---

## 8. CI / CD (all builds on GitHub Actions — nothing is installed locally)

### 8.1 `ci.yml` — triggers: `push` to `main` and `pull_request`; runner `windows-latest`.

```yaml
name: ci
on:
  push: { branches: [main] }
  pull_request:
permissions: { contents: read }
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
jobs:
  rust:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
  frontend:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: npm }
      - run: npm ci
      - run: npm run lint
      - run: npm run build
  bundle:
    runs-on: windows-latest
    needs: [rust, frontend]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: npm }
      - run: npm ci
      - run: npm run tauri build -- --no-bundle
      - uses: actions/upload-artifact@v4
        with:
          name: nevixplore-portable
          path: src-tauri/target/release/nevixplore.exe
```

### 8.2 `release.yml` — trigger: `push` tag `v*`. Same steps, plus Inno Setup for the installer, `softprops/action-gh-release` to publish, and a portable zip alongside the installer. Release notes are generated from `git log` with `conventional-changelog`.

### 8.3 `nightly.yml` — cron `0 3 * * *` (UTC), builds from `main`, publishes to the `nightly` prerelease, keeps the last 7 builds. Runs the performance budget script and fails the job if a budget regresses by more than 20 %.

### 8.4 Branch protection on `main`
Require: `rust`, `frontend`, `bundle` checks; 1 approving review; linear history; no force-push; no deletion; signed commits **if** the user configures a key, otherwise skipped.

### 8.5 What the agent must NOT do
- Do not install Rust/Node/Inno Setup on the user's machine. Do not run `rustup`, `cargo build`, `npm install` locally. If a verification is needed, push a branch and let CI run it.
- Do not commit build artifacts, `target/`, `node_modules/`, `.exe`, or `.msi`.

---

## 9. DELIVERY ARTIFACTS (create all of these)

| Artifact | Where |
|---|---|
| README with a screenshot placeholder, install instructions, feature list, roadmap | `README.md` |
| Architecture overview + the layer diagram + data flows | `docs/ARCHITECTURE.md` |
| How to build (CI-only workflow, how to run the nightly, how to release) | `docs/BUILDING.md` |
| Plugin API reference: every `nv.*` function, signatures, the manifest schema, two full examples | `docs/PLUGIN-API.md` |
| ADRs ADR-1..ADR-15 | `docs/adr/NNNN-*.md` |
| Benchmark harness: enumerate, open, search, preview; outputs a markdown table | `crates/nv-bench/` (or `benches/` with criterion) |
| Performance budget doc with the numbers from the last nightly | `docs/PERF.md` |
| Contributing guide: PR flow, DoD, commit style | `CONTRIBUTING.md` |
| Security policy (no telemetry, how to report a vuln, what data never leaves the machine) | `SECURITY.md` |
| License (MIT) | `LICENSE` |
| Changelog, generated | `CHANGELOG.md` |
| Editor/agent config that does not commit secrets | `.gitignore`, `.editorconfig`, `rust-toolchain.toml` |
| CI workflows as above | `.github/workflows/` |

---

## 10. ACCEPTANCE CRITERIA (the user will check these by hand)

1. Double-click `NevIxplore.exe`. A window opens in under 1.2 s, in the system theme, with no splash screen and no network request.
2. Navigate to `C:\Windows\System32` (about 5 000 files). The first screen appears in under 250 ms. Scrolling to the bottom is smooth. The status bar shows the count.
3. Open a folder with 200 000 generated files. First screen under 250 ms, scroll 60 fps, memory under 180 MB.
4. Create a file in that folder from Explorer. It appears within one second, without pressing F5.
5. Type `repor` in the search box. Within 50 ms, files whose names contain those letters in order appear, with the matched characters highlighted in the name.
6. Search for `TODO` in a folder of mixed text files. Each matching file shows a badge with the hit count and expands to the matching lines, each with the matched text highlighted. Clicking a snippet opens the file at that line.
7. Select 20 files, press F2, run a batch rename that numbers them 01..20 with a prefix. All 20 are renamed atomically enough that a failure leaves no half-renamed set.
8. Start copying a 10 GB folder. A progress dialog shows file, speed, ETA. Cancel at 40 %. The destination contains no partial file.
9. Open a `.7z` file. Its tree appears in a new tab. Extract two entries. A `.zip` with `../` entries is rejected with a clear message.
10. Open the metadata panel on a photo: dimensions, camera, exposure, ISO, GPS. On an mp3: artist, album, duration, cover. On a `.rs` file: line count, encoding, line endings.
11. Enable preview. A 200 MB TIFF does not hang. Disable preview. Decoding stops entirely.
12. Drop a file from NevIxplore onto the desktop. It is copied. Drop a file from Explorer into NevIxplore. It is copied.
13. Copy the repo's example plugin into `%APPDATA%\NevIxplore\plugins\`, restart, and its menu item appears on right-click and its column appears in Details view.
14. Delete 5 000 files to the Recycle Bin. Confirm with one dialog that says "Delete 5 000 items (1.2 GB)?". Restore them. All 5 000 are back.
15. Disconnect a network drive mid-browse. The app shows a clear error and stays responsive.
16. Run the app for 8 hours with a folder open. Memory does not grow more than 20 % over the 1-hour mark.

---

## 11. ANTI-REQUIREMENTS (do not build these)

- Do not add telemetry, analytics, crash reporting to a third party, or an account system.
- Do not add any network call except the explicit, user-initiated update check.
- Do not use a web UI framework's default theme, a component library's default look, or an emoji-as-icon approach.
- Do not animate layout, do not add skeleton loaders that flash, do not add a loading spinner over content.
- Do not use `unsafe` outside FFI. Do not use `mem::transmute`. Do not call `std::process::Command` with a concatenated string.
- Do not build a plugin loader that executes a DLL, a shell command, or an arbitrary URL.
- Do not create a settings dialog with more than two levels of nesting.
- Do not implement a custom title bar on Windows 10 if it looks out of place — detect and fall back.
- Do not write code comments. Do not leave commented-out code. Do not leave `TODO` without an issue number.
- Do not implement a feature from §2 that is marked **C** if it risks the **M** items.
- Do not open a PR that mixes a feature and a refactor.
- Do not add a dependency to make a one-line thing shorter.

---

## 12. OPEN QUESTIONS FOR THE AGENT

Answer these **in the plan**, not by asking the user:

1. Windowing library for the virtualized list: `@tanstack/react-virtual` vs in-house. Benchmark both on 500 k rows and pick one; record it.
2. `notify` vs raw `ReadDirectoryChangesW` for the watcher: measure the debounce and the overflow path; justify.
3. Should `nv-scan` use `std::fs::read_dir` or a `ReadDir` over `NtQueryDirectoryFile`? Benchmark; default to std.
4. Where does the fuzzy scorer live — `nv-scan`, a new `nv-fuzzy`, or the frontend? Prefer Rust (shared with the index), but the UI needs highlight ranges, so the scorer must return `Vec<(usize, usize)>`.
5. Is `tauri-plugin-fs` scope configuration usable at all for the performance paths, or must the Rust side bypass it? Decide once and document.
6. Preview pipeline: data URL vs temp file vs custom protocol (`nevixplore://preview/<id>`). Custom protocol is likely best for memory; benchmark.
7. `mlua` sandbox: remove globals, or a full separate Lua state per plugin with a capability table? Prefer the former plus per-plugin state; verify `os.execute` is genuinely unreachable.
8. Inno Setup vs the Tauri WiX/MSI bundle: confirm Inno is the right call for a per-user install and record the installer size delta.

---

## 13. FIRST MESSAGE THE AGENT SHOULD SEND BACK

Before writing code, reply with exactly this structure (and nothing longer):

```
1. PLAN SUMMARY  - the PR sequence from §6, one line each, with the order and what each unblocks.
2. ADR DECISIONS  - for each [DECIDE] row in §5, the choice and one sentence of justification.
3. RISKS         - top 5 risks (e.g. WebView2 performance ceiling, mlua sandbox completeness, notify overflow on network drives) and the mitigation.
4. BENCHMARKS    - what you will measure, with what tool, at which PR, and the pass/fail threshold.
5. FIRST PR      - the exact file list of PR-0 and the commands CI will run.
6. QUESTIONS     - only questions whose answer changes the architecture. If there are none, say "none".
```

Do not write any code until this message is approved.
