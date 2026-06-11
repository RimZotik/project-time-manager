# Project Time Manager

[README на русском](README.md)

Project Time Manager is a Windows desktop time tracker for project work. You select a project, start tracking, and the app measures which applications and browser domains were active while you worked.

![Project Time Manager screenshot](docs/screenshot.png)

## Features

- Create, select, rename, and delete projects.
- Track work sessions: start, pause, and stop.
- Capture the active Windows foreground window through WinAPI.
- Show apps with real icons, time, and percentage of project time.
- Expand browsers into domain groups and store visited URLs for every domain.
- Include or exclude apps and domains from totals without deleting raw time.
- Export Excel and PDF reports.
- Store runtime data next to the app in the `data` folder.
- Switch the interface between Russian and English.
- Enable Windows autostart.

## Data Location

After installation, the app creates this folder next to the executable:

```text
data/
  workspace.json
  Проекты/
    <Project name>/
      project.json
      <Project name>.xlsx
      <Project name>.pdf
```

When a project is renamed, the project folder and newly generated reports use the project name.

## How To Use

1. Create a project in the left column or select an existing one.
2. Press `Start`.
3. Work as usual: editing, development, browser research, documents.
4. Disable checkboxes for apps or domains that should not count toward totals.
5. Press `Pause` or `Stop`.
6. Export an Excel or PDF report.

Import and export are disabled during active tracking so reports are not generated from half-written data.

## For Developers

### Stack

- Tauri 2
- Rust backend
- React + TypeScript
- Vite
- Tailwind CSS
- `rust_xlsxwriter` for Excel
- `printpdf` for PDF
- WinAPI through the `windows` crate

### Local Development

```bash
npm install
npm run tauri:dev
```

On macOS and Linux the UI can run, but real active-window tracking is Windows-only.

### Build

Build for the current OS:

```bash
npm run tauri:build
```

Windows builds are best produced on Windows or through GitHub Actions:

```text
.github/workflows/windows-build.yml
```

The workflow builds an NSIS installer and attaches `project-time-manager.exe` to GitHub Releases for tags that match `v*`.

### Structure

```text
src/
  App.tsx              # main React UI, localization, state handling
  main.tsx             # frontend entry point
  index.css            # Tailwind and shared styles

src-tauri/src/
  main.rs              # Tauri commands, tracking, settings
  storage.rs           # file storage, migrations, projects
  windows.rs           # WinAPI: active window, process, URL/icons
  export.rs            # Excel
  pdf.rs               # PDF
  models.rs            # shared data models

src-tauri/windows/
  nsis-hooks.nsh       # installer and uninstaller hooks
```

### Release

1. Update versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Verify the build:

```bash
npm run build
cd src-tauri
cargo check
cargo test
```

3. Create a tag:

```bash
git tag v0.1.7
git push origin develop --tags
```

4. GitHub Actions builds the Windows installer and creates the Release.
