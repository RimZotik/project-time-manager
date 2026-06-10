use crate::models::ProjectRecord;
use rust_xlsxwriter::{Format, Workbook, XlsxError};
use std::path::PathBuf;

pub fn export_project_xlsx(
    project: &ProjectRecord,
    output_path: PathBuf,
) -> Result<PathBuf, String> {
    let mut workbook = Workbook::new();
    let header = Format::new().set_bold();

    write_apps_sheet(&mut workbook, project, &header)?;
    write_tabs_sheet(&mut workbook, project, &header)?;
    write_sessions_sheet(&mut workbook, project, &header)?;
    write_dashboard_sheet(&mut workbook, project, &header)?;

    workbook
        .save(&output_path)
        .map_err(|err| format_xlsx_error(err))?;
    Ok(output_path)
}

fn write_apps_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    header: &Format,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Apps")
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 0, "Name", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 1, "Process", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 2, "Kind", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 3, "Enabled", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 4, "Time (sec)", header)
        .map_err(|err| format_xlsx_error(err))?;

    for (row, app) in project.apps.iter().enumerate() {
        let row = (row + 1) as u32;
        worksheet
            .write_string(row, 0, &app.name)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_string(row, 1, &app.process_name)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_string(row, 2, &app.kind)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_boolean(row, 3, app.enabled)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_number(row, 4, app.time_seconds as f64)
            .map_err(|err| format_xlsx_error(err))?;
    }

    Ok(())
}

fn write_tabs_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    header: &Format,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("BrowserTabs")
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 0, "Browser", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 1, "Title", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 2, "URL", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 3, "Enabled", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 4, "Time (sec)", header)
        .map_err(|err| format_xlsx_error(err))?;

    let mut row = 1;
    for app in &project.apps {
        for tab in &app.tabs {
            worksheet
                .write_string(row, 0, &app.name)
                .map_err(|err| format_xlsx_error(err))?;
            worksheet
                .write_string(row, 1, &tab.title)
                .map_err(|err| format_xlsx_error(err))?;
            worksheet
                .write_string(row, 2, tab.url.as_deref().unwrap_or(""))
                .map_err(|err| format_xlsx_error(err))?;
            worksheet
                .write_boolean(row, 3, tab.enabled)
                .map_err(|err| format_xlsx_error(err))?;
            worksheet
                .write_number(row, 4, tab.time_seconds as f64)
                .map_err(|err| format_xlsx_error(err))?;
            row += 1;
        }
    }

    Ok(())
}

fn write_sessions_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    header: &Format,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Sessions")
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 0, "Started", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 1, "Stopped", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 2, "Duration (sec)", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 3, "Apps", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 4, "Browser tabs", header)
        .map_err(|err| format_xlsx_error(err))?;

    for (row, session) in project.sessions.iter().enumerate() {
        let row = (row + 1) as u32;
        worksheet
            .write_string(row, 0, &session.started_at)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_string(row, 1, session.stopped_at.as_deref().unwrap_or(""))
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_number(row, 2, session.duration_seconds as f64)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_number(row, 3, session.app_count as f64)
            .map_err(|err| format_xlsx_error(err))?;
        worksheet
            .write_number(row, 4, session.browser_count as f64)
            .map_err(|err| format_xlsx_error(err))?;
    }

    Ok(())
}

fn write_dashboard_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    header: &Format,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Dashboard")
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(0, 0, "Project", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string(0, 1, &project.name)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(1, 0, "Client", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string(1, 1, &project.client)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(2, 0, "Sessions", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_number(2, 1, project.sessions.len() as f64)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_string_with_format(3, 0, "Apps", header)
        .map_err(|err| format_xlsx_error(err))?;
    worksheet
        .write_number(3, 1, project.apps.len() as f64)
        .map_err(|err| format_xlsx_error(err))?;
    Ok(())
}

fn format_xlsx_error(error: XlsxError) -> String {
    error.to_string()
}
