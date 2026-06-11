use crate::models::{AppUsageRecord, ProjectRecord};
use printpdf::{
    font::{ParsedFont, PdfFontParseWarning},
    ops::PdfFontHandle,
    text::TextItem,
    Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, PdfWarnMsg, Point, Pt,
};
use std::fs;
use std::path::{Path, PathBuf};

pub fn export_project_pdf(project: &ProjectRecord, output_path: PathBuf) -> Result<PathBuf, String> {
    let (font_bytes, _font_name) = load_font_bytes()?;
    let mut warnings = Vec::<PdfFontParseWarning>::new();
    let parsed_font = ParsedFont::from_bytes(&font_bytes, 0, &mut warnings)
        .ok_or_else(|| "Не удалось загрузить шрифт для PDF.".to_string())?;

    let mut doc = PdfDocument::new(&format!("Отчет {}", project.name));
    let font_id = doc.add_font(&parsed_font);

    let included_apps = included_apps(project);
    let included_tabs = included_tabs(project);
    let total_seconds: u64 = included_apps.iter().map(|app| app.seconds).sum();
    let sessions = &project.sessions;

    let pages = vec![
        PdfPage::new(
            Mm(210.0),
            Mm(297.0),
            build_dashboard_page(
                project,
                &font_id,
                total_seconds,
                &included_apps,
                &included_tabs,
            ),
        ),
        PdfPage::new(
            Mm(210.0),
            Mm(297.0),
            build_details_page(&font_id, &included_apps, &included_tabs),
        ),
        PdfPage::new(
            Mm(210.0),
            Mm(297.0),
            build_sessions_page(&font_id, project, sessions),
        ),
    ];

    let doc = doc.with_pages(pages);
    let bytes = doc.save(&PdfSaveOptions::default(), &mut Vec::<PdfWarnMsg>::new());
    fs::write(&output_path, bytes).map_err(|err| err.to_string())?;
    Ok(output_path)
}

fn build_dashboard_page(
    project: &ProjectRecord,
    font_id: &printpdf::FontId,
    total_seconds: u64,
    apps: &[IncludedApp],
    tabs: &[IncludedTab],
) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut y = 278.0;

    push_heading(&mut ops, font_id, &project.name, y);
    y -= 11.0;
    push_text(
        &mut ops,
        font_id,
        11.0,
        y,
        &format!("Проект: {}", project.name),
    );
    y -= 6.0;
    push_text(
        &mut ops,
        font_id,
        11.0,
        y,
        &format!("Создан: {}", project.created_at),
    );
    y -= 6.0;
    push_text(
        &mut ops,
        font_id,
        11.0,
        y,
        &format!(
            "Последняя сессия: {}",
            project
                .sessions
                .last()
                .and_then(|session| session.stopped_at.as_deref())
                .unwrap_or("-")
        ),
    );
    y -= 6.0;
    push_text(
        &mut ops,
        font_id,
        11.0,
        y,
        &format!("Общее время: {}", format_duration(total_seconds)),
    );

    y -= 14.0;
    push_section_title(&mut ops, font_id, "Ключевые показатели", y);
    y -= 9.0;
    push_text(
        &mut ops,
        font_id,
        10.5,
        y,
        &format!("Приложений: {}", apps.len()),
    );
    y -= 6.0;
    push_text(&mut ops, font_id, 10.5, y, &format!("Вкладок: {}", tabs.len()));
    y -= 6.0;
    push_text(
        &mut ops,
        font_id,
        10.5,
        y,
        &format!("Сеансов: {}", project.sessions.len()),
    );

    y -= 14.0;
    push_section_title(&mut ops, font_id, "Топ приложения", y);
    y -= 8.0;
    for app in apps.iter().take(6) {
        push_text(
            &mut ops,
            font_id,
            10.2,
            y,
            &format!("• {}  |  {}  |  {}", app.name, format_duration(app.seconds), percent(app.seconds, total_seconds)),
        );
        y -= 5.5;
    }

    y -= 6.0;
    push_section_title(&mut ops, font_id, "Топ вкладки", y);
    y -= 8.0;
    for tab in tabs.iter().take(6) {
        push_text(
            &mut ops,
            font_id,
            10.2,
            y,
            &format!("• {}  |  {}  |  {}", tab.title, format_duration(tab.seconds), percent(tab.seconds, total_seconds)),
        );
        y -= 5.5;
    }

    ops
}

