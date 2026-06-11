use crate::models::{AppUsageRecord, ProjectRecord};
use rust_xlsxwriter::{
    Chart, ChartType, Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError,
};
use std::path::PathBuf;

pub fn export_project_xlsx(
    project: &ProjectRecord,
    output_path: PathBuf,
) -> Result<PathBuf, String> {
    let mut workbook = Workbook::new();
    let styles = ReportStyles::new();
    let apps = included_apps(project);
    let tabs = included_tabs(project);

    write_report_sheet(&mut workbook, project, &apps, &tabs, &styles)?;
    write_apps_sheet(&mut workbook, &apps, &styles)?;
    write_tabs_sheet(&mut workbook, &tabs, &styles)?;
    write_sessions_sheet(&mut workbook, project, &styles)?;

    workbook.save(&output_path).map_err(format_xlsx_error)?;
    Ok(output_path)
}

struct ReportStyles {
    title: Format,
    subtitle: Format,
    section: Format,
    metric_label: Format,
    metric_value: Format,
    header: Format,
    text: Format,
    number: Format,
    percent: Format,
}

#[derive(Clone)]
struct IncludedApp {
    name: String,
    process_name: String,
    kind: String,
    seconds: u64,
}

#[derive(Clone)]
struct IncludedTab {
    browser: String,
    title: String,
    url: Option<String>,
    seconds: u64,
}

impl ReportStyles {
    fn new() -> Self {
        Self {
            title: Format::new()
                .set_bold()
                .set_font_size(22)
                .set_font_color(Color::RGB(0x064E3B)),
            subtitle: Format::new().set_font_color(Color::RGB(0x64748B)),
            section: Format::new()
                .set_bold()
                .set_font_color(Color::White)
                .set_background_color(Color::RGB(0x059669))
                .set_align(FormatAlign::Center),
            metric_label: Format::new()
                .set_bold()
                .set_font_color(Color::RGB(0x64748B))
                .set_background_color(Color::RGB(0xECFDF5))
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xD1FAE5)),
            metric_value: Format::new()
                .set_bold()
                .set_font_size(14)
                .set_font_color(Color::RGB(0x0F172A))
                .set_background_color(Color::RGB(0xFFFFFF))
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xD1FAE5)),
            header: Format::new()
                .set_bold()
                .set_font_color(Color::RGB(0x064E3B))
                .set_background_color(Color::RGB(0xD1FAE5))
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xA7F3D0)),
            text: Format::new()
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xE2E8F0)),
            number: Format::new()
                .set_num_format("0.00")
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xE2E8F0)),
            percent: Format::new()
                .set_num_format("0.0%")
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xE2E8F0)),
        }
    }
}

