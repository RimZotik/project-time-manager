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
  Download,
  FileText,
  FolderOpen,
  FolderPlus,
  Import,
  Link,
  Languages,
  PencilLine,
  Plus,
  RefreshCw,
  Settings,
  Square,
  Tags,
  TimerReset,
  Trash2,
  X,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";

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

type Category = {
  id: string;
  name: string;
  color: string;
  icon: string;
  order: number;
  created_at: string;
  updated_at: string;
};

type ProjectSummary = {
  id: string;
  name: string;
  client: string;
  updated_at: string;
  category_id?: string | null;
  color?: string | null;
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
  stages: SessionStageSnapshot[];
};

type SessionStageSnapshot = {
  id: string;
  name: string;
};

type ProjectRecord = ProjectSummary & {
  note: string;
  created_at: string;
  sessions: SessionRecord[];
  apps: AppUsageRecord[];
  selected_stage_ids: string[];
  stages: ProjectStageRecord[];
  category_id?: string | null;
  color?: string | null;
};

type AppState = {
  tracker: TrackerPayload;
  settings: AppSettings;
  projects: ProjectSummary[];
  selected_project: ProjectRecord | null;
  categories: Category[];
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
  categories: [],
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
    categoryLabel: "Категория",
    noCategory: "Без категории",
    manageCategoriesLabel: "Категории",
    categoriesTitle: "Категории проектов",
    categoriesDescription:
      "Создавайте категории (например, Монтаж, Программирование) и присваивайте их проектам, чтобы сравнивать аналитику.",
    categoryNamePlaceholder: "Название категории",
    addCategoryLabel: "Добавить",
    categoryEmpty: "Категорий пока нет.",
    deleteCategoryConfirm:
      "Удалить категорию? Проекты этой категории останутся без категории.",
    totalProjectLabel: "Всего по проекту",
    sessionsCountLabel: "Сеансов",
    topAppLabel: "Топ приложение",
    topTabLabel: "Топ сайт",
    appsTitle: "Приложения и вкладки",
    applicationHeader: "Приложение",
    timeHeader: "Время",
    percentHeader: "%",
    browserLabel: "Браузер",
    noProjects: "Пока нет проектов.",
    noSessions: "Сеансы появятся после старта записи.",
    emptyPrompt: "Выбери или создай проект, чтобы начать запись.",
    currentProjectHint:
      "Выбери проект, запусти запись и отслеживай активные окна.",
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
    helpDescription:
      "Основные действия для работы с проектами, записью времени и отчетами.",
    helpProjectsTitle: "Проекты",
    helpProjectsText:
      "Создай проект в левом столбце или выбери существующий. Все записи и отчеты хранятся в папке проекта.",
    helpTrackingTitle: "Запись времени",
    helpTrackingText:
      "Нажми Старт, работай как обычно и затем поставь запись на паузу или останови ее. Во время записи импорт и экспорт отключены.",
    helpAppsTitle: "Приложения",
    helpAppsText:
      "В таблице видно, что реально использовалось в проекте. Если снять галочку у приложения, сайта или ссылки, они исчезнут из итогового времени и отчетов.",
    helpSitesTitle: "Сайты",
    helpSitesText:
      "У каждого браузера показываются сайты, а внутри сайта доступны его ссылки. Нажми на значок цепочки, чтобы открыть список ссылок и перейти по нужной.",
    helpReportsTitle: "Отчеты",
    helpReportsText:
      "Excel и PDF собираются только из включенных приложений, сайтов, ссылок и реально использованных этапов. Если файл уже открыт, новый отчет сохранится с временной меткой.",
    projectMenuRename: "Переименовать",
    projectMenuDelete: "Удалить",
    projectMenuStages: "Этапы",
    stageTitle: "Этапы проекта",
    stageDescription:
      "Создавай этапы проекта и выбирай их на главной панели до старта записи. В отчет попадают только реально использованные этапы.",
    stageCreateLabel: "Создать этап",
    stageRenameLabel: "Переименовать",
    stageDeleteLabel: "Удалить",
    stageUpLabel: "Выше",
    stageDownLabel: "Ниже",
    stageEmpty: "Этапов пока нет.",
    stageNamePlaceholder: "Название этапа",
    stageCreateHint:
      "Серые этапы не учитываются. Выбранные этапы фиксируются на всю сессию от старта до стопа.",
    stageMetricsTime: "Выбрано",
    stageMetricsShare: "Всего этапов",
    stageMetricsApps: "Порядок",
    stageMetricsTabs: "Статус",
    stageDeleteConfirm: "Удалить этап полностью? Это действие нельзя отменить.",
    stageSelectionHint:
      "Нажми на этап, чтобы включить его в следующую сессию. Можно выбрать несколько.",
    stageLockedHint: "Сначала остановите сессию, потом меняйте этапы.",
    stageManageLabel: "Управление этапами",
    stageUnusedLabel: "Не выбран",
    stageSelectedLabel: "Выбрано",
    stageMetricsCount: "Количество",
    stageMetricsSelectedCount: "Выбрано",
    stageMetricsAll: "Все",
    stageProjectDisabledHint:
      "Сначала включите этот элемент в проекте, потом его можно включать в этапе.",
    reportTimeLabel: "В отчетах",
    actualTimeLabel: "Фактически",
    renameTitle: "Переименование проекта",
    renameDescription: "Введите новое название проекта.",
    renamePlaceholder: "Новое название",
    saveLabel: "Сохранить",
    cancelLabel: "Отмена",
    deleteConfirm: "Удалить проект полностью? Это действие нельзя отменить.",
    openReportLabel: "Открыть файл",
    openUrlLabel: "Открыть ссылку",
    urlUnavailable: "URL недоступен",
    linksInDomain: (count: number) => `${count} ссылок у этого сайта`,
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
    categoryLabel: "Category",
    noCategory: "No category",
    manageCategoriesLabel: "Categories",
    categoriesTitle: "Project categories",
    categoriesDescription:
      "Create categories (e.g. Editing, Programming) and assign them to projects to compare analytics.",
    categoryNamePlaceholder: "Category name",
    addCategoryLabel: "Add",
    categoryEmpty: "No categories yet.",
    deleteCategoryConfirm:
      "Delete this category? Its projects will simply have no category.",
    totalProjectLabel: "Total project",
    sessionsCountLabel: "Sessions",
    topAppLabel: "Top app",
    topTabLabel: "Top site",
    appsTitle: "Applications and tabs",
    applicationHeader: "Application",
    timeHeader: "Time",
    percentHeader: "%",
    browserLabel: "Browser",
    noProjects: "No projects yet.",
    noSessions: "Sessions will appear after recording starts.",
    emptyPrompt: "Choose or create a project to start recording.",
    currentProjectHint:
      "Choose a project, start recording, and track active windows.",
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
      "The table shows what was actually used in the project. If you clear a checkbox for an app, site, or link, it no longer counts in totals or reports.",
    helpSitesTitle: "Sites",
    helpSitesText:
      "Each browser expands into sites, and each site contains its links. Click the chain icon to open the captured links and jump to the one you need.",
    helpReportsTitle: "Reports",
    helpReportsText:
      "Excel and PDF are built only from enabled apps, sites, links, and stages that were actually used. If a file is already open, the new report is saved with a timestamp.",
    projectMenuRename: "Rename",
    projectMenuDelete: "Delete",
    projectMenuStages: "Stages",
    stageTitle: "Project stages",
    stageDescription:
      "Create project stages and choose them on the main screen before recording starts. Reports include only stages that were actually used.",
    stageCreateLabel: "Create stage",
    stageRenameLabel: "Rename",
    stageDeleteLabel: "Delete",
    stageUpLabel: "Up",
    stageDownLabel: "Down",
    stageEmpty: "No stages yet.",
    stageNamePlaceholder: "Stage name",
    stageCreateHint:
      "Gray stages are ignored. Selected stages are fixed for the whole session from start to stop.",
    stageMetricsTime: "Selected",
    stageMetricsShare: "Total stages",
    stageMetricsApps: "Order",
    stageMetricsTabs: "Status",
    stageDeleteConfirm: "Delete this stage completely? This cannot be undone.",
    stageSelectionHint:
      "Click a stage to include it in the next session. Multiple stages are allowed.",
    stageLockedHint: "Stop the session first, then change stages.",
    stageManageLabel: "Manage stages",
    stageUnusedLabel: "Not selected",
    stageSelectedLabel: "Selected",
    stageMetricsCount: "Count",
    stageMetricsSelectedCount: "Selected",
    stageMetricsAll: "All",
    stageProjectDisabledHint:
      "Enable this item in the project first, then it can be enabled in the stage.",
    reportTimeLabel: "In reports",
    actualTimeLabel: "Actual",
    renameTitle: "Rename project",
    renameDescription: "Enter the new project name.",
    renamePlaceholder: "New name",
    saveLabel: "Save",
    cancelLabel: "Cancel",
    deleteConfirm: "Delete this project completely? This cannot be undone.",
    openReportLabel: "Open file",
    openUrlLabel: "Open link",
    urlUnavailable: "URL unavailable",
    linksInDomain: (count: number) => `${count} links for this site`,
    dataFolderLabel: "Open app folder",
    languageRu: "Russian",
    languageEn: "English",
  },
} as const;

type Copy = {
  [K in keyof (typeof copy)["ru"]]: (typeof copy)["ru"][K] extends (
    ...args: infer Args
  ) => infer Return
    ? (...args: Args) => Return
    : string;
};

async function invokeCommand<T>(
  name: string,
  args: Record<string, unknown> = {},
  fallback: T,
): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    console.error(name, error);
    return fallback;
  }
}

