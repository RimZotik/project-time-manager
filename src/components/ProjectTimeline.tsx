// NLE-таймлайн проекта (как в монтажных программах): горизонтальная ось
// времени от начала до конца проекта, дорожки-этапы и дорожки-сессии с
// отрезками. Зум колесом, прокрутка, перетаскивание, тултипы.
import {
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type WheelEvent as ReactWheelEvent,
} from "react";
import type { ProjectStageRecord, SessionRecord } from "../lib/types";

type Lang = "ru" | "en";

const STAGE_COLORS = [
  "#059669",
  "#2563eb",
  "#7c3aed",
  "#db2777",
  "#ea580c",
  "#0891b2",
  "#65a30d",
  "#dc2626",
  "#0d9488",
  "#9333ea",
];

const ROW_H = 40;
const AXIS_H = 30;
const LABEL_W = 150;
const MS = { hour: 3600e3, day: 86400e3 };
const TICK_STEPS = [
  MS.hour,
  2 * MS.hour,
  3 * MS.hour,
  6 * MS.hour,
  12 * MS.hour,
  MS.day,
  2 * MS.day,
  7 * MS.day,
  14 * MS.day,
  30 * MS.day,
];

type Seg = {
  id: string;
  start: number;
  stop: number;
  durationSec: number;
  stageIds: string[];
  stageNames: string[];
};

function parseTs(value: string | null | undefined): number | null {
  if (!value) return null;
  // Обрезаем нано/микросекунды до миллисекунд, иначе Date может не распарсить.
  const clean = value.replace(/\.(\d{3})\d+/, ".$1");
  const ms = Date.parse(clean);
  return Number.isNaN(ms) ? null : ms;
}

function fmtDuration(sec: number, lang: Lang): string {
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  if (h > 0) return lang === "en" ? `${h}h ${m}m` : `${h} ч ${m} мин`;
  return lang === "en" ? `${m}m` : `${m} мин`;
}

function fmtDateTime(ms: number, lang: Lang, withTime: boolean): string {
  return new Date(ms).toLocaleString(lang === "en" ? "en-US" : "ru-RU", {
    day: "2-digit",
    month: "short",
    ...(withTime ? { hour: "2-digit", minute: "2-digit" } : {}),
  });
}

