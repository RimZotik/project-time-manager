// TODO(phase3): почистить legacy-хелперы JSON и экспорт; пока часть функций
// остаётся ради миграции и отчётов — глушим предупреждения переходного периода.
#![allow(dead_code)]

use crate::models::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub projects_dir: PathBuf,
    pub workspace_file: PathBuf,
    pub db_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIndex {
    pub selected_project_id: Option<String>,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "ru".to_string()
}

impl Default for WorkspaceIndex {
    fn default() -> Self {
        Self {
            selected_project_id: None,
            autostart: false,
            language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StoreData {
    pub workspace: WorkspaceIndex,
    pub projects: Vec<ProjectRecord>,
    pub categories: Vec<Category>,
}

pub fn storage_paths() -> Result<StoragePaths, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let root = exe
        .parent()
        .map(|parent| parent.join("data"))
        .unwrap_or_else(|| PathBuf::from("data"));
    let projects_dir = root.join("Проекты");
    let workspace_file = root.join("workspace.json");
    let db_file = root.join("ptm.db");
    Ok(StoragePaths {
        projects_dir,
        workspace_file,
        db_file,
    })
}

/// Гарантирует наличие базы и схемы. Вызывается перед любой записью.
pub fn ensure_storage(paths: &StoragePaths) -> Result<(), String> {
    if let Some(parent) = paths.db_file.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    crate::db::connect(&paths.db_file).map(|_| ())
}

/// При первом запуске: если база пуста, но рядом лежат старые JSON-данные —
/// переносим их в SQLite. Вызывается один раз из main().
pub fn migrate_legacy_if_needed(paths: &StoragePaths) -> Result<(), String> {
    let mut conn = crate::db::connect(&paths.db_file)?;
    if crate::db::is_populated(&conn)? {
        return Ok(());
    }
    let has_projects = paths.projects_dir.exists()
        && fs::read_dir(&paths.projects_dir)
            .map(|mut dir| dir.next().is_some())
            .unwrap_or(false);
    if !has_projects && !paths.workspace_file.exists() {
        return Ok(());
    }
    let report = crate::db::migrate_from_json(&mut conn, paths)?;
    eprintln!("Миграция из JSON выполнена: {report:?}");
    Ok(())
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Legacy: чтение старого workspace.json (используется только миграцией).
pub fn load_workspace_json(paths: &StoragePaths) -> Result<WorkspaceIndex, String> {
    if !paths.workspace_file.exists() {
        return Ok(WorkspaceIndex::default());
    }

    let content = fs::read_to_string(&paths.workspace_file).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

pub fn load_workspace(paths: &StoragePaths) -> Result<WorkspaceIndex, String> {
    let conn = crate::db::connect(&paths.db_file)?;
    Ok(WorkspaceIndex {
        selected_project_id: crate::db::get_setting(&conn, "selected_project_id")?,
        autostart: crate::db::get_setting(&conn, "autostart")?.as_deref() == Some("1"),
        language: crate::db::get_setting(&conn, "language")?.unwrap_or_else(default_language),
    })
}

pub fn save_workspace(paths: &StoragePaths, workspace: &WorkspaceIndex) -> Result<(), String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::set_setting(&conn, "language", &workspace.language)?;
    crate::db::set_setting(
        &conn,
        "autostart",
        if workspace.autostart { "1" } else { "0" },
    )?;
    match &workspace.selected_project_id {
        Some(id) => crate::db::set_setting(&conn, "selected_project_id", id)?,
        None => crate::db::clear_setting(&conn, "selected_project_id")?,
    }
    Ok(())
}

pub fn list_project_files(paths: &StoragePaths) -> Result<Vec<ProjectRecord>, String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::list_projects(&conn)
}

/// Legacy: чтение старых project.json (используется только миграцией).
pub fn list_project_files_json(paths: &StoragePaths) -> Result<Vec<ProjectRecord>, String> {
    if !paths.projects_dir.exists() {
        return Ok(Vec::new());
    }
    let mut projects = Vec::new();

    for entry in fs::read_dir(&paths.projects_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let project_file = path.join("project.json");
            if project_file.exists() {
                let content = fs::read_to_string(&project_file).map_err(|err| err.to_string())?;
                if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                    let mut project = project;
                    normalize_project_structure(&mut project);
                    projects.push(project);
                }
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                let mut project = project;
                normalize_project_structure(&mut project);
                projects.push(project);
            }
        }
    }

