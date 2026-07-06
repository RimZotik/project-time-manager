import { Settings } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import PlaceholderPage from "../components/ui/PlaceholderPage";

export default function SettingsPage() {
  const { language } = useAppState();
  const t = shellCopy[language];
  const description =
    language === "en"
      ? "Language, autostart, data folder and theme will move here from the modal window."
      : "Сюда переедут язык, автозапуск, папка данных и тема из модального окна.";
  return (
    <PlaceholderPage
      icon={Settings}
      title={t.nav.settings}
      subtitle={t.appTagline}
      description={description}
    />
  );
}
