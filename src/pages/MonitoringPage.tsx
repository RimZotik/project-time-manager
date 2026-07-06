import { MonitorPlay } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import PlaceholderPage from "../components/ui/PlaceholderPage";

export default function MonitoringPage() {
  const { language } = useAppState();
  const p = shellCopy[language].pages.monitoring;
  return (
    <PlaceholderPage
      icon={MonitorPlay}
      title={p.title}
      subtitle={p.subtitle}
      description={p.placeholder}
    />
  );
}
