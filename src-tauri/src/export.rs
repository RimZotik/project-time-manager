use crate::models::{
    AppUsageRecord, ProjectRecord, SessionRecord, SessionStageSnapshot, TabUsageRecord,
};
use chrono::{DateTime, Duration, FixedOffset};
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

    write_report_sheet(&mut workbook, project, &apps, &tabs, &styles)?;
    write_apps_sheet(&mut workbook, &apps, &styles)?;
    write_tabs_sheet(&mut workbook, &tabs, &styles)?;
    write_sessions_sheet(&mut workbook, project, &styles)?;
    if !stages.is_empty() {
        write_stages_sheet(&mut workbook, project, &stages, &styles)?;
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
    timeline_header: Format,
    timeline_empty: Format,
    timeline_block: Format,
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
    id: String,
    name: String,
    order: usize,
    seconds: u64,
    session_count: usize,
    first_used_at: String,
    last_used_at: String,
    segments: Vec<StageSegment>,
}

#[derive(Clone)]
struct StageSegment {
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
}

#[derive(Clone)]
struct SessionWindow {
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    started_at: String,
    stopped_at: String,
    duration_seconds: u64,
    stage_ids: Vec<String>,
    stages: Vec<SessionStageSnapshot>,
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
            timeline_header: Format::new()
                .set_align(FormatAlign::Center)
                .set_font_color(Color::RGB(0x475569))
                .set_background_color(Color::RGB(0xF8FAFC))
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xCBD5E1)),
            timeline_empty: Format::new()
                .set_background_color(Color::RGB(0xF8FAFC))
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0xE2E8F0)),
            timeline_block: Format::new()
                .set_align(FormatAlign::Center)
                .set_font_color(Color::White)
                .set_background_color(Color::RGB(0x059669))
                .set_border(FormatBorder::Thin)
                .set_border_color(Color::RGB(0x047857)),
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
    worksheet.set_column_width(0, 22).map_err(format_xlsx_error)?;
    worksheet.set_column_width(1, 18).map_err(format_xlsx_error)?;
    worksheet.set_column_width(2, 18).map_err(format_xlsx_error)?;
    worksheet.set_column_width(3, 18).map_err(format_xlsx_error)?;
    worksheet.set_column_width(4, 22).map_err(format_xlsx_error)?;
    worksheet.set_column_width(5, 16).map_err(format_xlsx_error)?;
    worksheet.set_column_width(6, 16).map_err(format_xlsx_error)?;
    worksheet.set_column_width(9, 24).map_err(format_xlsx_error)?;
    worksheet.set_column_width(10, 12).map_err(format_xlsx_error)?;
    worksheet.set_column_width(12, 24).map_err(format_xlsx_error)?;
    worksheet.set_column_width(13, 12).map_err(format_xlsx_error)?;

    let total_seconds: u64 = apps.iter().map(|app| app.seconds).sum();

    worksheet
        .merge_range(0, 0, 0, 6, &format!("Отчет по проекту: {}", project.name), &styles.title)
        .map_err(format_xlsx_error)?;
    worksheet
        .merge_range(1, 0, 1, 6, "Учитываются только включенные приложения, сайты и ссылки.", &styles.subtitle)
        .map_err(format_xlsx_error)?;

    write_metric_text(worksheet, 3, 0, "Всего времени", &format_duration(total_seconds), styles)?;
    write_metric(worksheet, 3, 2, "Приложений", apps.len() as f64, styles)?;
    write_metric(worksheet, 3, 4, "Сеансов", project.sessions.len() as f64, styles)?;

    worksheet
        .merge_range(6, 0, 6, 2, "Топ приложений", &styles.section)
        .map_err(format_xlsx_error)?;
    worksheet
        .merge_range(6, 4, 6, 6, "Топ сайтов", &styles.section)
        .map_err(format_xlsx_error)?;

    write_small_table_header(worksheet, 7, 0, &["Приложение", "Длительность", "%"], styles)?;
    write_small_table_header(worksheet, 7, 4, &["Сайт", "Длительность", "%"], styles)?;

    for (index, app) in apps.iter().take(8).enumerate() {
        let row = 8 + index as u32;
        worksheet.write_string_with_format(row, 0, &app.name, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 1, &format_duration(app.seconds), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 2, ratio(app.seconds, total_seconds), &styles.percent).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 9, &app.name, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 10, seconds_to_hours(app.seconds), &styles.number).map_err(format_xlsx_error)?;
    }

    for (index, tab) in tabs.iter().take(8).enumerate() {
        let row = 8 + index as u32;
        worksheet.write_string_with_format(row, 4, &tab.title, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 5, &format_duration(tab.seconds), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 6, ratio(tab.seconds, total_seconds), &styles.percent).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 12, &tab.title, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 13, seconds_to_hours(tab.seconds), &styles.number).map_err(format_xlsx_error)?;
    }

    worksheet.write_string_with_format(7, 9, "Данные графика приложений", &styles.header).map_err(format_xlsx_error)?;
    worksheet.write_string_with_format(7, 10, "Часы", &styles.header).map_err(format_xlsx_error)?;
    worksheet.write_string_with_format(7, 12, "Данные графика сайтов", &styles.header).map_err(format_xlsx_error)?;
    worksheet.write_string_with_format(7, 13, "Часы", &styles.header).map_err(format_xlsx_error)?;

    if !apps.is_empty() {
        let last_row = 8 + apps.len().min(8) as u32 - 1;
        let mut chart = Chart::new(ChartType::Bar);
        chart.title().set_name("Приложения проекта");
        chart.set_style(10);
        chart
            .add_series()
            .set_name("Приложения")
            .set_categories(("Отчет", 8, 9, last_row, 9))
            .set_values(("Отчет", 8, 10, last_row, 10));
        worksheet.insert_chart(20, 0, &chart).map_err(format_xlsx_error)?;
    }

    if !tabs.is_empty() {
        let last_row = 8 + tabs.len().min(8) as u32 - 1;
        let mut chart = Chart::new(ChartType::Bar);
        chart.title().set_name("Сайты проекта");
        chart.set_style(10);
        chart
            .add_series()
            .set_name("Сайты")
            .set_categories(("Отчет", 8, 12, last_row, 12))
            .set_values(("Отчет", 8, 13, last_row, 13));
        worksheet.insert_chart(20, 8, &chart).map_err(format_xlsx_error)?;
    }

    Ok(())
}

