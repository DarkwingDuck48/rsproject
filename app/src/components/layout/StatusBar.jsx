import { Space, message } from "antd";
import {
  CheckSquareOutlined,
  FieldTimeOutlined,
  SaveOutlined,
  TeamOutlined,
} from "@ant-design/icons";
import { useEffect, useState } from "react";
import {
  getCriticalPath,
  getProjectInfo,
  getResources,
  getTaskTree,
  getTasks,
} from "../../lib/api";
import { isNoProjectError } from "../../lib/errors";
import { criticalPathSpan } from "../../lib/analytics";
import { formatDateTime, pluralDays } from "../../lib/format";

/** Количество узлов в дереве задач (задача + все дочерние). */
function countTaskNodes(nodes) {
  let count = 0;
  for (const node of nodes) {
    count += 1 + countTaskNodes(node.children ?? []);
  }
  return count;
}

/**
 * Статус-бар (задача 5.18): нижняя панель со сводкой по открытому проекту —
 * количество задач и ресурсов, длительность критического пути
 * и дата последнего сохранения.
 *
 * Задачи/ресурсы и критический путь перечитываются при каждом изменении
 * данных (`dataVersion`). Дата сохранения приходит из App: TopPanel
 * сообщает о каждом успешном `save_project`.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения.
 * @param {?Date}  props.lastSavedAt - Момент последнего успешного сохранения.
 */
export default function StatusBar({ dataVersion, lastSavedAt }) {
  const [stats, setStats] = useState({
    projectOpen: false,
    taskCount: 0,
    resourceCount: 0,
    criticalDurationDays: null,
  });

  useEffect(() => {
    let cancelled = false;

    async function loadStats() {
      try {
        const info = await getProjectInfo();
        if (cancelled) return;
        if (!info) {
          // Проект не открыт — показываем прочерки.
          setStats({
            projectOpen: false,
            taskCount: 0,
            resourceCount: 0,
            criticalDurationDays: null,
          });
          return;
        }

        const tree = await getTaskTree();
        const resources = await getResources();

        // Критический путь может быть ещё не рассчитан (бэкенд отдаёт null).
        let criticalDurationDays = null;
        const criticalPath = await getCriticalPath();
        if (criticalPath && criticalPath.length > 0) {
          // Задачи нужны только если путь есть — тянем их лениво.
          const span = criticalPathSpan(criticalPath, await getTasks());
          criticalDurationDays = span?.days ?? null;
        }

        if (cancelled) return;
        setStats({
          projectOpen: true,
          taskCount: countTaskNodes(tree),
          resourceCount: resources.length,
          criticalDurationDays,
        });
      } catch (err) {
        if (cancelled) return;
        setStats({
          projectOpen: false,
          taskCount: 0,
          resourceCount: 0,
          criticalDurationDays: null,
        });
        if (!isNoProjectError(err)) {
          message.error(`Не удалось загрузить статус-бар: ${err}`);
        }
      }
    }

    void loadStats();
    return () => {
      cancelled = true;
    };
  }, [dataVersion]);

  const { projectOpen, taskCount, resourceCount, criticalDurationDays } = stats;
  const criticalText =
    criticalDurationDays === null ? "—" : pluralDays(criticalDurationDays);

  return (
    <footer className="app-status-bar">
      <Space size={12} className="app-status-bar__stats" wrap>
        <span className="app-status-bar__item">
          <CheckSquareOutlined />
          Задачи: {projectOpen ? taskCount : "—"}
        </span>
        <span className="app-status-bar__item">
          <TeamOutlined />
          Ресурсы: {projectOpen ? resourceCount : "—"}
        </span>
        <span className="app-status-bar__item">
          <FieldTimeOutlined />
          Крит. путь: {projectOpen ? criticalText : "—"}
        </span>
      </Space>
      <span className="app-status-bar__spacer" />
      <span className="app-status-bar__item">
        <SaveOutlined />
        Сохранено: {projectOpen ? formatDateTime(lastSavedAt) : "—"}
      </span>
    </footer>
  );
}
