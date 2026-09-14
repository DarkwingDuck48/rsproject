import { ConfigProvider, theme } from "antd";
import { useState } from "react";
import CentralPanel from "./components/layout/CentralPanel";
import SidePanel from "./components/layout/SidePanel";
import StatusBar from "./components/layout/StatusBar";
import TopPanel from "./components/layout/TopPanel";
import { SelectionProvider } from "./context/SelectionContext";
import { useTheme } from "./hooks/useTheme";
import "./App.css";

/** Вкладка приложения, активная по умолчанию. */
const DEFAULT_TAB = "project";

/**
 * Алгоритмы antd для светлой и тёмной темы (задача 5.16).
 * Переключение темы пересобирает CSS-переменные `--ant-*` на лету.
 */
const APP_THEMES = {
  light: { algorithm: theme.defaultAlgorithm },
  dark: { algorithm: theme.darkAlgorithm },
};

/**
 * Корневой компонент приложения.
 *
 * Хранит состояние активной вкладки, счётчик изменений данных, тему
 * (5.16) и дату последнего сохранения (5.18). После любой мутации
 * (создание/редактирование/закрытие проекта и т.п.) `dataVersion`
 * инкрементируется, благодаря чему открытые вкладки перезагружают
 * свои данные через useEffect({}, [dataVersion]).
 */
function App() {
  const [activeTab, setActiveTab] = useState(DEFAULT_TAB);
  const [dataVersion, setDataVersion] = useState(0);
  const { themeMode, toggleTheme } = useTheme();
  /** Момент последнего успешного сохранения проекта (null — ещё не сохраняли). */
  const [lastSavedAt, setLastSavedAt] = useState(null);

  /** Уведомить панели о том, что данные изменились (нужен перезапрос). */
  function onDataChange() {
    setDataVersion((version) => version + 1);
  }

  return (
    <ConfigProvider theme={APP_THEMES[themeMode]}>
      <SelectionProvider>
        <div className="app" data-theme={themeMode}>
          <TopPanel
            activeTab={activeTab}
            onTabChange={setActiveTab}
            onDataChange={onDataChange}
            themeMode={themeMode}
            onThemeToggle={toggleTheme}
            onProjectSaved={() => setLastSavedAt(new Date())}
          />
          <SidePanel activeTab={activeTab} dataVersion={dataVersion} />
          <CentralPanel
            activeTab={activeTab}
            dataVersion={dataVersion}
            onDataChange={onDataChange}
          />
          <StatusBar dataVersion={dataVersion} lastSavedAt={lastSavedAt} />
        </div>
      </SelectionProvider>
    </ConfigProvider>
  );
}

export default App;