fn write_apps_sheet(
    workbook: &mut Workbook,
    apps: &[IncludedApp],
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Приложения").map_err(format_xlsx_error)?;
    worksheet.set_column_width(0, 32).map_err(format_xlsx_error)?;
    worksheet.set_column_width(1, 24).map_err(format_xlsx_error)?;
    worksheet.set_column_width(2, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(3, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(4, 12).map_err(format_xlsx_error)?;

    write_small_table_header(worksheet, 0, 0, &["Название", "Процесс", "Тип", "Длительность", "Часы"], styles)?;
    for (index, app) in apps.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet.write_string_with_format(row, 0, &app.name, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 1, &app.process_name, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 2, app_kind_label(&app.kind), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 3, &format_duration(app.seconds), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 4, seconds_to_hours(app.seconds), &styles.number).map_err(format_xlsx_error)?;
    }

    if !apps.is_empty() {
        worksheet.autofilter(0, 0, apps.len() as u32, 4).map_err(format_xlsx_error)?;
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
    worksheet.set_column_width(0, 22).map_err(format_xlsx_error)?;
    worksheet.set_column_width(1, 42).map_err(format_xlsx_error)?;
    worksheet.set_column_width(2, 48).map_err(format_xlsx_error)?;
    worksheet.set_column_width(3, 12).map_err(format_xlsx_error)?;
    worksheet.set_column_width(4, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(5, 12).map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        0,
        0,
        &["Браузер", "Сайт", "Основная ссылка", "Ссылок", "Длительность", "Часы"],
        styles,
    )?;

    for (index, tab) in tabs.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet.write_string_with_format(row, 0, &tab.browser, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 1, &tab.title, &styles.text).map_err(format_xlsx_error)?;
        if let Some(url) = tab.url.as_deref().filter(|url| !url.trim().is_empty()) {
            worksheet.write_url_with_text(row, 2, url, url).map_err(format_xlsx_error)?;
        } else {
            worksheet.write_string_with_format(row, 2, "", &styles.text).map_err(format_xlsx_error)?;
        }
        worksheet.write_number_with_format(row, 3, tab.url_count as f64, &styles.number).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 4, &format_duration(tab.seconds), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 5, seconds_to_hours(tab.seconds), &styles.number).map_err(format_xlsx_error)?;
    }

    if !tabs.is_empty() {
        worksheet.autofilter(0, 0, tabs.len() as u32, 5).map_err(format_xlsx_error)?;
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
    worksheet.set_column_width(0, 28).map_err(format_xlsx_error)?;
    worksheet.set_column_width(1, 28).map_err(format_xlsx_error)?;
    worksheet.set_column_width(2, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(3, 12).map_err(format_xlsx_error)?;
    worksheet.set_column_width(4, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(5, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(6, 28).map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        0,
        0,
        &["Начало", "Окончание", "Длительность", "Часы", "Приложения", "Браузер", "Этапы"],
        styles,
    )?;

    for (index, session) in project.sessions.iter().enumerate() {
        let row = (index + 1) as u32;
        let stage_names = if session.stages.is_empty() {
            String::new()
        } else {
            session
                .stages
                .iter()
                .map(|stage| stage.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        worksheet.write_string_with_format(row, 0, &session.started_at, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 1, session.stopped_at.as_deref().unwrap_or(""), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 2, &format_duration(session.duration_seconds), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 3, seconds_to_hours(session.duration_seconds), &styles.number).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 4, session.app_count as f64, &styles.number).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 5, session.browser_count as f64, &styles.number).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 6, &stage_names, &styles.text).map_err(format_xlsx_error)?;
    }

    if !project.sessions.is_empty() {
        worksheet.autofilter(0, 0, project.sessions.len() as u32, 6).map_err(format_xlsx_error)?;
    }

    Ok(())
}

fn write_stages_sheet(
    workbook: &mut Workbook,
    project: &ProjectRecord,
    stages: &[IncludedStage],
    styles: &ReportStyles,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Этапы").map_err(format_xlsx_error)?;
    worksheet.set_column_width(0, 24).map_err(format_xlsx_error)?;
    worksheet.set_column_width(1, 12).map_err(format_xlsx_error)?;
    worksheet.set_column_width(2, 14).map_err(format_xlsx_error)?;
    worksheet.set_column_width(3, 22).map_err(format_xlsx_error)?;
    worksheet.set_column_width(4, 22).map_err(format_xlsx_error)?;
    worksheet.set_column_width(5, 12).map_err(format_xlsx_error)?;
    worksheet.set_column_width(6, 16).map_err(format_xlsx_error)?;

    write_small_table_header(
        worksheet,
        0,
        0,
        &["Название", "Сеансов", "Время", "Первое использование", "Последнее использование"],
        styles,
    )?;

    for (index, stage) in stages.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet.write_string_with_format(row, 0, &stage.name, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_number_with_format(row, 1, stage.session_count as f64, &styles.number).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 2, &format_duration(stage.seconds), &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 3, &stage.first_used_at, &styles.text).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(row, 4, &stage.last_used_at, &styles.text).map_err(format_xlsx_error)?;
    }

    if !stages.is_empty() {
        worksheet.autofilter(0, 0, stages.len() as u32, 4).map_err(format_xlsx_error)?;
    }

    let timeline_sessions = project_session_windows(project);
    if let Some((timeline_start, timeline_end)) = timeline_bounds(&timeline_sessions) {
        let slot_seconds = timeline_slot_seconds((timeline_end - timeline_start).num_seconds());
        let slot_count = timeline_slot_count(timeline_start, timeline_end, slot_seconds);
        let header_row = stages.len() as u32 + 4;
        let axis_row = header_row + 1;

        worksheet
            .merge_range(header_row, 0, header_row, 6, "Таймлайн этапов", &styles.section)
            .map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(axis_row, 0, "Этап", &styles.header).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(axis_row, 1, "Сеансов", &styles.header).map_err(format_xlsx_error)?;
        worksheet.write_string_with_format(axis_row, 2, "Время", &styles.header).map_err(format_xlsx_error)?;

        for slot_index in 0..slot_count {
            let column = 3 + slot_index as u16;
            worksheet.set_column_width(column, 5).map_err(format_xlsx_error)?;
            let slot_start = timeline_start + Duration::seconds(slot_seconds * slot_index as i64);
            let label = timeline_label(slot_start, slot_index, slot_count, slot_seconds);
            worksheet.write_string_with_format(axis_row, column, &label, &styles.timeline_header).map_err(format_xlsx_error)?;
        }

        for (stage_index, stage) in stages.iter().enumerate() {
            let row = axis_row + 1 + stage_index as u32;
            worksheet.write_string_with_format(row, 0, &stage.name, &styles.text).map_err(format_xlsx_error)?;
            worksheet.write_number_with_format(row, 1, stage.session_count as f64, &styles.number).map_err(format_xlsx_error)?;
            worksheet.write_string_with_format(row, 2, &format_duration(stage.seconds), &styles.text).map_err(format_xlsx_error)?;

            for slot_index in 0..slot_count {
                worksheet.write_string_with_format(row, 3 + slot_index as u16, "", &styles.timeline_empty).map_err(format_xlsx_error)?;
            }

            for segment in &stage.segments {
                let start_col = timeline_slot_index(timeline_start, segment.start, slot_seconds);
                let end_col = timeline_slot_end_index(timeline_start, segment.end, slot_seconds, slot_count);
                let label = if end_col.saturating_sub(start_col) >= 3 {
                    format!(
                        "{} - {}",
                        segment.start.format("%d.%m %H:%M"),
                        segment.end.format("%d.%m %H:%M")
                    )
                } else {
                    String::new()
                };
                if start_col == end_col {
                    worksheet.write_string_with_format(row, 3 + start_col as u16, &label, &styles.timeline_block).map_err(format_xlsx_error)?;
                } else {
                    worksheet.merge_range(row, 3 + start_col as u16, row, 3 + end_col as u16, &label, &styles.timeline_block).map_err(format_xlsx_error)?;
                }
            }
        }
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
    worksheet.write_string_with_format(row, col, label, &styles.metric_label).map_err(format_xlsx_error)?;
    worksheet.write_number_with_format(row + 1, col, value, &styles.metric_value).map_err(format_xlsx_error)?;
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
    worksheet.write_string_with_format(row, col, label, &styles.metric_label).map_err(format_xlsx_error)?;
    worksheet.write_string_with_format(row + 1, col, value, &styles.metric_value).map_err(format_xlsx_error)?;
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
        worksheet.write_string_with_format(row, start_col + offset as u16, *label, &styles.header).map_err(format_xlsx_error)?;
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
                    url_count: if tab.urls.is_empty() { usize::from(tab.url.is_some()) } else { tab.urls.len() },
                    seconds,
                })
            })
        })
        .collect::<Vec<_>>();
    tabs.sort_by(|left, right| right.seconds.cmp(&left.seconds));
    tabs
}

