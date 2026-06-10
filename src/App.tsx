import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  Check,
  ChevronDown,
  ChevronRight,
  CirclePause,
  CirclePlay,
  Download,
  FileText,
  FolderPlus,
  Import,
  RefreshCw,
  Square,
  TimerReset,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type TrackerStatus = "stopped" | "paused" | "running";

type TrackerPayload = {
  status: TrackerStatus;
  active_project_id: string | null;
  running_since: string | null;
};

type ProjectSummary = {
  id: string;
  name: string;
  client: string;
  updated_at: string;
};

type TabUsageRecord = {
  key: string;
  title: string;
  url: string | null;
  enabled: boolean;
  time_seconds: number;
};

type AppUsageRecord = {
  key: string;
  name: string;
  process_name: string;
  process_path: string;
  icon_data_url?: string | null;
  kind: "app" | "browser";
  enabled: boolean;
  time_seconds: number;
  tabs: TabUsageRecord[];
};

type SessionRecord = {
  id: string;
  started_at: string;
  stopped_at: string | null;
  duration_seconds: number;
  app_count: number;
  browser_count: number;
};

type ProjectRecord = ProjectSummary & {
  note: string;
  created_at: string;
  sessions: SessionRecord[];
  apps: AppUsageRecord[];
};

type AppState = {
  tracker: TrackerPayload;
  projects: ProjectSummary[];
  selected_project: ProjectRecord | null;
};

type ExportResult = {
  message: string;
  path: string;
};

const fallbackState: AppState = {
  tracker: {
    status: "stopped",
    active_project_id: null,
    running_since: null,
  },
  projects: [],
  selected_project: null,
};

async function invokeCommand<T>(name: string, args: Record<string, unknown> = {}, fallback: T): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    console.error(name, error);
    return fallback;
  }
}