function formatDuration(
  seconds: number,
  language: AppSettings["language"] = "ru",
): string {
  const value = Number(seconds || 0);
  const days = Math.floor(value / 86400);
  const hours = Math.floor((value % 86400) / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  const secs = value % 60;
  const clock = `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
  return days > 0 ? `${days} ${language === "en" ? "d" : "д"} ${clock}` : clock;
}

function formatDurationInput(
  seconds: number,
  language: AppSettings["language"] = "ru",
): string {
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
  if (
    parts.length < 2 ||
    parts.length > 3 ||
    parts.some((part) => part === "" || Number.isNaN(Number(part)))
  ) {
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

function formatDateTime(
  value?: string | null,
  language: AppSettings["language"] = "ru",
): string {
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

function sortRankedApps(
  apps: AppUsageRecord[],
  totalSeconds: number,
): RankedApp[] {
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
    .sort(
      (left, right) =>
        right.includedSeconds - left.includedSeconds ||
        right.actualSeconds - left.actualSeconds ||
        left.app.name.localeCompare(right.app.name),
    )
    .map((item) => item);
}

function sortRankedTabs(
  app: AppUsageRecord,
  totalSeconds: number,
): RankedTab[] {
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
    .sort(
      (left, right) =>
        right.includedSeconds - left.includedSeconds ||
        right.actualSeconds - left.actualSeconds ||
        left.tab.title.localeCompare(right.tab.title),
    )
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
  return history.sort(
    (left, right) =>
      right.time_seconds - left.time_seconds ||
      right.hits - left.hits ||
      right.last_seen_at.localeCompare(left.last_seen_at),
  );
}

function stageAppRecord(stage: ProjectStageRecord, appKey: string) {
  return stage.apps.find((item) => item.app_key === appKey) ?? null;
}

function stageTabRecord(
  stage: ProjectStageRecord,
  appKey: string,
  tabKey: string,
) {
  return (
    stageAppRecord(stage, appKey)?.tabs.find(
      (item) => item.tab_key === tabKey,
    ) ?? null
  );
}

function stageUrlRecord(
  stage: ProjectStageRecord,
  appKey: string,
  tabKey: string,
  url: string,
) {
  return (
    stageTabRecord(stage, appKey, tabKey)?.urls.find(
      (item) => item.url === url,
    ) ?? null
  );
}

function stageAppEnabled(
  stage: ProjectStageRecord,
  app: AppUsageRecord,
): boolean {
  if (!app.enabled) return false;
  return stageAppRecord(stage, app.key)?.enabled ?? true;
}

function stageTabEnabled(
  stage: ProjectStageRecord,
  app: AppUsageRecord,
  tab: TabUsageRecord,
): boolean {
  if (!stageAppEnabled(stage, app) || !tab.enabled) return false;
  return stageTabRecord(stage, app.key, tab.key)?.enabled ?? true;
}

function stageLinkEnabled(
  stage: ProjectStageRecord,
  app: AppUsageRecord,
  tab: TabUsageRecord,
  link: VisitedUrlRecord,
): boolean {
  if (!stageTabEnabled(stage, app, tab) || !link.enabled) return false;
  return stageUrlRecord(stage, app.key, tab.key, link.url)?.enabled ?? true;
}

function stageTabIncludedSeconds(
  stage: ProjectStageRecord,
  app: AppUsageRecord,
  tab: TabUsageRecord,
): number {
  if (!stageTabEnabled(stage, app, tab)) return 0;
  if (tab.urls?.length) {
    return tab.urls.reduce(
      (sum, link) =>
        sum +
        (stageLinkEnabled(stage, app, tab, link) ? actualLinkSeconds(link) : 0),
      0,
    );
  }
  return tab.time_seconds;
}

function stageAppIncludedSeconds(
  stage: ProjectStageRecord,
  app: AppUsageRecord,
): number {
  if (app.kind === "browser") {
    return app.tabs.reduce(
      (sum, tab) => sum + stageTabIncludedSeconds(stage, app, tab),
      0,
    );
  }
  return stageAppEnabled(stage, app) ? app.time_seconds : 0;
}

function sortRankedStageApps(
  stage: ProjectStageRecord,
  apps: AppUsageRecord[],
  totalSeconds: number,
): RankedApp[] {
  return [...apps]
    .map((app) => {
      const includedSeconds = stageAppIncludedSeconds(stage, app);
      const actualSeconds = appActualSeconds(app);
      return {
        app,
        includedSeconds,
        actualSeconds,
        includedPercent: percentOf(totalSeconds, includedSeconds),
        actualPercent: percentOf(totalSeconds, actualSeconds),
      };
    })
    .sort(
      (left, right) =>
        right.includedSeconds - left.includedSeconds ||
        right.actualSeconds - left.actualSeconds ||
        left.app.name.localeCompare(right.app.name),
    );
}

function sortRankedStageTabs(
  stage: ProjectStageRecord,
  app: AppUsageRecord,
  totalSeconds: number,
): RankedTab[] {
  return [...app.tabs]
    .map((tab) => {
      const includedSeconds = stageTabIncludedSeconds(stage, app, tab);
      const actualSeconds = tabActualSeconds(tab);
      return {
        tab,
        includedSeconds,
        actualSeconds,
        includedPercent: percentOf(totalSeconds, includedSeconds),
        actualPercent: percentOf(totalSeconds, actualSeconds),
      };
    })
    .sort(
      (left, right) =>
        right.includedSeconds - left.includedSeconds ||
        right.actualSeconds - left.actualSeconds ||
        left.tab.title.localeCompare(right.tab.title),
    );
}

function stageEntityTotals(stage: ProjectStageRecord, apps: AppUsageRecord[]) {
  let enabled = 0;
  let total = 0;

  for (const app of apps) {
    total += 1;
    if (stageAppEnabled(stage, app)) {
      enabled += 1;
    }

    for (const tab of app.tabs) {
      total += 1;
      if (stageTabEnabled(stage, app, tab)) {
        enabled += 1;
      }

      for (const link of tabUrlList(tab)) {
        total += 1;
        if (stageLinkEnabled(stage, app, tab, link)) {
          enabled += 1;
        }
      }
    }
  }

  return { enabled, total };
}

export default function ProjectWorkspace() {
  const [state, setState] = useState<AppState>(fallbackState);
  const [expandedApps, setExpandedApps] = useState<Record<string, boolean>>({});
  const [linkMenu, setLinkMenu] = useState<string | null>(null);
  const [projectMenu, setProjectMenu] = useState<ProjectMenuState | null>(null);
  const [renameProject, setRenameProject] = useState<RenameState | null>(null);
  const [categoryManagerOpen, setCategoryManagerOpen] = useState(false);
  const [categoryFilter, setCategoryFilter] = useState<string | null>(null);
  const [modal, setModal] = useState<"settings" | "help" | "stages" | null>(
    null,
  );
  const [stageModal, setStageModal] = useState<StageModalState | null>(null);
  const [newProjectName, setNewProjectName] = useState("");
  const [toast, setToast] = useState<ToastState | null>(null);
  const [timeDrafts, setTimeDrafts] = useState<Record<string, string>>({});
  const [editingTimeKey, setEditingTimeKey] = useState<string | null>(null);
  const importInputRef = useRef<HTMLInputElement>(null);
  const toastTimerRef = useRef<number | null>(null);
  const settingsSaveSeqRef = useRef(0);
  const settingsSavePendingRef = useRef(0);

  async function refresh() {
    const next = await invokeCommand<AppState>(
      "get_app_state",
      {},
      fallbackState,
    );
    setState((current) => ({
      ...next,
      settings:
        settingsSavePendingRef.current > 0 ? current.settings : next.settings,
    }));
  }

  useEffect(() => {
    refresh();
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
  const filteredProjects = categoryFilter
    ? state.projects.filter((project) => project.category_id === categoryFilter)
    : state.projects;
  const trackerStatus = state.tracker.status;
  const isRunning = trackerStatus === "running";
  const isPaused = trackerStatus === "paused";
  const isRecordingLocked = isRunning || isPaused;
  const isStageEditingLocked = isRunning || isPaused;
  const statusLabel =
    trackerStatus === "running"
      ? t.statusRunning
      : trackerStatus === "paused"
        ? t.statusPaused
        : t.statusStopped;

  const totals = useMemo(() => {
    const appTime = apps.reduce((sum, app) => sum + appIncludedSeconds(app), 0);
    const topApp = [...apps].sort(
      (a, b) => appIncludedSeconds(b) - appIncludedSeconds(a),
    )[0];
    const topTab = apps
      .flatMap((app) =>
        app.tabs
          .filter((tab) => app.enabled && tab.enabled)
          .map((tab) => ({
            ...tab,
            browser: app.name,
            included: tabIncludedSeconds(app, tab),
          })),
      )
      .sort(
        (a, b) =>
          b.included - a.included || tabActualSeconds(b) - tabActualSeconds(a),
      )[0];

    return { appTime, topApp, topTab };
  }, [apps]);

  const rankedApps = useMemo(
    () => sortRankedApps(apps, totals.appTime),
    [apps, totals.appTime],
  );

  useEffect(() => {
    setTimeDrafts({});
    setEditingTimeKey(null);
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

  async function assignCategory(projectId: string, categoryId: string | null) {
    await invokeCommand<ProjectRecord | null>(
      "set_project_category",
      { projectId, categoryId },
      null,
    );
    refresh();
  }

  async function createCategory(name: string, color: string, icon: string) {
    await invokeCommand<Category | null>(
      "create_category",
      { name, color, icon },
      null,
    );
    refresh();
  }

  async function updateCategory(
    id: string,
    name: string,
    color: string,
    icon: string,
  ) {
    await invokeCommand<void>(
      "update_category",
      { id, name, color, icon },
      undefined,
    );
    refresh();
  }

  async function deleteCategory(id: string) {
    await invokeCommand<void>("delete_category", { id }, undefined);
    refresh();
  }

  async function toggleTracking(
    command: "start_tracking" | "pause_tracking" | "stop_tracking",
  ) {
    await invokeCommand<void>(command, {}, undefined);
    refresh();
  }

  async function setSelectedStages(stageIds: string[]) {
    if (!selectedProject) return;
    const updated = await invokeCommand<ProjectRecord | null>(
      "set_selected_project_stages",
      { projectId: selectedProject.id, stageIds },
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

  async function toggleApp(
    projectId: string,
    appKey: string,
    enabled: boolean,
  ) {
    const updated = await invokeCommand<ProjectRecord | null>(
      "toggle_app_included",
      { projectId, appKey, enabled },
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

  async function toggleTab(
    projectId: string,
    appKey: string,
    tabKey: string,
    enabled: boolean,
  ) {
    const updated = await invokeCommand<ProjectRecord | null>(
      "toggle_tab_included",
      { projectId, appKey, tabKey, enabled },
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

  async function toggleUrl(
    projectId: string,
    appKey: string,
    tabKey: string,
    url: string,
    enabled: boolean,
  ) {
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

  async function commitAppTime(
    projectId: string,
    appKey: string,
    seconds: number,
  ) {
    const updated = await invokeCommand<ProjectRecord | null>(
      "set_app_time",
      { projectId, appKey, seconds },
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

  async function commitTabTime(
    projectId: string,
    appKey: string,
    tabKey: string,
    seconds: number,
  ) {
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

  function timeDraftValue(
    kind: "app" | "tab",
    valueSeconds: number,
    key: string,
    tabKey?: string,
  ) {
    return (
      timeDrafts[timeDraftKey(kind, key, tabKey)] ??
      formatDurationInput(valueSeconds, language)
    );
  }

  function setTimeDraft(
    kind: "app" | "tab",
    key: string,
    value: string,
    tabKey?: string,
  ) {
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

  function startTimeEditing(
    kind: "app" | "tab",
    key: string,
    valueSeconds: number,
    tabKey?: string,
  ) {
    const draftKey = timeDraftKey(kind, key, tabKey);
    setEditingTimeKey(draftKey);
    setTimeDraft(
      kind,
      key,
      formatDurationInput(valueSeconds, language),
      tabKey,
    );
  }

  function cancelTimeEditing(
    kind: "app" | "tab",
    key: string,
    tabKey?: string,
  ) {
    clearTimeDraft(kind, key, tabKey);
    setEditingTimeKey((current) =>
      current === timeDraftKey(kind, key, tabKey) ? null : current,
    );
  }

  async function submitTimeDraft(
    kind: "app" | "tab",
    key: string,
    rawValue: string,
    tabKey?: string,
  ) {
    const parsed = parseDurationInput(rawValue);
    if (parsed === null) {
      cancelTimeEditing(kind, key, tabKey);
      return;
    }
    if (!selectedProject) return;
    if (kind === "app") {
      await commitAppTime(selectedProject.id, key, parsed);
    } else if (tabKey) {
      await commitTabTime(selectedProject.id, key, tabKey, parsed);
    }
    clearTimeDraft(kind, key, tabKey);
    setEditingTimeKey(null);
  }


  async function handleImportFile(event: React.ChangeEvent<HTMLInputElement>) {
    if (isRecordingLocked) return;
    const file = event.target.files?.[0];
    if (!file) return;

    const jsonText = await file.text();
    const result = await invokeCommand<ProjectSummary | null>(
      "import_project_json",
      { jsonText },
      null,
    );
    setToast({
      text: result?.name
        ? `${t.importedProject}: ${result.name}`
        : t.importDone,
    });
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
      const updated = await invokeCommand<AppSettings>(
        "update_app_settings",
        next,
        next,
      );
      if (settingsSaveSeqRef.current === saveSeq) {
        setState((current) => ({
          ...current,
          settings: updated,
        }));
      }
    } finally {
      settingsSavePendingRef.current = Math.max(
        0,
        settingsSavePendingRef.current - 1,
      );
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
          project.id === updated.id
            ? { ...project, name: updated.name, updated_at: updated.updated_at }
            : project,
        ),
        selected_project:
          current.selected_project?.id === updated.id
            ? updated
            : current.selected_project,
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
      className="flex h-full min-h-0 flex-col overflow-hidden text-slate-900"
      onContextMenu={(event) => event.preventDefault()}
      onClick={() => {
        setLinkMenu(null);
        setProjectMenu(null);
      }}
    >
      <div className="flex h-full w-full min-w-0 flex-col gap-4 overflow-hidden px-5 py-5">
        <main className="grid min-h-0 flex-1 grid-cols-[340px_minmax(0,1fr)] gap-4 overflow-hidden">
          <aside className="flex min-h-0 flex-col gap-4">
            <section className="flex min-h-0 flex-[1.08] flex-col rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="flex items-center justify-between gap-3">
                <h2 className="text-base font-semibold text-slate-900">
                  {t.projectLabel}
                </h2>
                <button
                  className="icon-button"
                  onClick={refresh}
                  title={t.projectLabel}
                >
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
                <div className="grid grid-cols-2 gap-2">
                  <button
                    className="secondary-button min-h-10 px-3"
                    onClick={() => importInputRef.current?.click()}
                    disabled={isStageEditingLocked}
                  >
                    <Import size={16} />
                    {t.importJsonLabel}
                  </button>
                  <button
                    className="secondary-button min-h-10 px-3"
                    onClick={() => setCategoryManagerOpen(true)}
                  >
                    <Tags size={16} />
                    {t.manageCategoriesLabel}
                  </button>
                </div>
              </div>

              {state.categories.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-1.5">
                  <button
                    onClick={() => setCategoryFilter(null)}
                    className={`rounded-full px-3 py-1 text-xs font-medium transition-colors ${
                      categoryFilter === null
                        ? "bg-emerald-600 text-white"
                        : "bg-slate-100 text-slate-500 hover:bg-emerald-50 hover:text-emerald-700"
                    }`}
                  >
                    {language === "en" ? "All" : "Все"}
                  </button>
                  {state.categories.map((category) => (
                    <button
                      key={category.id}
                      onClick={() => setCategoryFilter(category.id)}
                      className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium transition-colors ${
                        categoryFilter === category.id
                          ? "bg-emerald-600 text-white"
                          : "bg-slate-100 text-slate-600 hover:bg-emerald-50 hover:text-emerald-700"
                      }`}
                    >
                      <span
                        className="size-2 rounded-full"
                        style={{ background: category.color }}
                      />
                      {category.name}
                    </button>
                  ))}
                </div>
              )}

              <div className="stable-scroll mt-3 min-h-0 flex-1 overflow-y-scroll pr-1">
                <div className="grid w-full gap-2">
                  {filteredProjects.length ? (
                    filteredProjects.map((project) => (
                      <div key={project.id}>
                        <div
                          className={`flex w-full items-center justify-between gap-2 rounded-2xl border px-3 py-3 text-left transition ${
                            project.id === selectedProject?.id
                              ? "border-emerald-300 bg-emerald-50"
                              : "border-slate-200 bg-white hover:border-emerald-200"
                          }`}
                        >
                          <button
                            className="flex min-w-0 flex-1 items-center gap-2.5 text-left"
                            onClick={() => selectProject(project.id)}
                          >
                            <span
                              className="size-2.5 shrink-0 rounded-full"
                              style={{
                                background:
                                  state.categories.find(
                                    (c) => c.id === project.category_id,
                                  )?.color ?? "#d1d5db",
                              }}
                              title={
                                state.categories.find(
                                  (c) => c.id === project.category_id,
                                )?.name ?? t.noCategory
                              }
                            />
                            <span className="min-w-0 flex-1">
                              <strong className="block truncate text-sm text-slate-900">
                                {project.name}
                              </strong>
                              <span className="block truncate text-xs text-slate-500">
                                {formatDateTime(project.updated_at, language)}
                              </span>
                            </span>
                          </button>
                          <button
                            className="flex size-8 shrink-0 items-center justify-center rounded-full text-slate-400 transition-colors hover:bg-white hover:text-emerald-700"
                            onClick={(event) => {
                              event.stopPropagation();
                              const rect =
                                event.currentTarget.getBoundingClientRect();
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
              <SectionTitle
                icon={<TimerReset size={16} />}
                title={t.sessionsLabel}
              />
              <div className="stable-scroll mt-3 min-h-0 flex-1 overflow-y-scroll pr-1">
                <div className="grid gap-2">
                  {sessions.length ? (
                    sessions.map((session) => (
                      <article
                        key={session.id}
                        className="rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3"
                      >
                        <strong className="block text-sm text-slate-900">
                          {formatDateTime(session.started_at, language)} —{" "}
                          {formatDateTime(session.stopped_at, language)}
                        </strong>
                        <span className="mt-1 block text-sm text-slate-500">
                          {formatDuration(session.duration_seconds, language)}
                        </span>
                      </article>
                    ))
                  ) : (
                    <EmptyState text={t.noSessions} />
                  )}
                </div>
              </div>
            </section>

          </aside>

          <section className="grid min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] gap-4 overflow-hidden">
            <section className="rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
              <div className="grid grid-cols-[minmax(420px,1fr)_auto_auto] items-center gap-4">
                <div className="space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <h1 className="text-xl font-semibold text-slate-900">
                      {selectedProject?.name ?? t.appName}
                    </h1>
                    <span className="inline-flex items-center gap-2 rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs font-semibold uppercase text-emerald-700">
                      <Activity size={13} />
                      {statusLabel}
                    </span>
                  </div>
                  <p className="whitespace-nowrap text-sm text-slate-500">
                    {t.currentProjectHint}
                  </p>
                </div>

                <div className="flex flex-nowrap gap-2">
                  <button
                    className="primary-button"
                    onClick={() => toggleTracking("start_tracking")}
                    disabled={isRunning || !selectedProject}
                  >
                    <CirclePlay size={16} />
                    {isPaused ? t.continueLabel : t.startLabel}
                  </button>
                  <button
                    className="secondary-button"
                    onClick={() => toggleTracking("pause_tracking")}
                    disabled={!isRunning || !selectedProject}
                  >
                    <CirclePause size={16} />
                    {t.pauseLabel}
                  </button>
                  <button
                    className="secondary-button"
                    onClick={() => toggleTracking("stop_tracking")}
                    disabled={(!isRunning && !isPaused) || !selectedProject}
                  >
                    <Square size={16} />
                    {t.stopLabel}
                  </button>
                </div>

                <div className="flex items-center justify-end">
                  {selectedProject && (
                    <CategoryPicker
                      categories={state.categories}
                      value={selectedProject.category_id ?? null}
                      language={language}
                      onChange={(id) => assignCategory(selectedProject.id, id)}
                    />
                  )}
                </div>
              </div>

              <input
                ref={importInputRef}
                type="file"
                accept=".json"
                hidden
                onChange={handleImportFile}
              />
            </section>

            {selectedProject ? (
              <>
                <section className="grid grid-cols-4 gap-3">
                  <Metric
                    label={t.totalProjectLabel}
                    value={formatDuration(totals.appTime, language)}
                    accent="emerald"
                  />
                  <Metric
                    label={t.sessionsCountLabel}
                    value={String(sessions.length)}
                    accent="emerald"
                  />
                  <StageSelectorCard
                    stages={[...(selectedProject.stages ?? [])].sort(
                      (left, right) => left.order - right.order,
                    )}
                    selectedStageIds={selectedProject.selected_stage_ids ?? []}
                    disabled={isStageEditingLocked}
                    t={t}
                    onToggle={(stageId) => {
                      const next = selectedProject.selected_stage_ids.includes(
                        stageId,
                      )
                        ? selectedProject.selected_stage_ids.filter(
                            (item) => item !== stageId,
                          )
                        : [...selectedProject.selected_stage_ids, stageId];
                      setSelectedStages(next);
                    }}
                    onManage={() =>
                      setStageModal({ projectId: selectedProject.id })
                    }
                  />
                </section>

                <section className="flex min-h-0 flex-col rounded-[24px] border border-emerald-100 bg-white p-4 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
                  <SectionTitle
                    icon={<Activity size={16} />}
                    title={t.appsTitle}
                  />

                  <div className="mt-4 grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)] gap-2 overflow-hidden">
                    <div className="grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] gap-2 px-3 text-xs font-medium uppercase tracking-[0.14em] text-slate-400 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4">
                      <span />
                      <span>{t.applicationHeader}</span>
                      <span>{t.timeHeader}</span>
                      <span>{t.percentHeader}</span>
                    </div>

                    <div className="stable-scroll min-h-0 overflow-y-scroll pr-1">
                      <div className="grid gap-2">
                        {rankedApps.map(
                          ({
                            app,
                            includedSeconds,
                            actualSeconds,
                            includedPercent,
                            actualPercent,
                          }) => {
                            const isOpen = expandedApps[app.key];
                            const isIncluded = app.enabled;
                            const appEditorKey = timeDraftKey("app", app.key);

                            return (
                              <div key={app.key} className="grid gap-2">
                                <div
                                  className={`grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4 ${
                                    isIncluded
                                      ? "border-slate-200 bg-slate-50"
                                      : "border-slate-200 bg-slate-100/80 text-slate-500"
                                  }`}
                                >
                                  <Checkbox
                                    checked={app.enabled}
                                    onChange={(checked) =>
                                      toggleApp(
                                        selectedProject.id,
                                        app.key,
                                        checked,
                                      )
                                    }
                                  />

                                  <button
                                    className="flex min-w-0 items-center gap-3 text-left"
                                    onClick={() =>
                                      setExpandedApps((prev) => ({
                                        ...prev,
                                        [app.key]: !prev[app.key],
                                      }))
                                    }
                                  >
                                    <AppIcon app={app} />
                                    <span className="min-w-0">
                                      <strong
                                        className={`block truncate text-sm ${isIncluded ? "text-slate-900" : "text-slate-500"}`}
                                      >
                                        {app.name}
                                      </strong>
                                      <small
                                        className={`block truncate text-xs ${isIncluded ? "text-slate-500" : "text-slate-400"}`}
                                      >
                                        {app.kind === "browser"
                                          ? t.browserLabel
                                          : app.process_path ||
                                            app.process_name}
                                      </small>
                                    </span>
                                    {app.kind === "browser" ? (
                                      <span className="ml-auto text-slate-400">
                                        {isOpen ? (
                                          <ChevronDown size={16} />
                                        ) : (
                                          <ChevronRight size={16} />
                                        )}
                                      </span>
                                    ) : null}
                                  </button>

                                  <InlineDurationEditor
                                    active={editingTimeKey === appEditorKey}
                                    value={formatDuration(
                                      actualSeconds,
                                      language,
                                    )}
                                    draft={timeDraftValue(
                                      "app",
                                      actualSeconds,
                                      app.key,
                                    )}
                                    secondary={
                                      isIncluded
                                        ? actualSeconds !== includedSeconds
                                          ? `${t.reportTimeLabel}: ${formatDuration(includedSeconds, language)}`
                                          : undefined
                                        : `${t.reportTimeLabel}: 00:00:00`
                                    }
                                    onActivate={() =>
                                      startTimeEditing(
                                        "app",
                                        app.key,
                                        actualSeconds,
                                      )
                                    }
                                    onChange={(value) =>
                                      setTimeDraft("app", app.key, value)
                                    }
                                    onSubmit={(value) =>
                                      submitTimeDraft("app", app.key, value)
                                    }
                                    onCancel={() =>
                                      cancelTimeEditing("app", app.key)
                                    }
                                  />
                                  <span className="text-sm text-slate-500">
                                    {isIncluded ? includedPercent : "0%"}
                                    {!isIncluded ? (
                                      <small className="mt-1 block text-xs text-slate-400">
                                        {actualPercent}
                                      </small>
                                    ) : null}
                                  </span>
                                </div>

                                {app.kind === "browser" && isOpen ? (
                                  <div className="ml-4 grid gap-2 sm:ml-10">
                                    {sortRankedTabs(app, totals.appTime).map(
                                      ({
                                        tab,
                                        includedSeconds: tabIncluded,
                                        actualSeconds: tabActual,
                                        includedPercent: tabIncludedPercent,
                                        actualPercent: tabActualPercent,
                                      }) => {
                                        const urls = tabUrlList(tab);
                                        const menuKey = `${app.key}:${tab.key}`;
                                        const isMenuOpen = linkMenu === menuKey;

                                        return (
                                          <div
                                            key={tab.key}
                                            className={`grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4 ${
                                              tab.enabled
                                                ? "border-slate-200 bg-white"
                                                : "border-slate-200 bg-slate-50 text-slate-500"
                                            }`}
                                          >
                                            <Checkbox
                                              checked={tab.enabled}
                                              onChange={(checked) =>
                                                toggleTab(
                                                  selectedProject.id,
                                                  app.key,
                                                  tab.key,
                                                  checked,
                                                )
                                              }
                                            />
                                            <span className="flex min-w-0 items-center gap-3">
                                              <TabIcon tab={tab} />
                                              <span className="relative min-w-0">
                                                <span className="flex max-w-full items-center gap-2">
                                                  <strong
                                                    className={`block truncate text-sm ${tab.enabled ? "text-slate-900" : "text-slate-500"}`}
                                                  >
                                                    {tab.title}
                                                  </strong>
                                                  {urls.length ? (
                                                    <button
                                                      className="inline-flex size-7 shrink-0 items-center justify-center rounded-full border border-emerald-100 bg-emerald-50 text-emerald-600 transition-colors hover:border-emerald-200 hover:bg-emerald-100"
                                                      onClick={(event) => {
                                                        event.stopPropagation();
                                                        setLinkMenu(
                                                          isMenuOpen
                                                            ? null
                                                            : menuKey,
                                                        );
                                                      }}
                                                      title={t.openUrlLabel}
                                                    >
                                                      <Link size={13} />
                                                    </button>
                                                  ) : null}
                                                </span>
                                                <small
                                                  className={`block truncate text-xs ${tab.enabled ? "text-slate-500" : "text-slate-400"}`}
                                                >
                                                  {urls.length
                                                    ? t.linksInDomain(
                                                        urls.length,
                                                      )
                                                    : t.urlUnavailable}
                                                </small>
                                                {isMenuOpen ? (
                                                  <div
                                                    className="absolute left-0 top-12 z-20 w-[min(460px,70vw)] rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
                                                    onClick={(event) =>
                                                      event.stopPropagation()
                                                    }
                                                  >
                                                    <div className="max-h-60 overflow-y-auto pr-1">
                                                      {urls.map((item) => (
                                                        <div
                                                          key={item.url}
                                                          className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-xl px-3 py-2 text-left transition-colors hover:bg-emerald-50"
                                                        >
                                                          <Checkbox
                                                            checked={
                                                              item.enabled
                                                            }
                                                            onChange={(
                                                              checked,
                                                            ) =>
                                                              toggleUrl(
                                                                selectedProject.id,
                                                                app.key,
                                                                tab.key,
                                                                item.url,
                                                                checked,
                                                              )
                                                            }
                                                          />
                                                          <button
                                                            className="min-w-0 text-left"
                                                            onClick={() => {
                                                              setLinkMenu(null);
                                                              openTabUrl(
                                                                item.url,
                                                              );
                                                            }}
                                                          >
                                                            <strong className="block truncate text-xs text-slate-900">
                                                              {item.title ||
                                                                item.url}
                                                            </strong>
                                                            <span className="block truncate text-xs text-slate-500">
                                                              {item.url}
                                                            </span>
                                                          </button>
                                                          <span className="font-mono text-xs text-slate-700">
                                                            {formatDuration(
                                                              item.time_seconds,
                                                              language,
                                                            )}
                                                          </span>
                                                        </div>
                                                      ))}
                                                    </div>
                                                  </div>
                                                ) : null}
                                              </span>
                                            </span>
                                            <InlineDurationEditor
                                              active={
                                                editingTimeKey ===
                                                timeDraftKey(
                                                  "tab",
                                                  app.key,
                                                  tab.key,
                                                )
                                              }
                                              value={formatDuration(
                                                tabActual,
                                                language,
                                              )}
                                              draft={timeDraftValue(
                                                "tab",
                                                tabActual,
                                                app.key,
                                                tab.key,
                                              )}
                                              secondary={
                                                tab.enabled
                                                  ? tabActual !== tabIncluded
                                                    ? `${t.reportTimeLabel}: ${formatDuration(tabIncluded, language)}`
                                                    : undefined
                                                  : `${t.reportTimeLabel}: 00:00:00`
                                              }
                                              onActivate={() =>
                                                startTimeEditing(
                                                  "tab",
                                                  app.key,
                                                  tabActual,
                                                  tab.key,
                                                )
                                              }
                                              onChange={(value) =>
                                                setTimeDraft(
                                                  "tab",
                                                  app.key,
                                                  value,
                                                  tab.key,
                                                )
                                              }
                                              onSubmit={(value) =>
                                                submitTimeDraft(
                                                  "tab",
                                                  app.key,
                                                  value,
                                                  tab.key,
                                                )
                                              }
                                              onCancel={() =>
                                                cancelTimeEditing(
                                                  "tab",
                                                  app.key,
                                                  tab.key,
                                                )
                                              }
                                            />
                                            <span className="text-sm text-slate-500">
                                              {tab.enabled
                                                ? tabIncludedPercent
                                                : "0%"}
                                              {!tab.enabled ? (
                                                <small className="mt-1 block text-xs text-slate-400">
                                                  {tabActualPercent}
                                                </small>
                                              ) : null}
                                            </span>
                                          </div>
                                        );
                                      },
                                    )}
                                  </div>
                                ) : null}
                              </div>
                            );
                          },
                        )}
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
      {categoryManagerOpen ? (
        <CategoryManagerModal
          categories={state.categories}
          language={language}
          onClose={() => setCategoryManagerOpen(false)}
          onCreate={createCategory}
          onUpdate={updateCategory}
          onDelete={deleteCategory}
        />
      ) : null}
      {projectMenu ? (
        <div
          className="fixed z-40 w-52 rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
          style={{ left: projectMenu.x, top: projectMenu.y }}
          onClick={(event) => event.stopPropagation()}
        >
          <button
            className="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm text-slate-700 transition-colors hover:bg-emerald-50 disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-transparent"
            onClick={async () => {
              if (isStageEditingLocked) return;
              setLinkMenu(null);
              if (selectedProject?.id !== projectMenu.projectId) {
                await selectProject(projectMenu.projectId);
              }
              setStageModal({ projectId: projectMenu.projectId });
              setProjectMenu(null);
            }}
            disabled={isStageEditingLocked}
            title={isStageEditingLocked ? t.stageLockedHint : undefined}
          >
            <Activity size={15} />
            {t.projectMenuStages}
          </button>
          <button
            className="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm text-slate-700 transition-colors hover:bg-emerald-50"
            onClick={() => {
              setRenameProject({
                projectId: projectMenu.projectId,
                value: projectMenu.projectName,
              });
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
        </div>
      ) : null}
      {renameProject ? (
        <RenameProjectModal
          state={renameProject}
          t={t}
          onChange={(value) =>
            setRenameProject((current) =>
              current ? { ...current, value } : current,
            )
          }
          onClose={() => setRenameProject(null)}
          onSubmit={submitRenameProject}
        />
      ) : null}
      {toast ? (
        <Toast
          toast={toast}
          label={t.openReportLabel}
          onOpenExport={openExportLocation}
        />
      ) : null}
      {stageModal ? (
        <ProjectStagesModal
          project={selectedProject}
          language={language}
          t={t}
          isLocked={isStageEditingLocked}
          onClose={() => setStageModal(null)}
          onSetProjectState={(updated) => {
            setState((current) => ({
              ...current,
              selected_project:
                current.selected_project?.id === updated.id
                  ? updated
                  : current.selected_project,
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

function Metric({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent: "emerald" | "slate";
}) {
  return (
    <article className="rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
      <span className="block text-xs font-semibold uppercase tracking-[0.18em] text-slate-400">
        {label}
      </span>
      <div
        className={`mt-3 h-1.5 w-12 rounded-full ${accent === "emerald" ? "bg-emerald-500" : "bg-slate-300"}`}
      />
      <strong className="mt-4 block truncate text-lg font-semibold text-slate-900">
        {value}
      </strong>
    </article>
  );
}

function StageSelectorCard({
  stages,
  selectedStageIds,
  disabled,
  t,
  onToggle,
  onManage,
}: {
  stages: ProjectStageRecord[];
  selectedStageIds: string[];
  disabled: boolean;
  t: Copy;
  onToggle: (stageId: string) => void;
  onManage: () => void;
}) {
  const tooltip = disabled ? t.stageLockedHint : undefined;

  return (
    <article className="col-span-2 rounded-[28px] border border-emerald-100 bg-white p-5 shadow-[0_10px_30px_rgba(15,23,42,0.05)]">
      <div className="flex items-start justify-between gap-3">
        <div>
          <span className="block text-xs font-semibold uppercase tracking-[0.18em] text-slate-400">
            {t.stagesLabel}
          </span>
          <strong className="mt-3 block text-base font-semibold text-slate-900">
            {selectedStageIds.length
              ? `${t.stageSelectedLabel}: ${selectedStageIds.length}`
              : t.stageUnusedLabel}
          </strong>
        </div>
        <div title={tooltip}>
          <button
            className="secondary-button min-h-10 px-3"
            onClick={() => !disabled && onManage()}
            disabled={disabled}
          >
            <PencilLine size={15} />
            {t.stageManageLabel}
          </button>
        </div>
      </div>

      <div className="stable-scroll mt-4 overflow-x-auto pb-1">
        <div className="flex min-w-max gap-2">
          {stages.length ? (
            stages.map((stage) => {
              const isSelected = selectedStageIds.includes(stage.id);
              return (
                <div key={stage.id} title={tooltip}>
                  <button
                    className={`rounded-2xl border px-4 py-2 text-sm font-medium transition ${
                      isSelected
                        ? "border-emerald-300 bg-emerald-100 text-emerald-900"
                        : "border-slate-200 bg-slate-100 text-slate-500"
                    } ${disabled ? "cursor-not-allowed opacity-70" : "hover:border-emerald-200 hover:text-slate-800"}`}
                    onClick={() => !disabled && onToggle(stage.id)}
                    disabled={disabled}
                  >
                    {stage.name}
                  </button>
                </div>
              );
            })
          ) : (
            <span className="text-sm text-slate-500">{t.stageEmpty}</span>
          )}
        </div>
      </div>

      <p className="mt-3 text-xs leading-5 text-slate-500">
        {disabled ? t.stageLockedHint : t.stageSelectionHint}
      </p>
    </article>
  );
}

function InlineDurationEditor({
  active,
  value,
  draft,
  secondary,
  onActivate,
  onChange,
  onSubmit,
  onCancel,
}: {
  active: boolean;
  value: string;
  draft: string;
  secondary?: string;
  onActivate: () => void;
  onChange: (value: string) => void;
  onSubmit: (value: string) => void;
  onCancel: () => void;
}) {
  if (active) {
    return (
      <span className="min-w-0">
        <input
          className="w-full min-w-0 rounded-xl border border-emerald-300 bg-white px-2 py-1 text-right font-mono text-sm text-slate-700 outline-none ring-2 ring-emerald-100"
          value={draft}
          autoFocus
          onChange={(event) => onChange(event.currentTarget.value)}
          onBlur={(event) => onSubmit(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.currentTarget.blur();
            }
            if (event.key === "Escape") {
              event.preventDefault();
              onCancel();
            }
          }}
          inputMode="text"
        />
        {secondary ? (
          <small className="mt-1 block text-right text-xs text-slate-400">
            {secondary}
          </small>
        ) : null}
      </span>
    );
  }

  return (
    <button className="min-w-0 text-right" onClick={onActivate}>
      <span className="block font-mono text-sm text-slate-700">{value}</span>
      {secondary ? (
        <small className="mt-1 block text-xs text-slate-400">{secondary}</small>
      ) : null}
    </button>
  );
}

function AppIcon({ app }: { app: AppUsageRecord }) {
  if (app.icon_data_url) {
    return (
      <span className="flex size-10 shrink-0 items-center justify-center rounded-2xl border border-white bg-white shadow-sm">
        <img
          className="size-7 object-contain"
          src={app.icon_data_url}
          alt=""
          draggable={false}
        />
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
      {tab.favicon_url ? (
        <img
          className="size-4 object-contain"
          src={tab.favicon_url}
          alt=""
          draggable={false}
        />
      ) : (
        "W"
      )}
    </span>
  );
}

function SectionTitle({
  icon,
  title,
}: {
  icon: React.ReactNode;
  title: string;
}) {
  return (
    <div className="flex items-center gap-2 text-slate-700">
      {icon}
      <h3 className="text-base font-semibold text-slate-900">{title}</h3>
    </div>
  );
}

function Checkbox({
  checked,
  onChange,
  disabled = false,
  title,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  title?: string;
}) {
  return (
    <label
      className={`relative flex size-8 items-center justify-center ${disabled ? "cursor-not-allowed" : "cursor-pointer"}`}
      title={title}
    >
      <input
        className="peer absolute inset-0 opacity-0"
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="flex size-5 items-center justify-center rounded-md border border-slate-300 bg-white text-transparent peer-checked:border-emerald-500 peer-checked:bg-emerald-500 peer-checked:text-white peer-disabled:border-slate-200 peer-disabled:bg-slate-100 peer-disabled:text-slate-300">
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

function Toast({
  toast,
  label,
  onOpenExport,
}: {
  toast: ToastState;
  label: string;
  onOpenExport: (path?: string) => void;
}) {
  return (
    <footer className="fixed bottom-5 right-5 flex max-w-[560px] items-center gap-3 rounded-2xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900 shadow-[0_10px_30px_rgba(15,23,42,0.1)]">
      <span className="min-w-0 truncate">{toast.text}</span>
      {toast.exportPath ? (
        <button
          className="icon-button bg-white"
          onClick={() => onOpenExport(toast.exportPath)}
          title={label}
        >
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
            <h2 className="mt-4 text-2xl font-semibold text-slate-900">
              {isHelp ? t.helpTitle : t.settingsTitle}
            </h2>
            <p className="mt-2 max-w-xl text-sm leading-6 text-slate-600">
              {isHelp ? t.helpDescription : t.settingsDescription}
            </p>
          </div>
          <button
            className="icon-button shrink-0"
            onClick={onClose}
            title={t.closeLabel}
          >
            <X size={18} />
          </button>
        </header>

        <div className="stable-scroll max-h-[58vh] overflow-y-auto px-7 py-6">
          {isHelp ? (
            <div className="grid gap-4">
              <HelpItem title={t.helpProjectsTitle} text={t.helpProjectsText} />
              <HelpItem title={t.helpTrackingTitle} text={t.helpTrackingText} />
              <HelpItem title={t.helpAppsTitle} text={t.helpAppsText} />
              <HelpItem title={t.helpSitesTitle} text={t.helpSitesText} />
              <HelpItem title={t.helpReportsTitle} text={t.helpReportsText} />
            </div>
          ) : (
            <div className="grid gap-4">
              <label className="flex items-center justify-between gap-4 rounded-3xl border border-emerald-100 bg-slate-50 px-5 py-4">
                <span>
                  <strong className="block text-sm text-slate-900">
                    {t.autostartLabel}
                  </strong>
                  <span className="mt-1 block text-xs text-slate-500">
                    Windows
                  </span>
                </span>
                <input
                  className="size-5 accent-emerald-600"
                  type="checkbox"
                  checked={settings.autostart}
                  onChange={(event) =>
                    onSettingsChange({
                      ...settings,
                      autostart: event.target.checked,
                    })
                  }
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
                    onChange={(event) =>
                      onSettingsChange({
                        ...settings,
                        language: event.target.value as AppSettings["language"],
                      })
                    }
                  >
                    <option value="ru">{t.languageRu}</option>
                    <option value="en">{t.languageEn}</option>
                  </select>
                  <ChevronDown
                    className="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-slate-400"
                    size={16}
                  />
                </span>
              </label>

              <button
                className="secondary-button w-fit"
                onClick={onOpenAppFolder}
              >
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
            <h2 className="text-xl font-semibold text-slate-900">
              {t.renameTitle}
            </h2>
            <p className="mt-2 text-sm text-slate-500">{t.renameDescription}</p>
          </div>
          <button
            className="icon-button shrink-0"
            onClick={onClose}
            title={t.closeLabel}
          >
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

const CATEGORY_PALETTE = [
  "#059669",
  "#2563eb",
  "#7c3aed",
  "#db2777",
  "#ea580c",
  "#0891b2",
  "#65a30d",
  "#dc2626",
];

function CategoryRow({
  category,
  language,
  onUpdate,
  onDelete,
}: {
  category: Category;
  language: "ru" | "en";
  onUpdate: (id: string, name: string, color: string, icon: string) => void;
  onDelete: (id: string) => void;
}) {
  const t = copy[language];
  const [name, setName] = useState(category.name);
  const [confirming, setConfirming] = useState(false);

  // Синхронизируем локальное имя, если категория обновилась извне.
  useEffect(() => {
    setName(category.name);
  }, [category.name]);

  function commitName() {
    const trimmed = name.trim();
    if (trimmed && trimmed !== category.name) {
      onUpdate(category.id, trimmed, category.color, category.icon);
    } else if (!trimmed) {
      setName(category.name);
    }
  }

  return (
    <div className="flex items-center gap-2.5 rounded-2xl border border-slate-200 px-3 py-2">
      <input
        type="color"
        value={category.color}
        onChange={(event) =>
          onUpdate(category.id, category.name, event.target.value, category.icon)
        }
        className="size-8 shrink-0 cursor-pointer rounded-lg border border-slate-200 bg-white p-0.5"
        title={t.categoryLabel}
      />
      <input
        className="min-w-0 flex-1 rounded-lg border border-transparent bg-transparent px-2 py-1 text-sm font-medium text-slate-800 outline-none transition-colors hover:border-slate-200 focus:border-emerald-300 focus:bg-white"
        value={name}
        onChange={(event) => setName(event.target.value)}
        onBlur={commitName}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
          if (event.key === "Escape") setName(category.name);
        }}
      />
      {confirming ? (
        <div className="flex shrink-0 items-center gap-1">
          <button
            onClick={() => onDelete(category.id)}
            className="rounded-lg bg-rose-600 px-2.5 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-rose-700"
          >
            {t.stageDeleteLabel}
          </button>
          <button
            onClick={() => setConfirming(false)}
            className="grid size-8 place-items-center rounded-lg text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600"
          >
            <X size={15} />
          </button>
        </div>
      ) : (
        <button
          onClick={() => setConfirming(true)}
          title={t.deleteCategoryConfirm}
          className="grid size-8 shrink-0 place-items-center rounded-lg text-slate-400 transition-colors hover:bg-rose-50 hover:text-rose-600"
        >
          <Trash2 size={16} />
        </button>
      )}
    </div>
  );
}

function CategoryManagerModal({
  categories,
  language,
  onClose,
  onCreate,
  onUpdate,
  onDelete,
}: {
  categories: Category[];
  language: "ru" | "en";
  onClose: () => void;
  onCreate: (name: string, color: string, icon: string) => void;
  onUpdate: (id: string, name: string, color: string, icon: string) => void;
  onDelete: (id: string) => void;
}) {
  const t = copy[language];
  const [name, setName] = useState("");
  const [icon, setIcon] = useState("");
  const [color, setColor] = useState(CATEGORY_PALETTE[0]);

  function submit() {
    if (!name.trim()) return;
    onCreate(name.trim(), color, icon.trim());
    setName("");
    setIcon("");
    setColor(CATEGORY_PALETTE[0]);
  }

  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center bg-slate-900/30 p-6 backdrop-blur-sm"
      onClick={onClose}
    >
      <section
        className="w-full max-w-lg overflow-hidden rounded-[28px] border border-emerald-100 bg-white shadow-[0_28px_90px_rgba(15,23,42,0.22)]"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-start justify-between gap-4 border-b border-emerald-100 px-6 py-5">
          <div>
            <h2 className="text-lg font-semibold text-slate-900">
              {t.categoriesTitle}
            </h2>
            <p className="mt-1 text-sm text-slate-500">
              {t.categoriesDescription}
            </p>
          </div>
          <button
            onClick={onClose}
            className="grid size-9 shrink-0 place-items-center rounded-xl text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600"
          >
            <X size={18} />
          </button>
        </header>

        <div className="space-y-4 p-6">
          {/* Добавление */}
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={color}
              onChange={(event) => setColor(event.target.value)}
              className="size-10 shrink-0 cursor-pointer rounded-xl border border-slate-200 bg-white p-1"
              title={t.categoryLabel}
            />
            <input
              className="field"
              value={name}
              placeholder={t.categoryNamePlaceholder}
              onChange={(event) => setName(event.target.value)}
              onKeyDown={(event) => event.key === "Enter" && submit()}
            />
            <input
              className="field w-16 text-center"
              value={icon}
              placeholder="🎬"
              maxLength={2}
              onChange={(event) => setIcon(event.target.value)}
            />
            <button
              className="primary-button shrink-0"
              onClick={submit}
              disabled={!name.trim()}
            >
              <Plus size={16} />
              {t.addCategoryLabel}
            </button>
          </div>

          {/* Список */}
          <div className="max-h-[45vh] space-y-2 overflow-y-auto">
            {categories.length ? (
              categories.map((category) => (
                <CategoryRow
                  key={category.id}
                  category={category}
                  language={language}
                  onUpdate={onUpdate}
                  onDelete={onDelete}
                />
              ))
            ) : (
              <EmptyState text={t.categoryEmpty} />
            )}
          </div>
        </div>
      </section>
    </div>
  );
}

function CategoryPicker({
  categories,
  value,
  language,
  onChange,
}: {
  categories: Category[];
  value: string | null;
  language: "ru" | "en";
  onChange: (categoryId: string | null) => void;
}) {
  const t = copy[language];
  const [open, setOpen] = useState(false);
  const current = categories.find((c) => c.id === value) ?? null;

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex min-w-[180px] items-center gap-2 rounded-2xl border border-slate-200 bg-white px-3 py-2.5 text-sm font-medium text-slate-700 transition-colors hover:border-emerald-200"
      >
        <span
          className="size-3 shrink-0 rounded-full"
          style={{ background: current?.color ?? "#cbd5e1" }}
        />
        <span className="min-w-0 flex-1 truncate text-left">
          {current ? `${current.icon ? `${current.icon} ` : ""}${current.name}` : t.noCategory}
        </span>
        <ChevronDown
          size={16}
          className={`shrink-0 text-slate-400 transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <motion.div
            initial={{ opacity: 0, y: -6, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            transition={{ duration: 0.14 }}
            className="absolute right-0 z-50 mt-2 max-h-72 w-64 overflow-y-auto rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
          >
            <button
              onClick={() => {
                onChange(null);
                setOpen(false);
              }}
              className="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-left text-sm transition-colors hover:bg-emerald-50"
            >
              <span className="size-3 shrink-0 rounded-full border-2 border-slate-300" />
              <span className="min-w-0 flex-1 truncate text-slate-600">
                {t.noCategory}
              </span>
              {value === null && <Check size={15} className="text-emerald-600" />}
            </button>
            {categories.map((category) => (
              <button
                key={category.id}
                onClick={() => {
                  onChange(category.id);
                  setOpen(false);
                }}
                className="flex w-full items-center gap-2.5 rounded-xl px-3 py-2 text-left text-sm transition-colors hover:bg-emerald-50"
              >
                <span
                  className="size-3 shrink-0 rounded-full"
                  style={{ background: category.color }}
                />
                <span className="min-w-0 flex-1 truncate text-slate-700">
                  {category.icon ? `${category.icon} ` : ""}
                  {category.name}
                </span>
                {value === category.id && (
                  <Check size={15} className="text-emerald-600" />
                )}
              </button>
            ))}
          </motion.div>
        </>
      )}
    </div>
  );
}

function HelpItem({ title, text }: { title: string; text: string }) {
  return (
    <article className="rounded-3xl border border-emerald-100 bg-slate-50 px-5 py-4">
      <strong className="block text-sm font-semibold text-emerald-800">
        {title}
      </strong>
      <p className="mt-2 text-sm leading-6 text-slate-600">{text}</p>
    </article>
  );
}

function ProjectStagesModal({
  project,
  language,
  t,
  isLocked,
  onClose,
  onSetProjectState,
}: {
  project: ProjectRecord | null;
  language: AppSettings["language"];
  t: Copy;
  isLocked: boolean;
  onClose: () => void;
  onSetProjectState: (updated: ProjectRecord) => void;
}) {
  const [localProject, setLocalProject] = useState(project);
  const [selectedStageId, setSelectedStageId] = useState<string | null>(
    project?.stages[0]?.id ?? null,
  );
  const [expandedStageApps, setExpandedStageApps] = useState<
    Record<string, boolean>
  >({});
  const [stageLinkMenu, setStageLinkMenu] = useState<string | null>(null);

  useEffect(() => {
    setLocalProject(project);
    const firstStage = project?.stages[0] ?? null;
    setSelectedStageId((current) =>
      current && project?.stages.some((stage) => stage.id === current)
        ? current
        : (firstStage?.id ?? null),
    );
  }, [project]);

  const selectedStage =
    localProject?.stages.find((stage) => stage.id === selectedStageId) ?? null;
  const orderedStages = [...(localProject?.stages ?? [])].sort(
    (left, right) => left.order - right.order,
  );
  const selectedCount = localProject?.selected_stage_ids.length ?? 0;
  const selectedStageApps = selectedStage
    ? sortRankedStageApps(selectedStage, localProject?.apps ?? [], 0)
    : [];
  const stageTotalSeconds = selectedStageApps.reduce(
    (sum, item) => sum + item.includedSeconds,
    0,
  );
  const rankedStageApps = selectedStage
    ? sortRankedStageApps(
        selectedStage,
        localProject?.apps ?? [],
        stageTotalSeconds,
      )
    : [];
  const stageTotals = selectedStage
    ? stageEntityTotals(selectedStage, localProject?.apps ?? [])
    : { enabled: 0, total: 0 };

  async function saveUpdatedProject(command: Promise<ProjectRecord | null>) {
    const updated = await command;
    if (!updated) return null;
    setLocalProject(updated);
    onSetProjectState(updated);
    const nextStage =
      updated.stages.find((stage) => stage.id === selectedStageId) ??
      updated.stages[0] ??
      null;
    setSelectedStageId(nextStage?.id ?? null);
    return updated;
  }

  async function createStage() {
    if (!localProject || isLocked) return;
    const name = window.prompt(t.stageNamePlaceholder, "");
    const trimmed = name?.trim();
    if (!trimmed) return;
    const updated = await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "create_stage",
        { projectId: localProject.id, name: trimmed },
        null,
      ),
    );
    if (!updated) return;
    const next =
      [...updated.stages]
        .sort((left, right) => left.order - right.order)
        .at(-1) ?? null;
    setSelectedStageId(next?.id ?? null);
  }

  async function renameStage() {
    if (!localProject || !selectedStage || isLocked) return;
    const name = window.prompt(t.stageRenameLabel, selectedStage.name);
    const trimmed = name?.trim();
    if (!trimmed) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "rename_stage",
        {
          projectId: localProject.id,
          stageId: selectedStage.id,
          name: trimmed,
        },
        null,
      ),
    );
  }

  async function removeStage() {
    if (!localProject || !selectedStage || isLocked) return;
    if (!window.confirm(t.stageDeleteConfirm)) return;
    const updated = await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "delete_stage",
        { projectId: localProject.id, stageId: selectedStage.id },
        null,
      ),
    );
    const next = updated?.stages[0] ?? null;
    setSelectedStageId(next?.id ?? null);
  }

  async function moveStage(stageId: string, direction: number) {
    if (!localProject || isLocked) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "reorder_stage",
        { projectId: localProject.id, stageId, direction },
        null,
      ),
    );
  }

  async function toggleStageSelected(stageId: string, enabled: boolean) {
    if (!localProject || isLocked) return;
    const nextIds = enabled
      ? [...localProject.selected_stage_ids, stageId]
      : localProject.selected_stage_ids.filter((item) => item !== stageId);
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "set_selected_project_stages",
        { projectId: localProject.id, stageIds: nextIds },
        null,
      ),
    );
  }

  async function toggleStageApp(
    stageId: string,
    appKey: string,
    enabled: boolean,
  ) {
    if (!localProject || isLocked) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "toggle_stage_app_included",
        { projectId: localProject.id, stageId, appKey, enabled },
        null,
      ),
    );
  }

  async function toggleStageTab(
    stageId: string,
    appKey: string,
    tabKey: string,
    enabled: boolean,
  ) {
    if (!localProject || isLocked) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "toggle_stage_tab_included",
        { projectId: localProject.id, stageId, appKey, tabKey, enabled },
        null,
      ),
    );
  }

  async function toggleStageUrl(
    stageId: string,
    appKey: string,
    tabKey: string,
    url: string,
    enabled: boolean,
  ) {
    if (!localProject || isLocked) return;
    await saveUpdatedProject(
      invokeCommand<ProjectRecord | null>(
        "toggle_stage_url_included",
        { projectId: localProject.id, stageId, appKey, tabKey, url, enabled },
        null,
      ),
    );
  }

  async function openStageUrl(url?: string | null) {
    if (!url) return;
    await invokeCommand<void>("open_external_url", { url }, undefined);
  }

  if (!localProject) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/24 px-5 backdrop-blur-sm">
      <section className="flex h-[78vh] w-full max-w-[1380px] flex-col overflow-hidden rounded-[30px] border border-emerald-100 bg-white shadow-[0_28px_90px_rgba(15,23,42,0.22)]">
        <header className="flex items-start justify-between gap-4 bg-[linear-gradient(135deg,#ecfdf5_0%,#ffffff_62%,#dcfce7_100%)] px-7 py-6">
          <div>
            <span className="inline-flex rounded-full bg-emerald-600 px-3 py-1 text-xs font-semibold uppercase text-white">
              Project Time Manager
            </span>
            <h2 className="mt-4 text-2xl font-semibold text-slate-900">
              {t.stageTitle}
            </h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-slate-600">
              {t.stageDescription}
            </p>
          </div>
          <button
            className="icon-button shrink-0"
            onClick={onClose}
            title={t.closeLabel}
          >
            <X size={18} />
          </button>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-[320px_minmax(0,1fr)] gap-4 px-6 py-5">
          <section className="flex min-h-0 flex-col rounded-[26px] border border-emerald-100 bg-white p-4">
            {isLocked ? (
              <div className="mb-4 rounded-2xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
                {t.stageLockedHint}
              </div>
            ) : null}
            <div className="stable-scroll min-h-0 flex-1 overflow-y-scroll pr-1">
              <div className="grid gap-2">
                {orderedStages.length ? (
                  orderedStages.map((stage, index) => {
                    const isCurrent = stage.id === selectedStage?.id;
                    const isStageSelected =
                      localProject.selected_stage_ids.includes(stage.id);

                    return (
                      <button
                        key={stage.id}
                        className={`grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-2xl border px-3 py-3 text-left transition ${
                          isCurrent
                            ? "border-emerald-300 bg-emerald-50"
                            : "border-slate-200 bg-white hover:border-emerald-200"
                        }`}
                        onClick={() => setSelectedStageId(stage.id)}
                        title={isLocked ? t.stageLockedHint : stage.name}
                      >
                        <div
                          className="flex flex-col gap-2"
                          onClick={(event) => event.stopPropagation()}
                        >
                          <button
                            className="icon-button h-8 w-8"
                            onClick={() => moveStage(stage.id, -1)}
                            title={t.stageUpLabel}
                            disabled={index === 0 || isLocked}
                          >
                            <ArrowUp size={14} />
                          </button>
                          <button
                            className="icon-button h-8 w-8"
                            onClick={() => moveStage(stage.id, 1)}
                            title={t.stageDownLabel}
                            disabled={
                              index === orderedStages.length - 1 || isLocked
                            }
                          >
                            <ArrowDown size={14} />
                          </button>
                        </div>

                        <span className="min-w-0">
                          <strong className="block truncate text-sm text-slate-900">
                            {stage.name}
                          </strong>
                          <small className="mt-1 block truncate text-xs text-slate-500">
                            {formatDateTime(stage.updated_at, language)}
                          </small>
                        </span>

                        <span onClick={(event) => event.stopPropagation()}>
                          <Checkbox
                            checked={isStageSelected}
                            disabled={isLocked}
                            title={
                              isLocked
                                ? t.stageLockedHint
                                : t.stageSelectionHint
                            }
                            onChange={(checked) =>
                              toggleStageSelected(stage.id, checked)
                            }
                          />
                        </span>
                      </button>
                    );
                  })
                ) : (
                  <EmptyState text={t.stageEmpty} />
                )}
              </div>
            </div>

            <div className="mt-4 grid grid-cols-3 gap-2">
              <button
                className="icon-button h-11 w-full"
                onClick={createStage}
                title={isLocked ? t.stageLockedHint : t.stageCreateLabel}
                disabled={isLocked}
              >
                <Plus size={16} />
              </button>
              <button
                className="icon-button h-11 w-full"
                onClick={renameStage}
                title={isLocked ? t.stageLockedHint : t.stageRenameLabel}
                disabled={!selectedStage || isLocked}
              >
                <PencilLine size={16} />
              </button>
              <button
                className="icon-button h-11 w-full text-rose-700 hover:bg-rose-50"
                onClick={removeStage}
                title={isLocked ? t.stageLockedHint : t.stageDeleteLabel}
                disabled={!selectedStage || isLocked}
              >
                <Trash2 size={16} />
              </button>
            </div>
          </section>

          <section className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] gap-4">
            <section className="grid grid-cols-3 gap-3">
              <Metric
                label={t.stageMetricsCount}
                value={String(stageTotals.enabled)}
                accent="emerald"
              />
              <Metric
                label={t.stageMetricsSelectedCount}
                value={String(selectedCount)}
                accent="emerald"
              />
              <Metric
                label={t.stageMetricsAll}
                value={String(stageTotals.total)}
                accent="slate"
              />
            </section>

            <section className="flex min-h-0 flex-col rounded-[26px] border border-emerald-100 bg-white p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <h3 className="text-base font-semibold text-slate-900">
                    {selectedStage?.name ?? t.stageTitle}
                  </h3>
                  <p className="mt-1 text-sm text-slate-500">
                    {selectedStage
                      ? formatDateTime(selectedStage.updated_at, language)
                      : t.stageCreateHint}
                  </p>
                </div>
                {selectedStage ? (
                  <span className="rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.12em] text-emerald-700">
                    {localProject.selected_stage_ids.includes(selectedStage.id)
                      ? t.stageSelectedLabel
                      : t.stageUnusedLabel}
                  </span>
                ) : null}
              </div>

              {selectedStage ? (
                <div className="mt-4 grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)] gap-2 overflow-hidden">
                  <div className="grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] gap-2 px-3 text-xs font-medium uppercase tracking-[0.14em] text-slate-400 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4">
                    <span />
                    <span>{t.applicationHeader}</span>
                    <span>{t.timeHeader}</span>
                    <span>{t.percentHeader}</span>
                  </div>

                  <div className="stable-scroll min-h-0 overflow-y-scroll pr-1">
                    <div className="grid gap-2">
                      {rankedStageApps.map(
                        ({
                          app,
                          includedSeconds,
                          actualSeconds,
                          includedPercent,
                          actualPercent,
                        }) => {
                          const isOpen = expandedStageApps[app.key];
                          const isIncluded = stageAppEnabled(
                            selectedStage,
                            app,
                          );
                          const checkboxTitle = !app.enabled
                            ? t.stageProjectDisabledHint
                            : isLocked
                              ? t.stageLockedHint
                              : undefined;

                          return (
                            <div key={app.key} className="grid gap-2">
                              <div
                                className={`grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4 ${
                                  isIncluded
                                    ? "border-slate-200 bg-slate-50"
                                    : "border-slate-200 bg-slate-100/80 text-slate-500"
                                }`}
                              >
                                <Checkbox
                                  checked={isIncluded}
                                  disabled={!app.enabled || isLocked}
                                  title={checkboxTitle}
                                  onChange={(checked) =>
                                    toggleStageApp(
                                      selectedStage.id,
                                      app.key,
                                      checked,
                                    )
                                  }
                                />

                                <button
                                  className="flex min-w-0 items-center gap-3 text-left"
                                  onClick={() =>
                                    setExpandedStageApps((prev) => ({
                                      ...prev,
                                      [app.key]: !prev[app.key],
                                    }))
                                  }
                                >
                                  <AppIcon app={app} />
                                  <span className="min-w-0">
                                    <strong
                                      className={`block truncate text-sm ${isIncluded ? "text-slate-900" : "text-slate-500"}`}
                                    >
                                      {app.name}
                                    </strong>
                                    <small
                                      className={`block truncate text-xs ${isIncluded ? "text-slate-500" : "text-slate-400"}`}
                                    >
                                      {app.kind === "browser"
                                        ? t.browserLabel
                                        : app.process_path || app.process_name}
                                    </small>
                                  </span>
                                  {app.kind === "browser" ? (
                                    <span className="ml-auto text-slate-400">
                                      {isOpen ? (
                                        <ChevronDown size={16} />
                                      ) : (
                                        <ChevronRight size={16} />
                                      )}
                                    </span>
                                  ) : null}
                                </button>

                                <span className="text-sm text-slate-900">
                                  {formatDuration(actualSeconds, language)}
                                  <small className="mt-1 block text-xs text-slate-500">
                                    {isIncluded
                                      ? `${t.reportTimeLabel}: ${formatDuration(includedSeconds, language)}`
                                      : `${t.reportTimeLabel}: 00:00:00`}
                                  </small>
                                </span>

                                <span className="text-sm text-slate-500">
                                  {isIncluded ? includedPercent : "0%"}
                                  {!isIncluded ? (
                                    <small className="mt-1 block text-xs text-slate-400">
                                      {actualPercent}
                                    </small>
                                  ) : null}
                                </span>
                              </div>

                              {app.kind === "browser" && isOpen ? (
                                <div className="ml-4 grid gap-2 sm:ml-10">
                                  {sortRankedStageTabs(
                                    selectedStage,
                                    app,
                                    stageTotalSeconds,
                                  ).map(
                                    ({
                                      tab,
                                      includedSeconds: tabIncludedSeconds,
                                      actualSeconds: tabActual,
                                      includedPercent: tabIncludedPercent,
                                      actualPercent: tabActualPercent,
                                    }) => {
                                      const urls = tabUrlList(tab);
                                      const menuKey = `stage:${selectedStage.id}:${app.key}:${tab.key}`;
                                      const isMenuOpen =
                                        stageLinkMenu === menuKey;
                                      const isTabIncluded = stageTabEnabled(
                                        selectedStage,
                                        app,
                                        tab,
                                      );
                                      const tabCheckboxTitle = !tab.enabled
                                        ? t.stageProjectDisabledHint
                                        : isLocked
                                          ? t.stageLockedHint
                                          : undefined;

                                      return (
                                        <div
                                          key={tab.key}
                                          className={`grid grid-cols-[36px_minmax(120px,1fr)_86px_52px] items-start gap-2 rounded-2xl border px-3 py-3 sm:grid-cols-[44px_minmax(220px,1fr)_110px_72px] sm:gap-3 sm:px-4 ${
                                            isTabIncluded
                                              ? "border-slate-200 bg-white"
                                              : "border-slate-200 bg-slate-50 text-slate-500"
                                          }`}
                                        >
                                          <Checkbox
                                            checked={isTabIncluded}
                                            disabled={!tab.enabled || isLocked}
                                            title={tabCheckboxTitle}
                                            onChange={(checked) =>
                                              toggleStageTab(
                                                selectedStage.id,
                                                app.key,
                                                tab.key,
                                                checked,
                                              )
                                            }
                                          />
                                          <span className="flex min-w-0 items-center gap-3">
                                            <TabIcon tab={tab} />
                                            <span className="relative min-w-0">
                                              <span className="flex max-w-full items-center gap-2">
                                                <strong
                                                  className={`block truncate text-sm ${isTabIncluded ? "text-slate-900" : "text-slate-500"}`}
                                                >
                                                  {tab.title}
                                                </strong>
                                                {urls.length ? (
                                                  <button
                                                    className="inline-flex size-7 shrink-0 items-center justify-center rounded-full border border-emerald-100 bg-emerald-50 text-emerald-600 transition-colors hover:border-emerald-200 hover:bg-emerald-100"
                                                    onClick={(event) => {
                                                      event.stopPropagation();
                                                      setStageLinkMenu(
                                                        isMenuOpen
                                                          ? null
                                                          : menuKey,
                                                      );
                                                    }}
                                                    title={t.openUrlLabel}
                                                  >
                                                    <Link size={13} />
                                                  </button>
                                                ) : null}
                                              </span>
                                              <small
                                                className={`block truncate text-xs ${isTabIncluded ? "text-slate-500" : "text-slate-400"}`}
                                              >
                                                {urls.length
                                                  ? t.linksInDomain(urls.length)
                                                  : t.urlUnavailable}
                                              </small>
                                              {isMenuOpen ? (
                                                <div
                                                  className="absolute left-0 top-12 z-20 w-[min(460px,70vw)] rounded-2xl border border-emerald-100 bg-white p-2 shadow-[0_18px_50px_rgba(15,23,42,0.16)]"
                                                  onClick={(event) =>
                                                    event.stopPropagation()
                                                  }
                                                >
                                                  <div className="max-h-60 overflow-y-auto pr-1">
                                                    {urls.map((item) => {
                                                      const linkIncluded =
                                                        stageLinkEnabled(
                                                          selectedStage,
                                                          app,
                                                          tab,
                                                          item,
                                                        );
                                                      const linkCheckboxTitle =
                                                        !item.enabled
                                                          ? t.stageProjectDisabledHint
                                                          : isLocked
                                                            ? t.stageLockedHint
                                                            : undefined;

                                                      return (
                                                        <div
                                                          key={item.url}
                                                          className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 rounded-xl px-3 py-2 text-left transition-colors hover:bg-emerald-50"
                                                        >
                                                          <Checkbox
                                                            checked={
                                                              linkIncluded
                                                            }
                                                            disabled={
                                                              !item.enabled ||
                                                              isLocked
                                                            }
                                                            title={
                                                              linkCheckboxTitle
                                                            }
                                                            onChange={(
                                                              checked,
                                                            ) =>
                                                              toggleStageUrl(
                                                                selectedStage.id,
                                                                app.key,
                                                                tab.key,
                                                                item.url,
                                                                checked,
                                                              )
                                                            }
                                                          />
                                                          <button
                                                            className="min-w-0 text-left"
                                                            onClick={() => {
                                                              setStageLinkMenu(
                                                                null,
                                                              );
                                                              openStageUrl(
                                                                item.url,
                                                              );
                                                            }}
                                                          >
                                                            <strong className="block truncate text-xs text-slate-900">
                                                              {item.title ||
                                                                item.url}
                                                            </strong>
                                                            <span className="block truncate text-xs text-slate-500">
                                                              {item.url}
                                                            </span>
                                                          </button>
                                                          <span className="font-mono text-xs text-slate-700">
                                                            {formatDuration(
                                                              item.time_seconds,
                                                              language,
                                                            )}
                                                          </span>
                                                        </div>
                                                      );
                                                    })}
                                                  </div>
                                                </div>
                                              ) : null}
                                            </span>
                                          </span>
                                          <span className="text-sm text-slate-900">
                                            {formatDuration(
                                              tabActual,
                                              language,
                                            )}
                                            <small className="mt-1 block text-xs text-slate-500">
                                              {isTabIncluded
                                                ? `${t.reportTimeLabel}: ${formatDuration(tabIncludedSeconds, language)}`
                                                : `${t.reportTimeLabel}: 00:00:00`}
                                            </small>
                                          </span>
                                          <span className="text-sm text-slate-500">
                                            {isTabIncluded
                                              ? tabIncludedPercent
                                              : "0%"}
                                            {!isTabIncluded ? (
                                              <small className="mt-1 block text-xs text-slate-400">
                                                {tabActualPercent}
                                              </small>
                                            ) : null}
                                          </span>
                                        </div>
                                      );
                                    },
                                  )}
                                </div>
                              ) : null}
                            </div>
                          );
                        },
                      )}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="mt-4">
                  <EmptyState text={t.stageEmpty} />
                </div>
              )}
            </section>
          </section>
        </div>
      </section>
    </div>
  );
}
