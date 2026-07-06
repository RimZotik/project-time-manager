import { FolderOpen } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import { invokeCommand } from "../lib/api";
import type { AppSettings, Language } from "../lib/types";
import PageHeader from "../components/ui/PageHeader";

export default function SettingsPage() {
  const { state, language, refresh } = useAppState();
  const t = shellCopy[language];
  const s = t.settings;
  const settings = state.settings;

  async function save(next: AppSettings) {
    await invokeCommand<AppSettings>("update_app_settings", next, next);
    await refresh();
  }

  async function openFolder() {
    await invokeCommand<void>("open_app_folder", {}, undefined);
  }

  const setLanguage = (lang: Language) => save({ ...settings, language: lang });

  return (
    <div className="flex h-full flex-col">
      <PageHeader title={s.title} subtitle={s.subtitle} />
      <div className="min-h-0 flex-1 overflow-y-auto px-8 pb-10">
        <div className="mx-auto flex max-w-2xl flex-col gap-4">
          {/* Язык */}
          <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <h2 className="text-sm font-semibold text-slate-900">
              {s.language}
            </h2>
            <div className="mt-4 inline-flex rounded-2xl border border-slate-200 bg-slate-50 p-1">
              {(
                [
                  ["ru", s.languageRu],
                  ["en", s.languageEn],
                ] as const
              ).map(([code, label]) => (
                <button
                  key={code}
                  onClick={() => setLanguage(code)}
                  className={`rounded-xl px-5 py-2 text-sm font-medium transition-colors ${
                    settings.language === code
                      ? "bg-emerald-600 text-white shadow-[0_8px_18px_rgba(5,150,105,0.25)]"
                      : "text-slate-600 hover:text-emerald-700"
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>
          </section>

          {/* Автозапуск */}
          <section className="flex items-center justify-between gap-4 rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <div>
              <h2 className="text-sm font-semibold text-slate-900">
                {s.autostart}
              </h2>
              <p className="mt-1 text-sm text-slate-500">{s.autostartHint}</p>
            </div>
            <button
              role="switch"
              aria-checked={settings.autostart}
              onClick={() => save({ ...settings, autostart: !settings.autostart })}
              className={`relative h-7 w-12 shrink-0 rounded-full transition-colors ${
                settings.autostart ? "bg-emerald-600" : "bg-slate-300"
              }`}
            >
              <span
                className={`absolute top-1 size-5 rounded-full bg-white shadow transition-all ${
                  settings.autostart ? "left-6" : "left-1"
                }`}
              />
            </button>
          </section>

          {/* Папка данных */}
          <section className="flex items-center justify-between gap-4 rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <div>
              <h2 className="text-sm font-semibold text-slate-900">
                {s.dataFolder}
              </h2>
              <p className="mt-1 text-sm text-slate-500">{s.dataFolderHint}</p>
            </div>
            <button className="secondary-button shrink-0" onClick={openFolder}>
              <FolderOpen size={16} />
              {s.openFolder}
            </button>
          </section>
        </div>
      </div>
    </div>
  );
}
