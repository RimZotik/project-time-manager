// Общие типы данных приложения. Держим их отдельно от монолитного
// ProjectWorkspace, чтобы shell, контекст состояния и новые страницы
// использовали единый источник правды.

export type TrackerStatus = "stopped" | "paused" | "running";

export type TrackerPayload = {
  status: TrackerStatus;
  active_project_id: string | null;
  running_since: string | null;
};

export type Language = "ru" | "en";

export type AppSettings = {
  autostart: boolean;
  language: Language;
};

export type ProjectSummary = {
  id: string;
  name: string;
  client: string;
  updated_at: string;
};

export type VisitedUrlRecord = {
  url: string;
  title: string;
  last_seen_at: string;
  hits: number;
  enabled: boolean;
  time_seconds: number;
};

export type TabUsageRecord = {
  key: string;
  title: string;
  url: string | null;
  urls?: VisitedUrlRecord[];
  favicon_url?: string | null;
  enabled: boolean;
  time_seconds: number;
};

export type AppUsageRecord = {
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

export type SessionStageSnapshot = {
  id: string;
  name: string;
};

export type SessionRecord = {
  id: string;
  started_at: string;
  stopped_at: string | null;
  duration_seconds: number;
  app_count: number;
  browser_count: number;
  stages: SessionStageSnapshot[];
};

export type ProjectStageRecord = {
  id: string;
  name: string;
  order: number;
  created_at: string;
  updated_at: string;
  apps: unknown[];
};

export type ProjectRecord = ProjectSummary & {
  note: string;
  created_at: string;
  sessions: SessionRecord[];
  apps: AppUsageRecord[];
  selected_stage_ids: string[];
  stages: ProjectStageRecord[];
};

export type AppState = {
  tracker: TrackerPayload;
  settings: AppSettings;
  projects: ProjectSummary[];
  selected_project: ProjectRecord | null;
};

export const fallbackState: AppState = {
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