fn included_stages(project: &ProjectRecord) -> Vec<IncludedStage> {
    let mut stages = Vec::<IncludedStage>::new();

    let timeline_sessions = project_session_windows(project);

    for session in timeline_sessions.iter().filter(|session| !session.stage_ids.is_empty()) {
        let last_used_at = session.stopped_at.clone();
        for snapshot in &session.stages {
            if let Some(stage) = stages.iter_mut().find(|item| item.id == snapshot.id) {
                stage.seconds = stage.seconds.saturating_add(session.duration_seconds);
                stage.session_count += 1;
                if stage.first_used_at.is_empty() || session.started_at < stage.first_used_at {
                    stage.first_used_at = session.started_at.clone();
                }
                if stage.last_used_at.is_empty() || last_used_at > stage.last_used_at {
                    stage.last_used_at = last_used_at.clone();
                }
                if let Some(current_stage) = project.stages.iter().find(|item| item.id == snapshot.id) {
                    stage.name = current_stage.name.clone();
                    stage.order = current_stage.order;
                }
            } else {
                let stage_name = project
                    .stages
                    .iter()
                    .find(|item| item.id == snapshot.id)
                    .map(|item| (item.name.clone(), item.order))
                    .unwrap_or_else(|| (snapshot.name.clone(), usize::MAX));
                stages.push(IncludedStage {
                    id: snapshot.id.clone(),
                    name: stage_name.0,
                    order: stage_name.1,
                    seconds: session.duration_seconds,
                    session_count: 1,
                    first_used_at: session.started_at.clone(),
                    last_used_at: last_used_at.clone(),
                    segments: Vec::new(),
                });
            }
        }
    }

    for stage in &mut stages {
        stage.segments = stage_segments(&timeline_sessions, &stage.id);
    }

    stages.sort_by(|left, right| left.order.cmp(&right.order).then(left.name.cmp(&right.name)));
    stages
}

