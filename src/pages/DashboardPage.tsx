import { LayoutDashboard } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import PlaceholderPage from "../components/ui/PlaceholderPage";

export default function DashboardPage() {
  const { language } = useAppState();
  const p = shellCopy[language].pages.dashboard;
  return (
    <PlaceholderPage
      icon={LayoutDashboard}
      title={p.title}
      subtitle={p.subtitle}
      description={p.placeholder}
    />
  );
}
