import { useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import { CirclePlay, FolderKanban } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import { invokeCommand } from "../lib/api";
import type { AnalyticsPayload } from "../lib/types";
import PageHeader from "../components/ui/PageHeader";

const EMPTY: AnalyticsPayload = { projects: [], sessions: [], top_apps: [] };
const EMERALD = "#059669";

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

function fmtClock(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(sec)}` : `${pad(m)}:${pad(sec)}`;
}

export default function DashboardPage() {
  const { state, language, refresh } = useAppState();
  const t = shellCopy[language].pages.dashboard;
  const lang = language;
  const [data, setData] = useState<AnalyticsPayload>(EMPTY);
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    invokeCommand<AnalyticsPayload>("get_analytics", {}, EMPTY).then(setData);
  }, [state.projects.length]);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const L =
    lang === "en"
      ? {
          recording: "Recording now",
          idle: "Not recording",
          idleHint: "Pick a project below to start tracking.",
          totalTime: "Total time",
          projects: "Projects",
          sessions: "Sessions",
          avgSession: "Avg. session",
          topProjects: "Top projects",
          week: "This week",
          quickStart: "Quick start",
          start: "Start",
          noProjects: "No projects yet.",
        }
      : {
          recording: "Идёт запись",
          idle: "Запись не идёт",
          idleHint: "Выбери проект ниже, чтобы начать учёт.",
          totalTime: "Всего времени",
          projects: "Проектов",
          sessions: "Сессий",
          avgSession: "Средняя сессия",
          topProjects: "Топ проекты",
          week: "За неделю",
          quickStart: "Быстрый старт",
          start: "Старт",
          noProjects: "Проектов пока нет.",
        };

  const weekdays =
    lang === "en"
      ? ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
      : ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

  const totalSeconds = data.projects.reduce((s, p) => s + p.total_seconds, 0);
  const totalSessions = data.projects.reduce((s, p) => s + p.session_count, 0);
  const avgSession = totalSessions ? totalSeconds / totalSessions : 0;

  const topProjects = useMemo(
    () =>
      [...data.projects].sort((a, b) => b.total_seconds - a.total_seconds).slice(0, 5),
    [data.projects],
  );
  const projMax = Math.max(1, ...topProjects.map((p) => p.total_seconds));

  // Последние 7 дней.
  const week = useMemo(() => {
    const days: { label: string; seconds: number }[] = [];
    const base = new Date();
    base.setHours(0, 0, 0, 0);
    for (let i = 6; i >= 0; i--) {
      const d = new Date(base);
      d.setDate(base.getDate() - i);
      days.push({ label: weekdays[(d.getDay() + 6) % 7], seconds: 0 });
    }
    const startBound = new Date(base);
    startBound.setDate(base.getDate() - 6);
    for (const s of data.sessions) {
      const ms = parseTs(s.started_at);
      if (ms === null || ms < startBound.getTime()) continue;
      const idx = Math.floor((ms - startBound.getTime()) / 86400000);
      if (idx >= 0 && idx < 7) days[idx].seconds += s.duration_seconds;
    }
    return days;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data.sessions]);
  const weekMax = Math.max(1, ...week.map((d) => d.seconds));

  const recording = state.tracker.status === "running";
  const activeName =
    state.projects.find((p) => p.id === state.tracker.active_project_id)?.name ??
    null;
  const runSince = state.tracker.running_since
    ? parseTs(state.tracker.running_since)
    : null;
  const elapsed = recording && runSince ? now - runSince : 0;

  const recentProjects = [...state.projects]
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
    .slice(0, 6);

  async function quickStart(projectId: string) {
    await invokeCommand<void>("select_project", { projectId }, undefined);
    await invokeCommand<void>("start_tracking", {}, undefined);
    refresh();
  }

  return (
    <div className="flex h-full flex-col">
      <PageHeader title={t.title} subtitle={t.subtitle} />
      <div className="min-h-0 flex-1 overflow-y-auto px-8 pb-10">
        <div className="mx-auto flex max-w-5xl flex-col gap-5">
          {/* Статус записи */}
          <section
            className={`flex items-center gap-4 rounded-[24px] border p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] ${
              recording
                ? "border-emerald-200 bg-emerald-50/70"
                : "border-slate-200 bg-white/80"
            } backdrop-blur`}
          >
            <span className="relative grid size-4 place-items-center">
              {recording ? (
                <>
                  <motion.span
                    className="absolute inset-0 rounded-full bg-emerald-500"
                    animate={{ scale: [1, 2], opacity: [0.6, 0] }}
                    transition={{ duration: 1.6, repeat: Infinity }}
                  />
                  <span className="size-4 rounded-full bg-emerald-500" />
                </>
              ) : (
                <span className="size-4 rounded-full border-2 border-slate-300" />
              )}
            </span>
            <div className="min-w-0 flex-1">
              <p className="text-sm font-semibold text-slate-900">
                {recording ? L.recording : L.idle}
              </p>
              <p className="truncate text-sm text-slate-500">
                {recording ? activeName : L.idleHint}
              </p>
            </div>
            {recording && (
              <span className="text-2xl font-bold tabular-nums text-emerald-700">
                {fmtClock(elapsed)}
              </span>
            )}
          </section>

          {/* KPI */}
          <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
            {[
              [L.totalTime, fmtH(totalSeconds, lang)],
              [L.projects, String(data.projects.length)],
              [L.sessions, String(totalSessions)],
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
            {/* Топ проекты */}
            <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
              <h2 className="mb-4 text-sm font-semibold text-slate-900">
                {L.topProjects}
              </h2>
              <div className="flex flex-col gap-2.5">
                {topProjects.length ? (
                  topProjects.map((p) => (
                    <div key={p.id} className="flex items-center gap-3">
                      <span className="w-24 shrink-0 truncate text-sm text-slate-600">
                        {p.name}
                      </span>
                      <div className="relative h-6 flex-1 overflow-hidden rounded-md bg-slate-100">
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
                  ))
                ) : (
                  <p className="text-sm text-slate-400">{L.noProjects}</p>
                )}
              </div>
            </section>

            {/* За неделю */}
            <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
              <h2 className="mb-4 text-sm font-semibold text-slate-900">
                {L.week}
              </h2>
              <div className="flex h-32 items-end gap-2">
                {week.map((d, i) => (
                  <div key={i} className="flex flex-1 flex-col items-center gap-1">
                    <div
                      className="w-full rounded-t-md"
                      style={{
                        height: `${Math.max(3, (d.seconds / weekMax) * 100)}px`,
                        background: `linear-gradient(180deg, #34d399, ${EMERALD})`,
                      }}
                      title={fmtH(d.seconds, lang)}
                    />
                    <span className="text-[11px] text-slate-400">{d.label}</span>
                  </div>
                ))}
              </div>
            </section>
          </div>

          {/* Быстрый старт */}
          <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <h2 className="mb-4 text-sm font-semibold text-slate-900">
              {L.quickStart}
            </h2>
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
              {recentProjects.map((p) => (
                <button
                  key={p.id}
                  onClick={() => quickStart(p.id)}
                  disabled={recording}
                  className="group flex items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-2.5 text-left transition-colors hover:border-emerald-300 hover:bg-emerald-50 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <span
                    className="grid size-8 shrink-0 place-items-center rounded-xl text-emerald-600"
                    style={{
                      background:
                        state.categories.find((c) => c.id === p.category_id)
                          ?.color ?? "#ecfdf5",
                    }}
                  >
                    <CirclePlay size={16} className="text-white" />
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-sm font-medium text-slate-800">
                      {p.name}
                    </span>
                  </span>
                </button>
              ))}
              {!recentProjects.length && (
                <div className="col-span-full flex items-center gap-2 text-sm text-slate-400">
                  <FolderKanban size={16} />
                  {L.noProjects}
                </div>
              )}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
