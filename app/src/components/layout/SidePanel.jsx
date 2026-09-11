import { Empty } from "antd";
import { TABS } from "../../tabs";

/** Пустые состояния списков по вкладкам (заполнятся в 5.11–5.13). */
const EMPTY_HINTS = {
  project: "Список проектов появится здесь",
  tasks: "Создайте проект и добавьте задачи",
  resources: "Добавьте ресурсы через меню",
  gantt: "Список задач сгруппируется здесь",
};

/**
 * Боковая панель: список элементов текущей вкладки.
 * В задаче 5.10 присутствует только каркас — содержимое списков
 * будет реализовано вместе с views (5.11–5.13).
 *
 * @param {Object} props
 * @param {string} props.activeTab - Текущая активная вкладка (TabKey).
 */
export default function SidePanel({ activeTab }) {
  const tab = TABS[activeTab] ?? TABS.project;

  return (
    <aside className="app-side-panel">
      <div className="app-side-panel__header">
        <span className="app-side-panel__header-icon">{tab.icon}</span>
        <span className="app-side-panel__header-title">{tab.label}</span>
      </div>

      <div className="app-side-panel__content">
        {/* TODO(5.11–5.13): список элементов текущей вкладки */}
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description={EMPTY_HINTS[tab.key]}
        />
      </div>
    </aside>
  );
}
