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
 *
 * Хранит состояние активной вкладки и счётчик изменений данных:
 * после любой мутации (создание/редактирование/закрытие проекта и т.п.)
 * `dataVersion` инкрементируется, благодаря чему открытые вкладки
 * перезагружают свои данные через useEffect({}, [dataVersion]).
 */
function App() {
  const [activeTab, setActiveTab] = useState(DEFAULT_TAB);
  const [dataVersion, setDataVersion] = useState(0);

  /** Уведомить панели о том, что данные изменились (нужен перезапрос). */
  function onDataChange() {
    setDataVersion((version) => version + 1);
  }

  return (
    <ConfigProvider theme={APP_THEME}>
      <div className="app">
        <TopPanel
          activeTab={activeTab}
          onTabChange={setActiveTab}
          onDataChange={onDataChange}
        />
        <SidePanel activeTab={activeTab} dataVersion={dataVersion} />
        <CentralPanel activeTab={activeTab} dataVersion={dataVersion} />
      </div>
    </ConfigProvider>
  );
}

export default App;
