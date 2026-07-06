// Локализация оболочки и новых страниц. Тексты самого рабочего экрана
// (ProjectWorkspace) пока живут внутри него; сюда вынесена навигация,
// заголовки страниц и общие подписи.
import type { Language } from "./types";

export const shellCopy = {
  ru: {
    appName: "Project Time Manager",
    appTagline: "Учёт времени по проектам",
    nav: {
      dashboard: "Дашборд",
      projects: "Проекты",
      analytics: "Аналитика",
      monitoring: "Мониторинг",
      categories: "Категории",
      settings: "Настройки",
    },
    collapse: "Свернуть меню",
    expand: "Развернуть меню",
    soon: "Скоро",
    inDevelopment: "Раздел в разработке",
    pages: {
      dashboard: {
        title: "Дашборд",
        subtitle: "Сводка по всем проектам и активности",
        placeholder:
          "Здесь появится общая сводка: суммарное время, топ-проекты, активность за неделю и средняя длительность сессии.",
      },
      analytics: {
        title: "Аналитика",
        subtitle: "Графики, сравнение проектов и средние по категориям",
        placeholder:
          "Здесь будут графики по проектам, этапам, приложениям и сайтам, тепловая карта активности и сравнение проектов между собой.",
      },
      monitoring: {
        title: "Мониторинг",
        subtitle: "Активное окно и приложения в реальном времени",
        placeholder:
          "Здесь появится живой монитор: активное окно прямо сейчас, счётчик текущей сессии и список всех отслеживаемых приложений.",
      },
      categories: {
        title: "Категории",
        subtitle: "Группировка проектов (Монтаж, Программирование…)",
        placeholder:
          "Здесь можно будет создавать категории с цветом и привязывать к ним проекты, чтобы смотреть аналитику по категориям.",
      },
    },
  },
  en: {
    appName: "Project Time Manager",
    appTagline: "Per-project time tracking",
    nav: {
      dashboard: "Dashboard",
      projects: "Projects",
      analytics: "Analytics",
      monitoring: "Monitoring",
      categories: "Categories",
      settings: "Settings",
    },
    collapse: "Collapse menu",
    expand: "Expand menu",
    soon: "Soon",
    inDevelopment: "Section in development",
    pages: {
      dashboard: {
        title: "Dashboard",
        subtitle: "Overview across all projects and activity",
        placeholder:
          "This will show a global overview: total time, top projects, weekly activity and average session length.",
      },
      analytics: {
        title: "Analytics",
        subtitle: "Charts, project comparison and category averages",
        placeholder:
          "This will hold charts by project, stage, app and site, an activity heatmap and cross-project comparison.",
      },
      monitoring: {
        title: "Monitoring",
        subtitle: "Active window and applications in real time",
        placeholder:
          "This will be a live monitor: the current active window, a running-session counter and the list of all tracked apps.",
      },
      categories: {
        title: "Categories",
        subtitle: "Group projects (Editing, Programming…)",
        placeholder:
          "Here you'll create colored categories and assign projects to them to view analytics per category.",
      },
    },
  },
} as const;

export type ShellCopy = (typeof shellCopy)[Language];
