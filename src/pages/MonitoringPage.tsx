import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { AppWindow, Globe } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import { invokeCommand } from "../lib/api";
import type { ActiveWindowInfo } from "../lib/types";
import PageHeader from "../components/ui/PageHeader";

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
function fmtDur(sec: number): string {
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
}

export default function MonitoringPage() {
  const { state, language } = useAppState();
  const t = shellCopy[language].pages.monitoring;
  const lang = language;
  const [win, setWin] = useState<ActiveWindowInfo | null>(null);
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    let alive = true;
    const poll = async () => {
      const w = await invokeCommand<ActiveWindowInfo | null>(
        "get_active_window",
        {},
        null,
      );
      if (alive) setWin(w);
    };
    poll();
    const timer = window.setInterval(poll, 2000);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const L =
    lang === "en"
      ? {
          status: "Session",
          recording: "Recording",
          paused: "Paused",
          stopped: "Stopped",
          project: "Project",
          none: "—",
          activeWindow: "Active window now",
          noWindow: "Active-window tracking works on Windows.",
          liveApps: "Applications in this session",
          noApps: "No activity captured yet.",
          site: "Site",
        }
      : {
          status: "Сессия",
          recording: "Идёт запись",
          paused: "Пауза",
          stopped: "Остановлено",
          project: "Проект",
          none: "—",
          activeWindow: "Активное окно сейчас",
          noWindow: "Отслеживание активного окна работает на Windows.",
          liveApps: "Приложения в этой сессии",
          noApps: "Активность ещё не зафиксирована.",
          site: "Сайт",
        };

  const status = state.tracker.status;
  const recording = status === "running";
  const statusLabel =
    status === "running" ? L.recording : status === "paused" ? L.paused : L.stopped;
  const activeName =
    state.projects.find((p) => p.id === state.tracker.active_project_id)?.name ??
    state.selected_project?.name ??
    L.none;
  const runSince = state.tracker.running_since
    ? parseTs(state.tracker.running_since)
    : null;
  const elapsed = recording && runSince ? now - runSince : 0;

  const apps = [...(state.selected_project?.apps ?? [])]
    .filter((a) => a.time_seconds > 0)
    .sort((a, b) => b.time_seconds - a.time_seconds)
    .slice(0, 15);
  const appMax = Math.max(1, ...apps.map((a) => a.time_seconds));

  return (
    <div className="flex h-full flex-col">
      <PageHeader title={t.title} subtitle={t.subtitle} />
      <div className="min-h-0 flex-1 overflow-y-auto px-8 pb-10">
        <div className="mx-auto flex max-w-4xl flex-col gap-5">
          {/* Статус сессии */}
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
            <section
              className={`flex items-center gap-4 rounded-[24px] border p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] ${
                recording ? "border-emerald-200 bg-emerald-50/70" : "border-slate-200 bg-white/80"
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
                <p className="text-sm font-semibold text-slate-900">{statusLabel}</p>
                <p className="truncate text-sm text-slate-500">{activeName}</p>
              </div>
              {recording && (
                <span className="text-2xl font-bold tabular-nums text-emerald-700">
                  {fmtClock(elapsed)}
                </span>
              )}
            </section>

            {/* Активное окно сейчас */}
            <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
              <p className="text-xs font-semibold uppercase tracking-wide text-slate-400">
                {L.activeWindow}
              </p>
              {win ? (
                <div className="mt-3 flex items-center gap-3">
                  <span className="grid size-10 shrink-0 place-items-center rounded-xl bg-emerald-50 text-emerald-600">
                    {win.kind === "browser" ? (
                      <Globe size={18} />
                    ) : (
                      <AppWindow size={18} />
                    )}
                  </span>
                  <div className="min-w-0">
                    <p className="truncate text-sm font-semibold text-slate-900">
                      {win.name}
                    </p>
                    <p className="truncate text-xs text-slate-500">
                      {win.url ?? win.title}
                    </p>
                  </div>
                </div>
              ) : (
                <p className="mt-3 text-sm text-slate-400">{L.noWindow}</p>
              )}
            </section>
          </div>

          {/* Живой список приложений */}
          <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <h2 className="mb-4 text-sm font-semibold text-slate-900">
              {L.liveApps}
            </h2>
            {apps.length ? (
              <div className="flex flex-col gap-2.5">
                {apps.map((a) => (
                  <div key={a.key} className="flex items-center gap-3">
                    <span className="grid size-6 shrink-0 place-items-center overflow-hidden rounded-md bg-slate-100">
                      {a.icon_data_url ? (
                        <img src={a.icon_data_url} alt="" className="size-5" />
                      ) : a.kind === "browser" ? (
                        <Globe size={13} className="text-slate-400" />
                      ) : (
                        <AppWindow size={13} className="text-slate-400" />
                      )}
                    </span>
                    <span className="w-40 shrink-0 truncate text-sm text-slate-600">
                      {a.name}
                    </span>
                    <div className="relative h-5 flex-1 overflow-hidden rounded-md bg-slate-100">
                      <div
                        className="absolute inset-y-0 left-0 rounded-md"
                        style={{
                          width: `${(a.time_seconds / appMax) * 100}%`,
                          background: "linear-gradient(90deg, #059669, #34d399)",
                        }}
                      />
                    </div>
                    <span className="w-16 shrink-0 text-right text-sm font-semibold tabular-nums text-slate-700">
                      {fmtDur(a.time_seconds)}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-slate-400">{L.noApps}</p>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}
