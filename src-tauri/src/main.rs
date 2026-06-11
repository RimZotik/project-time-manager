#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod export;
mod models;
mod storage;
mod windows;

use crate::export::export_project_xlsx;
use crate::models::*;
use crate::storage::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Manager, State};
use uuid::Uuid;

struct TrackerDraft {
    started_at: String,
    duration_seconds: u64,
    app_hits: u64,
    browser_hits: u64,
}

struct TrackerRuntime {
    status: String,
    active_project_id: Option<String>,
    running_since: Option<String>,
    draft: Option<TrackerDraft>,
}

struct AppRuntime {
    paths: StoragePaths,
    store: Arc<Mutex<StoreData>>,
    tracker: Arc<Mutex<TrackerRuntime>>,
}

#[tauri::command]
fn get_app_state(state: State<'_, AppRuntime>) -> Result<AppPayload, String> {
    let tracker = state.tracker.lock().map_err(|err| err.to_string())?;
    let tracker_payload = TrackerPayload {
        status: tracker.status.clone(),
        active_project_id: tracker.active_project_id.clone(),
        running_since: tracker.running_since.clone(),
    };
    drop(tracker);

    let store = state.store.lock().map_err(|err| err.to_string())?.clone();
    let selected_project = selected_project_from_store(&store);

    Ok(AppPayload {
        tracker: tracker_payload,
        projects: store.projects.iter().map(ProjectRecord::summary).collect(),
        selected_project,
    })
}

#[tauri::command]
fn create_project(
    state: State<'_, AppRuntime>,
    name: String,
    client: String,
) -> Result<ProjectSummary, String> {
    let mut store = state.store.lock().map_err(|err| err.to_string())?;
    let project = storage::create_project(&state.paths, &name, &client)?;
    store.projects.push(project.clone());
    store.workspace.selected_project_id = Some(project.id.clone());
    save_workspace(
        &state.paths,
        &WorkspaceIndex {
            selected_project_id: Some(project.id.clone()),
        },
    )?;
    Ok(project.summary())
}

#[tauri::command]
fn select_project(state: State<'_, AppRuntime>, project_id: String) -> Result<(), String> {
    {
        let mut store = state.store.lock().map_err(|err| err.to_string())?;
        store.workspace.selected_project_id = Some(project_id.clone());
    }
    set_selected_project(&state.paths, Some(project_id))
}

#[tauri::command]
fn start_tracking(state: State<'_, AppRuntime>) -> Result<(), String> {
    let mut tracker = state.tracker.lock().map_err(|err| err.to_string())?;
    let store = state.store.lock().map_err(|err| err.to_string())?;
    let project_id = store
        .workspace
        .selected_project_id
        .clone()
        .ok_or_else(|| "No project selected".to_string())?;

    if tracker.status == "running" {
        return Ok(());
    }

    tracker.status = "running".to_string();
    tracker.active_project_id = Some(project_id);
    if tracker.running_since.is_none() {
        tracker.running_since = Some(now_iso());
    }
    if tracker.draft.is_none() {
        tracker.draft = Some(TrackerDraft {
            started_at: now_iso(),
            duration_seconds: 0,
            app_hits: 0,
            browser_hits: 0,
        });
    }
    Ok(())
}

#[tauri::command]
fn pause_tracking(state: State<'_, AppRuntime>) -> Result<(), String> {
    let mut tracker = state.tracker.lock().map_err(|err| err.to_string())?;
    tracker.status = "paused".to_string();
    Ok(())
}

#[tauri::command]
fn stop_tracking(state: State<'_, AppRuntime>) -> Result<(), String> {
    finalize_session(&state)?;
    let mut tracker = state.tracker.lock().map_err(|err| err.to_string())?;
    tracker.status = "stopped".to_string();
    tracker.active_project_id = None;
    tracker.running_since = None;
    tracker.draft = None;
    Ok(())
}

