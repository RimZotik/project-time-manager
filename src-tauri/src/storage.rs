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
    pub root: PathBuf,
    pub projects_dir: PathBuf,
    pub workspace_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceIndex {
    pub selected_project_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StoreData {
    pub workspace: WorkspaceIndex,
    pub projects: Vec<ProjectRecord>,
}

pub fn storage_paths() -> Result<StoragePaths, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let root = exe
        .parent()
        .map(|parent| parent.join("Данные Project Time Manager"))
        .unwrap_or_else(|| PathBuf::from("Данные Project Time Manager"));
    let projects_dir = root.join("Проекты");
    let workspace_file = root.join("workspace.json");
    Ok(StoragePaths {
        root,
        projects_dir,
        workspace_file,
    })
}

pub fn ensure_storage(paths: &StoragePaths) -> Result<(), String> {
    fs::create_dir_all(&paths.projects_dir).map_err(|err| err.to_string())?;
    migrate_legacy_storage(paths)?;
    migrate_flat_project_files(paths)?;
    migrate_legacy_exports(paths)?;
    Ok(())
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn load_workspace(paths: &StoragePaths) -> Result<WorkspaceIndex, String> {
    if !paths.workspace_file.exists() {
        return Ok(WorkspaceIndex::default());
    }

    let content = fs::read_to_string(&paths.workspace_file).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

pub fn save_workspace(paths: &StoragePaths, workspace: &WorkspaceIndex) -> Result<(), String> {
    ensure_storage(paths)?;
    let content = serde_json::to_string_pretty(workspace).map_err(|err| err.to_string())?;
    write_text_file(&paths.workspace_file, &content)
}

pub fn list_project_files(paths: &StoragePaths) -> Result<Vec<ProjectRecord>, String> {
    ensure_storage(paths)?;
    let mut projects = Vec::new();

    for entry in fs::read_dir(&paths.projects_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
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

pub fn load_store(paths: &StoragePaths) -> Result<StoreData, String> {
    Ok(StoreData {
        workspace: load_workspace(paths)?,
        projects: list_project_files(paths)?,
    })
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
        let ext = path.extension().and_then(|value| value.to_str()).unwrap_or("bin");
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
    let content = serde_json::to_string_pretty(project).map_err(|err| err.to_string())?;
    let path = project_path(paths, project);
    write_text_file(&path, &content)?;

    let legacy_path = paths.projects_dir.join(format!("{}.json", project.id));
    if legacy_path != path && legacy_path.exists() {
        let _ = fs::remove_file(legacy_path);
    }

    Ok(())
}

pub fn load_project(paths: &StoragePaths, project_id: &str) -> Result<ProjectRecord, String> {
    let path = project_path_by_id(paths, project_id)?;
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

pub fn create_project(
    paths: &StoragePaths,
    name: &str,
    _client: &str,
) -> Result<ProjectRecord, String> {
    let project = ProjectRecord {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        client: String::new(),
        note: String::new(),
        created_at: now_iso(),
        updated_at: now_iso(),
        sessions: Vec::new(),
        apps: Vec::new(),
    };
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
    save_project(paths, &project)?;
    Ok(project)
}

pub fn set_selected_project(
    paths: &StoragePaths,
    project_id: Option<String>,
) -> Result<(), String> {
    save_workspace(
        paths,
        &WorkspaceIndex {
            selected_project_id: project_id,
        },
    )
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
        if let Some(url) = observation.url.as_deref().filter(|value| !value.trim().is_empty()) {
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
    });
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

    let legacy_root = exe_dir.join("data");
    if paths.workspace_file.exists() {
        return Ok(());
    }

    let legacy_projects = legacy_root.join("projects");
    if !legacy_projects.exists() {
        return Ok(());
    }

    let legacy_workspace = legacy_root.join("workspace.json");
    if legacy_workspace.exists() {
        let _ = fs::copy(&legacy_workspace, &paths.workspace_file);
    }

    for entry in fs::read_dir(&legacy_projects).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
            let target = project_path(paths, &project);
            if !target.exists() {
                write_text_file(&target, &content)?;
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
            let target = project_path(paths, &project);
            if !target.exists() {
                write_text_file(&target, &content)?;
            }
        }
    }
    Ok(())
}

fn migrate_legacy_exports(paths: &StoragePaths) -> Result<(), String> {
    let legacy_exports = paths.root.join("exports");
    if !legacy_exports.exists() {
        return Ok(());
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
