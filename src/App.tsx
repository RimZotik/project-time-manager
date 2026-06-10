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
  PanelsTopLeft,
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
  if (key.includes("premiere")) return "P";
  if (key.includes("after")) return "A";
  if (key.includes("code") || key.includes("studio")) return "D";
  return name.slice(0, 1).toUpperCase() || "?";
}

function appIncludedSeconds(app: AppUsageRecord): number {
  if (app.kind === "browser") {
    return app.tabs.reduce((sum, tab) => sum + (tab.enabled ? tab.time_seconds : 0), 0);
  }
  return app.enabled ? app.time_seconds : 0;
}

export default function App() {
  const [state, setState] = useState<AppState>(fallbackState);
  const [expandedApps, setExpandedApps] = useState<Record<string, boolean>>({});
  const [newProjectName, setNewProjectName] = useState("");
  const [newProjectClient, setNewProjectClient] = useState("");
  const [message, setMessage] = useState("");
  const importInputRef = useRef<HTMLInputElement>(null);

  async function refresh() {
    const next = await invokeCommand<AppState>("get_app_state", {}, fallbackState);
    setState(next);
  }

  useEffect(() => {
    refresh();
    const timer = window.setInterval(refresh, 3000);
    return () => window.clearInterval(timer);
  }, []);

  const selectedProject = state.selected_project;
  const apps = selectedProject?.apps ?? [];
  const sessions = selectedProject?.sessions ?? [];

  const totals = useMemo(() => {
    const appTime = apps.reduce((sum, app) => sum + appIncludedSeconds(app), 0);
    const topApp = [...apps].sort((a, b) => appIncludedSeconds(b) - appIncludedSeconds(a))[0];
    const topTab = apps
      .flatMap((app) => app.tabs.map((tab) => ({ ...tab, browser: app.name })))
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
    await invokeCommand<ProjectRecord | null>("toggle_app_included", { projectId, appKey, enabled }, null);
    refresh();
  }

  async function toggleTab(projectId: string, appKey: string, tabKey: string, enabled: boolean) {
    await invokeCommand<ProjectRecord | null>("toggle_tab_included", { projectId, appKey, tabKey, enabled }, null);
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
    <div className="min-h-screen bg-slate-950 text-slate-100">
      <div className="mx-auto flex min-h-screen w-full max-w-[1600px] flex-col gap-5 px-6 py-6">
        <header className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h1 className="text-3xl font-semibold tracking-normal text-white">Project Time Manager</h1>
            <p className="mt-1 text-sm text-slate-400">Трекер проектов, активных окон и браузерных вкладок для Windows.</p>
          </div>
          <div className="inline-flex items-center gap-2 rounded-full border border-slate-700 bg-slate-900 px-4 py-2 text-sm capitalize text-slate-200">
            <Activity size={14} />
            {state.tracker.status}
          </div>
        </header>

        <main className="grid flex-1 grid-cols-1 gap-5 xl:grid-cols-[340px_minmax(0,1fr)]">
          <section className="border border-slate-800 bg-slate-900/80 p-4">
            <div className="flex items-center justify-between gap-3">
              <h2 className="text-base font-semibold text-white">Проекты</h2>
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

            <div className="mt-4 grid gap-2">
              {state.projects.length ? (
                state.projects.map((project) => (
                  <button
                    key={project.id}
                    className={`flex w-full items-center justify-between gap-3 border p-3 text-left transition ${
                      project.id === selectedProject?.id
                        ? "border-blue-500 bg-blue-950/40"
                        : "border-slate-800 bg-slate-950/70 hover:border-slate-700"
                    }`}
                    onClick={() => selectProject(project.id)}
                  >
                    <span className="min-w-0">
                      <strong className="block truncate text-sm text-white">{project.name}</strong>
                      <span className="block truncate text-xs text-slate-400">{project.client || "Без клиента"}</span>
                    </span>
                    <small className="shrink-0 text-xs text-slate-500">{formatDateTime(project.updated_at)}</small>
                  </button>
                ))
              ) : (
                <EmptyState text="Пока нет проектов." />
              )}
            </div>
          </section>

          <section className="grid content-start gap-5 border border-slate-800 bg-slate-900/80 p-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h2 className="text-base font-semibold text-white">Панель проекта</h2>
              <div className="flex items-center gap-2">
                <button className="primary-button" onClick={() => toggleTracking("start_tracking")}>
                  <CirclePlay size={16} />
                  Start
                </button>
                <button className="icon-button" onClick={() => toggleTracking("pause_tracking")} title="Pause">
                  <CirclePause size={16} />
                </button>
                <button className="icon-button" onClick={() => toggleTracking("stop_tracking")} title="Stop">
                  <Square size={16} />
                </button>
              </div>
            </div>

            {selectedProject ? (
              <>
                <div className="grid gap-3 md:grid-cols-2 2xl:grid-cols-4">
                  <Metric label="Всего по проекту" value={formatDuration(totals.appTime)} />
                  <Metric label="Топ приложение" value={totals.topApp?.name ?? "-"} />
                  <Metric label="Топ вкладка" value={totals.topTab?.title ?? "-"} />
                  <Metric label="Сеансов" value={String(sessions.length)} />
                </div>

                <div className="flex flex-wrap justify-end gap-2">
                  <button className="secondary-button" onClick={() => importInputRef.current?.click()}>
                    <Import size={16} />
                    Импорт JSON
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
                <input ref={importInputRef} type="file" accept=".json" hidden onChange={handleImportFile} />

                <SectionTitle icon={<PanelsTopLeft size={16} />} title="Приложения и вкладки" />
                <div className="grid gap-2">
                  <div className="grid grid-cols-[44px_minmax(220px,1fr)_112px_72px] gap-3 px-3 text-xs text-slate-500">
                    <span />
                    <span>Приложение</span>
                    <span>Время</span>
                    <span>%</span>
                  </div>

                  {apps.map((app) => {
                    const appTotal = appIncludedSeconds(app);
                    const isOpen = expandedApps[app.key];

                    return (
                      <div key={app.key} className="grid gap-1">
                        <div className="grid grid-cols-[44px_minmax(220px,1fr)_112px_72px] items-center gap-3 border border-slate-800 bg-slate-950/70 px-3 py-3">
                          <Checkbox checked={app.enabled} onChange={(checked) => toggleApp(selectedProject.id, app.key, checked)} />

                          <button
                            className="flex min-w-0 items-center gap-3 text-left"
                            onClick={() => setExpandedApps((prev) => ({ ...prev, [app.key]: !prev[app.key] }))}
                          >
                            <span className="flex size-8 shrink-0 items-center justify-center rounded bg-blue-500/15 text-sm font-bold text-blue-100">
                              {iconForName(app.name)}
                            </span>
                            <span className="min-w-0">
                              <strong className="block truncate text-sm text-white">{app.name}</strong>
                              <small className="block truncate text-xs text-slate-500">{app.kind === "browser" ? "Browser" : app.process_name}</small>
                            </span>
                            {app.kind === "browser" ? (
                              <span className="ml-auto text-slate-400">{isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}</span>
                            ) : null}
                          </button>

                          <span className="font-mono text-sm text-slate-200">{formatDuration(appTotal)}</span>
                          <span className="text-sm text-slate-300">{percentOf(totals.appTime, appTotal)}</span>
                        </div>

                        {app.kind === "browser" && isOpen ? (
                          <div className="ml-10 grid gap-1">
                            {app.tabs.map((tab) => (
                              <div
                                key={tab.key}
                                className="grid grid-cols-[44px_minmax(220px,1fr)_112px_72px] items-center gap-3 border border-slate-800 bg-slate-950/40 px-3 py-2"
                              >
                                <Checkbox checked={tab.enabled} onChange={(checked) => toggleTab(selectedProject.id, app.key, tab.key, checked)} />
                                <span className="min-w-0">
                                  <strong className="block truncate text-sm text-slate-100">{tab.title}</strong>
                                  <small className="block truncate text-xs text-slate-500">{tab.url || "URL недоступен"}</small>
                                </span>
                                <span className="font-mono text-sm text-slate-300">{formatDuration(tab.time_seconds)}</span>
                                <span className="text-sm text-slate-400">{percentOf(totals.appTime, tab.time_seconds)}</span>
                              </div>
                            ))}
                          </div>
                        ) : null}
                      </div>
                    );
                  })}
                </div>

                <SectionTitle icon={<TimerReset size={16} />} title="Сеансы" />
                <div className="grid gap-2">
                  {sessions.length ? (
                    sessions.map((session) => (
                      <article
                        className="flex flex-wrap items-center justify-between gap-3 border border-slate-800 bg-slate-950/70 px-3 py-3"
                        key={session.id}
                      >
                        <strong className="text-sm text-slate-100">
                          {formatDateTime(session.started_at)} - {formatDateTime(session.stopped_at)}
                        </strong>
                        <span className="font-mono text-sm text-slate-300">{formatDuration(session.duration_seconds)}</span>
                      </article>
                    ))
                  ) : (
                    <EmptyState text="Сеансы появятся после старта записи." />
                  )}
                </div>
              </>
            ) : (
              <EmptyState text="Выбери или создай проект, чтобы начать запись." />
            )}
          </section>
        </main>
      </div>

      {message ? (
        <footer className="fixed bottom-5 right-5 max-w-[520px] border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-slate-100 shadow-2xl shadow-black/40">
          {message}
        </footer>
      ) : null}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <article className="border border-slate-800 bg-slate-950/70 p-4">
      <span className="block text-xs text-slate-500">{label}</span>
      <strong className="mt-2 block truncate text-lg font-semibold text-white">{value}</strong>
    </article>
  );
}

function SectionTitle({ icon, title }: { icon: React.ReactNode; title: string }) {
  return (
    <div className="flex items-center gap-2 text-slate-200">
      {icon}
      <h3 className="text-base font-semibold text-white">{title}</h3>
    </div>
  );
}

function Checkbox({ checked, onChange }: { checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <label className="relative flex size-8 items-center justify-center">
      <input className="peer absolute inset-0 cursor-pointer opacity-0" type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
      <span className="flex size-5 items-center justify-center border border-slate-600 bg-slate-900 text-transparent peer-checked:border-blue-500 peer-checked:bg-blue-500 peer-checked:text-white">
        <Check size={14} />
      </span>
    </label>
  );
}

function EmptyState({ text }: { text: string }) {
  return <div className="border border-dashed border-slate-700 bg-slate-950/40 p-4 text-sm text-slate-400">{text}</div>;
}
