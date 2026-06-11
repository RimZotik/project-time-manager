import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  Activity,
  ArrowDown,
  ArrowUp,
  Check,
  ChevronDown,
  ChevronRight,
  CircleAlert,
  CirclePause,
  CirclePlay,
  GripVertical,
  Download,
  FileText,
  FolderOpen,
  FolderPlus,
  Import,
  Link,
  Languages,
  PencilLine,
  RefreshCw,
  Settings,
  Square,
  TimerReset,
  Trash2,
  X,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type TrackerStatus = "stopped" | "paused" | "running";

type TrackerPayload = {
  status: TrackerStatus;
  active_project_id: string | null;
  running_since: string | null;
};

type AppSettings = {
  autostart: boolean;
  language: "ru" | "en";
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
  urls?: VisitedUrlRecord[];
  favicon_url?: string | null;
  enabled: boolean;
  time_seconds: number;
};

type VisitedUrlRecord = {
  url: string;
  title: string;
  last_seen_at: string;
  hits: number;
  enabled: boolean;
  time_seconds: number;
};

type StageUrlRecord = {
  url: string;
  enabled: boolean;
};

type StageTabRecord = {
  tab_key: string;
  enabled: boolean;
  urls: StageUrlRecord[];
};

type StageAppRecord = {
  app_key: string;
  enabled: boolean;
  tabs: StageTabRecord[];
};

type ProjectStageRecord = {
  id: string;
  name: string;
  order: number;
  created_at: string;
  updated_at: string;
  apps: StageAppRecord[];
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
  stages: ProjectStageRecord[];
};

type AppState = {
  tracker: TrackerPayload;
  settings: AppSettings;
  projects: ProjectSummary[];
  selected_project: ProjectRecord | null;
};

type ExportResult = {
  message: string;
  path: string;
};

type ToastState = {
  text: string;
  exportPath?: string;
};

type ProjectMenuState = {
  projectId: string;
  projectName: string;
  x: number;
  y: number;
};

type StageModalState = {
  projectId: string;
};

type RenameState = {
  projectId: string;
  value: string;
};

type RankedApp = {
  app: AppUsageRecord;
  includedSeconds: number;
  actualSeconds: number;
  includedPercent: string;
  actualPercent: string;
};

type RankedTab = {
  tab: TabUsageRecord;
  includedSeconds: number;
  actualSeconds: number;
  includedPercent: string;
  actualPercent: string;
};

const fallbackState: AppState = {
  tracker: {
    status: "stopped",
    active_project_id: null,
    running_since: null,
  },
  settings: {
    autostart: false,
    language: "ru",
  },
  projects: [],
  selected_project: null,
};

const copy = {
  ru: {
    appName: "Project Time Manager",
    projectLabel: "Проекты",
    sessionsLabel: "Сеансы",
    settingsLabel: "Настройки",
    helpLabel: "Помощь",
    stagesLabel: "Этапы",
    createLabel: "Создать",
    projectNamePlaceholder: "Название проекта",
    statusRunning: "Запись",
    statusPaused: "Пауза",
    statusStopped: "Остановлено",
    startLabel: "Старт",
    continueLabel: "Продолжить",
    pauseLabel: "Пауза",
    stopLabel: "Стоп",
    importJsonLabel: "Импорт JSON",
    exportExcelLabel: "Экспорт Excel",
    exportPdfLabel: "PDF",
    totalProjectLabel: "Всего по проекту",
    sessionsCountLabel: "Сеансов",
    topAppLabel: "Топ приложение",
    topTabLabel: "Топ вкладка",
    appsTitle: "Приложения и вкладки",
    applicationHeader: "Приложение",
    timeHeader: "Время",
    percentHeader: "%",
    browserLabel: "Браузер",
    noProjects: "Пока нет проектов.",
    noSessions: "Сеансы появятся после старта записи.",
    emptyPrompt: "Выбери или создай проект, чтобы начать запись.",
    currentProjectHint: "Выбери проект, запусти запись и отслеживай активные окна.",
    projectCreated: "Проект создан.",
    exportDone: "Экспорт выполнен.",
    importDone: "Импорт выполнен.",
    importedProject: "Импортирован проект",
    settingsTitle: "Настройки",
    settingsDescription: "Параметры приложения сохраняются автоматически.",
    autostartLabel: "Автозапуск",
    languageLabel: "Язык",
    openFolderLabel: "Открыть папку приложения",
    closeLabel: "Закрыть",
    helpTitle: "Краткая инструкция",
    helpDescription: "Основные действия для работы с проектами, записью времени и отчетами.",
    helpProjectsTitle: "Проекты",
    helpProjectsText:
      "Создай проект в левом столбце или выбери существующий. Все записи и отчеты хранятся в папке проекта.",
    helpTrackingTitle: "Запись времени",
    helpTrackingText:
      "Нажми Старт, работай как обычно и затем поставь запись на паузу или останови ее. Во время записи импорт и экспорт отключены.",
    helpAppsTitle: "Приложения",
    helpAppsText:
      "В таблице видно, какие окна были активны. Галочка управляет тем, учитывается ли приложение в итоговом времени и отчетах.",
    helpSitesTitle: "Сайты",
    helpSitesText:
      "Браузеры раскрываются как группы доменов. Нажми значок цепочки, чтобы увидеть все посещенные URL и открыть нужный в браузере.",
    helpReportsTitle: "Отчеты",
    helpReportsText:
      "Excel и PDF строятся только по включенным приложениям и сайтам. Если файл уже открыт, новый отчет сохранится с временной меткой.",
    projectMenuRename: "Переименовать",
    projectMenuDelete: "Удалить",
    projectMenuStages: "Этапы",
    stageTitle: "Этапы проекта",
    stageDescription: "У каждого этапа свои включения приложений, вкладок и ссылок.",
    stageCreateLabel: "Создать этап",
    stageRenameLabel: "Переименовать",
    stageDeleteLabel: "Удалить",
    stageUpLabel: "Выше",
    stageDownLabel: "Ниже",
    stageEmpty: "Этапов пока нет.",
    stageNamePlaceholder: "Название этапа",
    stageCreateHint: "Создавай этапы и затем уточняй, что в них учитывать.",
    stageMetricsTime: "Время этапа",
    stageMetricsShare: "От проекта",
    stageMetricsApps: "Приложений",
    stageMetricsTabs: "Вкладок",
    stageDeleteConfirm: "Удалить этап полностью? Это действие нельзя отменить.",
    renameTitle: "Переименование проекта",
    renameDescription: "Введите новое название проекта.",
    renamePlaceholder: "Новое название",
    saveLabel: "Сохранить",
    cancelLabel: "Отмена",
    deleteConfirm: "Удалить проект полностью? Это действие нельзя отменить.",
    openReportLabel: "Открыть файл",
    openUrlLabel: "Открыть ссылку",
    urlUnavailable: "URL недоступен",
    linksInDomain: (count: number) => `${count} ссылок в этом домене`,
    dataFolderLabel: "Открыть папку приложения",
    languageRu: "Русский",
    languageEn: "English",
  },
  en: {
    appName: "Project Time Manager",
    projectLabel: "Projects",
    sessionsLabel: "Sessions",
    settingsLabel: "Settings",
    helpLabel: "Help",
    stagesLabel: "Stages",
    createLabel: "Create",
    projectNamePlaceholder: "Project name",
    statusRunning: "Recording",
    statusPaused: "Paused",
    statusStopped: "Stopped",
    startLabel: "Start",
    continueLabel: "Continue",
    pauseLabel: "Pause",
    stopLabel: "Stop",
    importJsonLabel: "Import JSON",
    exportExcelLabel: "Export Excel",
    exportPdfLabel: "PDF",
    totalProjectLabel: "Total project",
    sessionsCountLabel: "Sessions",
    topAppLabel: "Top app",
    topTabLabel: "Top tab",
    appsTitle: "Applications and tabs",
    applicationHeader: "Application",
    timeHeader: "Time",
    percentHeader: "%",
    browserLabel: "Browser",
    noProjects: "No projects yet.",
    noSessions: "Sessions will appear after recording starts.",
    emptyPrompt: "Choose or create a project to start recording.",
    currentProjectHint: "Choose a project, start recording, and track active windows.",
    projectCreated: "Project created.",
    exportDone: "Export completed.",
    importDone: "Import completed.",
    importedProject: "Imported project",
    settingsTitle: "Settings",
    settingsDescription: "App preferences are saved automatically.",
    autostartLabel: "Autostart",
    languageLabel: "Language",
    openFolderLabel: "Open app folder",
    closeLabel: "Close",
    helpTitle: "Quick guide",
    helpDescription: "Main actions for projects, time tracking, and reports.",
    helpProjectsTitle: "Projects",
    helpProjectsText:
      "Create a project in the left column or select an existing one. All records and reports live inside the project folder.",
    helpTrackingTitle: "Tracking",
    helpTrackingText:
      "Press Start, work as usual, then pause or stop the session. Import and export are disabled during active tracking.",
    helpAppsTitle: "Applications",
    helpAppsText:
      "The table shows which windows were active. The checkbox controls whether the app counts toward totals and reports.",
    helpSitesTitle: "Sites",
    helpSitesText:
      "Browsers expand into domain groups. Click the chain icon to see every URL captured for that domain and open any one in your browser.",
    helpReportsTitle: "Reports",
    helpReportsText:
      "Excel and PDF are built only from enabled apps and sites. If a file is already open, the new report is saved with a timestamp.",
    projectMenuRename: "Rename",
    projectMenuDelete: "Delete",
    projectMenuStages: "Stages",
    stageTitle: "Project stages",
    stageDescription: "Each stage can include its own apps, tabs, and links.",
    stageCreateLabel: "Create stage",
    stageRenameLabel: "Rename",
    stageDeleteLabel: "Delete",
    stageUpLabel: "Up",
    stageDownLabel: "Down",
    stageEmpty: "No stages yet.",
    stageNamePlaceholder: "Stage name",
    stageCreateHint: "Create stages and tune what each one counts.",
    stageMetricsTime: "Stage time",
    stageMetricsShare: "Of project",
    stageMetricsApps: "Apps",
    stageMetricsTabs: "Tabs",
    stageDeleteConfirm: "Delete this stage completely? This cannot be undone.",
    renameTitle: "Rename project",
    renameDescription: "Enter the new project name.",
    renamePlaceholder: "New name",
    saveLabel: "Save",
    cancelLabel: "Cancel",
    deleteConfirm: "Delete this project completely? This cannot be undone.",
    openReportLabel: "Open file",
    openUrlLabel: "Open link",
    urlUnavailable: "URL unavailable",
    linksInDomain: (count: number) => `${count} links in this domain`,
    dataFolderLabel: "Open app folder",
    languageRu: "Russian",
    languageEn: "English",
  },
} as const;

