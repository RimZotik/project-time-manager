// Глобальная верхняя панель: индикатор записи (какой проект + идёт ли запись)
// и кнопка помощи. Видна на всех страницах.
import { useState } from "react";
import { motion } from "framer-motion";
import { HelpCircle } from "lucide-react";
import { useAppState } from "../../store/AppState";
import { shellCopy } from "../../lib/i18n";
import HelpModal from "../ui/HelpModal";

export default function TopBar() {
  const { state, language } = useAppState();
  const t = shellCopy[language];
  const [helpOpen, setHelpOpen] = useState(false);

  const recording = state.tracker.status === "running";
  const paused = state.tracker.status === "paused";
  const activeId = state.tracker.active_project_id;
  const activeName =
    state.projects.find((p) => p.id === activeId)?.name ??
    state.selected_project?.name ??
    null;

  const label = recording
    ? t.recording
    : paused
      ? t.notRecording
      : activeName
        ? activeName
        : t.noProject;

  return (
    <div className="flex items-center justify-end gap-2 px-6 pt-4">
      {/* Индикатор записи */}
      <div
        className={`flex items-center gap-2.5 rounded-full border px-3.5 py-2 text-sm font-medium transition-colors ${
          recording
            ? "border-emerald-200 bg-emerald-50 text-emerald-800"
            : "border-slate-200 bg-white/70 text-slate-500 backdrop-blur"
        }`}
        title={recording ? `${t.recording}: ${activeName ?? ""}` : label}
      >
        <span className="relative grid size-3 place-items-center">
          {recording ? (
            <>
              <motion.span
                className="absolute inset-0 rounded-full bg-emerald-500"
                animate={{ scale: [1, 1.9], opacity: [0.6, 0] }}
                transition={{ duration: 1.6, repeat: Infinity, ease: "easeOut" }}
              />
              <span className="size-3 rounded-full bg-emerald-500" />
            </>
          ) : (
            <span className="size-3 rounded-full border-2 border-slate-300" />
          )}
        </span>
        <span className="max-w-[220px] truncate">
          {recording && activeName ? activeName : label}
        </span>
      </div>

      {/* Помощь */}
      <button
        type="button"
        onClick={() => setHelpOpen(true)}
        title={t.helpButton}
        className="grid size-10 place-items-center rounded-full border border-slate-200 bg-white/70 text-slate-500 backdrop-blur transition-colors hover:border-emerald-200 hover:bg-emerald-50 hover:text-emerald-700"
      >
        <HelpCircle className="size-5" />
      </button>

      <HelpModal open={helpOpen} onClose={() => setHelpOpen(false)} />
    </div>
  );
}
