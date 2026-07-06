// Единая заглушка для разделов, которые ещё в разработке (Фаза 1 — каркас).
import type { LucideIcon } from "lucide-react";
import { motion } from "framer-motion";
import { useAppState } from "../../store/AppState";
import { shellCopy } from "../../lib/i18n";
import PageHeader from "./PageHeader";

export default function PlaceholderPage({
  icon: Icon,
  title,
  subtitle,
  description,
}: {
  icon: LucideIcon;
  title: string;
  subtitle: string;
  description: string;
}) {
  const { language } = useAppState();
  const t = shellCopy[language];

  return (
    <div className="flex h-full flex-col">
      <PageHeader title={title} subtitle={subtitle} />
      <div className="grid flex-1 place-items-center p-8">
        <motion.div
          initial={{ opacity: 0, y: 14, scale: 0.98 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ type: "spring", stiffness: 260, damping: 26 }}
          className="w-full max-w-md rounded-[28px] border border-emerald-100 bg-white/80 p-8 text-center shadow-[0_20px_60px_rgba(15,23,42,0.08)] backdrop-blur-xl"
        >
          <motion.div
            animate={{ y: [0, -6, 0] }}
            transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
            className="mx-auto grid size-16 place-items-center rounded-3xl bg-emerald-50 text-emerald-600"
          >
            <Icon className="size-8" />
          </motion.div>
          <span className="mt-5 inline-block rounded-full bg-emerald-100 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-emerald-700">
            {t.soon}
          </span>
          <h2 className="mt-3 text-lg font-semibold text-slate-900">
            {t.inDevelopment}
          </h2>
          <p className="mt-2 text-sm leading-relaxed text-slate-500">
            {description}
          </p>
        </motion.div>
      </div>
    </div>
  );
}
