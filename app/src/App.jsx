import { ConfigProvider } from "antd";
import { theme } from "antd";
import { useState } from "react";
import CentralPanel from "./components/layout/CentralPanel";
import SidePanel from "./components/layout/SidePanel";
import TopPanel from "./components/layout/TopPanel";
import "./App.css";

/** Вкладка приложения, активная по умолчанию. */
const DEFAULT_TAB = "project";

/**
 * Тема antd. До задачи 5.16 явно фиксируем светлую —
 * с переключателем темы будем менять этот алгоритм.
 */
const APP_THEME = { algorithm: theme.defaultAlgorithm };

/**
 * Корневой компонент приложения.
 * Хранит состояние активной вкладки (React state) и собирает layout из трёх панелей.
 */
function App() {
  const [activeTab, setActiveTab] = useState(DEFAULT_TAB);

  return (
    <ConfigProvider theme={APP_THEME}>
      <div className="app">
        <TopPanel activeTab={activeTab} onTabChange={setActiveTab} />
        <SidePanel activeTab={activeTab} />
        <CentralPanel activeTab={activeTab} />
      </div>
    </ConfigProvider>
  );
}

export default App;