export default function ProjectTimeline({
  sessions,
  stages,
  language,
}: {
  sessions: SessionRecord[];
  stages: ProjectStageRecord[];
  language: Lang;
}) {
  const lang = language;
  const labels =
    lang === "en"
      ? {
          stages: "Stages",
          sessions: "Sessions",
          empty: "No sessions yet — start tracking to see the timeline.",
          fit: "Fit",
          hint: "Scroll to zoom, drag to pan",
          untagged: "No stage",
        }
      : {
          stages: "Этапы",
          sessions: "Сессии",
          empty: "Сессий пока нет — начните запись, и появится таймлайн.",
          fit: "Вписать",
          hint: "Колесо — зум, перетаскивание — прокрутка",
          untagged: "Без этапа",
        };

  const [mode, setMode] = useState<"stages" | "sessions">("stages");
  const wrapRef = useRef<HTMLDivElement>(null);
  const [pxPerMs, setPxPerMs] = useState<number | null>(null);
  const [fitPx, setFitPx] = useState<number>(0);
  const pendingScroll = useRef<number | null>(null);
  const drag = useRef<{ x: number; scroll: number } | null>(null);

  const segs = useMemo<Seg[]>(() => {
    return sessions
      .map((s) => {
        const start = parseTs(s.started_at);
        if (start === null) return null;
        const stop =
          parseTs(s.stopped_at) ?? start + (s.duration_seconds || 0) * 1000;
        return {
          id: s.id,
          start,
          stop: Math.max(stop, start + 1000),
          durationSec: s.duration_seconds,
          stageIds: s.stages.map((x) => x.id),
          stageNames: s.stages.map((x) => x.name),
        } as Seg;
      })
      .filter((x): x is Seg => x !== null)
      .sort((a, b) => a.start - b.start);
  }, [sessions]);

  const domain = useMemo(() => {
    if (!segs.length) return null;
    const start = Math.min(...segs.map((s) => s.start));
    const end = Math.max(...segs.map((s) => s.stop));
    const pad = Math.max((end - start) * 0.03, MS.hour);
    return { start: start - pad, end: end + pad };
  }, [segs]);

  const totalMs = domain ? domain.end - domain.start : 0;

  // Начальный масштаб — вписать в ширину контейнера.
  useLayoutEffect(() => {
    if (!wrapRef.current || !totalMs) return;
    const w = wrapRef.current.clientWidth - 16;
    const fit = w / totalMs;
    setFitPx(fit);
    setPxPerMs((prev) => prev ?? fit);
  }, [totalMs]);

  // Применяем отложенную прокрутку после смены зума (якорь под курсором).
  useLayoutEffect(() => {
    if (pendingScroll.current !== null && wrapRef.current) {
      wrapRef.current.scrollLeft = pendingScroll.current;
      pendingScroll.current = null;
    }
  }, [pxPerMs]);

  const stageColor = (stageId: string) => {
    const idx = stages.findIndex((s) => s.id === stageId);
    return STAGE_COLORS[(idx < 0 ? 0 : idx) % STAGE_COLORS.length];
  };

  if (!domain || pxPerMs === null) {
    return (
      <div
        ref={wrapRef}
        className="grid min-h-[200px] flex-1 place-items-center rounded-[24px] border border-dashed border-emerald-200 bg-white/60 p-8 text-center text-sm text-slate-500"
      >
        {segs.length ? "…" : labels.empty}
      </div>
    );
  }

  const trackW = totalMs * pxPerMs;

  const rows: { key: string; label: string; color: string; items: Seg[] }[] =
    mode === "stages"
      ? [...stages]
          .sort((a, b) => a.order - b.order)
          .map((stage) => ({
            key: stage.id,
            label: stage.name,
            color: stageColor(stage.id),
            items: segs.filter((s) => s.stageIds.includes(stage.id)),
          }))
      : segs.map((s, i) => ({
          key: s.id,
          label: fmtDateTime(s.start, lang, true),
          color: STAGE_COLORS[i % STAGE_COLORS.length],
          items: [s],
        }));

  // Тики оси времени.
  const step =
    TICK_STEPS.find((s) => s * pxPerMs >= 90) ?? TICK_STEPS[TICK_STEPS.length - 1];
  const withTime = step < MS.day;
  const ticks: number[] = [];
  const first = Math.ceil(domain.start / step) * step;
  for (let tMs = first; tMs <= domain.end; tMs += step) ticks.push(tMs);

  function onWheel(e: ReactWheelEvent<HTMLDivElement>) {
    if (!wrapRef.current || pxPerMs === null) return;
    e.preventDefault();
    const rect = wrapRef.current.getBoundingClientRect();
    const cursorX = e.clientX - rect.left + wrapRef.current.scrollLeft;
    const timeAtCursor = cursorX / pxPerMs;
    const factor = e.deltaY < 0 ? 1.2 : 1 / 1.2;
    const next = Math.min(Math.max(pxPerMs * factor, fitPx), fitPx * 600);
    pendingScroll.current = timeAtCursor * next - (e.clientX - rect.left);
    setPxPerMs(next);
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
      {/* Панель управления */}
      <div className="flex shrink-0 items-center justify-between gap-3 pb-3">
        <div className="inline-flex rounded-2xl border border-slate-200 bg-slate-50 p-1">
          {(
            [
              ["stages", labels.stages],
              ["sessions", labels.sessions],
            ] as const
          ).map(([key, label]) => (
            <button
              key={key}
              onClick={() => setMode(key)}
              className={`rounded-xl px-4 py-1.5 text-sm font-medium transition-colors ${
                mode === key
                  ? "bg-emerald-600 text-white"
                  : "text-slate-500 hover:text-emerald-700"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-3">
          <span className="hidden text-xs text-slate-400 sm:inline">
            {labels.hint}
          </span>
          <button
            onClick={() => {
              pendingScroll.current = 0;
              setPxPerMs(fitPx);
            }}
            className="rounded-xl border border-slate-200 px-3 py-1.5 text-sm font-medium text-slate-600 transition-colors hover:border-emerald-200 hover:text-emerald-700"
          >
            {labels.fit}
          </button>
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        {/* Колонка подписей */}
        <div
          className="shrink-0 overflow-hidden"
          style={{ width: LABEL_W }}
        >
          <div style={{ height: AXIS_H }} />
          <div className="overflow-y-auto" style={{ maxHeight: "100%" }}>
            {rows.map((row) => (
              <div
                key={row.key}
                className="flex items-center gap-2 truncate pr-2 text-sm text-slate-600"
                style={{ height: ROW_H }}
              >
                <span
                  className="size-2.5 shrink-0 rounded-full"
                  style={{ background: row.color }}
                />
                <span className="truncate">{row.label}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Область таймлайна */}
        <div
          ref={wrapRef}
          onWheel={onWheel}
          onMouseDown={(e) => {
            if (!wrapRef.current) return;
            drag.current = { x: e.clientX, scroll: wrapRef.current.scrollLeft };
          }}
          onMouseMove={(e) => {
            if (!drag.current || !wrapRef.current) return;
            wrapRef.current.scrollLeft =
              drag.current.scroll - (e.clientX - drag.current.x);
          }}
          onMouseUp={() => (drag.current = null)}
          onMouseLeave={() => (drag.current = null)}
          className="min-h-0 flex-1 cursor-grab overflow-x-auto overflow-y-auto active:cursor-grabbing"
        >
          <div className="relative" style={{ width: trackW }}>
            {/* Ось + вертикальные линии сетки */}
            <div
              className="sticky top-0 z-10 border-b border-slate-100 bg-white"
              style={{ height: AXIS_H }}
            >
              {ticks.map((tMs) => (
                <div
                  key={tMs}
                  className="absolute top-0 h-full border-l border-slate-100"
                  style={{ left: (tMs - domain.start) * pxPerMs }}
                >
                  <span className="ml-1 whitespace-nowrap text-[11px] text-slate-400">
                    {fmtDateTime(tMs, lang, withTime)}
                  </span>
                </div>
              ))}
            </div>

            {/* Дорожки */}
            {rows.map((row, rowIdx) => (
              <div
                key={row.key}
                className={`relative border-b border-slate-50 ${
                  rowIdx % 2 ? "bg-slate-50/40" : ""
                }`}
                style={{ height: ROW_H }}
              >
                {/* линии сетки внутри дорожки */}
                {ticks.map((tMs) => (
                  <div
                    key={tMs}
                    className="absolute top-0 h-full border-l border-slate-100"
                    style={{ left: (tMs - domain.start) * pxPerMs }}
                  />
                ))}
                {row.items.map((seg) => {
                  const left = (seg.start - domain.start) * pxPerMs;
                  const width = Math.max(3, (seg.stop - seg.start) * pxPerMs);
                  const tip = `${fmtDateTime(seg.start, lang, true)} — ${fmtDateTime(
                    seg.stop,
                    lang,
                    true,
                  )}\n${fmtDuration(seg.durationSec, lang)}${
                    seg.stageNames.length ? `\n${seg.stageNames.join(", ")}` : ""
                  }`;
                  return (
                    <div
                      key={seg.id}
                      title={tip}
                      className="absolute top-1.5 rounded-md shadow-sm transition-[filter] hover:brightness-95"
                      style={{
                        left,
                        width,
                        height: ROW_H - 12,
                        background: `linear-gradient(90deg, ${row.color}, ${row.color}cc)`,
                      }}
                    />
                  );
                })}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
