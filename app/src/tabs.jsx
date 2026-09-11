import {
  BarChartOutlined,
  CheckSquareOutlined,
  ProjectOutlined,
  TeamOutlined,
} from "@ant-design/icons";

/**
 * Идентификаторы вкладок приложения.
 * @typedef {'project'|'tasks'|'resources'|'gantt'} TabKey
 */

/**
 * Реестр вкладок — единый источник истины для TopPanel, SidePanel и CentralPanel.
 * @type {Record<TabKey, { key: TabKey; label: string; icon: React.ReactNode }>}
 */
export const TABS = {
  project: { key: "project", label: "Проект", icon: <ProjectOutlined /> },
  tasks: { key: "tasks", label: "Задачи", icon: <CheckSquareOutlined /> },
  resources: { key: "resources", label: "Ресурсы", icon: <TeamOutlined /> },
  gantt: { key: "gantt", label: "Диаграмма Ганта", icon: <BarChartOutlined /> },
};

/** Порядок следования вкладок в интерфейсе. @type {TabKey[]} */
export const TAB_ORDER = ["project", "tasks", "resources", "gantt"];
