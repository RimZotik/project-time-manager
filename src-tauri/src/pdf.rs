use crate::models::{AppUsageRecord, ProjectRecord};
use printpdf::{Base64OrRaw, GeneratePdfOptions, PdfDocument, PdfSaveOptions, PdfWarnMsg};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const APP_ROWS_PER_PAGE: usize = 20;
const TAB_ROWS_PER_PAGE: usize = 18;
const SESSION_ROWS_PER_PAGE: usize = 22;

pub fn export_project_pdf(
    project: &ProjectRecord,
    output_path: PathBuf,
) -> Result<PathBuf, String> {
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
    let mut pages = Vec::new();
    pages.push(overview_page(project, apps, tabs, total_seconds));
    pages.extend(app_pages(apps, total_seconds));
    pages.extend(tab_pages(tabs, total_seconds));
    pages.extend(session_pages(project));

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
      font-size: 10.5px;
    }}
    .page {{
      width: 210mm;
      height: 297mm;
      box-sizing: border-box;
      padding: 13mm 12mm;
      page-break-after: always;
      overflow: hidden;
      background: #f8fbf8;
    }}
    .page-title {{
      margin: 0 0 12px;
      padding: 11px 16px;
      border-radius: 18px 18px 18px 5px;
      background: #059669;
      color: #ffffff;
      text-align: center;
      font-size: 18px;
      line-height: 1.15;
      font-weight: 800;
    }}
    .hero {{
      margin-bottom: 12px;
      padding: 18px 20px;
      border-radius: 24px;
      background: linear-gradient(135deg, #047857 0%, #10b981 58%, #a7f3d0 100%);
      color: #ffffff;
    }}
    .eyebrow {{
      display: inline-block;
      padding: 5px 10px;
      border-radius: 999px;
      background: rgba(255,255,255,0.22);
      font-size: 9px;
      font-weight: 800;
      letter-spacing: 0.9px;
      text-transform: uppercase;
    }}
    h1 {{
      margin: 11px 0 6px;
      font-size: 27px;
      line-height: 1.12;
    }}
    .muted {{ color: #64748b; }}
    .metrics {{
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 8px;
      margin-bottom: 12px;
    }}
    .metric {{
      min-height: 55px;
      padding: 10px;
      border: 1px solid #d1fae5;
      border-radius: 16px;
      background: #ffffff;
    }}
    .metric span {{
      display: block;
      color: #64748b;
      font-size: 8.5px;
      font-weight: 800;
      letter-spacing: 0.65px;
      text-transform: uppercase;
    }}
    .metric strong {{
      display: block;
      margin-top: 7px;
      color: #064e3b;
      font-size: 13.5px;
      line-height: 1.15;
    }}
    .panel {{
      margin-top: 10px;
      padding: 12px;
      border: 1px solid #d1fae5;
      border-radius: 20px;
      background: #ffffff;
    }}
    .panel h2 {{
      margin: 0 0 9px;
      color: #065f46;
      font-size: 13px;
    }}
    .info-grid {{
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 7px;
    }}
    .info {{
      padding: 8px 10px;
      border-radius: 13px;
      background: #f8fafc;
    }}
    .info span {{
      display: block;
      color: #64748b;
      font-size: 9px;
    }}
    .info strong {{
      display: block;
      margin-top: 4px;
      color: #0f172a;
      font-size: 11px;
    }}
    .bar-row {{
      display: grid;
      grid-template-columns: 118px 1fr 62px;
      gap: 8px;
      align-items: center;
      margin-top: 8px;
      font-size: 9.5px;
    }}
    .bar-label {{
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      color: #0f172a;
      font-weight: 700;
    }}
    .bar-track {{
      height: 9px;
      border-radius: 999px;
      background: #ecfdf5;
      overflow: hidden;
    }}
    .bar-fill {{
      height: 9px;
      background: linear-gradient(90deg, #10b981, #84cc16);
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
      font-size: 9.5px;
    }}
    th {{
      padding: 7px 8px;
      background: #d1fae5;
      color: #064e3b;
      border: 1px solid #bbf7d0;
      text-align: left;
      font-weight: 800;
    }}
    td {{
      padding: 7px 8px;
      border: 1px solid #e2e8f0;
      background: #ffffff;
      vertical-align: top;
    }}
    .truncate {{
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
    }}
    .pill {{
      display: inline-block;
      padding: 3px 7px;
      border-radius: 999px;
      background: #ecfdf5;
      color: #047857;
      font-weight: 800;
    }}
    .empty {{
      margin: 0;
      padding: 14px;
      border: 1px dashed #cbd5e1;
      border-radius: 16px;
      background: #f8fafc;
      color: #64748b;
    }}
  </style>
</head>
<body>{pages}</body>
</html>"#,
        pages = pages.join("")
    )
}

fn overview_page(
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

    page(
        "Отчет по проекту",
        &format!(
            r#"<div class="hero">
  <span class="eyebrow">Project Time Manager</span>
  <h1>{project_name}</h1>
  <div>Учитываются только включенные приложения и домены.</div>
</div>
<div class="metrics">
  {metric_total}
  {metric_sessions}
  {metric_top_app}
  {metric_top_tab}
</div>
<div class="panel">
  <h2>Общая информация</h2>
  <div class="info-grid">
    {info_first}
    {info_last}
    {info_apps}
    {info_tabs}
  </div>
</div>
<div class="panel">
  <h2>Распределение по приложениям</h2>
  {app_bars}
</div>"#,
            project_name = escape_html(&project.name),
            metric_total = metric_card("Всего по проекту", &format_duration(total_seconds)),
            metric_sessions = metric_card("Всего сеансов", &project.sessions.len().to_string()),
            metric_top_app = metric_card("Топ приложение", top_app),
            metric_top_tab = metric_card("Топ домен", top_tab),
            info_first = info_tile("Дата начала", first_session),
            info_last = info_tile("Последняя сессия", last_session),
            info_apps = info_tile("Приложений", &apps.len().to_string()),
            info_tabs = info_tile("Доменов", &tabs.len().to_string()),
            app_bars = bars_html(
                apps.iter().take(7).map(|app| (&app.name, app.seconds)),
                total_seconds
            ),
        ),
    )
}

fn app_pages(apps: &[IncludedApp], total_seconds: u64) -> Vec<String> {
    if apps.is_empty() {
        return vec![page("Приложения", &empty_html())];
    }

    apps.chunks(APP_ROWS_PER_PAGE)
        .enumerate()
        .map(|(index, chunk)| {
            let suffix = if index == 0 {
                ""
            } else {
                " продолжение"
            };
            page(
                &format!("Приложения{suffix}"),
                &simple_table(
                    &[
                        ("Название", "44%"),
                        ("Тип", "18%"),
                        ("Время", "20%"),
                        ("%", "18%"),
                    ],
                    &chunk
                        .iter()
                        .map(|app| {
                            vec![
                                escape_html(&app.name),
                                format!(
                                    r#"<span class="pill">{}</span>"#,
                                    app_kind_label(&app.kind)
                                ),
                                format_duration(app.seconds),
                                percent(app.seconds, total_seconds),
                            ]
                        })
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect()
}

fn tab_pages(tabs: &[IncludedTab], total_seconds: u64) -> Vec<String> {
    if tabs.is_empty() {
        return vec![page("Домены и ссылки", &empty_html())];
    }

    tabs.chunks(TAB_ROWS_PER_PAGE)
        .enumerate()
        .map(|(index, chunk)| {
            let suffix = if index == 0 {
                ""
            } else {
                " продолжение"
            };
            page(
                &format!("Домены и ссылки{suffix}"),
                &simple_table(
                    &[
                        ("Браузер", "24%"),
                        ("Домен", "34%"),
                        ("Ссылок", "14%"),
                        ("Время", "16%"),
                        ("%", "12%"),
                    ],
                    &chunk
                        .iter()
                        .map(|tab| {
                            vec![
                                escape_html(&tab.browser),
                                escape_html(&tab.title),
                                tab.url_count.to_string(),
                                format_duration(tab.seconds),
                                percent(tab.seconds, total_seconds),
                            ]
                        })
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect()
}

fn session_pages(project: &ProjectRecord) -> Vec<String> {
    if project.sessions.is_empty() {
        return vec![page("Сеансы", &empty_html())];
    }

    project
        .sessions
        .chunks(SESSION_ROWS_PER_PAGE)
        .enumerate()
        .map(|(index, chunk)| {
            let suffix = if index == 0 {
                ""
            } else {
                " продолжение"
            };
            page(
                &format!("Сеансы{suffix}"),
                &simple_table(
                    &[
                        ("Начало", "24%"),
                        ("Окончание", "24%"),
                        ("Длительность", "18%"),
                        ("Приложения", "17%"),
                        ("Браузеры", "17%"),
                    ],
                    &chunk
                        .iter()
                        .map(|session| {
                            vec![
                                escape_html(&session.started_at),
                                escape_html(session.stopped_at.as_deref().unwrap_or("-")),
                                format_duration(session.duration_seconds),
                                session.app_count.to_string(),
                                session.browser_count.to_string(),
                            ]
                        })
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect()
}

fn page(title: &str, body: &str) -> String {
    format!(
        r#"<section class="page"><h1 class="page-title">{}</h1>{}</section>"#,
        escape_html(title),
        body
    )
}

fn metric_card(label: &str, value: &str) -> String {
    format!(
        r#"<div class="metric"><span>{}</span><strong>{}</strong></div>"#,
        escape_html(label),
        escape_html(value)
    )
}

fn info_tile(label: &str, value: &str) -> String {
    format!(
        r#"<div class="info"><span>{}</span><strong>{}</strong></div>"#,
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
            (seconds as f64 / total_seconds as f64 * 100.0).clamp(4.0, 100.0)
        };
        output.push_str(&format!(
            r#"<div class="bar-row"><div class="bar-label">{}</div><div class="bar-track"><div class="bar-fill" style="width:{:.1}%"></div></div><span>{}</span></div>"#,
            escape_html(label),
            width,
            format_duration(seconds)
        ));
    }
    if output.is_empty() {
        empty_html()
    } else {
        output
    }
}

fn simple_table(headers: &[(&str, &str)], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return empty_html();
    }

    let header_cells = headers
        .iter()
        .map(|(label, width)| {
            format!(
                r#"<th style="width:{}">{}</th>"#,
                escape_html(width),
                escape_html(label)
            )
        })
        .collect::<String>();
    let body = rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .map(|value| format!(r#"<td><div class="truncate">{}</div></td>"#, value))
                .collect::<String>();
            format!("<tr>{cells}</tr>")
        })
        .collect::<String>();

    format!("<table><thead><tr>{header_cells}</tr></thead><tbody>{body}</tbody></table>")
}

fn empty_html() -> String {
    r#"<p class="empty">Данных пока нет.</p>"#.to_string()
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