    Ok(sort_and_dedupe_projects(projects))
}

pub fn load_store(paths: &StoragePaths) -> Result<StoreData, String> {
    Ok(StoreData {
        workspace: load_workspace(paths)?,
        projects: list_project_files(paths)?,
        categories: list_categories(paths)?,
    })
}

// ── Категории (обёртки над db) ──────────────────────────────────────────────

pub fn list_categories(paths: &StoragePaths) -> Result<Vec<Category>, String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::list_categories(&conn)
}

pub fn create_category(
    paths: &StoragePaths,
    name: &str,
    color: &str,
    icon: &str,
) -> Result<Category, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Название категории не может быть пустым.".to_string());
    }
    let conn = crate::db::connect(&paths.db_file)?;
    let category = Category {
        id: Uuid::new_v4().to_string(),
        name: trimmed.to_string(),
        color: color.to_string(),
        icon: icon.to_string(),
        order: crate::db::next_category_order(&conn)?,
        created_at: now_iso(),
        updated_at: now_iso(),
    };
    crate::db::insert_category(&conn, &category)?;
    Ok(category)
}

pub fn update_category(
    paths: &StoragePaths,
    id: &str,
    name: &str,
    color: &str,
    icon: &str,
) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Название категории не может быть пустым.".to_string());
    }
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::update_category(&conn, id, trimmed, color, icon, &now_iso())
}

pub fn delete_category(paths: &StoragePaths, id: &str) -> Result<(), String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::delete_category(&conn, id)
}

pub fn set_project_category(
    paths: &StoragePaths,
    project_id: &str,
    category_id: Option<&str>,
) -> Result<ProjectRecord, String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::set_project_category(&conn, project_id, category_id, &now_iso())?;
    drop(conn);
    load_project(paths, project_id)
}

pub fn project_path(paths: &StoragePaths, project: &ProjectRecord) -> PathBuf {
    project_dir(paths, project).join("project.json")
}

pub fn project_dir(paths: &StoragePaths, project: &ProjectRecord) -> PathBuf {
    paths.projects_dir.join(project_file_stem(project))
}

pub fn project_report_path(paths: &StoragePaths, project: &ProjectRecord) -> PathBuf {
    project_dir(paths, project).join(format!("{}.xlsx", project_file_stem(project)))
}

pub fn project_report_pdf_path(paths: &StoragePaths, project: &ProjectRecord) -> PathBuf {
    project_dir(paths, project).join(format!("{}.pdf", project_file_stem(project)))
}

pub fn reserve_report_path(path: &Path) -> PathBuf {
    if path.exists() && !can_write_target(path) {
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("report");
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("bin");
        let stamp = Utc::now().format("%Y%m%d-%H%M%S");
        let fallback = path.with_file_name(format!("{stem}-{stamp}.{ext}"));
        if let Some(parent) = fallback.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fallback
    } else {
        path.to_path_buf()
    }
}

pub fn project_path_by_id(paths: &StoragePaths, project_id: &str) -> Result<PathBuf, String> {
    for entry in fs::read_dir(&paths.projects_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            let project_file = path.join("project.json");
            if project_file.exists() {
                let content = fs::read_to_string(&project_file).map_err(|err| err.to_string())?;
                if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                    if project.id == project_id {
                        return Ok(project_file);
                    }
                }
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                if project.id == project_id {
                    return Ok(path);
                }
            }
        }
    }

    Ok(paths.projects_dir.join(format!("{project_id}.json")))
}

pub fn save_project(paths: &StoragePaths, project: &ProjectRecord) -> Result<(), String> {
    ensure_storage(paths)?;
    let mut conn = crate::db::connect(&paths.db_file)?;
    crate::db::save_project(&mut conn, project)
}

