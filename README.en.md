# Project Time Manager

[README на русском](README.md)

**Project Time Manager** is a desktop time tracker for Windows. It measures not just "time at the computer" but the **real time spent on a specific project**: which apps, browser tabs and sites were active, and which project stages the work belonged to — so you can view rich analytics and reports right inside the app.

> The app targets Windows (real active-window tracking via WinAPI). On macOS/Linux the UI opens for development, but active-window tracking is unavailable.

## Features

- **Projects** with categories (e.g. Editing, Programming) — color, icon, category filter.
- **Time tracking**: start / pause / stop, with a recording indicator always visible top-right.
- **Activity tracking** (Windows): active window, apps with real icons, browsers expanded to sites and visited links.
- **Include / exclude** apps, sites and links from totals without deleting the data.
- **Project stages**: chosen before a session; time is split across stages.
- **Reports with a timeline** — editing-software style: tracks by stage and by session, wheel zoom, scroll, drag.
- **Analytics** across projects: time by project, by category (donut), daily activity, an hour×weekday heatmap, top apps.
- **Dashboard**: live recording status with a clock, KPIs, top projects, weekly activity, quick start.
- **Monitoring**: current active window, session clock, live app list.
- **Auto-categorization**: "app → category" rules and a "Detect category from apps" action on a project.
- **Automatic DB backup** on startup (keeps the last 10 copies).
- **Settings**: language (Russian / English), Windows autostart, data folder.

## Data

Stored in a SQLite database next to the executable:

```text
data/
  ptm.db      # main database
  backups/    # automatic backups (last 10)
```

On first launch, legacy JSON data from earlier versions (`data/workspace.json`, `data/Проекты/<Name>/project.json`) is **migrated automatically**.

## Install (Windows)

Download `project-time-manager.exe` from the latest [release](../../releases/latest) and run it.

## Tech stack

Tauri 2 + Rust, React 19 + TypeScript + Vite + Tailwind CSS 4, SQLite (`rusqlite`), WinAPI (`windows` crate), `framer-motion`.

## Development

```bash
npm install
npm run tauri:dev
npm run build                # typecheck + frontend build
cd src-tauri && cargo test   # backend tests
```

## Build & release

Pushing a `vX.Y.Z` tag triggers `.github/workflows/windows-build.yml`, which builds the NSIS installer and publishes a GitHub Release with `project-time-manager.exe`.