fn write_report_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    apps: &[IncludedApp],
    tabs: &[IncludedTab],
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Отчет").map_err(format_xlsx_error)?;
    worksheet.set_tab_color(Color::RGB(0x059669));
    worksheet
        .set_column_width(0, 22)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(1, 18)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(2, 18)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(3, 18)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(4, 22)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(5, 16)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(6, 16)
        .map_err(format_xlsx_error)?;
    worksheet.set_column_hidden(7).map_err(format_xlsx_error)?;
    worksheet.set_column_hidden(8).map_err(format_xlsx_error)?;

    let total_seconds: u64 = apps.iter().map(|app| app.seconds).sum();

    worksheet
        .merge_range(
            0,
            0,
            0,
            6,
            &format!("Отчет по проекту: {}", project.name),
            &styles.title,
        )
        .map_err(format_xlsx_error)?;
    worksheet
        .merge_range(
            1,
            0,
            1,
            6,
            "Учитываются только включенные приложения и вкладки.",
            &styles.subtitle,
        )
        .map_err(format_xlsx_error)?;

    write_metric_text(
        worksheet,
        3,
        0,
        "Всего времени",
        &format_duration(total_seconds),
        styles,
    )?;
    write_metric(worksheet, 3, 2, "Приложений", apps.len() as f64, styles)?;
    write_metric(
        worksheet,
        3,
        4,
        "Сеансов",
        project.sessions.len() as f64,
        styles,
    )?;

    worksheet
        .merge_range(6, 0, 6, 2, "Топ приложений", &styles.section)
        .map_err(format_xlsx_error)?;
    worksheet
        .merge_range(6, 4, 6, 6, "Топ вкладок", &styles.section)
        .map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        7,
        0,
        &["Приложение", "Длительность", "%"],
        styles,
    )?;
    write_small_table_header(worksheet, 7, 4, &["Вкладка", "Длительность", "%"], styles)?;

    for (index, app) in apps.iter().take(8).enumerate() {
        let row = 8 + index as u32;
        worksheet
            .write_string_with_format(row, 0, &app.name, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 1, &format_duration(app.seconds), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 2, ratio(app.seconds, total_seconds), &styles.percent)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number(row, 7, seconds_to_hours(app.seconds))
            .map_err(format_xlsx_error)?;
    }

    for (index, tab) in tabs.iter().take(8).enumerate() {
        let row = 8 + index as u32;
        worksheet
            .write_string_with_format(row, 4, &tab.title, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 5, &format_duration(tab.seconds), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 6, ratio(tab.seconds, total_seconds), &styles.percent)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number(row, 8, seconds_to_hours(tab.seconds))
            .map_err(format_xlsx_error)?;
    }

    if !apps.is_empty() {
        let last_row = 8 + apps.len().min(8) as u32 - 1;
        let categories = format!("'Отчет'!$A$9:$A${}", last_row + 1);
        let values = format!("'Отчет'!$H$9:$H${}", last_row + 1);
        let mut chart = Chart::new(ChartType::Pie);
        chart.title().set_name("Распределение по приложениям");
        chart.set_style(10);
        chart
            .add_series()
            .set_name("Приложения")
            .set_categories(&categories)
            .set_values(&values);
        worksheet
            .insert_chart(18, 0, &chart)
            .map_err(format_xlsx_error)?;
    }

    if !tabs.is_empty() {
        let last_row = 8 + tabs.len().min(8) as u32 - 1;
        let categories = format!("'Отчет'!$E$9:$E${}", last_row + 1);
        let values = format!("'Отчет'!$I$9:$I${}", last_row + 1);
        let mut chart = Chart::new(ChartType::Column);
        chart.title().set_name("Время по вкладкам");
        chart.set_style(11);
        chart
            .add_series()
            .set_name("Вкладки")
            .set_categories(&categories)
            .set_values(&values);
        worksheet
            .insert_chart(18, 4, &chart)
            .map_err(format_xlsx_error)?;
    }

    Ok(())
}

fn write_apps_sheet(
    workbook: &mut Workbook,
    apps: &[IncludedApp],
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Приложения")
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(0, 32)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(1, 24)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(2, 14)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(3, 12)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(4, 12)
        .map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        0,
        0,
        &["Название", "Процесс", "Тип", "Длительность", "Часы"],
        styles,
    )?;
    for (index, app) in apps.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet
            .write_string_with_format(row, 0, &app.name, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 1, &app.process_name, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 2, app_kind_label(&app.kind), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 3, &format_duration(app.seconds), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 4, seconds_to_hours(app.seconds), &styles.number)
            .map_err(format_xlsx_error)?;
    }

    if !apps.is_empty() {
        worksheet
            .autofilter(0, 0, apps.len() as u32, 4)
            .map_err(format_xlsx_error)?;
    }

    Ok(())
}

fn write_tabs_sheet(
    workbook: &mut Workbook,
    tabs: &[IncludedTab],
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Вкладки").map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(0, 22)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(1, 42)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(2, 48)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(3, 12)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(4, 12)
        .map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        0,
        0,
        &["Браузер", "Вкладка", "URL", "Длительность", "Часы"],
        styles,
    )?;
    for (index, tab) in tabs.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet
            .write_string_with_format(row, 0, &tab.browser, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 1, &tab.title, &styles.text)
            .map_err(format_xlsx_error)?;
        if let Some(url) = tab.url.as_deref().filter(|url| !url.trim().is_empty()) {
            worksheet
                .write_url_with_text(row, 2, url, url)
                .map_err(format_xlsx_error)?;
        } else {
            worksheet
                .write_string_with_format(row, 2, "", &styles.text)
                .map_err(format_xlsx_error)?;
        }
        worksheet
            .write_string_with_format(row, 3, &format_duration(tab.seconds), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 4, seconds_to_hours(tab.seconds), &styles.number)
            .map_err(format_xlsx_error)?;
    }

    if !tabs.is_empty() {
        worksheet
            .autofilter(0, 0, tabs.len() as u32, 4)
            .map_err(format_xlsx_error)?;
    }

    Ok(())
}

