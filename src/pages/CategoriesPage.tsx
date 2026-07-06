import { Tags } from "lucide-react";
import { useAppState } from "../store/AppState";
import { shellCopy } from "../lib/i18n";
import PlaceholderPage from "../components/ui/PlaceholderPage";

export default function CategoriesPage() {
  const { language } = useAppState();
  const p = shellCopy[language].pages.categories;
  return (
    <PlaceholderPage
      icon={Tags}
      title={p.title}
      subtitle={p.subtitle}
      description={p.placeholder}
    />
  );
}