function formatDuration(seconds: number): string {
  const value = Number(seconds || 0);
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const secs = value % 60;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

function formatDateTime(value?: string | null): string {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return String(value);
  return new Intl.DateTimeFormat("ru-RU", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function percentOf(total: number, value: number): string {
  if (!total || !value) return "0%";
  return `${((value / total) * 100).toFixed(1)}%`;
}

function iconForName(name: string): string {
  const key = name.toLowerCase();
  if (key.includes("chrome")) return "C";
  if (key.includes("edge")) return "E";
  if (key.includes("firefox")) return "F";
  if (key.includes("after")) return "Ae";
  if (key.includes("premiere")) return "Pr";
  if (key.includes("figma")) return "Fg";
  if (key.includes("code") || key.includes("studio")) return "VS";
  return name.slice(0, 1).toUpperCase() || "?";
}

function appIncludedSeconds(app: AppUsageRecord): number {
  if (app.kind === "browser") {
    if (!app.enabled) return 0;
    return app.tabs.reduce((sum, tab) => sum + (tab.enabled ? tab.time_seconds : 0), 0);
  }
  return app.enabled ? app.time_seconds : 0;
}

function tabIncludedSeconds(app: AppUsageRecord, tab: TabUsageRecord): number {
  return app.enabled && tab.enabled ? tab.time_seconds : 0;
}

export default function App() {
  const [state, setState] = useState<AppState>(fallbackState);
  const [expandedApps, setExpandedApps] = useState<Record<string, boolean>>({});
  const [newProjectName, setNewProjectName] = useState("");
  const [newProjectClient, setNewProjectClient] = useState("");
  const [message, setMessage] = useState("");
  const importInputRef = useRef<HTMLInputElement>(null);
  const toastTimerRef = useRef<number | null>(null);

  async function refresh() {
    const next = await invokeCommand<AppState>("get_app_state", {}, fallbackState);
    setState(next);
  }

  useEffect(() => {
    refresh();
    const timer = window.setInterval(refresh, 1500);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (!message) return;
    if (toastTimerRef.current) {
      window.clearTimeout(toastTimerRef.current);
    }
    toastTimerRef.current = window.setTimeout(() => setMessage(""), 2500);
    return () => {
      if (toastTimerRef.current) {
        window.clearTimeout(toastTimerRef.current);
      }
    };
  }, [message]);

  const selectedProject = state.selected_project;
  const apps = selectedProject?.apps ?? [];
  const sessions = selectedProject?.sessions ?? [];
  const trackerStatus = state.tracker.status;
  const isRunning = trackerStatus === "running";
  const isPaused = trackerStatus === "paused";

  const totals = useMemo(() => {
    const appTime = apps.reduce((sum, app) => sum + appIncludedSeconds(app), 0);
    const topApp = [...apps].sort((a, b) => appIncludedSeconds(b) - appIncludedSeconds(a))[0];
    const topTab = apps
      .flatMap((app) =>
        app.tabs
          .filter((tab) => app.enabled && tab.enabled)
          .map((tab) => ({ ...tab, browser: app.name })),
      )
      .sort((a, b) => b.time_seconds - a.time_seconds)[0];

    return { appTime, topApp, topTab };
  }, [apps]);

  async function createProject() {
    if (!newProjectName.trim()) return;
    await invokeCommand<ProjectSummary | null>(
      "create_project",
      {
        name: newProjectName.trim(),
        client: newProjectClient.trim(),
      },
      null,
    );
    setNewProjectName("");
    setNewProjectClient("");
    setMessage("Проект создан.");
    refresh();
  }

  async function selectProject(projectId: string) {
    await invokeCommand<void>("select_project", { projectId }, undefined);
    refresh();
  }

  async function toggleTracking(command: "start_tracking" | "pause_tracking" | "stop_tracking") {
    await invokeCommand<void>(command, {}, undefined);
    refresh();
  }

  async function toggleApp(projectId: string, appKey: string, enabled: boolean) {
    const updated = await invokeCommand<ProjectRecord | null>("toggle_app_included", { projectId, appKey, enabled }, null);
    if (updated) {
      setState((current) => ({
        ...current,
        selected_project: updated,
      }));
    }
    refresh();
  }

  async function toggleTab(projectId: string, appKey: string, tabKey: string, enabled: boolean) {
    const updated = await invokeCommand<ProjectRecord | null>("toggle_tab_included", { projectId, appKey, tabKey, enabled }, null);
    if (updated) {
      setState((current) => ({
        ...current,
        selected_project: updated,
      }));
    }
    refresh();
  }

  async function exportXlsx() {
    const result = await invokeCommand<ExportResult | null>("export_selected_project_xlsx", {}, null);
    setMessage(result?.message ?? "Экспорт выполнен.");
    refresh();
  }

  async function handleImportFile(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;

    const jsonText = await file.text();
    const result = await invokeCommand<ProjectSummary | null>("import_project_json", { jsonText }, null);
    setMessage(result?.name ? `Импортирован проект: ${result.name}` : "Импорт выполнен.");
    event.target.value = "";
    refresh();
  }

  return (
    <div
      className="min-h-screen bg-[linear-gradient(180deg,#f8fbf8_0%,#eef6ef_100%)] text-slate-900"
      onContextMenu={(event) => event.preventDefault()}
    >
      <div className="mx-auto flex min-h-screen w-full max-w-[1680px] flex-col gap-5 px-5 py-5 lg:px-6">
        <main className="grid min-h-0 flex-1 grid-cols-1 gap-5 xl:grid-cols-[340px_minmax(0,1fr)]">
          <aside className="flex min-h-0 flex-col gap-5">
            <section className="rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="flex items-center justify-between gap-3">
                <h2 className="text-base font-semibold text-slate-900">Проекты</h2>
                <button className="icon-button" onClick={refresh} title="Обновить">
                  <RefreshCw size={16} />
                </button>
              </div>

              <div className="mt-4 grid gap-2">
                <input
                  className="field"
                  value={newProjectName}
                  onChange={(event) => setNewProjectName(event.target.value)}
                  placeholder="Название проекта"
                />
                <input
                  className="field"
                  value={newProjectClient}
                  onChange={(event) => setNewProjectClient(event.target.value)}
                  placeholder="Клиент или метка"
                />
                <button className="primary-button" onClick={createProject}>
                  <FolderPlus size={16} />
                  Создать
                </button>
              </div>

              <div className="mt-4 max-h-[calc(100vh-390px)] overflow-auto pr-1">
                <div className="grid gap-2">
                  {state.projects.length ? (
                    state.projects.map((project) => (
                      <button
                        key={project.id}
                        className={`flex w-full items-center justify-between gap-3 rounded-2xl border px-4 py-3 text-left transition ${
                          project.id === selectedProject?.id
                            ? "border-emerald-300 bg-emerald-50"
                            : "border-slate-200 bg-white hover:border-emerald-200"
                        }`}
                        onClick={() => selectProject(project.id)}
                      >
                        <span className="min-w-0">
                          <strong className="block truncate text-sm text-slate-900">{project.name}</strong>
                          <span className="block truncate text-xs text-slate-500">{project.client || "Без клиента"}</span>
                        </span>
                        <small className="shrink-0 text-xs text-slate-400">{formatDateTime(project.updated_at)}</small>
                      </button>
                    ))
                  ) : (
                    <EmptyState text="Пока нет проектов." />
                  )}
                </div>
              </div>
            </section>

            <section className="rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <SectionTitle icon={<TimerReset size={16} />} title="Сеансы" />
              <div className="mt-3 grid gap-2">
                {sessions.length ? (
                  sessions.map((session) => (
                    <article key={session.id} className="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
                      <strong className="block text-sm text-slate-900">
                        {formatDateTime(session.started_at)} - {formatDateTime(session.stopped_at)}
                      </strong>
                      <span className="mt-1 block text-sm text-slate-500">{formatDuration(session.duration_seconds)}</span>
                    </article>
                  ))
                ) : (
                  <EmptyState text="Сеансы появятся после старта записи." />
                )}
              </div>
            </section>
          </aside>

          <section className="grid min-h-0 content-start gap-5">
            <section className="rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
                <div className="space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <h1 className="text-xl font-semibold text-slate-900">{selectedProject?.name ?? "Project Time Manager"}</h1>
                    <span className="inline-flex items-center gap-2 rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs font-semibold uppercase text-emerald-700">
                      <Activity size={13} />
                      {trackerStatus}
                    </span>
                  </div>
                  <p className="text-sm text-slate-500">{selectedProject?.client || "Выбери проект и запускай запись времени."}</p>
                </div>

                <div className="grid grid-cols-3 gap-2 sm:flex sm:flex-wrap">
                  <button className="primary-button" onClick={() => toggleTracking("start_tracking")} disabled={isRunning || !selectedProject}>
                    <CirclePlay size={16} />
                    {isPaused ? "Resume" : "Start"}
                  </button>
                  <button className="secondary-button" onClick={() => toggleTracking("pause_tracking")} disabled={!isRunning || !selectedProject}>
                    <CirclePause size={16} />
                    Pause
                  </button>
                  <button className="secondary-button" onClick={() => toggleTracking("stop_tracking")} disabled={(!isRunning && !isPaused) || !selectedProject}>
                    <Square size={16} />
                    Stop
                  </button>
                </div>

                <div className="flex flex-wrap gap-2">
                  <button className="secondary-button" onClick={() => importInputRef.current?.click()}>
                    <Import size={16} />
                    JSON
                  </button>
                  <button className="secondary-button" onClick={exportXlsx}>
                    <Download size={16} />
                    Excel
                  </button>
                  <button className="secondary-button opacity-50" disabled title="PDF export пока не включен">
                    <FileText size={16} />
                    PDF
                  </button>
                </div>
              </div>

              <input ref={importInputRef} type="file" accept=".json" hidden onChange={handleImportFile} />
            </section>

            {selectedProject ? (
              <>
                <section className="grid gap-3 md:grid-cols-2 2xl:grid-cols-4">
                  <Metric label="Всего по проекту" value={formatDuration(totals.appTime)} accent="emerald" />
                  <Metric label="Топ приложение" value={totals.topApp?.name ?? "-"} accent="slate" />
                  <Metric label="Топ вкладка" value={totals.topTab?.title ?? "-"} accent="slate" />
                  <Metric label="Сеансов" value={String(sessions.length)} accent="emerald" />
                </section>

                <section className="rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
                  <SectionTitle icon={<Activity size={16} />} title="Приложения и вкладки" />

                  <div className="mt-4 grid gap-2">
                    <div className="grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] gap-2 px-3 text-xs font-medium uppercase tracking-[0.14em] text-slate-400 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4">
                      <span />
                      <span>Приложение</span>
                      <span>Время</span>
                      <span>%</span>
                    </div>

                    <div className="max-h-[46vh] overflow-auto pr-1">
                      <div className="grid gap-2">
                        {apps.map((app) => {
                          const appTotal = appIncludedSeconds(app);
                          const isOpen = expandedApps[app.key];

                          return (
                            <div key={app.key} className="grid gap-2">
                              <div className="grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-center gap-2 rounded-2xl border border-slate-200 bg-slate-50 px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4">
                                <Checkbox checked={app.enabled} onChange={(checked) => toggleApp(selectedProject.id, app.key, checked)} />

                                <button
                                  className="flex min-w-0 items-center gap-3 text-left"
                                  onClick={() => setExpandedApps((prev) => ({ ...prev, [app.key]: !prev[app.key] }))}
                                >
                                  <AppIcon app={app} />
                                  <span className="min-w-0">
                                    <strong className="block truncate text-sm text-slate-900">{app.name}</strong>
                                    <small className="block truncate text-xs text-slate-500">
                                      {app.kind === "browser" ? "Browser" : app.process_path || app.process_name}
                                    </small>
                                  </span>
                                  {app.kind === "browser" ? (
                                    <span className="ml-auto text-slate-400">{isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}</span>
                                  ) : null}
                                </button>

                                <span className="font-mono text-sm text-slate-700">{formatDuration(appTotal)}</span>
                                <span className="text-sm text-slate-500">{percentOf(totals.appTime, appTotal)}</span>
                              </div>

                              {app.kind === "browser" && isOpen ? (
                                <div className="ml-4 grid gap-2 sm:ml-10">
                                  {app.tabs.map((tab) => (
                                    <div
                                      key={tab.key}
                                      className="grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4"
                                    >
                                      <Checkbox
                                        checked={tab.enabled}
                                        onChange={(checked) => toggleTab(selectedProject.id, app.key, tab.key, checked)}
                                      />
                                      <span className="min-w-0">
                                        <strong className="block truncate text-sm text-slate-900">{tab.title}</strong>
                                        <small className="block truncate text-xs text-slate-500">{tab.url || "URL недоступен"}</small>
                                      </span>
                                      <span className="font-mono text-sm text-slate-700">{formatDuration(tab.time_seconds)}</span>
                                      <span className="text-sm text-slate-500">{percentOf(totals.appTime, tabIncludedSeconds(app, tab))}</span>
                                    </div>
                                  ))}
                                </div>
                              ) : null}
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  </div>
                </section>
              </>
            ) : (
              <section className="rounded-[28px] border border-dashed border-emerald-200 bg-white p-8 text-sm text-slate-500 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
                Выбери или создай проект, чтобы начать запись.
              </section>
            )}
          </section>
        </main>
      </div>

      {message ? <Toast text={message} /> : null}
    </div>
  );
}

function Metric({ label, value, accent }: { label: string; value: string; accent: "emerald" | "slate" }) {
  return (
    <article className="rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
      <span className="block text-xs font-semibold uppercase tracking-[0.18em] text-slate-400">{label}</span>
      <div
        className={`mt-3 h-1.5 w-12 rounded-full ${accent === "emerald" ? "bg-emerald-500" : "bg-slate-300"}`}
      />
      <strong className="mt-4 block truncate text-lg font-semibold text-slate-900">{value}</strong>
    </article>
  );
}

function AppIcon({ app }: { app: AppUsageRecord }) {
  if (app.icon_data_url) {
    return (
      <span className="flex size-10 shrink-0 items-center justify-center rounded-2xl border border-white bg-white shadow-sm">
        <img className="size-7 object-contain" src={app.icon_data_url} alt="" draggable={false} />
      </span>
    );
  }

  return (
    <span className="flex size-10 shrink-0 items-center justify-center rounded-2xl bg-emerald-100 text-sm font-semibold text-emerald-700">
      {iconForName(app.name)}
    </span>
  );
}

function SectionTitle({ icon, title }: { icon: React.ReactNode; title: string }) {
  return (
    <div className="flex items-center gap-2 text-slate-700">
      {icon}
      <h3 className="text-base font-semibold text-slate-900">{title}</h3>
    </div>
  );
}

function Checkbox({ checked, onChange }: { checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="relative flex size-8 items-center justify-center">
      <input
        className="peer absolute inset-0 cursor-pointer opacity-0"
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="flex size-5 items-center justify-center rounded-md border border-slate-300 bg-white text-transparent peer-checked:border-emerald-500 peer-checked:bg-emerald-500 peer-checked:text-white">
        <Check size={14} />
      </span>
    </label>
  );
}

function EmptyState({ text }: { text: string }) {
  return <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 px-4 py-4 text-sm text-slate-500">{text}</div>;
}

function Toast({ text }: { text: string }) {
  return (
    <footer className="fixed bottom-5 right-5 max-w-[520px] rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900 shadow-[0_10px_30px_rgba(15,23,42,0.1)]">
      {text}
    </footer>
  );
}