#[tauri::command]
fn toggle_app_included(
    state: State<'_, AppRuntime>,
    project_id: String,
    app_key: String,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let updated = toggle_app(&state.paths, &project_id, &app_key, enabled)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn toggle_tab_included(
    state: State<'_, AppRuntime>,
    project_id: String,
    app_key: String,
    tab_key: String,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let updated = toggle_tab(&state.paths, &project_id, &app_key, &tab_key, enabled)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn import_project_json(
    state: State<'_, AppRuntime>,
    json_text: String,
) -> Result<ProjectSummary, String> {
    let project = import_project_from_json(&state.paths, &json_text)?;
    sync_project_in_store(&state, project.clone())?;
    {
        let mut store = state.store.lock().map_err(|err| err.to_string())?;
        store.workspace.selected_project_id = Some(project.id.clone());
    }
    set_selected_project(&state.paths, Some(project.id.clone()))?;
    Ok(project.summary())
}

#[tauri::command]
fn export_selected_project_xlsx(state: State<'_, AppRuntime>) -> Result<ExportResult, String> {
    let store = state.store.lock().map_err(|err| err.to_string())?.clone();
    let project =
        selected_project_from_store(&store).ok_or_else(|| "No project selected".to_string())?;
    let output = state.paths.root.join("exports");
    std::fs::create_dir_all(&output).map_err(|err| err.to_string())?;
    let file_name = project_file_stem(&project);
    let path = output.join(format!("{file_name}.xlsx"));
    export_project_xlsx(&project, path.clone())?;
    Ok(ExportResult {
        message: format!("Excel export saved to {}", path.display()),
        path: path.to_string_lossy().to_string(),
    })
}

#[derive(serde::Serialize)]
struct ExportResult {
    message: String,
    path: String,
}

fn selected_project_from_store(store: &StoreData) -> Option<ProjectRecord> {
    let selected_id = store.workspace.selected_project_id.as_ref()?;
    store
        .projects
        .iter()
        .find(|project| &project.id == selected_id)
        .cloned()
}

fn sync_project_in_store(
    state: &State<'_, AppRuntime>,
    project: ProjectRecord,
) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|err| err.to_string())?;
    if let Some(item) = store
        .projects
        .iter_mut()
        .find(|current| current.id == project.id)
    {
        *item = project;
    } else {
        store.projects.push(project);
    }
    Ok(())
}

fn finalize_session(state: &State<'_, AppRuntime>) -> Result<(), String> {
    let mut tracker = state.tracker.lock().map_err(|err| err.to_string())?;
    let Some(project_id) = tracker.active_project_id.clone() else {
        return Ok(());
    };

    let Some(draft) = tracker.draft.take() else {
        return Ok(());
    };

    let mut store = state.store.lock().map_err(|err| err.to_string())?;
    let Some(project) = store.projects.iter_mut().find(|item| item.id == project_id) else {
        return Ok(());
    };

    project.sessions.push(SessionRecord {
        id: Uuid::new_v4().to_string(),
        started_at: draft.started_at,
        stopped_at: Some(now_iso()),
        duration_seconds: draft.duration_seconds,
        app_count: draft.app_hits as usize,
        browser_count: draft.browser_hits as usize,
    });
    project.updated_at = now_iso();
    save_project(&state.paths, project)?;
    Ok(())
}

fn tracker_loop(state: tauri::State<'_, AppRuntime>) {
    let store = state.store.clone();
    let tracker = state.tracker.clone();
    let paths = state.paths.clone();

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));

        let is_running = {
            let tracker_guard = match tracker.lock() {
                Ok(guard) => guard,
                Err(_) => continue,
            };
            tracker_guard.status == "running"
        };

        if !is_running {
            continue;
        }

        let observation = match windows::capture_active_window() {
            Some(value) => value,
            None => continue,
        };

        if let Err(error) = apply_sample(&store, &tracker, &paths, observation) {
            eprintln!("tracker error: {error}");
        }
    });
}

fn apply_sample(
    store: &Arc<Mutex<StoreData>>,
    tracker: &Arc<Mutex<TrackerRuntime>>,
    paths: &StoragePaths,
    observation: WindowObservation,
) -> Result<(), String> {
    let mut tracker = tracker.lock().map_err(|err| err.to_string())?;
    let project_id = tracker
        .active_project_id
        .clone()
        .ok_or_else(|| "No active project".to_string())?;

    let mut store = store.lock().map_err(|err| err.to_string())?;
    let project = store
        .projects
        .iter_mut()
        .find(|item| item.id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;

    let app_index = touch_app_time(project, &observation, 1);
    if observation.browser_name.is_some() {
        touch_tab_time(project, app_index, &observation, 1);
        if let Some(draft) = tracker.draft.as_mut() {
            draft.browser_hits = draft.browser_hits.saturating_add(1);
        }
    } else if let Some(draft) = tracker.draft.as_mut() {
        draft.app_hits = draft.app_hits.saturating_add(1);
    }

    if let Some(draft) = tracker.draft.as_mut() {
        draft.duration_seconds = draft.duration_seconds.saturating_add(1);
    }

    project.updated_at = now_iso();
    save_project(paths, project)?;
    Ok(())
}

fn main() {
    let paths = storage_paths().expect("failed to resolve storage paths");
    ensure_storage(&paths).expect("failed to create storage");
    let store = load_store(&paths).expect("failed to load store");
    let tracker = TrackerRuntime {
        status: "stopped".to_string(),
        active_project_id: store.workspace.selected_project_id.clone(),
        running_since: None,
        draft: None,
    };

    tauri::Builder::default()
        .manage(AppRuntime {
            paths,
            store: Arc::new(Mutex::new(store)),
            tracker: Arc::new(Mutex::new(tracker)),
        })
        .setup(|app| {
            tracker_loop(app.state::<AppRuntime>());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            create_project,
            select_project,
            start_tracking,
            pause_tracking,
            stop_tracking,
            toggle_app_included,
            toggle_tab_included,
            import_project_json,
            export_selected_project_xlsx
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
