// SQLite-хранилище: схема + миграция из старого JSON-формата.
//
// Схема отражает новую структуру приложения: проекты получают категорию и
// цвет, добавлены таблицы для правил автокатегоризации и настроек. Богатые
// данные (сессии, приложения, вкладки, ссылки, этапы) переносятся 1:1.
use crate::models::*;
use crate::storage::{self, StoragePaths};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;

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

pub fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(SCHEMA).map_err(|e| e.to_string())
}

/// Открыть базу и гарантировать схему + прагмы (foreign_keys, WAL).
pub fn connect(db_path: &std::path::Path) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO setting(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT value FROM setting WHERE key = ?1", params![key], |r| {
        r.get::<_, String>(0)
    })
    .optional()
    .map_err(|e| e.to_string())
}

pub fn clear_setting(conn: &Connection, key: &str) -> Result<(), String> {
    conn.execute("DELETE FROM setting WHERE key = ?1", params![key])
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
    // Читаем именно legacy-JSON (а не SQLite-обёртки storage), иначе миграция
    // читала бы из пустой базы саму себя.
    let workspace = storage::load_workspace_json(paths)?;
    let projects = storage::list_project_files_json(paths)?;

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
    insert_children(conn, project, report)
}

/// Вставить дочерние сущности проекта (этапы, сессии, приложения…).
/// Строка самого проекта должна уже существовать.
fn insert_children(
    conn: &Connection,
    project: &ProjectRecord,
    report: &mut MigrationReport,
) -> Result<(), String> {
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

// ── Аналитика ─────────────────────────────────────────────────────────────

/// Агрегированные данные по всем проектам для страницы аналитики.
pub fn analytics(conn: &Connection) -> Result<AnalyticsPayload, String> {
    // Базовый список проектов (без вычисляемых полей).
    let base: Vec<(String, String, Option<String>)> = {
        let mut stmt = conn
            .prepare("SELECT id, name, category_id FROM project")
            .map_err(|e| e.to_string())?;
        let rows: Vec<(String, String, Option<String>)> = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let mut projects = Vec::with_capacity(base.len());
    for (id, name, category_id) in base {
        let total_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(time_seconds),0) FROM app_usage WHERE project_id=?1 AND enabled=1",
                params![id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let session_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session WHERE project_id=?1",
                params![id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        projects.push(ProjectAnalytics {
            id,
            name,
            category_id,
            total_seconds: total_seconds as u64,
            session_count: session_count as u64,
        });
    }

    // Все сессии (облегчённое представление).
    let sessions: Vec<SessionLite> = {
        let mut stmt = conn
            .prepare("SELECT project_id, started_at, duration_seconds FROM session")
            .map_err(|e| e.to_string())?;
        let rows: Vec<SessionLite> = stmt
            .query_map([], |r| {
                Ok(SessionLite {
                    project_id: r.get::<_, String>(0)?,
                    started_at: r.get::<_, String>(1)?,
                    duration_seconds: r.get::<_, i64>(2)? as u64,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    // Топ приложений по суммарному времени.
    let top_apps: Vec<AppTotal> = {
        let mut stmt = conn
            .prepare(
                "SELECT name, kind, COALESCE(SUM(time_seconds),0) AS s
                 FROM app_usage WHERE enabled=1
                 GROUP BY name, kind ORDER BY s DESC LIMIT 20",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<AppTotal> = stmt
            .query_map([], |r| {
                Ok(AppTotal {
                    name: r.get::<_, String>(0)?,
                    kind: r.get::<_, String>(1)?,
                    seconds: r.get::<_, i64>(2)? as u64,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    Ok(AnalyticsPayload {
        projects,
        sessions,
        top_apps,
    })
}

// ── Runtime: чтение/запись проекта поверх SQLite ────────────────────────────

/// Удалить проект целиком (дочерние строки уходят по ON DELETE CASCADE).
pub fn delete_project(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM project WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ── Категории ───────────────────────────────────────────────────────────────

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, icon, ord, created_at, updated_at FROM category ORDER BY ord")
        .map_err(|e| e.to_string())?;
    let rows: Vec<Category> = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get::<_, String>(0)?,
                name: r.get::<_, String>(1)?,
                color: r.get::<_, String>(2)?,
                icon: r.get::<_, String>(3)?,
                order: r.get::<_, i64>(4)? as usize,
                created_at: r.get::<_, String>(5)?,
                updated_at: r.get::<_, String>(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn next_category_order(conn: &Connection) -> Result<usize, String> {
    let max: i64 = conn
        .query_row("SELECT COALESCE(MAX(ord), -1) FROM category", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok((max + 1) as usize)
}

pub fn insert_category(conn: &Connection, category: &Category) -> Result<(), String> {
    conn.execute(
        "INSERT INTO category(id, name, color, icon, ord, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            category.id,
            category.name,
            category.color,
            category.icon,
            category.order as i64,
            category.created_at,
            category.updated_at,
        ],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn update_category(
    conn: &Connection,
    id: &str,
    name: &str,
    color: &str,
    icon: &str,
    updated_at: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE category SET name = ?2, color = ?3, icon = ?4, updated_at = ?5 WHERE id = ?1",
        params![id, name, color, icon, updated_at],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn delete_category(conn: &Connection, id: &str) -> Result<(), String> {
    // Проекты этой категории обнулятся автоматически (ON DELETE SET NULL).
    conn.execute("DELETE FROM category WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn set_project_category(
    conn: &Connection,
    project_id: &str,
    category_id: Option<&str>,
    updated_at: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE project SET category_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![project_id, category_id, updated_at],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

// ── Правила автокатегоризации ───────────────────────────────────────────────

pub fn list_rules(conn: &Connection) -> Result<Vec<AppRule>, String> {
    let mut stmt = conn
        .prepare("SELECT id, match_process, category_id, created_at FROM app_rule ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let rows: Vec<AppRule> = stmt
        .query_map([], |r| {
            Ok(AppRule {
                id: r.get::<_, String>(0)?,
                match_process: r.get::<_, String>(1)?,
                category_id: r.get::<_, String>(2)?,
                created_at: r.get::<_, String>(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn insert_rule(conn: &Connection, rule: &AppRule) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_rule(id, match_process, category_id, created_at)
         VALUES(?1, ?2, ?3, ?4)",
        params![rule.id, rule.match_process, rule.category_id, rule.created_at],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

pub fn delete_rule(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM app_rule WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Подсказать категорию проекта: сумма времени приложений, чьи имена/процессы
/// подходят под правила, по категориям — берём категорию с максимумом.
pub fn suggest_category(conn: &Connection, project_id: &str) -> Result<Option<String>, String> {
    let rules = list_rules(conn)?;
    if rules.is_empty() {
        return Ok(None);
    }
    let apps: Vec<(String, String, i64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT process_name, name, time_seconds FROM app_usage
                 WHERE project_id = ?1 AND enabled = 1",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(String, String, i64)> = stmt
            .query_map(params![project_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let mut scores: HashMap<String, u64> = HashMap::new();
    for (proc_name, name, secs) in apps {
        let p = proc_name.to_lowercase();
        let n = name.to_lowercase();
        for rule in &rules {
            let m = rule.match_process.trim().to_lowercase();
            if !m.is_empty() && (p.contains(&m) || n.contains(&m)) {
                *scores.entry(rule.category_id.clone()).or_default() += secs.max(0) as u64;
            }
        }
    }
    Ok(scores.into_iter().max_by_key(|(_, v)| *v).map(|(k, _)| k))
}

/// Сохранить проект: строка проекта обновляется (категория и цвет
/// сохраняются), дочерние сущности перезаписываются целиком.
pub fn save_project(conn: &mut Connection, project: &ProjectRecord) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO project(id, name, client, note, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           client = excluded.client,
           note = excluded.note,
           updated_at = excluded.updated_at",
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

    for table in ["stage", "session", "app_usage", "project_selected_stage"] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE project_id = ?1"),
            params![project.id],
        )
        .map_err(|e| e.to_string())?;
    }

    let mut discard = MigrationReport::default();
    insert_children(&tx, project, &mut discard)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_projects(conn: &Connection) -> Result<Vec<ProjectRecord>, String> {
    let ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM project ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
    };
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(project) = load_project(conn, &id)? {
            out.push(project);
        }
    }
    Ok(out)
}

pub fn load_project(conn: &Connection, id: &str) -> Result<Option<ProjectRecord>, String> {
    let base = conn
        .query_row(
            "SELECT name, client, note, created_at, updated_at, category_id, color
             FROM project WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let Some((name, client, note, created_at, updated_at, category_id, color)) = base else {
        return Ok(None);
    };

    let mut project = ProjectRecord {
        id: id.to_string(),
        name,
        client,
        note,
        created_at,
        updated_at,
        sessions: load_sessions(conn, id)?,
        apps: load_apps(conn, id)?,
        selected_stage_ids: load_selected_stage_ids(conn, id)?,
        stages: load_stages(conn, id)?,
        category_id,
        color,
    };
    crate::storage::normalize_project_structure(&mut project);
    Ok(Some(project))
}

fn load_selected_stage_ids(conn: &Connection, project_id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT stage_id FROM project_selected_stage WHERE project_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![project_id], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

fn load_sessions(conn: &Connection, project_id: &str) -> Result<Vec<SessionRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, stopped_at, duration_seconds, app_count, browser_count
             FROM session WHERE project_id = ?1 ORDER BY started_at",
        )
        .map_err(|e| e.to_string())?;
    let raw: Vec<SessionRecord> = stmt
        .query_map(params![project_id], |r| {
            Ok(SessionRecord {
                id: r.get::<_, String>(0)?,
                started_at: r.get::<_, String>(1)?,
                stopped_at: r.get::<_, Option<String>>(2)?,
                duration_seconds: r.get::<_, i64>(3)? as u64,
                app_count: r.get::<_, i64>(4)? as usize,
                browser_count: r.get::<_, i64>(5)? as usize,
                stages: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut sessions = Vec::with_capacity(raw.len());
    for mut session in raw {
        let mut stmt = conn
            .prepare("SELECT stage_id, name FROM session_stage WHERE session_id = ?1")
            .map_err(|e| e.to_string())?;
        session.stages = stmt
            .query_map(params![session.id], |r| {
                Ok(SessionStageSnapshot {
                    id: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    name: r.get::<_, String>(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        sessions.push(session);
    }
    Ok(sessions)
}

fn load_apps(conn: &Connection, project_id: &str) -> Result<Vec<AppUsageRecord>, String> {
    let raw: Vec<(i64, AppUsageRecord)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, key, name, process_name, process_path, icon_data_url, kind, enabled, time_seconds
                 FROM app_usage WHERE project_id = ?1 ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, AppUsageRecord)> = stmt
            .query_map(params![project_id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    AppUsageRecord {
                        key: r.get::<_, String>(1)?,
                        name: r.get::<_, String>(2)?,
                        process_name: r.get::<_, String>(3)?,
                        process_path: r.get::<_, String>(4)?,
                        icon_data_url: r.get::<_, Option<String>>(5)?,
                        kind: r.get::<_, String>(6)?,
                        enabled: r.get::<_, i64>(7)? != 0,
                        time_seconds: r.get::<_, i64>(8)? as u64,
                        tabs: Vec::new(),
                    },
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let mut apps = Vec::with_capacity(raw.len());
    for (app_id, mut app) in raw {
        app.tabs = load_tabs(conn, app_id)?;
        apps.push(app);
    }
    Ok(apps)
}

fn load_tabs(conn: &Connection, app_id: i64) -> Result<Vec<TabUsageRecord>, String> {
    let raw: Vec<(i64, TabUsageRecord)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, key, title, url, favicon_url, enabled, time_seconds
                 FROM tab_usage WHERE app_id = ?1 ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, TabUsageRecord)> = stmt
            .query_map(params![app_id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    TabUsageRecord {
                        key: r.get::<_, String>(1)?,
                        title: r.get::<_, String>(2)?,
                        url: r.get::<_, Option<String>>(3)?,
                        urls: Vec::new(),
                        favicon_url: r.get::<_, Option<String>>(4)?,
                        enabled: r.get::<_, i64>(5)? != 0,
                        time_seconds: r.get::<_, i64>(6)? as u64,
                    },
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let mut tabs = Vec::with_capacity(raw.len());
    for (tab_id, mut tab) in raw {
        let mut stmt = conn
            .prepare(
                "SELECT url, title, last_seen_at, hits, enabled, time_seconds
                 FROM visited_url WHERE tab_id = ?1 ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        tab.urls = stmt
            .query_map(params![tab_id], |r| {
                Ok(VisitedUrlRecord {
                    url: r.get::<_, String>(0)?,
                    title: r.get::<_, String>(1)?,
                    last_seen_at: r.get::<_, String>(2)?,
                    hits: r.get::<_, i64>(3)? as u64,
                    enabled: r.get::<_, i64>(4)? != 0,
                    time_seconds: r.get::<_, i64>(5)? as u64,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        tabs.push(tab);
    }
    Ok(tabs)
}

fn load_stages(conn: &Connection, project_id: &str) -> Result<Vec<ProjectStageRecord>, String> {
    let raw: Vec<ProjectStageRecord> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, ord, created_at, updated_at
                 FROM stage WHERE project_id = ?1 ORDER BY ord",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<ProjectStageRecord> = stmt
            .query_map(params![project_id], |r| {
                Ok(ProjectStageRecord {
                    id: r.get::<_, String>(0)?,
                    name: r.get::<_, String>(1)?,
                    order: r.get::<_, i64>(2)? as usize,
                    created_at: r.get::<_, String>(3)?,
                    updated_at: r.get::<_, String>(4)?,
                    apps: Vec::new(),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let mut stages = Vec::with_capacity(raw.len());
    for mut stage in raw {
        stage.apps = load_stage_apps(conn, &stage.id)?;
        stages.push(stage);
    }
    Ok(stages)
}

fn load_stage_apps(conn: &Connection, stage_id: &str) -> Result<Vec<StageAppRecord>, String> {
    let mut apps: Vec<StageAppRecord> = {
        let mut stmt = conn
            .prepare("SELECT app_key, enabled FROM stage_app WHERE stage_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows: Vec<StageAppRecord> = stmt
            .query_map(params![stage_id], |r| {
                Ok(StageAppRecord {
                    app_key: r.get::<_, String>(0)?,
                    enabled: r.get::<_, i64>(1)? != 0,
                    tabs: Vec::new(),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    // Вкладки этапа
    {
        let mut stmt = conn
            .prepare("SELECT app_key, tab_key, enabled FROM stage_tab WHERE stage_id = ?1")
            .map_err(|e| e.to_string())?;
        let tabs: Vec<(String, StageTabRecord)> = stmt
            .query_map(params![stage_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    StageTabRecord {
                        tab_key: r.get::<_, String>(1)?,
                        enabled: r.get::<_, i64>(2)? != 0,
                        urls: Vec::new(),
                    },
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (app_key, tab) in tabs {
            if let Some(app) = apps.iter_mut().find(|a| a.app_key == app_key) {
                app.tabs.push(tab);
            }
        }
    }

    // Ссылки этапа
    {
        let mut stmt = conn
            .prepare("SELECT app_key, tab_key, url, enabled FROM stage_url WHERE stage_id = ?1")
            .map_err(|e| e.to_string())?;
        let urls: Vec<(String, String, StageUrlRecord)> = stmt
            .query_map(params![stage_id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    StageUrlRecord {
                        url: r.get::<_, String>(2)?,
                        enabled: r.get::<_, i64>(3)? != 0,
                    },
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (app_key, tab_key, url) in urls {
            if let Some(app) = apps.iter_mut().find(|a| a.app_key == app_key) {
                if let Some(tab) = app.tabs.iter_mut().find(|t| t.tab_key == tab_key) {
                    tab.urls.push(url);
                }
            }
        }
    }

    Ok(apps)
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
            db_file: tmp.join("ptm.db"),
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

    // Полный рантайм-путь запуска приложения на файловой базе:
    // ensure_storage → migrate_legacy_if_needed → load_store.
    #[test]
    fn startup_path_migrates_and_loads() {
        let real = PathBuf::from("/Users/rimzotik/Downloads/Project Time Manager-datas/data");
        if !real.exists() {
            eprintln!("SKIP: реальные данные не найдены");
            return;
        }
        let tmp = std::env::temp_dir().join(format!("ptm_startup_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        copy_dir_all(&real, &tmp);
        let paths = StoragePaths {
            projects_dir: tmp.join("Проекты"),
            workspace_file: tmp.join("workspace.json"),
            db_file: tmp.join("ptm.db"),
        };

        // Как в main(): создаём базу, мигрируем legacy, читаем store.
        storage::ensure_storage(&paths).unwrap();
        storage::migrate_legacy_if_needed(&paths).unwrap();
        let store = storage::load_store(&paths).unwrap();

        assert_eq!(store.projects.len(), 1, "проектов в store");
        let project = &store.projects[0];
        assert_eq!(project.sessions.len(), 14);
        assert_eq!(project.apps.len(), 41);
        assert_eq!(project.stages.len(), 9);
        assert_eq!(store.workspace.language, "ru");

        // Повторный вызов не должен дублировать данные (idempotent).
        storage::migrate_legacy_if_needed(&paths).unwrap();
        let store2 = storage::load_store(&paths).unwrap();
        assert_eq!(store2.projects.len(), 1, "миграция не идемпотентна");
        assert_eq!(store2.projects[0].apps.len(), 41);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn roundtrip_load_save_load() {
        let real = PathBuf::from("/Users/rimzotik/Downloads/Project Time Manager-datas/data");
        if !real.exists() {
            eprintln!("SKIP: реальные данные не найдены");
            return;
        }
        let tmp = std::env::temp_dir().join(format!("ptm_roundtrip_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        copy_dir_all(&real, &tmp);
        let paths = StoragePaths {
            projects_dir: tmp.join("Проекты"),
            workspace_file: tmp.join("workspace.json"),
            db_file: tmp.join("ptm.db"),
        };

        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        migrate_from_json(&mut conn, &paths).unwrap();

        // Реконструкция проекта из базы
        let projects = list_projects(&conn).unwrap();
        assert_eq!(projects.len(), 1);
        let project = projects.into_iter().next().unwrap();
        let count_urls = |p: &ProjectRecord| -> usize {
            p.apps.iter().flat_map(|a| a.tabs.iter()).map(|t| t.urls.len()).sum()
        };
        let sessions0 = project.sessions.len();
        let apps0 = project.apps.len();
        let tabs0: usize = project.apps.iter().map(|a| a.tabs.len()).sum();
        let urls0 = count_urls(&project);
        let dur0: u64 = project.sessions.iter().map(|s| s.duration_seconds).sum();
        let stages0 = project.stages.len();
        assert_eq!(sessions0, 14);
        assert_eq!(apps0, 41);
        assert_eq!(dur0, 229_946);

        // Пересохранение (delete+insert) не должно терять данные
        let id = project.id.clone();
        save_project(&mut conn, &project).unwrap();
        let reloaded = load_project(&conn, &id).unwrap().unwrap();
        assert_eq!(reloaded.sessions.len(), sessions0, "сессии после re-save");
        assert_eq!(reloaded.apps.len(), apps0, "приложения после re-save");
        assert_eq!(reloaded.apps.iter().map(|a| a.tabs.len()).sum::<usize>(), tabs0, "вкладки");
        assert_eq!(count_urls(&reloaded), urls0, "ссылки");
        assert_eq!(reloaded.stages.len(), stages0, "этапы");
        assert_eq!(
            reloaded.sessions.iter().map(|s| s.duration_seconds).sum::<u64>(),
            dur0,
            "суммарное время"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn categories_crud_and_assignment() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO project(id, name, client, note, created_at, updated_at)
             VALUES('p1', 'Проект', '', '', 't', 't')",
            [],
        )
        .unwrap();

        let cat = Category {
            id: "c1".into(),
            name: "Монтаж".into(),
            color: "#059669".into(),
            icon: "🎬".into(),
            order: next_category_order(&conn).unwrap(),
            created_at: "t".into(),
            updated_at: "t".into(),
        };
        insert_category(&conn, &cat).unwrap();
        assert_eq!(list_categories(&conn).unwrap().len(), 1);

        set_project_category(&conn, "p1", Some("c1"), "t2").unwrap();
        let p = load_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(p.category_id.as_deref(), Some("c1"), "категория привязана");

        // Удаление категории обнуляет её у проектов (ON DELETE SET NULL).
        delete_category(&conn, "c1").unwrap();
        assert_eq!(list_categories(&conn).unwrap().len(), 0);
        let p2 = load_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(p2.category_id, None, "категория обнулена после удаления");
    }
}
