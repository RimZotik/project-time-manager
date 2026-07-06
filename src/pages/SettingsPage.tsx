import { useEffect, useState } from "react";
import { FolderOpen, Plus, Trash2 } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import { invokeCommand } from "../lib/api";
import type { AppRule, AppSettings, Language } from "../lib/types";
import PageHeader from "../components/ui/PageHeader";

export default function SettingsPage() {
  const { state, language, refresh } = useAppState();
  const t = shellCopy[language];
  const s = t.settings;
  const settings = state.settings;
  const categories = state.categories;

  const [rules, setRules] = useState<AppRule[]>([]);
  const [ruleProcess, setRuleProcess] = useState("");
  const [ruleCategory, setRuleCategory] = useState<string>("");

  async function loadRules() {
    setRules(await invokeCommand<AppRule[]>("list_app_rules", {}, []));
  }
  useEffect(() => {
    loadRules();
  }, []);

  async function save(next: AppSettings) {
    await invokeCommand<AppSettings>("update_app_settings", next, next);
    await refresh();
  }
  const setLanguage = (lang: Language) => save({ ...settings, language: lang });

  async function addRule() {
    if (!ruleProcess.trim() || !ruleCategory) return;
    await invokeCommand<AppRule | null>(
      "create_app_rule",
      { matchProcess: ruleProcess.trim(), categoryId: ruleCategory },
      null,
    );
    setRuleProcess("");
    setRuleCategory("");
    loadRules();
  }
  async function removeRule(id: string) {
    await invokeCommand<void>("delete_app_rule", { id }, undefined);
    loadRules();
  }

  const rl =
    language === "en"
      ? {
          rules: "Auto-categorization rules",
          rulesHint:
            "Map an app (by name or process) to a category. On a project, “Detect category” picks the best match.",
          process: "App or process (e.g. Premiere)",
          pickCategory: "Pick a category",
          noRules: "No rules yet.",
          arrow: "→",
        }
      : {
          rules: "Правила автокатегоризации",
          rulesHint:
            "Свяжите приложение (по имени или процессу) с категорией. На проекте «Определить категорию» подберёт лучшее совпадение.",
          process: "Приложение или процесс (напр. Premiere)",
          pickCategory: "Выберите категорию",
          noRules: "Правил пока нет.",
          arrow: "→",
        };

  const catById = (id: string) => categories.find((c) => c.id === id);

  return (
    <div className="flex h-full flex-col">
      <PageHeader title={s.title} subtitle={s.subtitle} />
      <div className="min-h-0 flex-1 overflow-y-auto px-8 pb-10">
        <div className="mx-auto flex max-w-2xl flex-col gap-4">
          {/* Язык */}
          <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <h2 className="text-sm font-semibold text-slate-900">{s.language}</h2>
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
              <h2 className="text-sm font-semibold text-slate-900">{s.autostart}</h2>
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

          {/* Правила автокатегоризации */}
          <section className="rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <h2 className="text-sm font-semibold text-slate-900">{rl.rules}</h2>
            <p className="mt-1 text-sm text-slate-500">{rl.rulesHint}</p>

            <div className="mt-4 flex flex-col gap-3">
              <input
                className="field"
                value={ruleProcess}
                placeholder={rl.process}
                onChange={(e) => setRuleProcess(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addRule()}
              />
              <div className="flex flex-wrap items-center gap-1.5">
                {categories.length ? (
                  categories.map((c) => (
                    <button
                      key={c.id}
                      onClick={() => setRuleCategory(c.id)}
                      className={`flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-medium transition-colors ${
                        ruleCategory === c.id
                          ? "border-emerald-300 bg-emerald-50 text-emerald-800"
                          : "border-slate-200 text-slate-600 hover:bg-slate-50"
                      }`}
                    >
                      <span
                        className="size-2.5 rounded-full"
                        style={{ background: c.color }}
                      />
                      {c.name}
                    </button>
                  ))
                ) : (
                  <span className="text-xs text-slate-400">{rl.pickCategory}</span>
                )}
                <button
                  onClick={addRule}
                  disabled={!ruleProcess.trim() || !ruleCategory}
                  className="ml-auto inline-flex items-center gap-1.5 rounded-full bg-emerald-600 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-emerald-700 disabled:cursor-not-allowed disabled:bg-emerald-300"
                >
                  <Plus size={14} />
                  {language === "en" ? "Add" : "Добавить"}
                </button>
              </div>
            </div>

            <div className="mt-4 flex flex-col gap-1.5">
              {rules.length ? (
                rules.map((r) => {
                  const cat = catById(r.category_id);
                  return (
                    <div
                      key={r.id}
                      className="flex items-center gap-2 rounded-xl border border-slate-100 px-3 py-2 text-sm"
                    >
                      <span className="font-medium text-slate-700">
                        {r.match_process}
                      </span>
                      <span className="text-slate-300">{rl.arrow}</span>
                      <span className="flex items-center gap-1.5 text-slate-600">
                        <span
                          className="size-2.5 rounded-full"
                          style={{ background: cat?.color ?? "#cbd5e1" }}
                        />
                        {cat?.name ?? "—"}
                      </span>
                      <button
                        onClick={() => removeRule(r.id)}
                        className="ml-auto grid size-7 place-items-center rounded-lg text-slate-400 transition-colors hover:bg-rose-50 hover:text-rose-600"
                      >
                        <Trash2 size={15} />
                      </button>
                    </div>
                  );
                })
              ) : (
                <p className="text-sm text-slate-400">{rl.noRules}</p>
              )}
            </div>
          </section>

          {/* Папка данных */}
          <section className="flex items-center justify-between gap-4 rounded-[24px] border border-emerald-100 bg-white/80 p-6 shadow-[0_10px_30px_rgba(15,23,42,0.05)] backdrop-blur">
            <div>
              <h2 className="text-sm font-semibold text-slate-900">{s.dataFolder}</h2>
              <p className="mt-1 text-sm text-slate-500">{s.dataFolderHint}</p>
            </div>
            <button
              className="secondary-button shrink-0"
              onClick={() => invokeCommand<void>("open_app_folder", {}, undefined)}
            >
              <FolderOpen size={16} />
              {s.openFolder}
            </button>
          </section>
        </div>
      </div>
    </div>
  );
}