fn project_session_windows(project: &ProjectRecord) -> Vec<SessionWindow> {
    let mut sessions = project
        .sessions
        .iter()
        .filter_map(|session| {
            let start = parse_timestamp(&session.started_at)?;
            let end = session_end(session, start)?;
            if end <= start || session.duration_seconds == 0 {
                return None;
            }
            Some(SessionWindow {
                start,
                end,
                started_at: session.started_at.clone(),
                stopped_at: session
                    .stopped_at
                    .clone()
                    .unwrap_or_else(|| end.to_rfc3339()),
                duration_seconds: session.duration_seconds,
                stage_ids: session.stages.iter().map(|stage| stage.id.clone()).collect(),
                stages: session.stages.clone(),
            })
        })
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| left.start.cmp(&right.start));
    sessions
}

fn stage_segments(sessions: &[SessionWindow], stage_id: &str) -> Vec<StageSegment> {
    let mut segments = Vec::new();
    let mut current: Option<StageSegment> = None;

    for session in sessions {
        let has_stage = session.stage_ids.iter().any(|id| id == stage_id);
        if has_stage {
            if let Some(active) = &mut current {
                active.end = session.end;
            } else {
                current = Some(StageSegment {
                    start: session.start,
                    end: session.end,
                });
            }
        } else if let Some(active) = current.take() {
            segments.push(active);
        }
    }

    if let Some(active) = current {
        segments.push(active);
    }

    segments
}

