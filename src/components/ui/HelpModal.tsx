// Глобальное окно помощи (раньше «инфо» жило на странице проектов).
import { AnimatePresence, motion } from "framer-motion";
import { X } from "lucide-react";
import { useAppState } from "../../store/AppState";
import { shellCopy } from "../../lib/i18n";

export default function HelpModal({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const { language } = useAppState();
  const t = shellCopy[language].help;

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className="fixed inset-0 z-50 grid place-items-center bg-slate-900/30 p-6 backdrop-blur-sm"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
        >
          <motion.section
            className="w-full max-w-lg overflow-hidden rounded-[28px] border border-emerald-100 bg-white shadow-[0_28px_90px_rgba(15,23,42,0.22)]"
            initial={{ opacity: 0, y: 18, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 12, scale: 0.98 }}
            transition={{ type: "spring", stiffness: 300, damping: 28 }}
            onClick={(e) => e.stopPropagation()}
          >
            <header className="flex items-start justify-between gap-4 border-b border-emerald-100 px-6 py-5">
              <div>
                <h2 className="text-lg font-semibold text-slate-900">
                  {t.title}
                </h2>
                <p className="mt-1 text-sm text-slate-500">{t.subtitle}</p>
              </div>
              <button
                type="button"
                onClick={onClose}
                className="grid size-9 shrink-0 place-items-center rounded-xl text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600"
              >
                <X className="size-5" />
              </button>
            </header>
            <div className="max-h-[60vh] space-y-3 overflow-y-auto p-6">
              {t.items.map((item, i) => (
                <div
                  key={i}
                  className="rounded-2xl border border-emerald-100 bg-emerald-50/40 p-4"
                >
                  <h3 className="text-sm font-semibold text-emerald-900">
                    {item.title}
                  </h3>
                  <p className="mt-1 text-sm leading-relaxed text-slate-600">
                    {item.text}
                  </p>
                </div>
              ))}
            </div>
          </motion.section>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
