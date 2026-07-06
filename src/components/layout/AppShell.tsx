// Каркас приложения: фон с волнами + боковое меню + область страниц
// с мягкими переходами между разделами.
import { useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import WaveBackground from "./WaveBackground";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";

export default function AppShell() {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();

  return (
    <div className="relative flex h-screen min-h-[720px] overflow-hidden text-slate-900">
      <WaveBackground />
      <Sidebar collapsed={collapsed} onToggle={() => setCollapsed((v) => !v)} />

      <main className="relative z-10 flex min-w-0 flex-1 flex-col overflow-hidden">
        <TopBar />
        <div className="min-h-0 flex-1 overflow-hidden">
          <AnimatePresence mode="wait">
            <motion.div
              key={location.pathname}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              className="h-full min-h-0 overflow-hidden"
            >
              <Outlet />
            </motion.div>
          </AnimatePresence>
        </div>
      </main>
    </div>
  );
}
