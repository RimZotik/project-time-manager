import { useEffect, useMemo, useState, type ReactNode } from "react";
import { RefreshCw } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import { invokeCommand } from "../lib/api";
import type { AnalyticsPayload } from "../lib/types";
import PageHeader from "../components/ui/PageHeader";

const EMPTY: AnalyticsPayload = { projects: [], sessions: [], top_apps: [] };
const EMERALD = "#059669";
const AQUA = "#0891b2";
// Изумрудная секвенциальная шкала (монотонная по светлоте — CVD-безопасна).
const HEAT = ["#eef6f1", "#d1fae5", "#a7f3d0", "#6ee7b7", "#34d399", "#10b981", "#047857"];

function fmtH(sec: number, lang: "ru" | "en"): string {
  if (sec >= 3600) {
    const h = sec / 3600;
    return `${h >= 10 ? Math.round(h) : h.toFixed(1)}${lang === "en" ? "h" : " ч"}`;
  }
  return `${Math.round(sec / 60)}${lang === "en" ? "m" : " м"}`;
}

function parseTs(value: string): number | null {
  const ms = Date.parse(value.replace(/\.(\d{3})\d+/, ".$1"));
  return Number.isNaN(ms) ? null : ms;
}

export default function AnalyticsPage() {
  const { state, language } = useAppState();
  const t = shellCopy[language].pages.analytics;
  const lang = language;
  const [data, setData] = useState<AnalyticsPayload>(EMPTY);

  async function load() {
    setData(await invokeCommand<AnalyticsPayload>("get_analytics", {}, EMPTY));
  }
  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const L =
    lang === "en"
      ? {
          totalTime: "Total time",
          sessions: "Sessions",
          projects: "Projects",
          avgSession: "Avg. session",
          byProject: "Time by project",
          byCategory: "Time by category",
          daily: "Daily activity",
          heatmap: "When you work",
          topApps: "Top applications",
          noCategory: "No category",
          noData: "No data yet — track some time first.",
          avgPerProject: "avg / project",
        }
      : {
          totalTime: "Всего времени",
          sessions: "Сессий",
          projects: "Проектов",
          avgSession: "Средняя сессия",
          byProject: "Время по проектам",
          byCategory: "Время по категориям",
          daily: "Активность по дням",
          heatmap: "Когда ты работаешь",
          topApps: "Топ приложения",
          noCategory: "Без категории",
          noData: "Данных пока нет — сначала поучитывай время.",
          avgPerProject: "в среднем / проект",
        };

  const weekdays =
    lang === "en"
      ? ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
      : ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

  // ── Агрегации ───────────────────────────────────────────────
  const totalSeconds = data.projects.reduce((s, p) => s + p.total_seconds, 0);
  const totalSessions = data.projects.reduce((s, p) => s + p.session_count, 0);
  const avgSession = totalSessions ? totalSeconds / totalSessions : 0;

  const byProject = useMemo(
    () => [...data.projects].sort((a, b) => b.total_seconds - a.total_seconds),
    [data.projects],
  );

  const byCategory = useMemo(() => {
    const map = new Map<string, number>();
    const counts = new Map<string, number>();
    for (const p of data.projects) {
      const key = p.category_id ?? "__none__";
      map.set(key, (map.get(key) ?? 0) + p.total_seconds);
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return [...map.entries()]
      .map(([key, seconds]) => {
        const cat = state.categories.find((c) => c.id === key);
        return {
          key,
          label: cat ? cat.name : L.noCategory,
          color: cat ? cat.color : "#cbd5e1",
          seconds,
          projects: counts.get(key) ?? 0,
        };
      })
      .sort((a, b) => b.seconds - a.seconds);
  }, [data.projects, state.categories, L.noCategory]);

  const daily = useMemo(() => {
    const map = new Map<string, number>();
    for (const s of data.sessions) {
      const ms = parseTs(s.started_at);
      if (ms === null) continue;
      const d = new Date(ms);
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
      map.set(key, (map.get(key) ?? 0) + s.duration_seconds);
    }
    return [...map.entries()]
      .sort((a, b) => a[0].localeCompare(b[0]))
      .slice(-30)
      .map(([date, seconds]) => ({ date, seconds }));
  }, [data.sessions]);

  const heat = useMemo(() => {
    const grid: number[][] = Array.from({ length: 7 }, () =>
      Array(24).fill(0),
    );
    for (const s of data.sessions) {
      const start = parseTs(s.started_at);
      if (start === null) continue;
      let cur = start;
      const stop = start + s.duration_seconds * 1000;
      let guard = 0;
      while (cur < stop && guard++ < 1000) {
        const d = new Date(cur);
        const wd = (d.getDay() + 6) % 7;
        const hr = d.getHours();
        const next = new Date(d);
        next.setMinutes(0, 0, 0);
        next.setHours(hr + 1);
        const chunkEnd = Math.min(stop, next.getTime());
        grid[wd][hr] += (chunkEnd - cur) / 1000;
        cur = chunkEnd;
      }
    }
    return grid;
  }, [data.sessions]);

  const heatMax = Math.max(1, ...heat.flat());
  const projMax = Math.max(1, ...byProject.map((p) => p.total_seconds));
  const appMax = Math.max(1, ...data.top_apps.map((a) => a.seconds));

  const hasData = totalSeconds > 0 || data.sessions.length > 0;

  // Donut через conic-gradient.
  const donutStops = (() => {
    let acc = 0;
    const total = byCategory.reduce((s, c) => s + c.seconds, 0) || 1;
    return byCategory
      .map((c) => {
        const from = (acc / total) * 100;
        acc += c.seconds;
        const to = (acc / total) * 100;
        return `${c.color} ${from}% ${to}%`;
      })
      .join(", ");
  })();

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title={t.title}
        subtitle={t.subtitle}
        actions={
          <button
            onClick={load}
            className="grid size-10 place-items-center rounded-full border border-slate-200 bg-white/70 text-slate-500 backdrop-blur transition-colors hover:border-emerald-200 hover:text-emerald-700"
            title="↻"
          >
            <RefreshCw size={18} />
          </button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto px-8 pb-10">
        {!hasData ? (
          <div className="grid h-full place-items-center text-sm text-slate-500">
            {L.noData}
          </div>
        ) : (
          <div className="mx-auto flex max-w-5xl flex-col gap-5">
            {/* KPI */}
            <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
              {[
                [L.totalTime, fmtH(totalSeconds, lang)],
                [L.sessions, String(totalSessions)],
                [L.projects, String(data.projects.length)],
                [L.avgSession, fmtH(avgSession, lang)],
              ].map(([label, value]) => (
                <div
                  key={label}
                  className="rounded-[22px] border border-emerald-100 bg-white/80 p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur"
                >
                  <p className="text-xs font-semibold uppercase tracking-wide text-slate-400">
                    {label}
                  </p>
                  <p className="mt-2 text-2xl font-bold tabular-nums text-slate-900">
                    {value}
                  </p>
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
              {/* Время по проектам */}
              <Card title={L.byProject}>
                <div className="flex flex-col gap-2.5">
                  {byProject.map((p) => (
                    <div key={p.id} className="flex items-center gap-3">
                      <span className="w-28 shrink-0 truncate text-sm text-slate-600">
                        {p.name}
                      </span>
                      <div
                        className="relative h-6 flex-1 overflow-hidden rounded-md bg-slate-100"
                        title={`${p.name}: ${fmtH(p.total_seconds, lang)}`}
                      >
                        <div
                          className="absolute inset-y-0 left-0 rounded-md"
                          style={{
                            width: `${(p.total_seconds / projMax) * 100}%`,
                            background: `linear-gradient(90deg, ${EMERALD}, #34d399)`,
                          }}
                        />
                      </div>
                      <span className="w-14 shrink-0 text-right text-sm font-semibold tabular-nums text-slate-700">
                        {fmtH(p.total_seconds, lang)}
                      </span>
                    </div>
                  ))}
                </div>
              </Card>

              {/* Время по категориям */}
              <Card title={L.byCategory}>
                <div className="flex items-center gap-6">
                  <div className="relative grid size-40 shrink-0 place-items-center">
                    <div
                      className="size-40 rounded-full"
                      style={{ background: `conic-gradient(${donutStops})` }}
                    />
                    <div className="absolute grid size-24 place-items-center rounded-full bg-white text-center">
                      <span className="text-lg font-bold tabular-nums text-slate-900">
                        {fmtH(totalSeconds, lang)}
                      </span>
                    </div>
                  </div>
                  <div className="flex min-w-0 flex-1 flex-col gap-2">
                    {byCategory.map((c) => (
                      <div key={c.key} className="flex items-center gap-2 text-sm">
                        <span
                          className="size-3 shrink-0 rounded-full"
                          style={{ background: c.color }}
                        />
                        <span className="min-w-0 flex-1 truncate text-slate-600">
                          {c.label}
                        </span>
                        <span className="shrink-0 font-semibold tabular-nums text-slate-800">
                          {fmtH(c.seconds, lang)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </Card>
            </div>

            {/* Активность по дням */}
            <Card title={L.daily}>
              <div className="flex h-40 items-end gap-1">
                {daily.map((d) => {
                  const max = Math.max(1, ...daily.map((x) => x.seconds));
                  return (
                    <div
                      key={d.date}
                      className="group relative flex-1"
                      title={`${d.date}: ${fmtH(d.seconds, lang)}`}
                    >
                      <div
                        className="w-full rounded-t-md transition-colors group-hover:brightness-95"
                        style={{
                          height: `${Math.max(3, (d.seconds / max) * 150)}px`,
                          background: `linear-gradient(180deg, #34d399, ${EMERALD})`,
                        }}
                      />
                    </div>
                  );
                })}
              </div>
              <div className="mt-2 flex justify-between text-[11px] text-slate-400">
                <span>{daily[0]?.date.slice(5)}</span>
                <span>{daily[daily.length - 1]?.date.slice(5)}</span>
              </div>
            </Card>

            {/* Тепловая карта */}
            <Card title={L.heatmap}>
              <div className="overflow-x-auto">
                <div className="inline-block">
                  <div className="flex gap-[3px] pl-9">
                    {Array.from({ length: 24 }).map((_, h) => (
                      <div
                        key={h}
                        className="w-[14px] text-center text-[9px] text-slate-400"
                      >
                        {h % 3 === 0 ? h : ""}
                      </div>
                    ))}
                  </div>
                  {heat.map((row, wd) => (
                    <div key={wd} className="mt-[3px] flex items-center gap-[3px]">
                      <span className="w-9 shrink-0 text-[11px] text-slate-400">
                        {weekdays[wd]}
                      </span>
                      {row.map((val, h) => {
                        const idx = val
                          ? Math.min(
                              HEAT.length - 1,
                              1 + Math.floor((val / heatMax) * (HEAT.length - 2)),
                            )
                          : 0;
                        return (
                          <div
                            key={h}
                            className="size-[14px] rounded-[3px]"
                            style={{ background: HEAT[idx] }}
                            title={`${weekdays[wd]} ${h}:00 — ${fmtH(val, lang)}`}
                          />
                        );
                      })}
                    </div>
                  ))}
                </div>
              </div>
            </Card>

            {/* Топ приложения */}
            <Card title={L.topApps}>
              <div className="flex flex-col gap-2.5">
                {data.top_apps.slice(0, 12).map((a) => (
                  <div key={a.name} className="flex items-center gap-3">
                    <span className="w-40 shrink-0 truncate text-sm text-slate-600">
                      {a.name}
                    </span>
                    <div
                      className="relative h-6 flex-1 overflow-hidden rounded-md bg-slate-100"
                      title={`${a.name}: ${fmtH(a.seconds, lang)}`}
                    >
                      <div
                        className="absolute inset-y-0 left-0 rounded-md"
                        style={{
                          width: `${(a.seconds / appMax) * 100}%`,
                          background: `linear-gradient(90deg, ${AQUA}, #22d3ee)`,
                        }}
                      />
                    </div>
                    <span className="w-14 shrink-0 text-right text-sm font-semibold tabular-nums text-slate-700">
                      {fmtH(a.seconds, lang)}
                    </span>
                  </div>
                ))}
              </div>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}

function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
      <h2 className="mb-4 text-sm font-semibold text-slate-900">{title}</h2>
      {children}
    </section>
  );
}
