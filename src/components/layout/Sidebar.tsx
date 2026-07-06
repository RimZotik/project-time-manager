// Боковое меню: сворачиваемое, с пружинными анимациями. Активный пункт
// подсвечивается «переезжающей» подложкой (framer-motion layoutId) в стиле
// плавных Apple-переходов.
import { NavLink } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import {
  BarChart3,
  ChevronLeft,
  FolderKanban,
  LayoutDashboard,
  MonitorPlay,
  Settings,
  Tags,
  Timer,
  type LucideIcon,
} from "lucide-react";
import { useAppState } from "../../store/AppState";
import { shellCopy } from "../../lib/i18n";

type NavItem = {
  to: string;
  key: keyof typeof shellCopy.ru.nav;
  icon: LucideIcon;
  end?: boolean;
};

const NAV: NavItem[] = [
  { to: "/", key: "dashboard", icon: LayoutDashboard, end: true },
  { to: "/projects", key: "projects", icon: FolderKanban },
  { to: "/analytics", key: "analytics", icon: BarChart3 },
  { to: "/monitoring", key: "monitoring", icon: MonitorPlay },
  { to: "/categories", key: "categories", icon: Tags },
  { to: "/settings", key: "settings", icon: Settings },
];

export default function Sidebar({
  collapsed,
  onToggle,
}: {
  collapsed: boolean;
  onToggle: () => void;
}) {
  const { language } = useAppState();
  const t = shellCopy[language];

  return (
    <motion.aside
      initial={false}
      animate={{ width: collapsed ? 76 : 248 }}
      transition={{ type: "spring", stiffness: 360, damping: 32 }}
      className="relative z-20 flex h-full shrink-0 flex-col border-r border-emerald-100/80 bg-white/70 backdrop-blur-xl"
    >
      {/* Логотип */}
      <div className="flex items-center gap-3 px-4 py-5">
        <div className="grid size-11 shrink-0 place-items-center rounded-2xl bg-emerald-600 text-white shadow-[0_10px_22px_rgba(5,150,105,0.25)]">
          <Timer className="size-6" />
        </div>
        <AnimatePresence initial={false}>
          {!collapsed && (
            <motion.div
              initial={{ opacity: 0, x: -8 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -8 }}
              transition={{ duration: 0.18 }}
              className="min-w-0"
            >
              <p className="truncate text-sm font-semibold text-slate-900">
                {t.appName}
              </p>
              <p className="truncate text-xs text-emerald-700/80">
                {t.appTagline}
              </p>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Навигация */}
      <nav className="flex flex-1 flex-col gap-1 px-3">
        {NAV.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end}
              title={collapsed ? t.nav[item.key] : undefined}
              className="group relative flex items-center gap-3 rounded-2xl px-3 py-2.5 outline-none"
            >
              {({ isActive }) => (
                <>
                  {isActive && (
                    <motion.span
                      layoutId="nav-active"
                      transition={{
                        type: "spring",
                        stiffness: 420,
                        damping: 34,
                      }}
                      className="absolute inset-0 rounded-2xl bg-emerald-50 ring-1 ring-emerald-100"
                    />
                  )}
                  <Icon
                    className={`relative size-5 shrink-0 transition-colors ${
                      isActive
                        ? "text-emerald-700"
                        : "text-slate-400 group-hover:text-emerald-600"
                    }`}
                  />
                  <AnimatePresence initial={false}>
                    {!collapsed && (
                      <motion.span
                        initial={{ opacity: 0, x: -6 }}
                        animate={{ opacity: 1, x: 0 }}
                        exit={{ opacity: 0, x: -6 }}
                        transition={{ duration: 0.16 }}
                        className={`relative truncate text-sm font-medium ${
                          isActive ? "text-emerald-900" : "text-slate-600"
                        }`}
                      >
                        {t.nav[item.key]}
                      </motion.span>
                    )}
                  </AnimatePresence>
                </>
              )}
            </NavLink>
          );
        })}
      </nav>

      {/* Кнопка сворачивания */}
      <div className="p-3">
        <button
          type="button"
          onClick={onToggle}
          title={collapsed ? t.expand : t.collapse}
          className="flex w-full items-center gap-3 rounded-2xl px-3 py-2.5 text-slate-400 transition-colors hover:bg-emerald-50 hover:text-emerald-600"
        >
          <motion.span
            animate={{ rotate: collapsed ? 180 : 0 }}
            transition={{ type: "spring", stiffness: 360, damping: 30 }}
            className="grid place-items-center"
          >
            <ChevronLeft className="size-5 shrink-0" />
          </motion.span>
          {!collapsed && (
            <span className="truncate text-sm font-medium">{t.collapse}</span>
          )}
        </button>
      </div>
    </motion.aside>
  );
}