fn build_details_page(
    font_id: &printpdf::FontId,
    apps: &[IncludedApp],
    tabs: &[IncludedTab],
) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut y = 278.0;

    push_heading(&mut ops, font_id, "Подробности", y);
    y -= 14.0;
    push_section_title(&mut ops, font_id, "Приложения", y);
    y -= 8.0;
    push_text(
        &mut ops,
        font_id,
        9.5,
        y,
        "Название | Процесс | Тип | Время | Часы",
    );
    y -= 5.0;
    push_rule(&mut ops, y);
    y -= 6.0;
    for app in apps.iter() {
        push_text(
            &mut ops,
            font_id,
            9.2,
            y,
            &format!(
                "{} | {} | {} | {} | {:.2}",
                trim_field(&app.name, 20),
                trim_field(&app.process_name, 18),
                app_kind_label(&app.kind),
                format_duration(app.seconds),
                seconds_to_hours(app.seconds)
            ),
        );
        y -= 5.0;
        if y < 155.0 {
            break;
        }
    }

    y -= 8.0;
    push_section_title(&mut ops, font_id, "Вкладки", y);
    y -= 8.0;
    push_text(
        &mut ops,
        font_id,
        9.5,
        y,
        "Браузер | Вкладка | URL | Время | Часы",
    );
    y -= 5.0;
    push_rule(&mut ops, y);
    y -= 6.0;
    for tab in tabs.iter() {
        push_text(
            &mut ops,
            font_id,
            9.0,
            y,
            &format!(
                "{} | {} | {} | {} | {:.2}",
                trim_field(&tab.browser, 16),
                trim_field(&tab.title, 24),
                trim_field(tab.url.as_deref().unwrap_or("-"), 26),
                format_duration(tab.seconds),
                seconds_to_hours(tab.seconds)
            ),
        );
        y -= 5.0;
        if y < 40.0 {
            break;
        }
    }

    ops
}

fn build_sessions_page(font_id: &printpdf::FontId, project: &ProjectRecord, sessions: &[crate::models::SessionRecord]) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut y = 278.0;

    push_heading(&mut ops, font_id, "Сеансы", y);
    y -= 14.0;
    push_text(
        &mut ops,
        font_id,
        10.0,
        y,
        &format!(
            "Старт проекта: {}  |  Последняя сессия: {}",
            project.created_at,
            project
                .sessions
                .last()
                .and_then(|session| session.stopped_at.as_deref())
                .unwrap_or("-")
        ),
    );
    y -= 8.0;
    push_text(
        &mut ops,
        font_id,
        9.5,
        y,
        "Начало | Окончание | Длительность | Часы | Приложения | Браузер",
    );
    y -= 5.0;
    push_rule(&mut ops, y);
    y -= 6.0;

    for session in sessions.iter() {
        push_text(
            &mut ops,
            font_id,
            8.8,
            y,
            &format!(
                "{} | {} | {} | {:.2} | {} | {}",
                trim_field(&session.started_at, 19),
                trim_field(session.stopped_at.as_deref().unwrap_or("-"), 19),
                format_duration(session.duration_seconds),
                seconds_to_hours(session.duration_seconds),
                session.app_count,
                session.browser_count
            ),
        );
        y -= 5.0;
        if y < 35.0 {
            break;
        }
    }

    ops
}

fn push_heading(ops: &mut Vec<Op>, font_id: &printpdf::FontId, text: &str, y: f32) {
    push_text(ops, font_id, 20.0, y, text);
}

fn push_section_title(ops: &mut Vec<Op>, font_id: &printpdf::FontId, text: &str, y: f32) {
    push_text(ops, font_id, 13.0, y, text);
}

fn push_rule(_ops: &mut Vec<Op>, _y: f32) {
}

fn push_text(ops: &mut Vec<Op>, font_id: &printpdf::FontId, size: f32, y: f32, text: &str) {
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(14.0), Mm(y)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font_id.clone()),
        size: Pt(size),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.to_string())],
    });
    ops.push(Op::EndTextSection);
}

fn load_font_bytes() -> Result<(Vec<u8>, String), String> {
    let candidates = [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/Library/Fonts/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
    ];

    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists() {
            let bytes = fs::read(path).map_err(|err| err.to_string())?;
            return Ok((bytes, candidate.to_string()));
        }
    }

    Err("Не найден системный шрифт для PDF.".to_string())
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

fn percent(value: u64, total: u64) -> String {
    if total == 0 {
        "0%".to_string()
    } else {
        format!("{:.1}%", (value as f64 / total as f64) * 100.0)
    }
}

fn trim_field(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    let chars = trimmed.chars().count();
    if chars <= max_chars {
        trimmed.to_string()
    } else {
        let mut result = trimmed.chars().take(max_chars.saturating_sub(1)).collect::<String>();
        result.push('…');
        result
    }
}
