use crate::models::{AppUsageRecord, ProjectRecord, ProjectStageRecord, TabUsageRecord};
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
    let stages = included_stages(project);

    write_report_sheet(&mut workbook, project, &apps, &tabs, &stages, &styles)?;
    write_apps_sheet(&mut workbook, &apps, &styles)?;
    write_tabs_sheet(&mut workbook, &tabs, &styles)?;
    write_sessions_sheet(&mut workbook, project, &styles)?;
    if !stages.is_empty() {
        write_stages_sheet(&mut workbook, &stages, &styles)?;
    }

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
    url_count: usize,
    seconds: u64,
}

#[derive(Clone)]
struct IncludedStage {
    name: String,
    created_at: String,
    updated_at: String,
    seconds: u64,
    app_count: usize,
    tab_count: usize,
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
    stages: &[IncludedStage],
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
    worksheet
        .set_column_width(8, 3)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(9, 24)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(10, 12)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(12, 24)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(13, 12)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(15, 24)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(16, 12)
        .map_err(format_xlsx_error)?;

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
        worksheet
            .write_string_with_format(row, 9, &app.name, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 10, seconds_to_hours(app.seconds), &styles.number)
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
        worksheet
            .write_string_with_format(row, 12, &tab.title, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 13, seconds_to_hours(tab.seconds), &styles.number)
            .map_err(format_xlsx_error)?;
    }

    worksheet
        .write_string_with_format(7, 9, "Данные графика приложений", &styles.header)
        .map_err(format_xlsx_error)?;
    worksheet
        .write_string_with_format(7, 10, "Часы", &styles.header)
        .map_err(format_xlsx_error)?;
    worksheet
        .write_string_with_format(7, 12, "Данные графика доменов", &styles.header)
        .map_err(format_xlsx_error)?;
    worksheet
        .write_string_with_format(7, 13, "Часы", &styles.header)
        .map_err(format_xlsx_error)?;
    if !stages.is_empty() {
        worksheet
            .write_string_with_format(7, 15, "Этапы", &styles.header)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(7, 16, "Часы", &styles.header)
            .map_err(format_xlsx_error)?;
        for (index, stage) in stages.iter().take(8).enumerate() {
            let row = 8 + index as u32;
            worksheet
                .write_string_with_format(row, 15, &stage.name, &styles.text)
                .map_err(format_xlsx_error)?;
            worksheet
                .write_number_with_format(row, 16, seconds_to_hours(stage.seconds), &styles.number)
                .map_err(format_xlsx_error)?;
        }
        let last_row = 8 + stages.len().min(8) as u32 - 1;
        let mut chart = Chart::new(ChartType::Bar);
        chart.title().set_name("Этапы проекта");
        chart.set_style(10);
        chart
            .add_series()
            .set_name("Этапы")
            .set_categories(("Отчет", 8, 15, last_row, 15))
            .set_values(("Отчет", 8, 16, last_row, 16));
        worksheet
            .insert_chart(20, 8, &chart)
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
    worksheet
        .set_column_width(5, 12)
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
        &[
            "Браузер",
            "Домен",
            "Основной URL",
            "Ссылок",
            "Длительность",
            "Часы",
        ],
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
            .write_number_with_format(row, 3, tab.url_count as f64, &styles.number)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 4, &format_duration(tab.seconds), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 5, seconds_to_hours(tab.seconds), &styles.number)
            .map_err(format_xlsx_error)?;
    }

    if !tabs.is_empty() {
        worksheet
            .autofilter(0, 0, tabs.len() as u32, 5)
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

fn write_stages_sheet(
    workbook: &mut Workbook,
    stages: &[IncludedStage],
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Этапы").map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(0, 24)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(1, 20)
        .map_err(format_xlsx_error)?;
    worksheet
        .set_column_width(2, 20)
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
        &["Название", "Создан", "Обновлен", "Время", "Приложений", "Вкладок"],
        styles,
    )?;

    for (index, stage) in stages.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet
            .write_string_with_format(row, 0, &stage.name, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 1, &stage.created_at, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 2, &stage.updated_at, &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_string_with_format(row, 3, &format_duration(stage.seconds), &styles.text)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 4, stage.app_count as f64, &styles.number)
            .map_err(format_xlsx_error)?;
        worksheet
            .write_number_with_format(row, 5, stage.tab_count as f64, &styles.number)
            .map_err(format_xlsx_error)?;
    }

    if !stages.is_empty() {
        worksheet
            .autofilter(0, 0, stages.len() as u32, 5)
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
                let seconds = included_tab_seconds(tab);
                if !tab.enabled || seconds == 0 {
                    return None;
                }
                Some(IncludedTab {
                    browser: app.name.clone(),
                    title: tab.title.clone(),
                    url: tab.url.clone(),
                    url_count: if tab.urls.is_empty() {
                        usize::from(tab.url.is_some())
                    } else {
                        tab.urls.len()
                    },
                    seconds,
                })
            })
        })
        .collect::<Vec<_>>();
    tabs.sort_by(|left, right| right.seconds.cmp(&left.seconds));
    tabs
}

fn included_stages(project: &ProjectRecord) -> Vec<IncludedStage> {
    let mut stages = project
        .stages
        .iter()
        .map(|stage| {
            let seconds = project
                .apps
                .iter()
                .map(|app| included_stage_app_seconds(stage, app))
                .sum();
            let app_count = project
                .apps
                .iter()
                .filter(|app| stage_app_enabled(stage, app))
                .count();
            let tab_count = project
                .apps
                .iter()
                .filter(|app| stage_app_enabled(stage, app))
                .flat_map(|app| app.tabs.iter().filter(|tab| stage_tab_enabled(stage, app, tab)))
                .count();
            IncludedStage {
                name: stage.name.clone(),
                created_at: stage.created_at.clone(),
                updated_at: stage.updated_at.clone(),
                seconds,
                app_count,
                tab_count,
            }
        })
        .collect::<Vec<_>>();
    stages.sort_by(|left, right| right.seconds.cmp(&left.seconds).then(left.name.cmp(&right.name)));
    stages
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
            .map(included_tab_seconds)
            .sum();
    }
    app.time_seconds
}

fn stage_app_enabled(stage: &ProjectStageRecord, app: &AppUsageRecord) -> bool {
    if !app.enabled {
        return false;
    }
    stage
        .apps
        .iter()
        .find(|item| item.app_key == app.key)
        .map(|item| item.enabled)
        .unwrap_or(true)
}

fn stage_tab_enabled(stage: &ProjectStageRecord, app: &AppUsageRecord, tab: &TabUsageRecord) -> bool {
    if !stage_app_enabled(stage, app) || !tab.enabled {
        return false;
    }
    stage
        .apps
        .iter()
        .find(|item| item.app_key == app.key)
        .and_then(|item| item.tabs.iter().find(|item| item.tab_key == tab.key))
        .map(|item| item.enabled)
        .unwrap_or(true)
}

fn included_stage_app_seconds(stage: &ProjectStageRecord, app: &AppUsageRecord) -> u64 {
    if !stage_app_enabled(stage, app) {
        return 0;
    }
    if app.kind == "browser" {
        app.tabs
            .iter()
            .filter(|tab| stage_tab_enabled(stage, app, tab))
            .map(included_tab_seconds)
            .sum()
    } else {
        app.time_seconds
    }
}

fn included_tab_seconds(tab: &TabUsageRecord) -> u64 {
    if tab.urls.is_empty() {
        return tab.time_seconds;
    }
    tab
        .urls
        .iter()
        .filter(|item| item.enabled)
        .map(|item| item.time_seconds)
        .sum::<u64>()
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
