use crate::models::{AppUsageRecord, ProjectRecord};
use printpdf::{Base64OrRaw, GeneratePdfOptions, PdfDocument, PdfSaveOptions, PdfWarnMsg};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn export_project_pdf(project: &ProjectRecord, output_path: PathBuf) -> Result<PathBuf, String> {
    let (font_bytes, _) = load_font_bytes()?;
    let apps = included_apps(project);
    let tabs = included_tabs(project);
    let total_seconds: u64 = apps.iter().map(|app| app.seconds).sum();
    let html = build_report_html(project, &apps, &tabs, total_seconds);
    let mut fonts = BTreeMap::new();
    fonts.insert("ReportFont".to_string(), Base64OrRaw::Raw(font_bytes));

    let mut warnings = Vec::<PdfWarnMsg>::new();
    let document = PdfDocument::from_html(
        &html,
        &BTreeMap::new(),
        &fonts,
        &GeneratePdfOptions {
            page_width: Some(210.0),
            page_height: Some(297.0),
            margin_top: Some(0.0),
            margin_right: Some(0.0),
            margin_bottom: Some(0.0),
            margin_left: Some(0.0),
            show_page_numbers: Some(true),
            footer_text: Some("Project Time Manager".to_string()),
            ..GeneratePdfOptions::default()
        },
        &mut warnings,
    )?;

    let bytes = document.save(&PdfSaveOptions::default(), &mut warnings);
    fs::write(&output_path, bytes).map_err(|err| err.to_string())?;
    Ok(output_path)
}

