# Project Time Manager

Desktop time tracker for Windows projects.

## Stack

- Tauri 2 + Rust backend
- React + Vite + TypeScript frontend
- Tailwind CSS UI
- JSON project files stored beside the executable in `data/projects`
- Excel export through `rust_xlsxwriter`

## Current Scope

- Create and select projects.
- Start, pause, and stop a tracking session.
- Poll the active Windows foreground window every second.
- Group browser processes as expandable rows with page/window titles as tab rows.
- Toggle apps and tabs in or out of project totals.
- Save project data as JSON.
- Import project JSON through the UI.
- Export Excel with `Dashboard`, `Apps`, `BrowserTabs`, and `Sessions` sheets.

## Commands

```bash
npm install
npm run tauri:dev
```

Build:

```bash
npm run tauri:build
```

Windows cross-build attempt from macOS:

```bash
brew install llvm nsis
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
npm run tauri:build:windows
```

The reliable release path is still to run `npm run tauri:build` on a Windows machine or Windows CI runner. The app is aimed at Windows. Non-Windows development builds can open the UI, but active-window tracking returns no samples.

There is also a GitHub Actions workflow:

```text
.github/workflows/windows-build.yml
```

Run it manually from GitHub Actions to get `project-time-manager-windows-exe` as an artifact.

## Data

Runtime files are created next to the executable:

```text
data/
  workspace.json
  projects/
    <project-id>.json
  exports/
    <project-name>.xlsx
```

This keeps the app close to a portable workflow. Tauri can produce a runnable `.exe` in the Windows release target directory; installer bundling is disabled for now because the goal is a direct launcher rather than an installer.

## Known Next Steps

- Real browser URL capture. Windows foreground-window APIs expose the tab/window title, not the URL. URL tracking needs a browser extension, browser remote debugging, or an accessibility/automation bridge.
- Extract real Windows app icons instead of symbolic UI badges.
- Add PDF export.
- Add a proper portable `.exe` release script and CI job for Windows.