fn write_sessions_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Сеансы").map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(0, 28)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(1, 28)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(2, 14)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(3, 14)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(4, 14)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(5, 14)
        .map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        0,
        0,
        &[
            "Начало",
            "Окончание",
            "Длительность",
            "Часы",
            "Приложения",
            "Браузер",
        ],
        styles,
    )?;

    for (index, session) in project.sessions.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet
            .write_string_with_format(row, 0, &session.started_at, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(
                row,
                1,
                session.stopped_at.as_deref().unwrap_or(""),
                &styles.text,
            )
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(
                row,
                2,
                &format_duration(session.duration_seconds),
                &styles.text,
            )
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(
                row,
                3,
                seconds_to_hours(session.duration_seconds),
                &styles.number,
            )
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 4, session.app_count as f64, &styles.number)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 5, session.browser_count as f64, &styles.number)
            .map_err(format_xlsx_error)?;
    }

    if !project.sessions.is_empty() {
        worksheet
            .autofilter(0, 0, project.sessions.len() as u32, 5)
            .map_err(format_xlsx_error)?;
    }

    Ok(())
}

fn write_metric(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    label: &str,
    value: f64,
    styles: &ReportStyles,
) -> Result<(), String> {
    worksheet
        .write_string_with_format(row, col, label, &styles.metric_label)
        .map_err(format_xlsx_error)?;
    worksheet
        .write_number_with_format(row + 1, col, value, &styles.metric_value)
        .map_err(format_xlsx_error)?;
    Ok(())
}

fn write_metric_text(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    label: &str,
    value: &str,
    styles: &ReportStyles,
) -> Result<(), String> {
    worksheet
        .write_string_with_format(row, col, label, &styles.metric_label)
        .map_err(format_xlsx_error)?;
    worksheet
        .write_string_with_format(row + 1, col, value, &styles.metric_value)
        .map_err(format_xlsx_error)?;
    Ok(())
}

fn write_small_table_header(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    start_col: u16,
    labels: &[&str],
    styles: &ReportStyles,
) -> Result<(), String> {
    for (offset, label) in labels.iter().enumerate() {
        worksheet
            .write_string_with_format(row, start_col + offset as u16, *label, &styles.header)
            .map_err(format_xlsx_error)?;
    }
    Ok(())
}

fn included_apps(project: &ProjectRecord) -> Vec<IncludedApp> {
    let mut apps = project
        .apps
        .iter()
        .filter_map(|app| {
            let seconds = included_app_seconds(app);
            if seconds == 0 {
                return None;
            }
            Some(IncludedApp {
                name: app.name.clone(),
                process_name: app.process_name.clone(),
                kind: app.kind.clone(),
                seconds,
            })
        })
        .collect::<Vec<_>>();
    apps.sort_by(|left, right| right.seconds.cmp(&left.seconds));
    apps
}

fn included_tabs(project: &ProjectRecord) -> Vec<IncludedTab> {
    let mut tabs = project
        .apps
        .iter()
        .filter(|app| app.enabled)
        .flat_map(|app| {
            app.tabs.iter().filter_map(|tab| {
                if !tab.enabled || tab.time_seconds == 0 {
                    return None;
                }
                Some(IncludedTab {
                    browser: app.name.clone(),
                    title: tab.title.clone(),
                    url: tab.url.clone(),
                    seconds: tab.time_seconds,
                })
            })
        })
        .collect::<Vec<_>>();
    tabs.sort_by(|left, right| right.seconds.cmp(&left.seconds));
    tabs
}

fn included_app_seconds(app: &AppUsageRecord) -> u64 {
    if !app.enabled {
        return 0;
    }
    if app.kind == "browser" && !app.tabs.is_empty() {
        return app
            .tabs
            .iter()
            .filter(|tab| tab.enabled)
            .map(|tab| tab.time_seconds)
            .sum();
    }
    app.time_seconds
}

fn app_kind_label(kind: &str) -> &str {
    if kind == "browser" {
        "Браузер"
    } else {
        "Приложение"
    }
}

fn seconds_to_hours(seconds: u64) -> f64 {
    seconds as f64 / 3600.0
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;
    let clock = format!("{hours:02}:{minutes:02}:{secs:02}");
    if days > 0 {
        format!("{days} д {clock}")
    } else {
        clock
    }
}

fn ratio(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 / total as f64
    }
}

fn format_xlsx_error(error: XlsxError) -> String {
    error.to_string()
}