pub fn rename_project_record(
    paths: &StoragePaths,
    project_id: &str,
    new_name: &str,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err("Название проекта не может быть пустым.".to_string());
    }
    project.name = trimmed.to_string();
    project.updated_at = now_iso();
    save_project(paths, &project)?;
    Ok(project)
}

pub fn delete_project_record(paths: &StoragePaths, project_id: &str) -> Result<(), String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::delete_project(&conn, project_id)
}

pub fn load_project(paths: &StoragePaths, project_id: &str) -> Result<ProjectRecord, String> {
    let conn = crate::db::connect(&paths.db_file)?;
    crate::db::load_project(&conn, project_id)?.ok_or_else(|| "Project not found".to_string())
}

pub fn create_project(
    paths: &StoragePaths,
    name: &str,
    _client: &str,
) -> Result<ProjectRecord, String> {
    let mut project = ProjectRecord {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        client: String::new(),
        note: String::new(),
        created_at: now_iso(),
        updated_at: now_iso(),
        sessions: Vec::new(),
        apps: Vec::new(),
        selected_stage_ids: Vec::new(),
        stages: Vec::new(),
        category_id: None,
        color: None,
    };
    normalize_project_structure(&mut project);
    save_project(paths, &project)?;
    Ok(project)
}

pub fn import_project_from_json(
    paths: &StoragePaths,
    json_text: &str,
) -> Result<ProjectRecord, String> {
    let mut project: ProjectRecord =
        serde_json::from_str(json_text).map_err(|err| err.to_string())?;
    if project.id.trim().is_empty() {
        project.id = Uuid::new_v4().to_string();
    }
    if project.created_at.trim().is_empty() {
        project.created_at = now_iso();
    }
    project.updated_at = now_iso();
    normalize_project_structure(&mut project);
    save_project(paths, &project)?;
    Ok(project)
}

pub fn toggle_app(
    paths: &StoragePaths,
    project_id: &str,
    app_key: &str,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    if let Some(app) = project.apps.iter_mut().find(|item| item.key == app_key) {
        app.enabled = enabled;
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn toggle_tab(
    paths: &StoragePaths,
    project_id: &str,
    app_key: &str,
    tab_key: &str,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    if let Some(app) = project.apps.iter_mut().find(|item| item.key == app_key) {
        if let Some(tab) = app.tabs.iter_mut().find(|item| item.key == tab_key) {
            tab.enabled = enabled;
            project.updated_at = now_iso();
            normalize_project_structure(&mut project);
            save_project(paths, &project)?;
        }
    }
    Ok(project)
}

pub fn touch_app_time(
    project: &mut ProjectRecord,
    observation: &WindowObservation,
    seconds: u64,
) -> usize {
    let app_name = friendly_app_name(
        &observation.process_name,
        &observation.process_path,
        observation.browser_name.as_deref(),
    );
    let kind = if observation.browser_name.is_some() {
        "browser"
    } else {
        "app"
    }
    .to_string();

    let index = if let Some(index) = project
        .apps
        .iter()
        .position(|item| item.key == observation.process_name)
    {
        index
    } else {
        project.apps.push(AppUsageRecord {
            key: observation.process_name.clone(),
            name: app_name,
            process_name: observation.process_name.clone(),
            process_path: observation.process_path.clone(),
            icon_data_url: observation.icon_data_url.clone(),
            kind,
            enabled: !is_own_app_process(&observation.process_name),
            time_seconds: 0,
            tabs: Vec::new(),
        });
        project.apps.len() - 1
    };

    if let Some(app) = project.apps.get_mut(index) {
        app.time_seconds = app.time_seconds.saturating_add(seconds);
        if !observation.process_path.is_empty()
            && (app.process_path.is_empty() || app.process_path != observation.process_path)
        {
            app.process_path = observation.process_path.clone();
        }
        if app.icon_data_url.is_none() {
            app.icon_data_url = observation.icon_data_url.clone();
        }
        app.name = friendly_app_name(
            &observation.process_name,
            &observation.process_path,
            observation.browser_name.as_deref(),
        );
        app.kind = if observation.browser_name.is_some() {
            "browser".to_string()
        } else {
            "app".to_string()
        };
    }

    index
}

pub fn touch_tab_time(
    project: &mut ProjectRecord,
    app_index: usize,
    observation: &WindowObservation,
    seconds: u64,
) {
    let Some(app) = project.apps.get_mut(app_index) else {
        return;
    };

    let title = observation
        .tab_title
        .clone()
        .unwrap_or_else(|| observation.window_title.clone());
    let domain = observation
        .url
        .as_deref()
        .and_then(extract_domain)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| title.trim().to_string());
    let tab_key = format!(
        "{}::{}",
        domain.to_lowercase(),
        observation.browser_name.clone().unwrap_or_default()
    );

    let index = if let Some(index) = app.tabs.iter().position(|item| item.key == tab_key) {
        index
    } else {
        app.tabs.push(TabUsageRecord {
            key: tab_key.clone(),
            title: domain.clone(),
            url: observation.url.clone(),
            urls: Vec::new(),
            favicon_url: observation.favicon_url.clone(),
            enabled: true,
            time_seconds: 0,
        });
        app.tabs.len() - 1
    };

    if let Some(tab) = app.tabs.get_mut(index) {
        tab.time_seconds = tab.time_seconds.saturating_add(seconds);
        if !domain.trim().is_empty() && tab.title != domain {
            tab.title = domain;
        }
        if tab.url.is_none() {
            tab.url = observation.url.clone();
        }
        if tab.favicon_url.is_none() {
            tab.favicon_url = observation.favicon_url.clone();
        }
        if let Some(url) = observation
            .url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            touch_url_history(tab, url, &title);
        }
    }
}