fn build_report_html(
    project: &ProjectRecord,
    apps: &[IncludedApp],
    tabs: &[IncludedTab],
    total_seconds: u64,
) -> String {
    let top_app = apps.first().map(|app| app.name.as_str()).unwrap_or("-");
    let top_tab = tabs.first().map(|tab| tab.title.as_str()).unwrap_or("-");
    let first_session = project
        .sessions
        .first()
        .map(|session| session.started_at.as_str())
        .unwrap_or(project.created_at.as_str());
    let last_session = project
        .sessions
        .last()
        .and_then(|session| session.stopped_at.as_deref())
        .unwrap_or("-");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <style>
    body {{
      margin: 0;
      padding: 0;
      background: #f4faf6;
      color: #0f172a;
      font-family: ReportFont, Arial, sans-serif;
      font-size: 12px;
    }}
    .page {{
      width: 210mm;
      min-height: 297mm;
      padding: 14mm;
      box-sizing: border-box;
      page-break-after: always;
      background: #f8fbf8;
    }}
    .hero {{
      padding: 20px 22px;
      border-radius: 22px;
      background: linear-gradient(135deg, #047857 0%, #10b981 56%, #d9f99d 100%);
      color: white;
    }}
    .eyebrow {{
      display: inline-block;
      padding: 5px 10px;
      border-radius: 999px;
      background: rgba(255,255,255,0.22);
      font-size: 10px;
      font-weight: 700;
      letter-spacing: 1px;
      text-transform: uppercase;
    }}
    h1 {{
      margin: 14px 0 8px;
      font-size: 28px;
      line-height: 1.1;
    }}
    h2 {{
      margin: 0;
      padding: 9px 14px;
      border-radius: 16px 16px 16px 4px;
      background: #059669;
      color: white;
      font-size: 16px;
    }}
    .subtle {{ color: #64748b; }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;
      margin-top: 14px;
    }}
    .metric {{
      padding: 14px;
      border: 1px solid #d1fae5;
      border-radius: 18px;
      background: white;
    }}
    .metric span {{
      display: block;
      color: #64748b;
      font-size: 10px;
      font-weight: 700;
      letter-spacing: 0.9px;
      text-transform: uppercase;
    }}
    .metric strong {{
      display: block;
      margin-top: 8px;
      font-size: 18px;
      color: #064e3b;
    }}
    .section {{
      margin-top: 18px;
      padding: 14px;
      border: 1px solid #d1fae5;
      border-radius: 22px;
      background: white;
    }}
    .bar-row {{
      display: grid;
      grid-template-columns: 118px 1fr 70px;
      gap: 10px;
      align-items: center;
      margin-top: 10px;
      font-size: 11px;
    }}
    .bar-track {{
      height: 10px;
      border-radius: 999px;
      background: #ecfdf5;
      overflow: hidden;
    }}
    .bar-fill {{
      height: 10px;
      border-radius: 999px;
      background: linear-gradient(90deg, #10b981, #84cc16);
    }}
    table {{
      width: 100%;
      border-collapse: separate;
      border-spacing: 0 6px;
      margin-top: 10px;
      font-size: 10.5px;
    }}
    th {{
      padding: 8px 10px;
      background: #d1fae5;
      color: #064e3b;
      text-align: left;
    }}
    td {{
      padding: 8px 10px;
      background: #ffffff;
      border-top: 1px solid #e2e8f0;
      border-bottom: 1px solid #e2e8f0;
    }}
    td:first-child {{ border-left: 1px solid #e2e8f0; border-radius: 12px 0 0 12px; }}
    td:last-child {{ border-right: 1px solid #e2e8f0; border-radius: 0 12px 12px 0; }}
    .pill {{
      display: inline-block;
      padding: 3px 8px;
      border-radius: 999px;
      background: #ecfdf5;
      color: #047857;
      font-weight: 700;
    }}
  </style>
</head>
<body>
  <section class="page">
    <div class="hero">
      <span class="eyebrow">Отчет по проекту</span>
      <h1>{project_name}</h1>
      <div>Учитываются только включенные приложения и домены.</div>
    </div>
    <div class="grid">
      {metric_total}
      {metric_sessions}
      {metric_top_app}
      {metric_top_tab}
    </div>
    <div class="section">
      <h2>Общая информация</h2>
      <table>
        <tr><td>Дата начала</td><td>{first_session}</td></tr>
        <tr><td>Последняя сессия</td><td>{last_session}</td></tr>
        <tr><td>Приложений</td><td>{app_count}</td></tr>
        <tr><td>Доменов</td><td>{tab_count}</td></tr>
      </table>
    </div>
    <div class="section">
      <h2>Распределение по приложениям</h2>
      {app_bars}
    </div>
  </section>
  <section class="page">
    <div class="section">
      <h2>Приложения</h2>
      {apps_table}
    </div>
    <div class="section">
      <h2>Домены и ссылки</h2>
      {tabs_table}
    </div>
  </section>
  <section class="page">
    <div class="section">
      <h2>Сеансы</h2>
      {sessions_table}
    </div>
  </section>
</body>
</html>"#,
        project_name = escape_html(&project.name),
        first_session = escape_html(first_session),
        last_session = escape_html(last_session),
        app_count = apps.len(),
        tab_count = tabs.len(),
        metric_total = metric_card("Всего по проекту", &format_duration(total_seconds)),
        metric_sessions = metric_card("Всего сеансов", &project.sessions.len().to_string()),
        metric_top_app = metric_card("Топ приложение", top_app),
        metric_top_tab = metric_card("Топ домен", top_tab),
        app_bars = bars_html(apps.iter().take(8).map(|app| (&app.name, app.seconds)), total_seconds),
        apps_table = apps_table_html(apps, total_seconds),
        tabs_table = tabs_table_html(tabs, total_seconds),
        sessions_table = sessions_table_html(project),
    )
}

fn metric_card(label: &str, value: &str) -> String {
    format!(
        r#"<div class="metric"><span>{}</span><strong>{}</strong></div>"#,
        escape_html(label),
        escape_html(value)
    )
}

fn bars_html<'a>(items: impl Iterator<Item = (&'a String, u64)>, total_seconds: u64) -> String {
    let mut output = String::new();
    for (label, seconds) in items {
        let width = if total_seconds == 0 {
            0.0
        } else {
            (seconds as f64 / total_seconds as f64 * 100.0).clamp(3.0, 100.0)
        };
        output.push_str(&format!(
            r#"<div class="bar-row"><strong>{}</strong><div class="bar-track"><div class="bar-fill" style="width:{:.1}%"></div></div><span>{}</span></div>"#,
            escape_html(label),
            width,
            format_duration(seconds)
        ));
    }
    if output.is_empty() {
        output.push_str(r#"<p class="subtle">Данных пока нет.</p>"#);
    }
    output
}

fn apps_table_html(apps: &[IncludedApp], total_seconds: u64) -> String {
    let mut rows = String::new();
    for app in apps {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td><span class=\"pill\">{}</span></td><td>{}</td><td>{}</td></tr>",
            escape_html(&app.name),
            escape_html(&app.process_name),
            app_kind_label(&app.kind),
            format_duration(app.seconds),
            percent(app.seconds, total_seconds)
        ));
    }
    table_or_empty("Название|Процесс|Тип|Время|%", rows)
}

fn tabs_table_html(tabs: &[IncludedTab], total_seconds: u64) -> String {
    let mut rows = String::new();
    for tab in tabs {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&tab.browser),
            escape_html(&tab.title),
            tab.url_count,
            format_duration(tab.seconds),
            percent(tab.seconds, total_seconds)
        ));
    }
    table_or_empty("Браузер|Домен|Ссылок|Время|%", rows)
}

fn sessions_table_html(project: &ProjectRecord) -> String {
    let mut rows = String::new();
    for session in &project.sessions {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.2}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&session.started_at),
            escape_html(session.stopped_at.as_deref().unwrap_or("-")),
            format_duration(session.duration_seconds),
            seconds_to_hours(session.duration_seconds),
            session.app_count,
            session.browser_count
        ));
    }
    table_or_empty("Начало|Окончание|Длительность|Часы|Приложения|Браузер", rows)
}

fn table_or_empty(headers: &str, rows: String) -> String {
    if rows.is_empty() {
        return r#"<p class="subtle">Данных пока нет.</p>"#.to_string();
    }

    let header_cells = headers
        .split('|')
        .map(|label| format!("<th>{}</th>", escape_html(label)))
        .collect::<String>();
    format!("<table><thead><tr>{header_cells}</tr></thead><tbody>{rows}</tbody></table>")
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
    url_count: usize,
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
                let url_count = if tab.urls.is_empty() {
                    usize::from(tab.url.is_some())
                } else {
                    tab.urls.len()
                };
                Some(IncludedTab {
                    browser: app.name.clone(),
                    title: tab.title.clone(),
                    url_count,
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

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
