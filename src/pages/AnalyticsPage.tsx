import { BarChart3 } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import PlaceholderPage from "../components/ui/PlaceholderPage";

export default function AnalyticsPage() {
  const { language } = useAppState();
  const p = shellCopy[language].pages.analytics;
  return (
    <PlaceholderPage
      icon={BarChart3}
      title={p.title}
      subtitle={p.subtitle}
      description={p.placeholder}
    />
  );
}