fn touch_url_history(tab: &mut TabUsageRecord, url: &str, title: &str) {
    let now = now_iso();
    let clean_title = title.trim();
    if let Some(entry) = tab.urls.iter_mut().find(|item| item.url == url) {
        entry.hits = entry.hits.saturating_add(1);
        entry.last_seen_at = now;
        entry.time_seconds = entry.time_seconds.saturating_add(1);
        if !clean_title.is_empty() {
            entry.title = clean_title.to_string();
        }
        return;
    }

    tab.urls.push(VisitedUrlRecord {
        url: url.to_string(),
        title: if clean_title.is_empty() {
            url.to_string()
        } else {
            clean_title.to_string()
        },
        last_seen_at: now,
        hits: 1,
        enabled: true,
        time_seconds: 1,
    });
}

pub fn set_app_time(
    paths: &StoragePaths,
    project_id: &str,
    app_key: &str,
    seconds: u64,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    if let Some(app) = project.apps.iter_mut().find(|item| item.key == app_key) {
        set_app_time_in_app(app, seconds);
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn set_tab_time(
    paths: &StoragePaths,
    project_id: &str,
    app_key: &str,
    tab_key: &str,
    seconds: u64,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    if let Some(app) = project.apps.iter_mut().find(|item| item.key == app_key) {
        if let Some(tab) = app.tabs.iter_mut().find(|item| item.key == tab_key) {
            set_tab_time_in_project(tab, seconds);
            if app.kind == "browser" {
                app.time_seconds = app.tabs.iter().map(|item| item.time_seconds).sum();
            }
            project.updated_at = now_iso();
            normalize_project_structure(&mut project);
            save_project(paths, &project)?;
        }
    }
    Ok(project)
}

pub fn toggle_url(
    paths: &StoragePaths,
    project_id: &str,
    app_key: &str,
    tab_key: &str,
    url: &str,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    if let Some(app) = project.apps.iter_mut().find(|item| item.key == app_key) {
        if let Some(tab) = app.tabs.iter_mut().find(|item| item.key == tab_key) {
            if let Some(item) = tab.urls.iter_mut().find(|item| item.url == url) {
                item.enabled = enabled;
                project.updated_at = now_iso();
                normalize_project_structure(&mut project);
                save_project(paths, &project)?;
            }
        }
    }
    Ok(project)
}

pub fn create_stage(paths: &StoragePaths, project_id: &str, name: &str) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Название этапа не может быть пустым.".to_string());
    }
    project.stages.push(ProjectStageRecord {
        id: Uuid::new_v4().to_string(),
        name: trimmed.to_string(),
        order: project.stages.len(),
        created_at: now_iso(),
        updated_at: now_iso(),
        apps: Vec::new(),
    });
    project.updated_at = now_iso();
    normalize_project_structure(&mut project);
    save_project(paths, &project)?;
    Ok(project)
}

