use crate::models::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
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
        .map(|parent| parent.join("data"))
        .unwrap_or_else(|| PathBuf::from("data"));
    let projects_dir = root.join("projects");
    let workspace_file = root.join("workspace.json");
    Ok(StoragePaths {
        root,
        projects_dir,
        workspace_file,
    })
}

pub fn ensure_storage(paths: &StoragePaths) -> Result<(), String> {
    fs::create_dir_all(&paths.projects_dir).map_err(|err| err.to_string())
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
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        if let Ok(project) = serde_json::from_str::<ProjectRecord>(&content) {
            projects.push(project);
        }
    }

    projects.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(projects)
}

pub fn load_store(paths: &StoragePaths) -> Result<StoreData, String> {
    Ok(StoreData {
        workspace: load_workspace(paths)?,
        projects: list_project_files(paths)?,
    })
}

pub fn project_path(paths: &StoragePaths, project_id: &str) -> PathBuf {
    paths.projects_dir.join(format!("{project_id}.json"))
}

pub fn save_project(paths: &StoragePaths, project: &ProjectRecord) -> Result<(), String> {
    ensure_storage(paths)?;
    let content = serde_json::to_string_pretty(project).map_err(|err| err.to_string())?;
    write_text_file(&project_path(paths, &project.id), &content)
}

pub fn load_project(paths: &StoragePaths, project_id: &str) -> Result<ProjectRecord, String> {
    let path = project_path(paths, project_id);
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

pub fn create_project(
    paths: &StoragePaths,
    name: &str,
    client: &str,
) -> Result<ProjectRecord, String> {
    let project = ProjectRecord {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        client: client.trim().to_string(),
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
            enabled: true,
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
    let tab_key = format!(
        "{title}::{}",
        observation.browser_name.clone().unwrap_or_default()
    );

    let index = if let Some(index) = app.tabs.iter().position(|item| item.key == tab_key) {
        index
    } else {
        app.tabs.push(TabUsageRecord {
            key: tab_key.clone(),
            title,
            url: observation.url.clone(),
            enabled: true,
            time_seconds: 0,
        });
        app.tabs.len() - 1
    };

    if let Some(tab) = app.tabs.get_mut(index) {
        tab.time_seconds = tab.time_seconds.saturating_add(seconds);
        if tab.url.is_none() {
            tab.url = observation.url.clone();
        }
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

fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut file = fs::File::create(path).map_err(|err| err.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|err| err.to_string())
}