type Copy = {
  [K in keyof (typeof copy)["ru"]]: (typeof copy)["ru"][K] extends (...args: infer Args) => infer Return
    ? (...args: Args) => Return
    : string;
};

async function invokeCommand<T>(name: string, args: Record<string, unknown> = {}, fallback: T): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    console.error(name, error);
    return fallback;
  }
}

function formatDuration(seconds: number, language: AppSettings["language"] = "ru"): string {
  const value = Number(seconds || 0);
  const days = Math.floor(value / 86400);
  const hours = Math.floor((value % 86400) / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const secs = value % 60;
  const clock = `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
  return days > 0 ? `${days} ${language === "en" ? "d" : "д"} ${clock}` : clock;
}

function formatDurationInput(seconds: number, language: AppSettings["language"] = "ru"): string {
  const value = Math.max(0, Math.floor(Number(seconds || 0)));
  const days = Math.floor(value / 86400);
  const hours = Math.floor((value % 86400) / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const secs = value % 60;
  const prefix = days > 0 ? `${days}${language === "en" ? "d" : "д"} ` : "";
  return `${prefix}${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

function parseDurationInput(value: string): number | null {
  const trimmed = value.trim().toLowerCase().replace(/,/g, ".");
  if (!trimmed) return null;

  const dayMatch = trimmed.match(/^(\d+)\s*[dд]\s*(.+)$/);
  const body = dayMatch ? dayMatch[2].trim() : trimmed;
  const days = dayMatch ? Number(dayMatch[1]) : 0;
  if (!Number.isFinite(days) || days < 0) return null;

  if (/^\d+$/.test(body)) {
    return days * 86400 + Number(body);
  }

  const parts = body.split(":").map((part) => part.trim());
  if (parts.length < 2 || parts.length > 3 || parts.some((part) => part === "" || Number.isNaN(Number(part)))) {
    return null;
  }

  const numbers = parts.map(Number);
  const [first, second, third = 0] = numbers;
  const hours = parts.length === 3 ? first : 0;
  const minutes = parts.length === 3 ? second : first;
  const seconds = parts.length === 3 ? third : second;
  if (minutes >= 60 || seconds >= 60) return null;
  return days * 86400 + hours * 3600 + minutes * 60 + seconds;
}

function formatDateTime(value?: string | null, language: AppSettings["language"] = "ru"): string {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return String(value);
  return new Intl.DateTimeFormat(language === "en" ? "en-US" : "ru-RU", {
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

function actualLinkSeconds(link: VisitedUrlRecord): number {
  return Math.max(0, Number(link.time_seconds || 0));
}

function includedLinkSeconds(link: VisitedUrlRecord): number {
  return link.enabled ? actualLinkSeconds(link) : 0;
}

function appIncludedSeconds(app: AppUsageRecord): number {
  if (app.kind === "browser") {
    if (!app.enabled) return 0;
    return app.tabs.reduce((sum, tab) => sum + tabIncludedSeconds(app, tab), 0);
  }
  return app.enabled ? app.time_seconds : 0;
}

function tabIncludedSeconds(app: AppUsageRecord, tab: TabUsageRecord): number {
  if (!app.enabled || !tab.enabled) return 0;
  if (tab.urls?.length) {
    return tab.urls.reduce((sum, link) => sum + includedLinkSeconds(link), 0);
  }
  return tab.time_seconds;
}

function appActualSeconds(app: AppUsageRecord): number {
  if (app.kind === "browser") {
    return app.tabs.reduce((sum, tab) => sum + tabActualSeconds(tab), 0);
  }
  return app.time_seconds;
}

function tabActualSeconds(tab: TabUsageRecord): number {
  if (tab.urls?.length) {
    return tab.urls.reduce((sum, link) => sum + actualLinkSeconds(link), 0);
  }
  return tab.time_seconds;
}

function sortRankedApps(apps: AppUsageRecord[], totalSeconds: number): RankedApp[] {
  return [...apps]
    .map((app) => {
      const includedSeconds = appIncludedSeconds(app);
      const actualSeconds = appActualSeconds(app);
      return {
        app,
        includedSeconds,
        actualSeconds,
        includedPercent: percentOf(totalSeconds, includedSeconds),
        actualPercent: percentOf(totalSeconds, actualSeconds),
      };
    })
    .sort((left, right) => right.includedSeconds - left.includedSeconds || right.actualSeconds - left.actualSeconds || left.app.name.localeCompare(right.app.name))
    .map((item) => item);
}

function sortRankedTabs(app: AppUsageRecord, totalSeconds: number): RankedTab[] {
  return [...app.tabs]
    .map((tab) => {
      const includedSeconds = tabIncludedSeconds(app, tab);
      return {
        tab,
        includedSeconds,
        actualSeconds: tabActualSeconds(tab),
        includedPercent: percentOf(totalSeconds, includedSeconds),
        actualPercent: percentOf(totalSeconds, tabActualSeconds(tab)),
      };
    })
    .sort((left, right) => right.includedSeconds - left.includedSeconds || right.actualSeconds - left.actualSeconds || left.tab.title.localeCompare(right.tab.title))
    .map((item) => item);
}

function tabUrlList(tab: TabUsageRecord): VisitedUrlRecord[] {
  const history = [...(tab.urls ?? [])].filter((item) => item.url?.trim());
  if (history.length === 0 && tab.url) {
    history.push({
      url: tab.url,
      title: tab.title,
      last_seen_at: "",
      hits: 1,
      enabled: true,
      time_seconds: tab.time_seconds,
    });
  }
  return history.sort((left, right) => right.time_seconds - left.time_seconds || right.hits - left.hits || right.last_seen_at.localeCompare(left.last_seen_at));
}

export default function App() {
  const [state, setState] = useState<AppState>(fallbackState);
  const [expandedApps, setExpandedApps] = useState<Record<string, boolean>>({});
  const [linkMenu, setLinkMenu] = useState<string | null>(null);
  const [projectMenu, setProjectMenu] = useState<ProjectMenuState | null>(null);
  const [renameProject, setRenameProject] = useState<RenameState | null>(null);
  const [modal, setModal] = useState<"settings" | "help" | "stages" | null>(null);
  const [stageModal, setStageModal] = useState<StageModalState | null>(null);
  const [newProjectName, setNewProjectName] = useState("");
  const [toast, setToast] = useState<ToastState | null>(null);
  const [timeDrafts, setTimeDrafts] = useState<Record<string, string>>({});
  const importInputRef = useRef<HTMLInputElement>(null);
  const toastTimerRef = useRef<number | null>(null);
  const settingsSaveSeqRef = useRef(0);
  const settingsSavePendingRef = useRef(0);

  async function refresh() {
    const next = await invokeCommand<AppState>("get_app_state", {}, fallbackState);
    setState((current) => ({
      ...next,
      settings: settingsSavePendingRef.current > 0 ? current.settings : next.settings,
    }));
  }

  useEffect(() => {
    refresh();
    if (window.localStorage.getItem("project-time-manager-help-seen") !== "1") {
      setModal("help");
      window.localStorage.setItem("project-time-manager-help-seen", "1");
    }
    const timer = window.setInterval(refresh, 1500);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (!toast) return;
    if (toastTimerRef.current) {
      window.clearTimeout(toastTimerRef.current);
    }
    toastTimerRef.current = window.setTimeout(() => setToast(null), 4200);
    return () => {
      if (toastTimerRef.current) {
        window.clearTimeout(toastTimerRef.current);
      }
    };
  }, [toast]);

  const selectedProject = state.selected_project;
  const settings = state.settings ?? fallbackState.settings;
  const language = settings.language === "en" ? "en" : "ru";
  const t = copy[language];
  const apps = selectedProject?.apps ?? [];
  const sessions = selectedProject?.sessions ?? [];
  const trackerStatus = state.tracker.status;
  const isRunning = trackerStatus === "running";
  const isPaused = trackerStatus === "paused";
  const isRecordingLocked = isRunning;
  const statusLabel = trackerStatus === "running" ? t.statusRunning : trackerStatus === "paused" ? t.statusPaused : t.statusStopped;

  const totals = useMemo(() => {
    const appTime = apps.reduce((sum, app) => sum + appIncludedSeconds(app), 0);
    const topApp = [...apps].sort((a, b) => appIncludedSeconds(b) - appIncludedSeconds(a))[0];
    const topTab = apps
      .flatMap((app) =>
        app.tabs
          .filter((tab) => app.enabled && tab.enabled)
          .map((tab) => ({ ...tab, browser: app.name, included: tabIncludedSeconds(app, tab) })),
      )
      .sort((a, b) => b.included - a.included || tabActualSeconds(b) - tabActualSeconds(a))[0];

    return { appTime, topApp, topTab };
  }, [apps]);

  const rankedApps = useMemo(() => sortRankedApps(apps, totals.appTime), [apps, totals.appTime]);

  useEffect(() => {
    setTimeDrafts({});
  }, [selectedProject?.id]);

  async function createProject() {
    if (!newProjectName.trim()) return;
    await invokeCommand<ProjectSummary | null>(
        "create_project",
        {
          name: newProjectName.trim(),
          client: "",
        },
        null,
      );
    setNewProjectName("");
    setToast({ text: t.projectCreated });
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

  async function toggleUrl(projectId: string, appKey: string, tabKey: string, url: string, enabled: boolean) {
    const updated = await invokeCommand<ProjectRecord | null>(
      "toggle_url_included",
      { projectId, appKey, tabKey, url, enabled },
      null,
    );
    if (updated) {
      setState((current) => ({
        ...current,
        selected_project: updated,
      }));
    }
    refresh();
  }

  async function commitAppTime(projectId: string, appKey: string, seconds: number) {
    const updated = await invokeCommand<ProjectRecord | null>("set_app_time", { projectId, appKey, seconds }, null);
    if (updated) {
      setState((current) => ({
        ...current,
        selected_project: updated,
      }));
    }
    refresh();
  }

  async function commitTabTime(projectId: string, appKey: string, tabKey: string, seconds: number) {
    const updated = await invokeCommand<ProjectRecord | null>(
      "set_tab_time",
      { projectId, appKey, tabKey, seconds },
      null,
    );
    if (updated) {
      setState((current) => ({
        ...current,
        selected_project: updated,
      }));
    }
    refresh();
  }

  function timeDraftKey(kind: "app" | "tab", key: string, tabKey?: string) {
    return kind === "app" ? `app:${key}` : `tab:${key}:${tabKey}`;
  }

  function timeDraftValue(kind: "app" | "tab", valueSeconds: number, key: string, tabKey?: string) {
    return timeDrafts[timeDraftKey(kind, key, tabKey)] ?? formatDurationInput(valueSeconds, language);
  }

  function setTimeDraft(kind: "app" | "tab", key: string, value: string, tabKey?: string) {
    setTimeDrafts((current) => ({
      ...current,
      [timeDraftKey(kind, key, tabKey)]: value,
    }));
  }

  function clearTimeDraft(kind: "app" | "tab", key: string, tabKey?: string) {
    setTimeDrafts((current) => {
      const next = { ...current };
      delete next[timeDraftKey(kind, key, tabKey)];
      return next;
    });
  }

  async function submitTimeDraft(kind: "app" | "tab", key: string, rawValue: string, tabKey?: string) {
    const parsed = parseDurationInput(rawValue);
    if (parsed === null) {
      clearTimeDraft(kind, key, tabKey);
      return;
    }
    if (!selectedProject) return;
    if (kind === "app") {
      await commitAppTime(selectedProject.id, key, parsed);
    } else if (tabKey) {
      await commitTabTime(selectedProject.id, key, tabKey, parsed);
    }
    clearTimeDraft(kind, key, tabKey);
  }

  async function exportXlsx() {
    if (isRecordingLocked) return;
    const result = await invokeCommand<ExportResult | null>("export_selected_project_xlsx", {}, null);
    setToast({ text: result?.message ?? t.exportDone, exportPath: result?.path });
    refresh();
  }

  async function exportPdf() {
    if (isRecordingLocked) return;
    const result = await invokeCommand<ExportResult | null>("export_selected_project_pdf", {}, null);
    setToast({ text: result?.message ?? t.exportDone, exportPath: result?.path });
    refresh();
  }

  async function handleImportFile(event: React.ChangeEvent<HTMLInputElement>) {
    if (isRecordingLocked) return;
    const file = event.target.files?.[0];
    if (!file) return;

    const jsonText = await file.text();
    const result = await invokeCommand<ProjectSummary | null>("import_project_json", { jsonText }, null);
    setToast({ text: result?.name ? `${t.importedProject}: ${result.name}` : t.importDone });
    event.target.value = "";
    refresh();
  }

  async function openExportLocation(path?: string) {
    if (!path) return;
    await invokeCommand<void>("open_report_file", { path }, undefined);
  }

  async function openTabUrl(url?: string | null) {
    if (!url) return;
    await invokeCommand<void>("open_external_url", { url }, undefined);
  }

  async function updateSettings(next: AppSettings) {
    const saveSeq = settingsSaveSeqRef.current + 1;
    settingsSaveSeqRef.current = saveSeq;
    settingsSavePendingRef.current += 1;
    setState((current) => ({
      ...current,
      settings: next,
    }));
    try {
      const updated = await invokeCommand<AppSettings>("update_app_settings", next, next);
      if (settingsSaveSeqRef.current === saveSeq) {
        setState((current) => ({
          ...current,
          settings: updated,
        }));
      }
    } finally {
      settingsSavePendingRef.current = Math.max(0, settingsSavePendingRef.current - 1);
    }
  }

  async function openAppFolder() {
    await invokeCommand<void>("open_app_folder", {}, undefined);
  }

  async function submitRenameProject() {
    if (!renameProject?.value.trim()) return;
    const updated = await invokeCommand<ProjectRecord | null>(
      "rename_project",
      { projectId: renameProject.projectId, name: renameProject.value.trim() },
      null,
    );
    if (updated) {
      setState((current) => ({
        ...current,
        projects: current.projects.map((project) =>
          project.id === updated.id ? { ...project, name: updated.name, updated_at: updated.updated_at } : project,
        ),
        selected_project: current.selected_project?.id === updated.id ? updated : current.selected_project,
      }));
    }
    setRenameProject(null);
    refresh();
  }

  async function removeProject(projectId: string) {
    if (!window.confirm(t.deleteConfirm)) return;
    await invokeCommand<void>("delete_project", { projectId }, undefined);
    setProjectMenu(null);
    refresh();
  }

  return (
    <div
      className="h-screen min-h-[900px] overflow-hidden bg-[linear-gradient(180deg,#f8fbf8_0%,#eef6ef_100%)] text-slate-900"
      onContextMenu={(event) => event.preventDefault()}
      onClick={() => {
        setLinkMenu(null);
        setProjectMenu(null);
      }}
    >
      <div className="mx-auto flex h-full w-full min-w-[1620px] max-w-[1840px] flex-col gap-4 overflow-hidden px-4 py-4 lg:px-5">
        <main className="grid min-h-0 flex-1 grid-cols-[340px_minmax(0,1fr)] gap-4 overflow-hidden">
          <aside className="flex min-h-0 flex-col gap-4">
            <section className="flex min-h-0 flex-[1.08] flex-col rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="flex items-center justify-between gap-3">
                <h2 className="text-base font-semibold text-slate-900">{t.projectLabel}</h2>
                <button className="icon-button" onClick={refresh} title={t.projectLabel}>
                  <RefreshCw size={16} />
                </button>
              </div>

              <div className="mt-4 grid gap-2">
                <input
                  className="field"
                  value={newProjectName}
                  onChange={(event) => setNewProjectName(event.target.value)}
                  placeholder={t.projectNamePlaceholder}
                />
                <button className="primary-button" onClick={createProject}>
                  <FolderPlus size={16} />
                  {t.createLabel}
                </button>
              </div>

              <div className="stable-scroll mt-4 min-h-0 flex-1 overflow-y-scroll pr-1">
                <div className="grid w-full gap-2">
                  {state.projects.length ? (
                    state.projects.map((project) => (
                      <div key={project.id}>
                        <div
                          className={`flex w-full items-center justify-between gap-2 rounded-2xl border px-3 py-3 text-left transition ${
                            project.id === selectedProject?.id
                              ? "border-emerald-300 bg-emerald-50"
                              : "border-slate-200 bg-white hover:border-emerald-200"
                          }`}
                        >
                          <button className="min-w-0 flex-1 text-left" onClick={() => selectProject(project.id)}>
                            <strong className="block truncate text-sm text-slate-900">{project.name}</strong>
                            <span className="block truncate text-xs text-slate-500">{formatDateTime(project.updated_at, language)}</span>
                          </button>
                          <button
                            className="flex size-8 shrink-0 items-center justify-center rounded-full text-slate-400 transition-colors hover:bg-white hover:text-emerald-700"
                            onClick={(event) => {
                              event.stopPropagation();
                              const rect = event.currentTarget.getBoundingClientRect();
                              setProjectMenu(
                                projectMenu?.projectId === project.id
                                  ? null
                                  : {
                                      projectId: project.id,
                                      projectName: project.name,
                                      x: Math.max(12, rect.right - 208),
                                      y: rect.bottom + 8,
                                    },
                              );
                            }}
                            title={t.projectLabel}
                          >
                            <ChevronRight size={16} />
                          </button>
                        </div>
                      </div>
                    ))
                  ) : (
                    <EmptyState text={t.noProjects} />
                  )}
                </div>
              </div>
            </section>

            <section className="flex min-h-0 flex-1 flex-col rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <SectionTitle icon={<TimerReset size={16} />} title={t.sessionsLabel} />
              <div className="stable-scroll mt-3 min-h-0 flex-1 overflow-y-scroll pr-1">
                <div className="grid gap-2">
                {sessions.length ? (
                  sessions.map((session) => (
                    <article key={session.id} className="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3">
                      <strong className="block text-sm text-slate-900">
                        {formatDateTime(session.started_at, language)} — {formatDateTime(session.stopped_at, language)}
                      </strong>
                      <span className="mt-1 block text-sm text-slate-500">{formatDuration(session.duration_seconds, language)}</span>
                    </article>
                  ))
                ) : (
                  <EmptyState text={t.noSessions} />
                )}
                </div>
              </div>
            </section>

            <section className="shrink-0 rounded-[24px] border border-emerald-100 bg-white p-3 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="grid grid-cols-2 gap-2">
                <button className="secondary-button min-h-10 px-3" onClick={() => setModal("settings")}>
                  <Settings size={16} />
                  {t.settingsLabel}
                </button>
                <button className="secondary-button min-h-10 px-3" onClick={() => setModal("help")}>
                  <CircleAlert size={16} />
                  {t.helpLabel}
                </button>
              </div>
            </section>
          </aside>

          <section className="grid min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-4 overflow-hidden">
            <section className="rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="grid grid-cols-[minmax(420px,1fr)_auto_auto] items-center gap-4">
                <div className="space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <h1 className="text-xl font-semibold text-slate-900">{selectedProject?.name ?? t.appName}</h1>
                    <span className="inline-flex items-center gap-2 rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs font-semibold uppercase text-emerald-700">
                      <Activity size={13} />
                      {statusLabel}
                    </span>
                  </div>
                  <p className="whitespace-nowrap text-sm text-slate-500">{t.currentProjectHint}</p>
                </div>

                <div className="flex flex-nowrap gap-2">
                  <button className="primary-button" onClick={() => toggleTracking("start_tracking")} disabled={isRunning || !selectedProject}>
                    <CirclePlay size={16} />
                    {isPaused ? t.continueLabel : t.startLabel}
                  </button>
                  <button className="secondary-button" onClick={() => toggleTracking("pause_tracking")} disabled={!isRunning || !selectedProject}>
                    <CirclePause size={16} />
                    {t.pauseLabel}
                  </button>
                  <button className="secondary-button" onClick={() => toggleTracking("stop_tracking")} disabled={(!isRunning && !isPaused) || !selectedProject}>
                    <Square size={16} />
                    {t.stopLabel}
                  </button>
                </div>

                <div className="flex flex-nowrap gap-2">
                  <button className="secondary-button" onClick={() => importInputRef.current?.click()} disabled={isRecordingLocked}>
                    <Import size={16} />
                    {t.importJsonLabel}
                  </button>
                  <button className="secondary-button" onClick={exportXlsx} disabled={isRecordingLocked}>
                    <Download size={16} />
                    {t.exportExcelLabel}
                  </button>
                  <button className="secondary-button" onClick={exportPdf} disabled={isRecordingLocked}>
                    <FileText size={16} />
                    {t.exportPdfLabel}
                  </button>
                </div>
              </div>

              <input ref={importInputRef} type="file" accept=".json" hidden onChange={handleImportFile} />
            </section>

            {selectedProject ? (
              <>
                <section className="grid grid-cols-4 gap-3">
                  <Metric label={t.totalProjectLabel} value={formatDuration(totals.appTime, language)} accent="emerald" />
                  <Metric label={t.sessionsCountLabel} value={String(sessions.length)} accent="emerald" />
                  <Metric label={t.topAppLabel} value={totals.topApp?.name ?? "-"} accent="slate" />
                  <Metric label={t.topTabLabel} value={totals.topTab?.title ?? "-"} accent="slate" />
                </section>

                <section className="flex min-h-0 flex-col rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
                  <SectionTitle icon={<Activity size={16} />} title={t.appsTitle} />

                  <div className="mt-4 grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)] gap-2 overflow-hidden">
                    <div className="grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] gap-2 px-3 text-xs font-medium uppercase tracking-[0.14em] text-slate-400 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4">
                      <span />
                      <span>{t.applicationHeader}</span>
                      <span>{t.timeHeader}</span>
                      <span>{t.percentHeader}</span>
                    </div>

                    <div className="stable-scroll min-h-0 overflow-y-scroll pr-1">
                      <div className="grid gap-2">
                        {rankedApps.map(({ app, includedSeconds, actualSeconds, includedPercent, actualPercent }) => {
                          const isOpen = expandedApps[app.key];
                          const isIncluded = app.enabled;
                          const appTimeValue = isIncluded ? includedSeconds : 0;

                          return (
                            <div key={app.key} className="grid gap-2">
                              <div
                                className={`grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4 ${
                                  isIncluded ? "border-slate-200 bg-slate-50" : "border-slate-200 bg-slate-100/80 text-slate-500"
                                }`}
                              >
                                <Checkbox checked={app.enabled} onChange={(checked) => toggleApp(selectedProject.id, app.key, checked)} />

                                <button
                                  className="flex min-w-0 items-center gap-3 text-left"
                                  onClick={() => setExpandedApps((prev) => ({ ...prev, [app.key]: !prev[app.key] }))}
                                >
                                  <AppIcon app={app} />
                                  <span className="min-w-0">
                                    <strong className={`block truncate text-sm ${isIncluded ? "text-slate-900" : "text-slate-500"}`}>{app.name}</strong>
                                    <small className={`block truncate text-xs ${isIncluded ? "text-slate-500" : "text-slate-400"}`}>
                                      {app.kind === "browser" ? t.browserLabel : app.process_path || app.process_name}
                                    </small>
                                  </span>
                                  {app.kind === "browser" ? (
                                    <span className="ml-auto text-slate-400">{isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}</span>
                                  ) : null}
                                </button>

                                {isIncluded ? (
                                  <span className="min-w-0">
                                    <input
                                      className="w-full min-w-0 rounded-xl border border-transparent bg-white px-2 py-1 text-right font-mono text-sm text-slate-700 outline-none transition focus:border-emerald-300 focus:ring-2 focus:ring-emerald-100"
                                      value={timeDraftValue("app", appTimeValue, app.key)}
                                      onFocus={(event) => setTimeDraft("app", app.key, event.currentTarget.value)}
                                      onChange={(event) => setTimeDraft("app", app.key, event.currentTarget.value)}
                                      onBlur={(event) => submitTimeDraft("app", app.key, event.currentTarget.value)}
                                      onKeyDown={(event) => {
                                        if (event.key === "Enter") {
                                          event.currentTarget.blur();
                                        }
                                      }}
                                      inputMode="text"
                                    />
                                    {actualSeconds !== appTimeValue ? (
                                      <small className="mt-1 block text-right text-xs text-slate-400">
                                        {formatDuration(actualSeconds, language)}
                                      </small>
                                    ) : null}
                                  </span>
                                ) : (
                                  <span className="font-mono text-sm text-slate-700">
                                    00:00:00
                                    <small className="mt-1 block text-xs text-slate-400">{formatDuration(actualSeconds, language)}</small>
                                  </span>
                                )}
                                <span className="text-sm text-slate-500">
                                  {isIncluded ? includedPercent : "0%"}
                                  {!isIncluded ? <small className="mt-1 block text-xs text-slate-400">{actualPercent}</small> : null}
                                </span>
                              </div>

                              {app.kind === "browser" && isOpen ? (
                                <div className="ml-4 grid gap-2 sm:ml-10">
                                  {sortRankedTabs(app, totals.appTime).map(({ tab, includedSeconds: tabIncluded, actualSeconds: tabActual, includedPercent: tabIncludedPercent, actualPercent: tabActualPercent }) => {
                                    const urls = tabUrlList(tab);
                                    const menuKey = `${app.key}:${tab.key}`;
                                    const isMenuOpen = linkMenu === menuKey;

                                    return (
                                      <div
                                        key={tab.key}
                                        className={`grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4 ${
                                          tab.enabled ? "border-slate-200 bg-white" : "border-slate-200 bg-slate-50 text-slate-500"
                                        }`}
                                      >
                                        <Checkbox
                                          checked={tab.enabled}
                                          onChange={(checked) => toggleTab(selectedProject.id, app.key, tab.key, checked)}
                                        />
                                        <span className="flex min-w-0 items-center gap-3">
                                          <TabIcon tab={tab} />
                                          <span className="relative min-w-0">
                                            <span className="flex max-w-full items-center gap-2">
                                              <strong className={`block truncate text-sm ${tab.enabled ? "text-slate-900" : "text-slate-500"}`}>
                                                {tab.title}
                                              </strong>
                                              {urls.length ? (
                                                <button
                                                  className="inline-flex size-7 shrink-0 items-center justify-center rounded-full border border-emerald-100 bg-emerald-50 text-emerald-600 transition-colors hover:border-emerald-200 hover:bg-emerald-100"
                                                  onClick={(event) => {
                                                    event.stopPropagation();
                                                    setLinkMenu(isMenuOpen ? null : menuKey);
                                                  }}
                                                  title={t.openUrlLabel}
                                                >
                                                  <Link size={13} />
                                                </button>
                                              ) : null}
                                            </span>
                                            <small className={`block truncate text-xs ${tab.enabled ? "text-slate-500" : "text-slate-400"}`}>
                                              {urls.length ? t.linksInDomain(urls.length) : t.urlUnavailable}
                                            </small>
                                            {isMenuOpen ? (
                                              <div
                                                className="absolute left-0 top-12 z-20 w-[min(460px,70vw)] rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
                                                onClick={(event) => event.stopPropagation()}
                                              >
                                                <div className="max-h-60 overflow-y-auto pr-1">
                                                  {urls.map((item) => (
                                                    <div
                                                      key={item.url}
                                                      className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-xl px-3 py-2 text-left transition-colors hover:bg-emerald-50"
                                                    >
                                                      <Checkbox
                                                        checked={item.enabled}
                                                        onChange={(checked) => toggleUrl(selectedProject.id, app.key, tab.key, item.url, checked)}
                                                      />
                                                      <button
                                                        className="min-w-0 text-left"
                                                        onClick={() => {
                                                          setLinkMenu(null);
                                                          openTabUrl(item.url);
                                                        }}
                                                      >
                                                        <strong className="block truncate text-xs text-slate-900">{item.title || item.url}</strong>
                                                        <span className="block truncate text-xs text-slate-500">{item.url}</span>
                                                      </button>
                                                      <span className="font-mono text-xs text-slate-700">{formatDuration(item.time_seconds, language)}</span>
                                                    </div>
                                                  ))}
                                                </div>
                                              </div>
                                            ) : null}
                                          </span>
                                        </span>
                                        {tab.enabled ? (
                                          <span className="min-w-0">
                                            <input
                                              className="w-full min-w-0 rounded-xl border border-transparent bg-white px-2 py-1 text-right font-mono text-sm text-slate-700 outline-none transition focus:border-emerald-300 focus:ring-2 focus:ring-emerald-100"
                                              value={timeDraftValue("tab", tabIncluded, app.key, tab.key)}
                                              onFocus={(event) => setTimeDraft("tab", app.key, event.currentTarget.value, tab.key)}
                                              onChange={(event) => setTimeDraft("tab", app.key, event.currentTarget.value, tab.key)}
                                              onBlur={(event) => submitTimeDraft("tab", app.key, event.currentTarget.value, tab.key)}
                                              onKeyDown={(event) => {
                                                if (event.key === "Enter") {
                                                  event.currentTarget.blur();
                                                }
                                              }}
                                              inputMode="text"
                                            />
                                            {tabActual !== tabIncluded ? (
                                              <small className="mt-1 block text-right text-xs text-slate-400">
                                                {formatDuration(tabActual, language)}
                                              </small>
                                            ) : null}
                                          </span>
                                        ) : (
                                          <span className="font-mono text-sm text-slate-700">
                                            00:00:00
                                            <small className="mt-1 block text-xs text-slate-400">{formatDuration(tabActual, language)}</small>
                                          </span>
                                        )}
                                        <span className="text-sm text-slate-500">
                                          {tab.enabled ? tabIncludedPercent : "0%"}
                                          {!tab.enabled ? <small className="mt-1 block text-xs text-slate-400">{tabActualPercent}</small> : null}
                                        </span>
                                      </div>
                                    );
                                  })}
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
                {t.emptyPrompt}
              </section>
            )}
          </section>
        </main>
      </div>

      {modal === "settings" || modal === "help" ? (
        <AppModal
          type={modal}
          settings={settings}
          t={t}
          onClose={() => setModal(null)}
          onSettingsChange={updateSettings}
          onOpenAppFolder={openAppFolder}
        />
      ) : null}
      {projectMenu ? (
        <div
          className="fixed z-40 w-52 rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
          style={{ left: projectMenu.x, top: projectMenu.y }}
          onClick={(event) => event.stopPropagation()}
        >
          <button
            className="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm text-slate-700 transition-colors hover:bg-emerald-50"
            onClick={() => {
              setRenameProject({ projectId: projectMenu.projectId, value: projectMenu.projectName });
              setProjectMenu(null);
            }}
          >
            <PencilLine size={15} />
            {t.projectMenuRename}
          </button>
          <button
            className="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm text-rose-700 transition-colors hover:bg-rose-50"
            onClick={() => removeProject(projectMenu.projectId)}
          >
            <Trash2 size={15} />
            {t.projectMenuDelete}
          </button>
          <button
            className="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm text-slate-700 transition-colors hover:bg-emerald-50"
            onClick={() => {
              setLinkMenu(null);
              setStageModal({ projectId: projectMenu.projectId });
              setProjectMenu(null);
            }}
          >
            <Activity size={15} />
            {t.projectMenuStages}
          </button>
        </div>
      ) : null}
      {renameProject ? (
        <RenameProjectModal
          state={renameProject}
          t={t}
          onChange={(value) => setRenameProject((current) => (current ? { ...current, value } : current))}
          onClose={() => setRenameProject(null)}
          onSubmit={submitRenameProject}
        />
      ) : null}
      {toast ? <Toast toast={toast} label={t.openReportLabel} onOpenExport={openExportLocation} /> : null}
      {stageModal ? (
        <ProjectStagesModal
          project={selectedProject}
          language={language}
          t={t}
          onClose={() => setStageModal(null)}
          onSetProjectState={(updated) => {
            setState((current) => ({
              ...current,
              selected_project: current.selected_project?.id === updated.id ? updated : current.selected_project,
              projects: current.projects.map((project) =>
                project.id === updated.id
                  ? {
                      id: updated.id,
                      name: updated.name,
                      client: updated.client,
                      updated_at: updated.updated_at,
                    }
                  : project,
              ),
            }));
          }}
        />
      ) : null}
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

function TabIcon({ tab }: { tab: TabUsageRecord }) {
  return (
    <span className="flex size-7 shrink-0 items-center justify-center rounded-xl bg-emerald-50 text-xs font-semibold text-emerald-700">
      {tab.favicon_url ? <img className="size-4 object-contain" src={tab.favicon_url} alt="" draggable={false} /> : "W"}
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
  return (
    <div className="flex min-h-11 w-full min-w-0 items-center self-stretch rounded-2xl border border-dashed border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-500">
      {text}
    </div>
  );
}

function Toast({ toast, label, onOpenExport }: { toast: ToastState; label: string; onOpenExport: (path?: string) => void }) {
  return (
    <footer className="fixed bottom-5 right-5 flex max-w-[560px] items-center gap-3 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900 shadow-[0_10px_30px_rgba(15,23,42,0.1)]">
      <span className="min-w-0 truncate">{toast.text}</span>
      {toast.exportPath ? (
        <button className="icon-button bg-white" onClick={() => onOpenExport(toast.exportPath)} title={label}>
          <FolderOpen size={16} />
        </button>
      ) : null}
    </footer>
  );
}

function AppModal({
  type,
  settings,
  t,
  onClose,
  onSettingsChange,
  onOpenAppFolder,
}: {
  type: "settings" | "help";
  settings: AppSettings;
  t: Copy;
  onClose: () => void;
  onSettingsChange: (settings: AppSettings) => void;
  onOpenAppFolder: () => void;
}) {
  const isHelp = type === "help";

  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-slate-900/24 px-5 backdrop-blur-sm">
      <section className="w-full max-w-[720px] overflow-hidden rounded-[30px] border border-emerald-100 bg-white shadow-[0_28px_90px_rgba(15,23,42,0.22)]">
        <header className="flex items-start justify-between gap-4 bg-[linear-gradient(135deg,#ecfdf5_0%,#ffffff_62%,#dcfce7_100%)] px-7 py-6">
          <div>
            <span className="inline-flex rounded-full bg-emerald-600 px-3 py-1 text-xs font-semibold uppercase text-white">
              Project Time Manager
            </span>
            <h2 className="mt-4 text-2xl font-semibold text-slate-900">{isHelp ? t.helpTitle : t.settingsTitle}</h2>
            <p className="mt-2 max-w-xl text-sm leading-6 text-slate-600">
              {isHelp ? t.helpDescription : t.settingsDescription}
            </p>
          </div>
          <button className="icon-button shrink-0" onClick={onClose} title={t.closeLabel}>
            <X size={18} />
          </button>
        </header>

        <div className="stable-scroll max-h-[58vh] overflow-y-auto px-7 py-6">
          {isHelp ? (
            <div className="grid gap-4">
              <HelpItem
                title={t.helpProjectsTitle}
                text={t.helpProjectsText}
              />
              <HelpItem
                title={t.helpTrackingTitle}
                text={t.helpTrackingText}
              />
              <HelpItem
                title={t.helpAppsTitle}
                text={t.helpAppsText}
              />
              <HelpItem
                title={t.helpSitesTitle}
                text={t.helpSitesText}
              />
              <HelpItem
                title={t.helpReportsTitle}
                text={t.helpReportsText}
              />
            </div>
          ) : (
            <div className="grid gap-4">
              <label className="flex items-center justify-between gap-4 rounded-3xl border border-emerald-100 bg-slate-50 px-5 py-4">
                <span>
                  <strong className="block text-sm text-slate-900">{t.autostartLabel}</strong>
                  <span className="mt-1 block text-xs text-slate-500">Windows</span>
                </span>
                <input
                  className="size-5 accent-emerald-600"
                  type="checkbox"
                  checked={settings.autostart}
                  onChange={(event) => onSettingsChange({ ...settings, autostart: event.target.checked })}
                />
              </label>

              <label className="grid gap-2 rounded-3xl border border-emerald-100 bg-slate-50 px-5 py-4">
                <span className="flex items-center gap-2 text-sm font-semibold text-slate-900">
                  <Languages size={16} />
                  {t.languageLabel}
                </span>
                <span className="relative block">
                  <select
                    className="field appearance-none pr-12"
                    value={settings.language}
                    onChange={(event) => onSettingsChange({ ...settings, language: event.target.value as AppSettings["language"] })}
                  >
                    <option value="ru">{t.languageRu}</option>
                    <option value="en">{t.languageEn}</option>
                  </select>
                  <ChevronDown className="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                </span>
              </label>

              <button className="secondary-button w-fit" onClick={onOpenAppFolder}>
                <FolderOpen size={16} />
                {t.openFolderLabel}
              </button>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}

function RenameProjectModal({
  state,
  t,
  onChange,
  onClose,
  onSubmit,
}: {
  state: RenameState;
  t: Copy;
  onChange: (value: string) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-slate-900/24 px-5 backdrop-blur-sm">
      <section className="w-full max-w-[460px] rounded-[28px] border border-emerald-100 bg-white p-6 shadow-[0_28px_90px_rgba(15,23,42,0.22)]">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold text-slate-900">{t.renameTitle}</h2>
            <p className="mt-2 text-sm text-slate-500">{t.renameDescription}</p>
          </div>
          <button className="icon-button shrink-0" onClick={onClose} title={t.closeLabel}>
            <X size={18} />
          </button>
        </div>
        <input
          className="field mt-5"
          value={state.value}
          autoFocus
          placeholder={t.renamePlaceholder}
          onChange={(event) => onChange(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") onSubmit();
            if (event.key === "Escape") onClose();
          }}
        />
        <div className="mt-5 flex justify-end gap-2">
          <button className="secondary-button" onClick={onClose}>
            {t.cancelLabel}
          </button>
          <button className="primary-button" onClick={onSubmit}>
            <Check size={16} />
            {t.saveLabel}
          </button>
        </div>
      </section>
    </div>
  );
}

function HelpItem({ title, text }: { title: string; text: string }) {
  return (
    <article className="rounded-3xl border border-emerald-100 bg-slate-50 px-5 py-4">
      <strong className="block text-sm font-semibold text-emerald-800">{title}</strong>
      <p className="mt-2 text-sm leading-6 text-slate-600">{text}</p>
    </article>
  );
}

function ProjectStagesModal({
  project,
  language,
  t,
  onClose,
  onSetProjectState,
}: {
  project: ProjectRecord | null;
  language: AppSettings["language"];
  t: Copy;
  onClose: () => void;
  onSetProjectState: (updated: ProjectRecord) => void;
}) {
  const [localProject, setLocalProject] = useState(project);
  const [selectedStageId, setSelectedStageId] = useState<string | null>(project?.stages[0]?.id ?? null);
  const [stageName, setStageName] = useState(project?.stages[0]?.name ?? "");
  const [expandedApps, setExpandedApps] = useState<Record<string, boolean>>({});
  const [dragStageId, setDragStageId] = useState<string | null>(null);
  const [linkMenu, setLinkMenu] = useState<string | null>(null);

  useEffect(() => {
    setLocalProject(project);
    const firstStage = project?.stages[0] ?? null;
    const preservedStage = project?.stages.find((stage) => stage.id === selectedStageId) ?? firstStage;
    setSelectedStageId((current) => (current && project?.stages.some((stage) => stage.id === current) ? current : firstStage?.id ?? null));
    setStageName((current) => preservedStage?.name ?? current);
    setDragStageId(null);
    setLinkMenu(null);
  }, [project?.id]);

  const selectedStage = localProject?.stages.find((stage) => stage.id === selectedStageId) ?? null;
  const orderedStages = [...(localProject?.stages ?? [])].sort((left, right) => left.order - right.order);
  const projectTotal = localProject?.apps.reduce((sum, app) => sum + appIncludedSeconds(app), 0) ?? 0;

  async function openStageUrl(url?: string | null) {
    if (!url) return;
    await invokeCommand<void>("open_external_url", { url }, undefined);
  }

  const stageTotals = useMemo(() => {
    if (!localProject || !selectedStage) {
      return {
        total: 0,
        share: "0%",
        apps: 0,
        tabs: 0,
        topApp: null as AppUsageRecord | null,
        topTab: null as { title: string; browser: string } | null,
      };
    }

    const includedApps = localProject.apps.map((app) => {
      const includedSeconds = stageIncludedAppSeconds(localProject, selectedStage, app);
      const actualSeconds = appActualSeconds(app);
      return { app, includedSeconds, actualSeconds };
    });

    const includedTabs = localProject.apps.flatMap((app) =>
      app.tabs.map((tab) => ({
        app,
        tab,
        includedSeconds: stageIncludedTabSeconds(localProject, selectedStage, app, tab),
        actualSeconds: tabActualSeconds(tab),
      })),
    );

    const total = includedApps.reduce((sum, item) => sum + item.includedSeconds, 0);
    const topApp = [...includedApps].sort((left, right) => right.includedSeconds - left.includedSeconds || right.actualSeconds - left.actualSeconds)[0]?.app ?? null;
    const topTab = [...includedTabs]
      .filter((item) => item.includedSeconds > 0)
      .sort((left, right) => right.includedSeconds - left.includedSeconds || right.actualSeconds - left.actualSeconds)[0];

    return {
      total,
      share: percentOf(projectTotal, total),
      apps: includedApps.filter((item) => item.includedSeconds > 0).length,
      tabs: includedTabs.filter((item) => item.includedSeconds > 0).length,
      topApp,
      topTab: topTab ? { title: topTab.tab.title, browser: topTab.app.name } : null,
    };
  }, [localProject, selectedStage, projectTotal]);

  async function saveUpdatedProject(command: Promise<ProjectRecord | null>) {
    const updated = await command;
    if (!updated) return null;
    setLocalProject(updated);
    onSetProjectState(updated);
    const nextStage = updated.stages.find((stage) => stage.id === selectedStageId) ?? updated.stages[0] ?? null;
    setSelectedStageId(nextStage?.id ?? null);
    return updated;
  }

  async function createStage() {
    if (!localProject || !stageName.trim()) return;
    const updated = await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>("create_stage", { projectId: localProject.id, name: stageName.trim() }, null),
    );
    if (!updated) return;
    const next = [...updated.stages].sort((left, right) => left.order - right.order).at(-1) ?? null;
    setSelectedStageId(next?.id ?? null);
    setStageName(next?.name ?? stageName);
  }

  async function renameStage() {
    if (!localProject || !selectedStage || !stageName.trim()) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "rename_stage",
        { projectId: localProject.id, stageId: selectedStage.id, name: stageName.trim() },
        null,
      ),
    );
    setStageName(stageName.trim());
  }

  async function removeStage() {
    if (!localProject || !selectedStage) return;
    if (!window.confirm(t.stageDeleteConfirm)) return;
    const updated = await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>("delete_stage", { projectId: localProject.id, stageId: selectedStage.id }, null),
    );
    const next = updated?.stages[0] ?? null;
    setSelectedStageId(next?.id ?? null);
    setStageName(next?.name ?? "");
  }

  async function moveStage(stageId: string, direction: number) {
    if (!localProject) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "reorder_stage",
        { projectId: localProject.id, stageId, direction },
        null,
      ),
    );
  }

  async function moveStageToIndex(stageId: string, targetIndex: number) {
    if (!localProject) return;
    const currentIndex = orderedStages.findIndex((stage) => stage.id === stageId);
    if (currentIndex < 0 || currentIndex === targetIndex) return;
    const direction = currentIndex < targetIndex ? 1 : -1;
    let next = localProject;
    let steps = Math.abs(targetIndex - currentIndex);
    while (steps > 0) {
      const updated = await invokeCommand<ProjectRecord | null>(
        "reorder_stage",
        { projectId: next.id, stageId, direction },
        null,
      );
      if (!updated) return;
      next = updated;
      setLocalProject(updated);
      onSetProjectState(updated);
      steps -= 1;
    }
    const stage = next.stages.find((item) => item.id === stageId) ?? null;
    setSelectedStageId(stage?.id ?? null);
    setStageName(stage?.name ?? "");
  }

  async function toggleStageApp(appKey: string, enabled: boolean) {
    if (!localProject || !selectedStage) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "toggle_stage_app_included",
        { projectId: localProject.id, stageId: selectedStage.id, appKey, enabled },
        null,
      ),
    );
  }

  async function toggleStageTab(appKey: string, tabKey: string, enabled: boolean) {
    if (!localProject || !selectedStage) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "toggle_stage_tab_included",
        { projectId: localProject.id, stageId: selectedStage.id, appKey, tabKey, enabled },
        null,
      ),
    );
  }

  async function toggleStageUrl(appKey: string, tabKey: string, url: string, enabled: boolean) {
    if (!localProject || !selectedStage) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "toggle_stage_url_included",
        { projectId: localProject.id, stageId: selectedStage.id, appKey, tabKey, url, enabled },
        null,
      ),
    );
  }

  if (!localProject) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/24 px-5 backdrop-blur-sm">
      <section className="flex h-[82vh] w-full max-w-[1560px] flex-col overflow-hidden rounded-[30px] border border-emerald-100 bg-white shadow-[0_28px_90px_rgba(15,23,42,0.22)]">
        <header className="flex items-start justify-between gap-4 bg-[linear-gradient(135deg,#ecfdf5_0%,#ffffff_62%,#dcfce7_100%)] px-7 py-6">
          <div>
            <span className="inline-flex rounded-full bg-emerald-600 px-3 py-1 text-xs font-semibold uppercase text-white">
              Project Time Manager
            </span>
            <h2 className="mt-4 text-2xl font-semibold text-slate-900">{t.stageTitle}</h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-600">{t.stageDescription}</p>
          </div>
          <button className="icon-button shrink-0" onClick={onClose} title={t.closeLabel}>
            <X size={18} />
          </button>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-[300px_minmax(0,1fr)] gap-4 px-6 py-5">
          <aside className="flex min-h-0 flex-col gap-4">
            <section className="flex min-h-0 flex-1 flex-col rounded-[26px] border border-emerald-100 bg-white p-4">
              <div className="flex items-center justify-between gap-3">
                <h3 className="text-base font-semibold text-slate-900">{t.stageTitle}</h3>
                <span className="text-xs font-medium uppercase tracking-[0.14em] text-slate-400">{orderedStages.length}</span>
              </div>

              <div className="stable-scroll mt-4 min-h-0 flex-1 overflow-y-scroll pr-1">
                <div className="grid gap-2">
                  {orderedStages.length ? (
                    orderedStages.map((stage, index) => {
                      const isSelected = stage.id === selectedStage?.id;
                      return (
                        <div
                          key={stage.id}
                          className={`rounded-2xl border px-3 py-3 transition ${
                            isSelected ? "border-emerald-300 bg-emerald-50" : "border-slate-200 bg-white hover:border-emerald-200"
                          }`}
                          draggable
                          onDragStart={() => setDragStageId(stage.id)}
                          onDragOver={(event) => event.preventDefault()}
                          onDrop={async () => {
                            if (!dragStageId || dragStageId === stage.id) return;
                            const sourceIndex = orderedStages.findIndex((item) => item.id === dragStageId);
                            const targetIndex = orderedStages.findIndex((item) => item.id === stage.id);
                            if (sourceIndex < 0 || targetIndex < 0) return;
                            await moveStageToIndex(dragStageId, targetIndex);
                            setDragStageId(null);
                          }}
                        >
                          <button className="flex w-full items-center gap-3 text-left" onClick={() => {
                            setSelectedStageId(stage.id);
                            setStageName(stage.name);
                          }}>
                            <GripVertical size={15} className="shrink-0 text-slate-300" />
                            <span className="min-w-0 flex-1">
                              <strong className="block truncate text-sm text-slate-900">{stage.name}</strong>
                              <small className="block text-xs text-slate-500">{formatDateTime(stage.updated_at, language)}</small>
                            </span>
                          </button>
                          <div className="mt-3 flex gap-2">
                            <button className="icon-button h-9 w-9" onClick={() => moveStage(stage.id, -1)} title={t.stageUpLabel} disabled={index === 0}>
                              <ArrowUp size={14} />
                            </button>
                            <button
                              className="icon-button h-9 w-9"
                              onClick={() => moveStage(stage.id, 1)}
                              title={t.stageDownLabel}
                              disabled={index === orderedStages.length - 1}
                            >
                              <ArrowDown size={14} />
                            </button>
                          </div>
                        </div>
                      );
                    })
                  ) : (
                    <EmptyState text={t.stageEmpty} />
                  )}
                </div>
              </div>
            </section>

            <section className="rounded-[26px] border border-emerald-100 bg-white p-4">
              <input
                className="field"
                value={stageName}
                onChange={(event) => setStageName(event.target.value)}
                placeholder={t.stageNamePlaceholder}
              />
              <div className="mt-3 grid grid-cols-3 gap-2">
                <button className="secondary-button px-3" onClick={createStage}>
                  <FolderPlus size={15} />
                  {t.stageCreateLabel}
                </button>
                <button className="secondary-button px-3" onClick={renameStage} disabled={!selectedStage}>
                  <PencilLine size={15} />
                  {t.stageRenameLabel}
                </button>
                <button className="secondary-button px-3 text-rose-700 hover:bg-rose-50" onClick={removeStage} disabled={!selectedStage}>
                  <Trash2 size={15} />
                  {t.stageDeleteLabel}
                </button>
              </div>
              <p className="mt-3 text-xs leading-5 text-slate-500">{t.stageCreateHint}</p>
            </section>
          </aside>

          <section className="grid min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-4 overflow-hidden">
            <section className="grid grid-cols-4 gap-3">
              <Metric label={t.stageMetricsTime} value={formatDuration(stageTotals.total, language)} accent="emerald" />
              <Metric label={t.stageMetricsShare} value={stageTotals.share} accent="slate" />
              <Metric label={t.stageMetricsApps} value={String(stageTotals.apps)} accent="emerald" />
              <Metric label={t.stageMetricsTabs} value={String(stageTotals.tabs)} accent="slate" />
            </section>

            <section className="flex min-h-0 flex-col rounded-[26px] border border-emerald-100 bg-white p-4">
              <SectionTitle icon={<Activity size={16} />} title={selectedStage?.name ?? t.stageTitle} />

              {!selectedStage ? (
                <div className="mt-4">
                  <EmptyState text={t.stageEmpty} />
                </div>
              ) : (
                <div className="mt-4 grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)] gap-2 overflow-hidden">
                  <div className="grid grid-cols-[36px_minmax(120px,1fr)_92px_56px] gap-2 px-3 text-xs font-medium uppercase tracking-[0.14em] text-slate-400 sm:grid-cols-[44px_minmax(220px,1fr)_120px_72px] sm:gap-3 sm:px-4">
                    <span />
                    <span>{t.applicationHeader}</span>
                    <span>{t.timeHeader}</span>
                    <span>{t.percentHeader}</span>
                  </div>

                  <div className="stable-scroll min-h-0 overflow-y-scroll pr-1">
                    <div className="grid gap-2">
                      {localProject.apps.length ? (
                        localProject.apps
                          .map((app) => ({
                            app,
                            includedSeconds: stageIncludedAppSeconds(localProject, selectedStage, app),
                            actualSeconds: appActualSeconds(app),
                            includedPercent: percentOf(stageTotals.total, stageIncludedAppSeconds(localProject, selectedStage, app)),
                            actualPercent: percentOf(stageTotals.total, appActualSeconds(app)),
                          }))
                          .sort((left, right) => right.includedSeconds - left.includedSeconds || right.actualSeconds - left.actualSeconds || left.app.name.localeCompare(right.app.name))
                          .map(({ app, includedSeconds, actualSeconds, includedPercent, actualPercent }) => {
                            const stageAppEnabled = stageAppIsEnabled(localProject, selectedStage, app);
                            const isOpen = expandedApps[app.key];

                            return (
                              <div key={app.key} className="grid gap-2">
                                <div
                                  className={`grid grid-cols-[36px_minmax(120px,1fr)_92px_56px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_120px_72px] sm:gap-3 sm:px-4 ${
                                    stageAppEnabled ? "border-slate-200 bg-slate-50" : "border-slate-200 bg-slate-100/80 text-slate-500"
                                  }`}
                                >
                                  <Checkbox
                                    checked={stageAppEnabled}
                                    onChange={(checked) => toggleStageApp(app.key, checked)}
                                  />

                                  <button
                                    className="flex min-w-0 items-center gap-3 text-left"
                                    onClick={() => setExpandedApps((prev) => ({ ...prev, [app.key]: !prev[app.key] }))}
                                  >
                                    <AppIcon app={app} />
                                    <span className="min-w-0">
                                      <strong className={`block truncate text-sm ${stageAppEnabled ? "text-slate-900" : "text-slate-500"}`}>{app.name}</strong>
                                      <small className={`block truncate text-xs ${stageAppEnabled ? "text-slate-500" : "text-slate-400"}`}>
                                        {app.kind === "browser" ? t.browserLabel : app.process_path || app.process_name}
                                      </small>
                                    </span>
                                    {app.kind === "browser" ? (
                                      <span className="ml-auto text-slate-400">{isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />}</span>
                                    ) : null}
                                  </button>

                                  <span className="font-mono text-sm text-slate-700">
                                    {stageAppEnabled ? formatDuration(includedSeconds, language) : "00:00:00"}
                                    {!stageAppEnabled ? <small className="mt-1 block text-xs text-slate-400">{formatDuration(actualSeconds, language)}</small> : null}
                                  </span>
                                  <span className="text-sm text-slate-500">
                                    {stageAppEnabled ? includedPercent : "0%"}
                                    {!stageAppEnabled ? <small className="mt-1 block text-xs text-slate-400">{actualPercent}</small> : null}
                                  </span>
                                </div>

                                {app.kind === "browser" && isOpen ? (
                                  <div className="ml-4 grid gap-2 sm:ml-10">
                                    {app.tabs
                                      .map((tab) => ({
                                        tab,
                                        includedSeconds: stageIncludedTabSeconds(localProject, selectedStage, app, tab),
                                        actualSeconds: tabActualSeconds(tab),
                                        includedPercent: percentOf(stageTotals.total, stageIncludedTabSeconds(localProject, selectedStage, app, tab)),
                                        actualPercent: percentOf(stageTotals.total, tabActualSeconds(tab)),
                                      }))
                                      .sort((left, right) => right.includedSeconds - left.includedSeconds || right.actualSeconds - left.actualSeconds || left.tab.title.localeCompare(right.tab.title))
                                      .map(({ tab, includedSeconds: tabIncluded, actualSeconds: tabActual, includedPercent: tabIncludedPercent, actualPercent: tabActualPercent }) => {
                                        const stageTabEnabled = stageTabIsEnabled(localProject, selectedStage, app, tab);
                                        const urls = tabUrlList(tab);
                                        const menuKey = `${app.key}:${tab.key}`;
                                        const isMenuOpen = linkMenu === menuKey;

                                        return (
                                          <div
                                            key={tab.key}
                                            className={`grid grid-cols-[36px_minmax(120px,1fr)_92px_56px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_120px_72px] sm:gap-3 sm:px-4 ${
                                              stageTabEnabled ? "border-slate-200 bg-white" : "border-slate-200 bg-slate-50 text-slate-500"
                                            }`}
                                          >
                                            <Checkbox
                                              checked={stageTabEnabled}
                                              onChange={(checked) => toggleStageTab(app.key, tab.key, checked)}
                                            />
                                            <span className="flex min-w-0 items-center gap-3">
                                              <TabIcon tab={tab} />
                                              <span className="relative min-w-0">
                                                <span className="flex max-w-full items-center gap-2">
                                                  <strong className={`block truncate text-sm ${stageTabEnabled ? "text-slate-900" : "text-slate-500"}`}>
                                                    {tab.title}
                                                  </strong>
                                                  {urls.length ? (
                                                    <button
                                                      className="inline-flex size-7 shrink-0 items-center justify-center rounded-full border border-emerald-100 bg-emerald-50 text-emerald-600 transition-colors hover:border-emerald-200 hover:bg-emerald-100"
                                                      onClick={(event) => {
                                                        event.stopPropagation();
                                                        setLinkMenu(isMenuOpen ? null : menuKey);
                                                      }}
                                                      title={t.openUrlLabel}
                                                    >
                                                      <Link size={13} />
                                                    </button>
                                                  ) : null}
                                                </span>
                                                <small className={`block truncate text-xs ${stageTabEnabled ? "text-slate-500" : "text-slate-400"}`}>
                                                  {urls.length ? t.linksInDomain(urls.length) : t.urlUnavailable}
                                                </small>
                                                {isMenuOpen ? (
                                                  <div
                                                    className="absolute left-0 top-12 z-20 w-[min(460px,70vw)] rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
                                                    onClick={(event) => event.stopPropagation()}
                                                  >
                                                    <div className="max-h-60 overflow-y-auto pr-1">
                                                      {urls.map((item) => (
                                                        <div
                                                          key={item.url}
                                                          className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-xl px-3 py-2 text-left transition-colors hover:bg-emerald-50"
                                                        >
                                                          <Checkbox
                                                            checked={stageUrlIsEnabled(localProject, selectedStage, app, tab, item)}
                                                            onChange={(checked) => toggleStageUrl(app.key, tab.key, item.url, checked)}
                                                          />
                                                          <button
                                                            className="min-w-0 text-left"
                                                            onClick={() => {
                                                              setLinkMenu(null);
                                                              openStageUrl(item.url);
                                                            }}
                                                          >
                                                            <strong className="block truncate text-xs text-slate-900">{item.title || item.url}</strong>
                                                            <span className="block truncate text-xs text-slate-500">{item.url}</span>
                                                          </button>
                                                          <span className="font-mono text-xs text-slate-700">{formatDuration(item.time_seconds, language)}</span>
                                                        </div>
                                                      ))}
                                                    </div>
                                                  </div>
                                                ) : null}
                                              </span>
                                            </span>
                                            <span className="font-mono text-sm text-slate-700">
                                              {stageTabEnabled ? formatDuration(tabIncluded, language) : "00:00:00"}
                                              {!stageTabEnabled ? <small className="mt-1 block text-xs text-slate-400">{formatDuration(tabActual, language)}</small> : null}
                                            </span>
                                            <span className="text-sm text-slate-500">
                                              {stageTabEnabled ? tabIncludedPercent : "0%"}
                                              {!stageTabEnabled ? <small className="mt-1 block text-xs text-slate-400">{tabActualPercent}</small> : null}
                                            </span>
                                          </div>
                                        );
                                      })}
                                  </div>
                                ) : null}
                              </div>
                            );
                          })
                      ) : (
                        <EmptyState text={t.stageEmpty} />
                      )}
                    </div>
                  </div>
                </div>
              )}
            </section>
          </section>
        </div>
      </section>
    </div>
  );
}

function stageAppIsEnabled(project: ProjectRecord, stage: ProjectStageRecord, app: AppUsageRecord) {
  if (!app.enabled) return false;
  const stageApp = stage.apps.find((item) => item.app_key === app.key);
  return stageApp ? stageApp.enabled && app.enabled : app.enabled;
}

function stageTabIsEnabled(project: ProjectRecord, stage: ProjectStageRecord, app: AppUsageRecord, tab: TabUsageRecord) {
  if (!stageAppIsEnabled(project, stage, app) || !tab.enabled) return false;
  const stageApp = stage.apps.find((item) => item.app_key === app.key);
  const stageTab = stageApp?.tabs.find((item) => item.tab_key === tab.key);
  return stageTab ? stageTab.enabled && tab.enabled : tab.enabled;
}

function stageUrlIsEnabled(
  project: ProjectRecord,
  stage: ProjectStageRecord,
  app: AppUsageRecord,
  tab: TabUsageRecord,
  url: VisitedUrlRecord,
) {
  if (!stageTabIsEnabled(project, stage, app, tab) || !url.enabled) return false;
  const stageApp = stage.apps.find((item) => item.app_key === app.key);
  const stageTab = stageApp?.tabs.find((item) => item.tab_key === tab.key);
  const stageUrl = stageTab?.urls.find((item) => item.url === url.url);
  return stageUrl ? stageUrl.enabled && url.enabled : url.enabled;
}

function stageIncludedTabSeconds(project: ProjectRecord, stage: ProjectStageRecord, app: AppUsageRecord, tab: TabUsageRecord) {
  if (!stageTabIsEnabled(project, stage, app, tab)) return 0;
  if (tab.urls?.length) {
    return tab.urls.reduce((sum, item) => sum + (stageUrlIsEnabled(project, stage, app, tab, item) ? item.time_seconds : 0), 0);
  }
  return tab.time_seconds;
}

function stageIncludedAppSeconds(project: ProjectRecord, stage: ProjectStageRecord, app: AppUsageRecord) {
  if (!stageAppIsEnabled(project, stage, app)) return 0;
  if (app.kind === "browser") {
    return app.tabs.reduce((sum, tab) => sum + stageIncludedTabSeconds(project, stage, app, tab), 0);
  }
  return app.time_seconds;
}