pub fn set_selected_stages(
    paths: &StoragePaths,
    project_id: &str,
    stage_ids: &[String],
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    project.selected_stage_ids = stage_ids
        .iter()
        .filter(|stage_id| project.stages.iter().any(|stage| stage.id == stage_id.as_str()))
        .cloned()
        .collect();
    project.updated_at = now_iso();
    normalize_project_structure(&mut project);
    save_project(paths, &project)?;
    Ok(project)
}

pub fn rename_stage(
    paths: &StoragePaths,
    project_id: &str,
    stage_id: &str,
    name: &str,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Название этапа не может быть пустым.".to_string());
    }
    if let Some(stage) = project.stages.iter_mut().find(|item| item.id == stage_id) {
        stage.name = trimmed.to_string();
        stage.updated_at = now_iso();
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn delete_stage(
    paths: &StoragePaths,
    project_id: &str,
    stage_id: &str,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    project.stages.retain(|stage| stage.id != stage_id);
    project.selected_stage_ids.retain(|item| item != stage_id);
    for (index, stage) in project.stages.iter_mut().enumerate() {
        stage.order = index;
        stage.updated_at = now_iso();
    }
    project.updated_at = now_iso();
    normalize_project_structure(&mut project);
    save_project(paths, &project)?;
    Ok(project)
}

pub fn reorder_stage(
    paths: &StoragePaths,
    project_id: &str,
    stage_id: &str,
    direction: i32,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let Some(index) = project.stages.iter().position(|item| item.id == stage_id) else {
        return Ok(project);
    };

    let new_index = if direction < 0 {
        index.saturating_sub(1)
    } else if direction > 0 {
        (index + 1).min(project.stages.len().saturating_sub(1))
    } else {
        index
    };

    if new_index != index {
        project.stages.swap(index, new_index);
        for (order, stage) in project.stages.iter_mut().enumerate() {
            stage.order = order;
            stage.updated_at = now_iso();
        }
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn toggle_stage_app(
    paths: &StoragePaths,
    project_id: &str,
    stage_id: &str,
    app_key: &str,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let global_enabled = project
        .apps
        .iter()
        .find(|item| item.key == app_key)
        .map(|app| app.enabled)
        .unwrap_or(false);
    if let Some(stage) = project.stages.iter_mut().find(|item| item.id == stage_id) {
        let app = ensure_stage_app(stage, app_key);
        app.enabled = enabled && global_enabled;
        stage.updated_at = now_iso();
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn toggle_stage_tab(
    paths: &StoragePaths,
    project_id: &str,
    stage_id: &str,
    app_key: &str,
    tab_key: &str,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let global_enabled = project
        .apps
        .iter()
        .find(|item| item.key == app_key)
        .and_then(|app| app.tabs.iter().find(|item| item.key == tab_key))
        .map(|tab| tab.enabled)
        .unwrap_or(false);
    if let Some(stage) = project.stages.iter_mut().find(|item| item.id == stage_id) {
        let app = ensure_stage_app(stage, app_key);
        let tab = ensure_stage_tab(app, tab_key);
        tab.enabled = enabled && global_enabled;
        stage.updated_at = now_iso();
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn toggle_stage_url(
    paths: &StoragePaths,
    project_id: &str,
    stage_id: &str,
    app_key: &str,
    tab_key: &str,
    url: &str,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let mut project = load_project(paths, project_id)?;
    let global_enabled = project
        .apps
        .iter()
        .find(|item| item.key == app_key)
        .and_then(|app| app.tabs.iter().find(|item| item.key == tab_key))
        .and_then(|tab| tab.urls.iter().find(|item| item.url == url))
        .map(|link| link.enabled)
        .unwrap_or(false);
    if let Some(stage) = project.stages.iter_mut().find(|item| item.id == stage_id) {
        let app = ensure_stage_app(stage, app_key);
        let tab = ensure_stage_tab(app, tab_key);
        let link = ensure_stage_url(tab, url);
        link.enabled = enabled && global_enabled;
        stage.updated_at = now_iso();
        project.updated_at = now_iso();
        normalize_project_structure(&mut project);
        save_project(paths, &project)?;
    }
    Ok(project)
}

pub fn normalize_project_structure(project: &mut ProjectRecord) {
    for app in &mut project.apps {
        for tab in &mut app.tabs {
            if tab.urls.is_empty() {
                continue;
            }
            for url in &mut tab.urls {
                if url.url.trim().is_empty() {
                    url.url = url.title.clone();
                }
                if url.title.trim().is_empty() {
                    url.title = url.url.clone();
                }
            }
        }
    }

    project
        .stages
        .sort_by(|left, right| left.order.cmp(&right.order).then(left.created_at.cmp(&right.created_at)));
    project
        .selected_stage_ids
        .retain(|stage_id| project.stages.iter().any(|stage| stage.id == stage_id.as_str()));
    project.selected_stage_ids.dedup();
    let project_apps = project.apps.clone();
    for (index, stage) in project.stages.iter_mut().enumerate() {
        stage.order = index;
        sync_stage_layout(stage, &project_apps);
    }
}

fn sync_stage_layout(stage: &mut ProjectStageRecord, apps: &[AppUsageRecord]) {
    stage.apps.retain(|item| apps.iter().any(|app| app.key == item.app_key));

    for app in apps {
        let stage_app = ensure_stage_app(stage, &app.key);
        if !app.enabled {
            stage_app.enabled = false;
        }
        stage_app.tabs.retain(|item| app.tabs.iter().any(|tab| tab.key == item.tab_key));
        for tab in &app.tabs {
            let stage_tab = ensure_stage_tab(stage_app, &tab.key);
            if !tab.enabled {
                stage_tab.enabled = false;
            }
            stage_tab.urls.retain(|item| tab.urls.iter().any(|url| url.url == item.url));
            for url in &tab.urls {
                let stage_url = ensure_stage_url(stage_tab, &url.url);
                if !url.enabled {
                    stage_url.enabled = false;
                }
            }
        }
    }
}

fn ensure_stage_app<'a>(
    stage: &'a mut ProjectStageRecord,
    app_key: &str,
) -> &'a mut StageAppRecord {
    if let Some(index) = stage.apps.iter().position(|item| item.app_key == app_key) {
        return stage.apps.get_mut(index).expect("stage app exists");
    }
    stage.apps.push(StageAppRecord {
        app_key: app_key.to_string(),
        enabled: true,
        tabs: Vec::new(),
    });
    stage.apps.last_mut().expect("stage app inserted")
}

fn ensure_stage_tab<'a>(app: &'a mut StageAppRecord, tab_key: &str) -> &'a mut StageTabRecord {
    if let Some(index) = app.tabs.iter().position(|item| item.tab_key == tab_key) {
        return app.tabs.get_mut(index).expect("stage tab exists");
    }
    app.tabs.push(StageTabRecord {
        tab_key: tab_key.to_string(),
        enabled: true,
        urls: Vec::new(),
    });
    app.tabs.last_mut().expect("stage tab inserted")
}

fn ensure_stage_url<'a>(tab: &'a mut StageTabRecord, url: &str) -> &'a mut StageUrlRecord {
    if let Some(index) = tab.urls.iter().position(|item| item.url == url) {
        return tab.urls.get_mut(index).expect("stage url exists");
    }
    tab.urls.push(StageUrlRecord {
        url: url.to_string(),
        enabled: true,
    });
    tab.urls.last_mut().expect("stage url inserted")
}

fn set_app_time_in_app(app: &mut AppUsageRecord, seconds: u64) {
    app.time_seconds = seconds;
    if app.kind == "browser" {
        let total = app.tabs.iter().map(|tab| tab.time_seconds).sum::<u64>();
        if total == 0 {
            let tab_count = app.tabs.len() as u64;
            if tab_count > 0 {
                let base = seconds / tab_count;
                let mut remainder = seconds % tab_count;
                for tab in &mut app.tabs {
                    let value = base + u64::from(remainder > 0);
                    tab.time_seconds = value;
                    redistribute_url_times(tab, value);
                    if remainder > 0 {
                        remainder -= 1;
                    }
                }
            }
        } else {
            let mut assigned = 0u64;
            let last_index = app.tabs.len().saturating_sub(1);
            for (index, tab) in app.tabs.iter_mut().enumerate() {
                let value = if index == last_index {
                    seconds.saturating_sub(assigned)
                } else {
                    ((tab.time_seconds as f64 / total as f64) * seconds as f64).round() as u64
                };
                tab.time_seconds = value;
                redistribute_url_times(tab, value);
                assigned = assigned.saturating_add(value);
            }
        }
    }
}

fn set_tab_time_in_project(tab: &mut TabUsageRecord, seconds: u64) {
    tab.time_seconds = seconds;
    redistribute_url_times(tab, seconds);
}

fn redistribute_url_times(tab: &mut TabUsageRecord, seconds: u64) {
    if tab.urls.is_empty() {
        return;
    }

    let total = tab.urls.iter().map(|url| url.time_seconds).sum::<u64>();
    if total == 0 {
        let url_count = tab.urls.len() as u64;
        if url_count == 0 {
            return;
        }
        let base = seconds / url_count;
        let mut remainder = seconds % url_count;
        for item in &mut tab.urls {
            let value = base + u64::from(remainder > 0);
            item.time_seconds = value;
            if remainder > 0 {
                remainder -= 1;
            }
        }
        return;
    }

    let mut assigned = 0u64;
    let last_index = tab.urls.len().saturating_sub(1);
    for (index, item) in tab.urls.iter_mut().enumerate() {
        let value = if index == last_index {
            seconds.saturating_sub(assigned)
        } else {
            ((item.time_seconds as f64 / total as f64) * seconds as f64).round() as u64
        };
        item.time_seconds = value;
        assigned = assigned.saturating_add(value);
    }
}

fn extract_domain(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .or_else(|| trimmed.strip_prefix("file://"))
        .unwrap_or(trimmed);
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .split('@')
        .last()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches("www.");

    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

pub fn friendly_app_name(
    process_name: &str,
    _process_path: &str,
    browser_name: Option<&str>,
) -> String {
    if let Some(browser) = browser_name {
        return browser.to_string();
    }

    let normalized = process_name.trim().trim_end_matches(".exe").to_lowercase();
    match normalized.as_str() {
        "afterfx" | "afterfx64" => "Adobe After Effects".to_string(),
        "premiere pro" | "premierepro" | "prproj" | "adobe premiere pro" => {
            "Adobe Premiere Pro".to_string()
        }
        "code" => "Visual Studio Code".to_string(),
        "devenv" => "Visual Studio".to_string(),
        "msedge" | "edge" => "Microsoft Edge".to_string(),
        "chrome" => "Google Chrome".to_string(),
        "firefox" => "Mozilla Firefox".to_string(),
        "explorer" => "File Explorer".to_string(),
        "notepad" => "Notepad".to_string(),
        _ => {
            let cleaned = process_name.trim().trim_end_matches(".exe");
            cleaned
                .split(['-', '_', '.'])
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

pub fn project_file_stem(project: &ProjectRecord) -> String {
    let name = sanitize_file_name(&project.name);
    if name.is_empty() {
        project.id.clone()
    } else {
        name
    }
}

fn is_own_app_process(process_name: &str) -> bool {
    let normalized = process_name
        .trim()
        .trim_end_matches(".exe")
        .to_lowercase()
        .replace(['_', ' '], "-");
    matches!(
        normalized.as_str(),
        "project-time-manager" | "projecttimemanager"
    )
}

pub fn sanitize_file_name(name: &str) -> String {
    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut out = String::new();

    for ch in name.trim().chars() {
        if forbidden.contains(&ch) || ch.is_control() {
            out.push('_');
        } else {
            out.push(ch);
        }
    }

    let cleaned = out
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(['.', ' '])
        .to_string();

    if cleaned.is_empty() {
        "project".to_string()
    } else {
        cleaned
    }
}

fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut file = fs::File::create(path).map_err(|err| err.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|err| err.to_string())
}

fn can_write_target(path: &Path) -> bool {
    OpenOptions::new().read(true).write(true).open(path).is_ok()
}

fn migrate_legacy_storage(paths: &StoragePaths) -> Result<(), String> {
    let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
    else {
        return Ok(());
    };

    let legacy_roots = [
        exe_dir.join("Данные Project Time Manager"),
        exe_dir.join("Данные приложения"),
    ];
    for legacy_root in legacy_roots {
        let legacy_workspace = legacy_root.join("workspace.json");
        if legacy_workspace.exists() && !paths.workspace_file.exists() {
            let _ = fs::copy(&legacy_workspace, &paths.workspace_file);
        }

        for legacy_projects in [legacy_root.join("projects"), legacy_root.join("Проекты")] {
            if !legacy_projects.exists() {
                continue;
            }

            for entry in fs::read_dir(&legacy_projects).map_err(|err| err.to_string())? {
                let path = entry.map_err(|err| err.to_string())?.path();
                if path.is_dir() {
                    let project_file = path.join("project.json");
                    if !project_file.exists() {
                        continue;
                    }
                    let content =
                        fs::read_to_string(&project_file).map_err(|err| err.to_string())?;
                    if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                        let mut project = project;
                        normalize_project_structure(&mut project);
                        let target = project_path(paths, &project);
                        if !target.exists() {
                            write_text_file(&target, &content)?;
                        }
                    }
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
                    if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                        let mut project = project;
                        normalize_project_structure(&mut project);
                        let target = project_path(paths, &project);
                        if !target.exists() {
                            write_text_file(&target, &content)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn migrate_flat_project_files(paths: &StoragePaths) -> Result<(), String> {
    for entry in fs::read_dir(&paths.projects_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
            let mut project = project;
            normalize_project_structure(&mut project);
            let target = project_path(paths, &project);
            if !target.exists() {
                write_text_file(&target, &content)?;
            }
        }
    }
    Ok(())
}

fn migrate_legacy_exports(paths: &StoragePaths) -> Result<(), String> {
    let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
    else {
        return Ok(());
    };

    for legacy_root in [
        exe_dir.join("Данные Project Time Manager"),
        exe_dir.join("Данные приложения"),
    ] {
        let legacy_exports = legacy_root.join("exports");
        if !legacy_exports.exists() {
            continue;
        }

        for project in list_project_files_without_migration(paths)? {
            let old_report = legacy_exports.join(format!("{}.xlsx", project_file_stem(&project)));
            if !old_report.exists() {
                continue;
            }
            let new_report = project_report_path(paths, &project);
            if !new_report.exists() {
                if let Some(parent) = new_report.parent() {
                    fs::create_dir_all(parent).map_err(|err| err.to_string())?;
                }
                let _ = fs::copy(old_report, new_report);
            }
        }
    }

    Ok(())
}

fn list_project_files_without_migration(
    paths: &StoragePaths,
) -> Result<Vec<ProjectRecord>, String> {
    let mut projects = Vec::new();

    for entry in fs::read_dir(&paths.projects_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            let project_file = path.join("project.json");
            if project_file.exists() {
                let content = fs::read_to_string(&project_file).map_err(|err| err.to_string())?;
                if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                    projects.push(project);
                }
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
                projects.push(project);
            }
        }
    }

    Ok(sort_and_dedupe_projects(projects))
}

fn sort_and_dedupe_projects(mut projects: Vec<ProjectRecord>) -> Vec<ProjectRecord> {
    projects.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    let mut seen = HashSet::new();
    projects
        .into_iter()
        .filter(|project| seen.insert(project.id.clone()))
        .collect()
}
