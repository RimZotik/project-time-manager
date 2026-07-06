// Корень приложения: провайдер состояния + hash-роутинг + каркас с боковым
// меню. Разделы — отдельные страницы; рабочий экран проекта (перенесённый
// монолит) живёт на маршруте /projects.
import { HashRouter, Route, Routes } from "react-router-dom";
import { AppStateProvider } from "./store/AppState";
import AppShell from "./components/layout/AppShell";
import DashboardPage from "./pages/DashboardPage";
import ProjectWorkspace from "./pages/ProjectWorkspace";
import AnalyticsPage from "./pages/AnalyticsPage";
import MonitoringPage from "./pages/MonitoringPage";
import CategoriesPage from "./pages/CategoriesPage";
import SettingsPage from "./pages/SettingsPage";

export default function App() {
  return (
    <AppStateProvider>
      <HashRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<DashboardPage />} />
            <Route path="projects" element={<ProjectWorkspace />} />
            <Route path="analytics" element={<AnalyticsPage />} />
            <Route path="monitoring" element={<MonitoringPage />} />
            <Route path="categories" element={<CategoriesPage />} />
            <Route path="settings" element={<SettingsPage />} />
          </Route>
        </Routes>
      </HashRouter>
    </AppStateProvider>
  );
}
