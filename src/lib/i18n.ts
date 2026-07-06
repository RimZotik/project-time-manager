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
    recording: "Идёт запись",
    notRecording: "Запись не идёт",
    noProject: "Проект не выбран",
    helpButton: "Помощь",
    help: {
      title: "Как пользоваться",
      subtitle: "Основные действия: проекты, запись времени, категории и отчёты.",
      items: [
        {
          title: "Проекты и категории",
          text: "Создайте проект или импортируйте JSON в разделе «Проекты». Присвойте проекту категорию (например, Монтаж или Программирование), чтобы сравнивать по ней аналитику.",
        },
        {
          title: "Запись времени",
          text: "Выберите проект и нажмите «Старт». Приложение считает активные окна, приложения и сайты. Ставьте на паузу или стоп. Индикатор записи виден в правом верхнем углу.",
        },
        {
          title: "Этапы",
          text: "Заведите этапы проекта и выбирайте их перед стартом сессии — так время распределяется по этапам, а в отчёте виден таймлайн.",
        },
        {
          title: "Включение и исключение",
          text: "Снимайте галочки у приложений, сайтов или ссылок, которые не должны попадать в итог — данные не удаляются, просто не учитываются.",
        },
      ],
    },
    settings: {
      title: "Настройки",
      subtitle: "Системные параметры приложения",
      language: "Язык интерфейса",
      languageRu: "Русский",
      languageEn: "English",
      autostart: "Автозапуск с Windows",
      autostartHint: "Запускать приложение при входе в систему",
      dataFolder: "Папка данных",
      dataFolderHint: "Все проекты и база хранятся рядом с приложением",
      openFolder: "Открыть папку",
    },
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
    recording: "Recording",
    notRecording: "Not recording",
    noProject: "No project selected",
    helpButton: "Help",
    help: {
      title: "How to use",
      subtitle: "Core actions: projects, time tracking, categories and reports.",
      items: [
        {
          title: "Projects and categories",
          text: "Create a project or import JSON on the Projects page. Assign a category (e.g. Editing or Programming) to compare analytics by it.",
        },
        {
          title: "Time tracking",
          text: "Pick a project and press Start. The app tracks active windows, apps and sites. Pause or stop anytime. The recording indicator sits in the top-right corner.",
        },
        {
          title: "Stages",
          text: "Create project stages and pick them before starting a session, so time is split across stages and the report shows a timeline.",
        },
        {
          title: "Include and exclude",
          text: "Uncheck apps, sites or links that should not count toward totals — the data is kept, just not counted.",
        },
      ],
    },
    settings: {
      title: "Settings",
      subtitle: "System-level app preferences",
      language: "Interface language",
      languageRu: "Русский",
      languageEn: "English",
      autostart: "Launch on Windows startup",
      autostartHint: "Start the app when you sign in",
      dataFolder: "Data folder",
      dataFolderHint: "All projects and the database live next to the app",
      openFolder: "Open folder",
    },
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
