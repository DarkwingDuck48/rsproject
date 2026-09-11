import GanttView from "../views/GanttView";
import ProjectView from "../views/ProjectView";
import ResourcesView from "../views/ResourcesView";
import TasksView from "../views/TasksView";

/** Соответствие вкладка → компонент-вью. */
const VIEWS = {
  project: ProjectView,
  tasks: TasksView,
  resources: ResourcesView,
  gantt: GanttView,
};

/**
 * Центральная панель: основной контент активной вкладки.
 *
 * @param {Object} props
 * @param {string} props.activeTab - Текущая активная вкладка (TabKey).
 */
export default function CentralPanel({ activeTab }) {
  const ActiveView = VIEWS[activeTab] ?? ProjectView;

  return (
    <main className="app-central-panel">
      <ActiveView />
    </main>
  );
}