fn parse_timestamp(value: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value).ok()
}

fn session_end(
    session: &SessionRecord,
    start: DateTime<FixedOffset>,
) -> Option<DateTime<FixedOffset>> {
    session
        .stopped_at
        .as_deref()
        .and_then(parse_timestamp)
        .or_else(|| Some(start + Duration::seconds(session.duration_seconds as i64)))
}

fn timeline_bounds(
    sessions: &[SessionWindow],
) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
    let first = sessions.first()?;
    let last = sessions.last()?;
    Some((first.start, last.end))
}

fn timeline_slot_seconds(total_span_seconds: i64) -> i64 {
    let span = total_span_seconds.max(1);
    for candidate in [900_i64, 1800, 3600, 7200, 14400, 28800, 43200, 86400] {
        if (span + candidate - 1) / candidate <= 64 {
            return candidate;
        }
    }
    86_400
}

fn timeline_slot_count(
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    slot_seconds: i64,
) -> usize {
    (((end - start).num_seconds().max(1) + slot_seconds - 1) / slot_seconds) as usize
}

fn timeline_slot_index(
    timeline_start: DateTime<FixedOffset>,
    value: DateTime<FixedOffset>,
    slot_seconds: i64,
) -> usize {
    ((value - timeline_start).num_seconds().max(0) / slot_seconds) as usize
}

fn timeline_slot_end_index(
    timeline_start: DateTime<FixedOffset>,
    value: DateTime<FixedOffset>,
    slot_seconds: i64,
    slot_count: usize,
) -> usize {
    let offset = (value - timeline_start).num_seconds().max(1) - 1;
    ((offset / slot_seconds) as usize).min(slot_count.saturating_sub(1))
}

fn timeline_label(
    slot_start: DateTime<FixedOffset>,
    slot_index: usize,
    slot_count: usize,
    slot_seconds: i64,
) -> String {
    if slot_index != 0 && slot_index != slot_count.saturating_sub(1) && slot_index % 4 != 0 {
        return String::new();
    }
    if slot_seconds >= 86_400 {
        slot_start.format("%d.%m").to_string()
    } else {
        slot_start.format("%d.%m\n%H:%M").to_string()
    }
}

fn included_app_seconds(app: &AppUsageRecord) -> u64 {
    if !app.enabled {
        return 0;
    }
    if app.kind == "browser" && !app.tabs.is_empty() {
        return app.tabs.iter().filter(|tab| tab.enabled).map(included_tab_seconds).sum();
    }
    app.time_seconds
}

fn included_tab_seconds(tab: &TabUsageRecord) -> u64 {
    if tab.urls.is_empty() {
        return tab.time_seconds;
    }
    tab.urls.iter().filter(|item| item.enabled).map(|item| item.time_seconds).sum::<u64>()
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
