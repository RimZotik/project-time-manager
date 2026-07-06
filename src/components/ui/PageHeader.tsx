// Заголовок страницы с волной-акцентом в стиле эко-темы.
import type { ReactNode } from "react";
import { motion } from "framer-motion";

export default function PageHeader({
  title,
  subtitle,
  actions,
}: {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
}) {
  return (
    <div className="relative px-8 pb-4 pt-7">
      <div className="relative flex items-end justify-between gap-4">
        <div className="min-w-0">
          <motion.h1
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.24 }}
            className="truncate text-2xl font-semibold text-slate-900"
          >
            {title}
          </motion.h1>
          {subtitle && (
            <p className="mt-1 truncate text-sm text-slate-500">{subtitle}</p>
          )}
        </div>
        {actions && <div className="flex shrink-0 items-center gap-2">{actions}</div>}
      </div>
    </div>
  );
}
