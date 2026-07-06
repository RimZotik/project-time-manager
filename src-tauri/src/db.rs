// SQLite-хранилище: схема + миграция из старого JSON-формата.
//
// Схема отражает новую структуру приложения: проекты получают категорию и
// цвет, добавлены таблицы для правил автокатегоризации и настроек. Богатые
// данные (сессии, приложения, вкладки, ссылки, этапы) переносятся 1:1.
use crate::models::*;
use crate::storage::{self, StoragePaths};
use rusqlite::{params, Connection};

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS setting (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS category (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  color      TEXT NOT NULL DEFAULT '#059669',
  icon       TEXT NOT NULL DEFAULT '',
  ord        INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS project (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  client      TEXT NOT NULL DEFAULT '',
  note        TEXT NOT NULL DEFAULT '',
  category_id TEXT REFERENCES category(id) ON DELETE SET NULL,
  color       TEXT,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stage (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  ord        INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS project_selected_stage (
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  stage_id   TEXT NOT NULL,
  PRIMARY KEY (project_id, stage_id)
);

CREATE TABLE IF NOT EXISTS session (
  id               TEXT PRIMARY KEY,
  project_id       TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  started_at       TEXT NOT NULL,
  stopped_at       TEXT,
  duration_seconds INTEGER NOT NULL DEFAULT 0,
  app_count        INTEGER NOT NULL DEFAULT 0,
  browser_count    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS session_stage (
  session_id TEXT NOT NULL REFERENCES session(id) ON DELETE CASCADE,
  stage_id   TEXT,
  name       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_usage (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id    TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  key           TEXT NOT NULL,
  name          TEXT NOT NULL,
  process_name  TEXT NOT NULL DEFAULT '',
  process_path  TEXT NOT NULL DEFAULT '',
  icon_data_url TEXT,
  kind          TEXT NOT NULL DEFAULT 'app',
  enabled       INTEGER NOT NULL DEFAULT 1,
  time_seconds  INTEGER NOT NULL DEFAULT 0,
  UNIQUE (project_id, key)
);

CREATE TABLE IF NOT EXISTS tab_usage (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  app_id       INTEGER NOT NULL REFERENCES app_usage(id) ON DELETE CASCADE,
  key          TEXT NOT NULL,
  title        TEXT NOT NULL DEFAULT '',
  url          TEXT,
  favicon_url  TEXT,
  enabled      INTEGER NOT NULL DEFAULT 1,
  time_seconds INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS visited_url (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  tab_id       INTEGER NOT NULL REFERENCES tab_usage(id) ON DELETE CASCADE,
  url          TEXT NOT NULL,
  title        TEXT NOT NULL DEFAULT '',
  last_seen_at TEXT NOT NULL DEFAULT '',
  hits         INTEGER NOT NULL DEFAULT 0,
  enabled      INTEGER NOT NULL DEFAULT 1,
  time_seconds INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS stage_app (
  stage_id TEXT NOT NULL REFERENCES stage(id) ON DELETE CASCADE,
  app_key  TEXT NOT NULL,
  enabled  INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (stage_id, app_key)
);

CREATE TABLE IF NOT EXISTS stage_tab (
  stage_id TEXT NOT NULL REFERENCES stage(id) ON DELETE CASCADE,
  app_key  TEXT NOT NULL,
  tab_key  TEXT NOT NULL,
  enabled  INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (stage_id, app_key, tab_key)
);

CREATE TABLE IF NOT EXISTS stage_url (
  stage_id TEXT NOT NULL REFERENCES stage(id) ON DELETE CASCADE,
  app_key  TEXT NOT NULL,
  tab_key  TEXT NOT NULL,
  url      TEXT NOT NULL,
  enabled  INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (stage_id, app_key, tab_key, url)
);

CREATE TABLE IF NOT EXISTS app_rule (
  id            TEXT PRIMARY KEY,
  match_process TEXT NOT NULL,
  category_id   TEXT REFERENCES category(id) ON DELETE CASCADE,
  created_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_session_project ON session(project_id);
CREATE INDEX IF NOT EXISTS idx_app_project    ON app_usage(project_id);
CREATE INDEX IF NOT EXISTS idx_tab_app        ON tab_usage(app_id);
CREATE INDEX IF NOT EXISTS idx_url_tab        ON visited_url(tab_id);
CREATE INDEX IF NOT EXISTS idx_stage_project  ON stage(project_id);
"#;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub projects: usize,
    pub sessions: usize,
    pub apps: usize,
    pub tabs: usize,
    pub urls: usize,
    pub stages: usize,
    pub total_duration_seconds: u64,
}

/// Открыть базу по пути и подготовить схему.
pub fn open(path: &std::path::Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(SCHEMA).map_err(|e| e.to_string())
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO setting(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Была ли база уже заполнена (есть ли проекты).
pub fn is_populated(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

/// Прочитать старые JSON-данные (через загрузчики storage) и перенести в SQLite.
pub fn migrate_from_json(
    conn: &mut Connection,
    paths: &StoragePaths,
) -> Result<MigrationReport, String> {
    let workspace = storage::load_workspace(paths)?;
    let projects = storage::list_project_files(paths)?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Настройки из workspace.json
    set_setting(&tx, "language", &workspace.language)?;
    set_setting(&tx, "autostart", if workspace.autostart { "1" } else { "0" })?;
    if let Some(selected) = &workspace.selected_project_id {
        set_setting(&tx, "selected_project_id", selected)?;
    }

    let mut report = MigrationReport::default();

    for project in &projects {
        insert_project(&tx, project, &mut report)?;
    }
    report.projects = projects.len();

    tx.commit().map_err(|e| e.to_string())?;
    Ok(report)
}

fn insert_project(
    conn: &Connection,
    project: &ProjectRecord,
    report: &mut MigrationReport,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO project(id, name, client, note, category_id, color, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, NULL, NULL, ?5, ?6)",
        params![
            project.id,
            project.name,
            project.client,
            project.note,
            project.created_at,
            project.updated_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    // Этапы
    for stage in &project.stages {
        conn.execute(
            "INSERT INTO stage(id, project_id, name, ord, created_at, updated_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                stage.id,
                project.id,
                stage.name,
                stage.order as i64,
                stage.created_at,
                stage.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        for sa in &stage.apps {
            conn.execute(
                "INSERT OR IGNORE INTO stage_app(stage_id, app_key, enabled) VALUES(?1, ?2, ?3)",
                params![stage.id, sa.app_key, sa.enabled as i64],
            )
            .map_err(|e| e.to_string())?;
            for st in &sa.tabs {
                conn.execute(
                    "INSERT OR IGNORE INTO stage_tab(stage_id, app_key, tab_key, enabled)
                     VALUES(?1, ?2, ?3, ?4)",
                    params![stage.id, sa.app_key, st.tab_key, st.enabled as i64],
                )
                .map_err(|e| e.to_string())?;
                for su in &st.urls {
                    conn.execute(
                        "INSERT OR IGNORE INTO stage_url(stage_id, app_key, tab_key, url, enabled)
                         VALUES(?1, ?2, ?3, ?4, ?5)",
                        params![stage.id, sa.app_key, st.tab_key, su.url, su.enabled as i64],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    report.stages += project.stages.len();

    // Выбранные этапы
    for stage_id in &project.selected_stage_ids {
        conn.execute(
            "INSERT OR IGNORE INTO project_selected_stage(project_id, stage_id) VALUES(?1, ?2)",
            params![project.id, stage_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Сессии
    for session in &project.sessions {
        conn.execute(
            "INSERT INTO session(id, project_id, started_at, stopped_at, duration_seconds, app_count, browser_count)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.id,
                project.id,
                session.started_at,
                session.stopped_at,
                session.duration_seconds as i64,
                session.app_count as i64,
                session.browser_count as i64,
            ],
        )
        .map_err(|e| e.to_string())?;
        for snap in &session.stages {
            conn.execute(
                "INSERT INTO session_stage(session_id, stage_id, name) VALUES(?1, ?2, ?3)",
                params![session.id, snap.id, snap.name],
            )
            .map_err(|e| e.to_string())?;
        }
        report.total_duration_seconds += session.duration_seconds;
    }
    report.sessions += project.sessions.len();

    // Приложения → вкладки → ссылки
    for app in &project.apps {
        conn.execute(
            "INSERT INTO app_usage(project_id, key, name, process_name, process_path, icon_data_url, kind, enabled, time_seconds)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                project.id,
                app.key,
                app.name,
                app.process_name,
                app.process_path,
                app.icon_data_url,
                app.kind,
                app.enabled as i64,
                app.time_seconds as i64,
            ],
        )
        .map_err(|e| e.to_string())?;
        let app_id = conn.last_insert_rowid();
        report.apps += 1;

        for tab in &app.tabs {
            conn.execute(
                "INSERT INTO tab_usage(app_id, key, title, url, favicon_url, enabled, time_seconds)
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    app_id,
                    tab.key,
                    tab.title,
                    tab.url,
                    tab.favicon_url,
                    tab.enabled as i64,
                    tab.time_seconds as i64,
                ],
            )
            .map_err(|e| e.to_string())?;
            let tab_id = conn.last_insert_rowid();
            report.tabs += 1;

            for url in &tab.urls {
                conn.execute(
                    "INSERT INTO visited_url(tab_id, url, title, last_seen_at, hits, enabled, time_seconds)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        tab_id,
                        url.url,
                        url.title,
                        url.last_seen_at,
                        url.hits as i64,
                        url.enabled as i64,
                        url.time_seconds as i64,
                    ],
                )
                .map_err(|e| e.to_string())?;
                report.urls += 1;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn copy_dir_all(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let target = dst.join(entry.file_name());
            if path.is_dir() {
                copy_dir_all(&path, &target);
            } else {
                fs::copy(&path, &target).unwrap();
            }
        }
    }

    #[test]
    fn migrate_real_montage_data() {
        // Реальные данные установленного приложения на этой машине.
        let real = PathBuf::from("/Users/rimzotik/Downloads/Project Time Manager-datas/data");
        if !real.exists() {
            eprintln!("SKIP: реальные данные не найдены ({real:?})");
            return;
        }

        // Копируем во временную папку, чтобы не тронуть оригинал.
        let tmp = std::env::temp_dir().join(format!("ptm_migrate_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        copy_dir_all(&real, &tmp);

        let paths = StoragePaths {
            projects_dir: tmp.join("Проекты"),
            workspace_file: tmp.join("workspace.json"),
        };

        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        let report = migrate_from_json(&mut conn, &paths).unwrap();
        eprintln!("Отчёт миграции: {report:?}");

        // Проверка счётчиков против фактических значений «Монтаж».
        assert_eq!(report.projects, 1, "проектов");
        assert_eq!(report.sessions, 14, "сессий");
        assert_eq!(report.apps, 41, "приложений");
        assert_eq!(report.stages, 9, "этапов");
        assert_eq!(report.total_duration_seconds, 229_946, "суммарное время");

        // Данные реально в базе, а не только в счётчиках.
        let db_sessions: i64 = conn
            .query_row("SELECT COUNT(*) FROM session", [], |r| r.get(0))
            .unwrap();
        assert_eq!(db_sessions, 14);
        let db_apps: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_usage", [], |r| r.get(0))
            .unwrap();
        assert_eq!(db_apps, 41);
        let db_duration: i64 = conn
            .query_row("SELECT COALESCE(SUM(duration_seconds),0) FROM session", [], |r| r.get(0))
            .unwrap();
        assert_eq!(db_duration, 229_946);

        // Топ-приложение по времени должно быть Premiere Pro.
        let top: String = conn
            .query_row(
                "SELECT name FROM app_usage ORDER BY time_seconds DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(top.contains("Premiere"), "топ-приложение = {top}");

        let _ = fs::remove_dir_all(&tmp);
    }
}
