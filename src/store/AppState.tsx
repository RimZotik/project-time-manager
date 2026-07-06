// Общий контекст состояния приложения для оболочки и новых страниц.
// Опрашивает backend раз в 1.5с (как и рабочий экран) и раздаёт данные +
// текущий язык. ProjectWorkspace пока держит собственное состояние —
// оба поллера читают один и тот же backend, поэтому расхождений нет.
import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { getAppState } from "../lib/api";
import { fallbackState, type AppState, type Language } from "../lib/types";

type AppStateContextValue = {
  state: AppState;
  language: Language;
  refresh: () => Promise<void>;
};

const AppStateContext = createContext<AppStateContextValue | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AppState>(fallbackState);

  async function refresh() {
    const next = await getAppState();
    setState(next);
  }

  useEffect(() => {
    refresh();
    const timer = window.setInterval(refresh, 1500);
    return () => window.clearInterval(timer);
  }, []);

  const language: Language = state.settings?.language === "en" ? "en" : "ru";

  return (
    <AppStateContext.Provider value={{ state, language, refresh }}>
      {children}
    </AppStateContext.Provider>
  );
}

export function useAppState(): AppStateContextValue {
  const ctx = useContext(AppStateContext);
  if (!ctx) {
    throw new Error("useAppState must be used within AppStateProvider");
  }
  return ctx;
}
