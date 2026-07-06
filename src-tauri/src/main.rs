#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod export;
mod models;
mod pdf;
mod storage;
mod windows;

use crate::export::export_project_xlsx;
use crate::models::*;
use crate::pdf::export_project_pdf;
use crate::storage::*;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Manager, State};
use uuid::Uuid;
#[cfg(target_os = "windows")]
use ::windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use ::windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use ::windows::Win32::UI::Shell::ShellExecuteW;
#[cfg(target_os = "windows")]
use ::windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

struct TrackerDraft {
    started_at: String,
    duration_seconds: u64,
    app_hits: u64,
    browser_hits: u64,
    stages: Vec<SessionStageSnapshot>,
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
        settings: AppSettings {
            autostart: store.workspace.autostart,
            language: normalized_language(&store.workspace.language),
        },
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
    save_workspace(&state.paths, &store.workspace)?;
    Ok(project.summary())
}

#[tauri::command]
fn select_project(state: State<'_, AppRuntime>, project_id: String) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|err| err.to_string())?;
    store.workspace.selected_project_id = Some(project_id);
    save_workspace(&state.paths, &store.workspace)
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
    let project = store
        .projects
        .iter()
        .find(|item| item.id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;
    let selected_stages = project
        .stages
        .iter()
        .filter(|stage| project.selected_stage_ids.iter().any(|stage_id| stage_id == &stage.id))
        .map(|stage| SessionStageSnapshot {
            id: stage.id.clone(),
            name: stage.name.clone(),
        })
        .collect::<Vec<_>>();

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
            stages: selected_stages,
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
fn toggle_url_included(
    state: State<'_, AppRuntime>,
    project_id: String,
    app_key: String,
    tab_key: String,
    url: String,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    let updated = toggle_url(&state.paths, &project_id, &app_key, &tab_key, &url, enabled)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn set_app_time(
    state: State<'_, AppRuntime>,
    project_id: String,
    app_key: String,
    seconds: u64,
) -> Result<ProjectRecord, String> {
    let updated = crate::storage::set_app_time(&state.paths, &project_id, &app_key, seconds)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn set_tab_time(
    state: State<'_, AppRuntime>,
    project_id: String,
    app_key: String,
    tab_key: String,
    seconds: u64,
) -> Result<ProjectRecord, String> {
    let updated = crate::storage::set_tab_time(&state.paths, &project_id, &app_key, &tab_key, seconds)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn create_stage(
    state: State<'_, AppRuntime>,
    project_id: String,
    name: String,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::create_stage(&state.paths, &project_id, &name)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn rename_stage(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_id: String,
    name: String,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::rename_stage(&state.paths, &project_id, &stage_id, &name)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn delete_stage(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_id: String,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::delete_stage(&state.paths, &project_id, &stage_id)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn reorder_stage(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_id: String,
    direction: i32,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::reorder_stage(&state.paths, &project_id, &stage_id, direction)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn set_selected_project_stages(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_ids: Vec<String>,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::set_selected_stages(&state.paths, &project_id, &stage_ids)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn toggle_stage_app_included(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_id: String,
    app_key: String,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::toggle_stage_app(&state.paths, &project_id, &stage_id, &app_key, enabled)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn toggle_stage_tab_included(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_id: String,
    app_key: String,
    tab_key: String,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::toggle_stage_tab(&state.paths, &project_id, &stage_id, &app_key, &tab_key, enabled)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn toggle_stage_url_included(
    state: State<'_, AppRuntime>,
    project_id: String,
    stage_id: String,
    app_key: String,
    tab_key: String,
    url: String,
    enabled: bool,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = crate::storage::toggle_stage_url(&state.paths, &project_id, &stage_id, &app_key, &tab_key, &url, enabled)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn rename_project(
    state: State<'_, AppRuntime>,
    project_id: String,
    name: String,
) -> Result<ProjectRecord, String> {
    ensure_tracker_stopped(&state)?;
    let updated = rename_project_record(&state.paths, &project_id, &name)?;
    sync_project_in_store(&state, updated.clone())?;
    Ok(updated)
}

#[tauri::command]
fn delete_project(state: State<'_, AppRuntime>, project_id: String) -> Result<(), String> {
    ensure_tracker_stopped(&state)?;
    delete_project_record(&state.paths, &project_id)?;
    let mut store = state.store.lock().map_err(|err| err.to_string())?;
    store.projects.retain(|project| project.id != project_id);
    if store.workspace.selected_project_id.as_deref() == Some(project_id.as_str()) {
        store.workspace.selected_project_id =
            store.projects.first().map(|project| project.id.clone());
    }
    save_workspace(&state.paths, &store.workspace)?;
    Ok(())
}

#[tauri::command]
fn update_app_settings(
    state: State<'_, AppRuntime>,
    autostart: bool,
    language: String,
) -> Result<AppSettings, String> {
    let normalized = normalized_language(&language);
    let previous_autostart;
    {
        let mut store = state.store.lock().map_err(|err| err.to_string())?;
        previous_autostart = store.workspace.autostart;
        store.workspace.autostart = autostart;
        store.workspace.language = normalized.clone();
        save_workspace(&state.paths, &store.workspace)?;
    }

    if previous_autostart != autostart {
        set_autostart_enabled(autostart)?;
    }
    Ok(AppSettings {
        autostart,
        language: normalized,
    })
}

#[tauri::command]
fn open_app_folder() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let folder = exe
        .parent()
        .ok_or_else(|| "Application folder not found".to_string())?;
    open_path(&folder.to_string_lossy())
}

#[tauri::command]
fn import_project_json(
    state: State<'_, AppRuntime>,
    json_text: String,
) -> Result<ProjectSummary, String> {
    ensure_tracker_stopped(&state)?;
    let project = import_project_from_json(&state.paths, &json_text)?;
    sync_project_in_store(&state, project.clone())?;
    {
        let mut store = state.store.lock().map_err(|err| err.to_string())?;
        store.workspace.selected_project_id = Some(project.id.clone());
        save_workspace(&state.paths, &store.workspace)?;
    }
    Ok(project.summary())
}

#[tauri::command]
fn export_selected_project_xlsx(state: State<'_, AppRuntime>) -> Result<ExportResult, String> {
    ensure_tracker_stopped(&state)?;
    let store = state.store.lock().map_err(|err| err.to_string())?.clone();
    let project =
        selected_project_from_store(&store).ok_or_else(|| "No project selected".to_string())?;
    let path = reserve_report_path(&project_report_path(&state.paths, &project));
    export_project_xlsx(&project, path.clone())?;
    Ok(ExportResult {
        message: format!("Excel-отчет сохранен: {}", path.display()),
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn export_selected_project_pdf(state: State<'_, AppRuntime>) -> Result<ExportResult, String> {
    ensure_tracker_stopped(&state)?;
    let store = state.store.lock().map_err(|err| err.to_string())?.clone();
    let project =
        selected_project_from_store(&store).ok_or_else(|| "No project selected".to_string())?;
    let path = reserve_report_path(&project_report_pdf_path(&state.paths, &project));
    export_project_pdf(&project, path.clone())?;
    Ok(ExportResult {
        message: format!("PDF-отчет сохранен: {}", path.display()),
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn open_report_file(path: String) -> Result<(), String> {
    open_path(&path)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("URL недоступен.".to_string());
    }
    open_path(trimmed)
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

fn ensure_tracker_stopped(state: &State<'_, AppRuntime>) -> Result<(), String> {
    let tracker = state.tracker.lock().map_err(|err| err.to_string())?;
    if tracker.status == "stopped" {
        Ok(())
    } else {
        Err("Изменять этапы можно только при остановленной записи.".to_string())
    }
}

fn open_path(target: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let operation = wide_null("open");
        let target = wide_null(target);
        let result = unsafe {
            ShellExecuteW(
                HWND(std::ptr::null_mut()),
                PCWSTR(operation.as_ptr()),
                PCWSTR(target.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        if result.0 as isize <= 32 {
            return Err("Не удалось открыть файл или ссылку.".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(target)
            .spawn()
            .map_err(|err| err.to_string())?;
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn normalized_language(language: &str) -> String {
    match language.trim().to_lowercase().as_str() {
        "en" => "en".to_string(),
        _ => "ru".to_string(),
    }
}

fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let exe = std::env::current_exe().map_err(|err| err.to_string())?;
        let exe = exe.to_string_lossy().to_string();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run_key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|err| err.to_string())?;
        if enabled {
            run_key
                .set_value("Project Time Manager", &format!("\"{}\"", exe))
                .map_err(|err| err.to_string())?;
        } else {
            let _ = run_key.delete_value("Project Time Manager");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
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
        stages: draft.stages,
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
    let _ = set_autostart_enabled(store.workspace.autostart);
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
            toggle_url_included,
            set_app_time,
            set_tab_time,
            rename_project,
            delete_project,
            create_stage,
            rename_stage,
            delete_stage,
            reorder_stage,
            set_selected_project_stages,
            toggle_stage_app_included,
            toggle_stage_tab_included,
            toggle_stage_url_included,
            update_app_settings,
            import_project_json,
            export_selected_project_xlsx,
            export_selected_project_pdf,
            open_report_file,
            open_external_url,
            open_app_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
