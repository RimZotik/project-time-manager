use crate::models::{AppUsageRecord, ProjectRecord, SessionStageSnapshot, TabUsageRecord};
use chrono::{DateTime, Duration, FixedOffset};
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
    let stages = included_stages(project);
    let total_seconds: u64 = apps.iter().map(|app| app.seconds).sum();
    let html = build_report_html(project, &apps, &tabs, &stages, total_seconds);
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
    stages: &[IncludedStage],
    total_seconds: u64,
) -> String {
    let mut pages = Vec::new();
    pages.push(overview_page(project, apps, tabs, total_seconds));
    pages.extend(app_pages(apps, total_seconds));
    pages.extend(tab_pages(tabs, total_seconds));
    pages.extend(session_pages(project));
    if !stages.is_empty() {
        pages.extend(stage_pages(stages));
        pages.extend(stage_timeline_pages(project, stages));
    }

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
        .timeline-meta {{
            margin: 0 0 10px;
            color: #475569;
            font-size: 9.5px;
        }}
        .timeline-axis {{
            position: relative;
            height: 24px;
            margin-bottom: 8px;
            border-bottom: 1px solid #cbd5e1;
        }}
        .timeline-axis span {{
            position: absolute;
            top: 0;
            transform: translateX(-50%);
            color: #64748b;
            font-size: 8px;
            white-space: nowrap;
        }}
        .timeline-axis span:first-child {{
            transform: none;
        }}
        .timeline-axis span:last-child {{
            transform: translateX(-100%);
        }}
        .timeline-row {{
            display: grid;
            grid-template-columns: 124px 1fr;
            gap: 10px;
            align-items: center;
            margin-bottom: 10px;
        }}
        .timeline-name {{
            font-size: 9px;
            font-weight: 700;
            color: #0f172a;
        }}
        .timeline-summary {{
            display: block;
            margin-top: 2px;
            color: #64748b;
            font-size: 8px;
            font-weight: 400;
        }}
        .timeline-track {{
            position: relative;
            height: 24px;
            border-radius: 999px;
            background: #ecfdf5;
            overflow: hidden;
            border: 1px solid #d1fae5;
        }}
        .timeline-block {{
            position: absolute;
            top: 3px;
            bottom: 3px;
            border-radius: 999px;
            background: linear-gradient(90deg, #059669 0%, #10b981 100%);
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
    <div>Учитываются только включенные приложения, сайты и ссылки.</div>
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
</div>"#,
            project_name = escape_html(&project.name),
            metric_total = metric_card("Всего по проекту", &format_duration(total_seconds)),
            metric_sessions = metric_card("Всего сеансов", &project.sessions.len().to_string()),
            metric_top_app = metric_card("Топ приложение", top_app),
            metric_top_tab = metric_card("Топ сайт", top_tab),
            info_first = info_tile("Дата начала", first_session),
            info_last = info_tile("Последняя сессия", last_session),
            info_apps = info_tile("Приложений", &apps.len().to_string()),
            info_tabs = info_tile("Сайтов", &tabs.len().to_string()),
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
        return vec![page("Сайты и ссылки", &empty_html())];
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
                &format!("Сайты и ссылки{suffix}"),
                &simple_table(
                    &[
                        ("Браузер", "24%"),
                        ("Сайт", "34%"),
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

    format!("<table><tbody><tr>{header_cells}</tr>{body}</tbody></table>")
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
                let seconds = included_tab_seconds(tab);
                if !tab.enabled || seconds == 0 {
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
                    seconds,
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
            .map(included_tab_seconds)
            .sum();
    }
    app.time_seconds
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
                continue;
            }

            let stage_data = project
                .stages
                .iter()
                .find(|item| item.id == snapshot.id)
                .map(|item| (item.name.clone(), item.order))
                .unwrap_or_else(|| (snapshot.name.clone(), usize::MAX));
            stages.push(IncludedStage {
                id: snapshot.id.clone(),
                name: stage_data.0,
                order: stage_data.1,
                seconds: session.duration_seconds,
                session_count: 1,
                first_used_at: session.started_at.clone(),
                last_used_at: last_used_at.clone(),
                segments: Vec::new(),
            });
        }
    }

    for stage in &mut stages {
        stage.segments = stage_segments(&timeline_sessions, &stage.id);
    }

    stages.sort_by(|left, right| left.order.cmp(&right.order).then(left.name.cmp(&right.name)));
    stages
}

fn stage_pages(stages: &[IncludedStage]) -> Vec<String> {
    if stages.is_empty() {
        return vec![];
    }

    stages
        .chunks(8)
        .enumerate()
        .map(|(index, chunk)| {
            let suffix = if index == 0 { "" } else { " продолжение" };
            page(
                &format!("Этапы{suffix}"),
                &simple_table(
                    &[
                        ("Название", "28%"),
                        ("Сеансов", "12%"),
                        ("Время", "16%"),
                        ("Первое использование", "22%"),
                        ("Последнее использование", "22%"),
                    ],
                    &chunk
                        .iter()
                        .map(|stage| {
                            vec![
                                escape_html(&stage.name),
                                stage.session_count.to_string(),
                                format_duration(stage.seconds),
                                escape_html(&stage.first_used_at),
                                escape_html(&stage.last_used_at),
                            ]
                        })
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect()
}

fn stage_timeline_pages(project: &ProjectRecord, stages: &[IncludedStage]) -> Vec<String> {
    let sessions = project_session_windows(project);
    let Some((timeline_start, timeline_end)) = timeline_bounds(&sessions) else {
        return Vec::new();
    };
    let total_seconds = (timeline_end - timeline_start).num_seconds().max(1) as f64;

    stages
        .chunks(8)
        .enumerate()
        .map(|(index, chunk)| {
            let suffix = if index == 0 { "" } else { " продолжение" };
            let axis = timeline_axis_html(timeline_start, timeline_end);
            let rows = chunk
                .iter()
                .map(|stage| {
                    let blocks = stage
                        .segments
                        .iter()
                        .map(|segment| {
                            let left = ((segment.start - timeline_start).num_seconds().max(0) as f64 / total_seconds) * 100.0;
                            let width = ((segment.end - segment.start).num_seconds().max(1) as f64 / total_seconds) * 100.0;
                            format!(
                                r#"<div class="timeline-block" style="left:{left:.2}%;width:{width:.2}%"></div>"#,
                            )
                        })
                        .collect::<String>();
                    format!(
                        r#"<div class="timeline-row"><div class="timeline-name">{name}<span class="timeline-summary">{summary}</span></div><div class="timeline-track">{blocks}</div></div>"#,
                        name = escape_html(&stage.name),
                        summary = escape_html(&format!("{} • {}", stage.session_count, format_duration(stage.seconds))),
                        blocks = blocks,
                    )
                })
                .collect::<String>();

            page(
                &format!("Таймлайн этапов{suffix}"),
                &format!(
                    r#"<p class="timeline-meta">Границы таймлайна: {} -> {}.</p>{axis}{rows}"#,
                    escape_html(&timeline_start.format("%d.%m.%Y %H:%M").to_string()),
                    escape_html(&timeline_end.format("%d.%m.%Y %H:%M").to_string()),
                ),
            )
        })
        .collect()
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
    session: &crate::models::SessionRecord,
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

fn timeline_axis_html(
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
) -> String {
    let total_seconds = (end - start).num_seconds().max(1) as f64;
    let markers = [0.0_f64, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|ratio| {
            let point = start + Duration::seconds((total_seconds * ratio) as i64);
            format!(
                r#"<span style="left:{:.2}%">{}</span>"#,
                ratio * 100.0,
                escape_html(&point.format("%d.%m %H:%M").to_string())
            )
        })
        .collect::<String>();
    format!(r#"<div class="timeline-axis">{markers}</div>"#)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProjectRecord, ProjectStageRecord, SessionRecord, SessionStageSnapshot};

    #[test]
    fn timeline_html_is_valid_for_pdf_generation() {
        let project = ProjectRecord {
            name: "Demo".to_string(),
            created_at: "2026-06-11T20:00:00+00:00".to_string(),
            sessions: vec![SessionRecord {
                id: "session-1".to_string(),
                started_at: "2026-06-11T20:00:00+00:00".to_string(),
                stopped_at: Some("2026-06-11T20:10:00+00:00".to_string()),
                duration_seconds: 600,
                app_count: 1,
                browser_count: 1,
                stages: vec![SessionStageSnapshot {
                    id: "stage-1".to_string(),
                    name: "Stage One".to_string(),
                }],
            }],
            stages: vec![ProjectStageRecord {
                id: "stage-1".to_string(),
                name: "Stage One".to_string(),
                order: 0,
                created_at: "2026-06-11T19:59:00+00:00".to_string(),
                updated_at: "2026-06-11T20:00:00+00:00".to_string(),
                ..ProjectStageRecord::default()
            }],
            ..ProjectRecord::default()
        };

        let html = build_report_html(&project, &[], &[], &included_stages(&project), 0);
        assert!(!html.contains("\\\"timeline-"));
        assert!(html.contains("class=\"timeline-block\""));

        let (font_bytes, _) = load_font_bytes().expect("font bytes");
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
                ..GeneratePdfOptions::default()
            },
            &mut warnings,
        );

        assert!(document.is_ok());
    }
}
