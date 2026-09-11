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
import { formatDateTime, pluralDays } from "../../lib/format";

const DAY_MS = 86_400_000;

/** Количество узлов в дереве задач (задача + все дочерние). */
function countTaskNodes(nodes) {
  let count = 0;
  for (const node of nodes) {
    count += 1 + countTaskNodes(node.children ?? []);
  }
  return count;
}

/** Длительность интервала [startIso, endIso] включительно, в днях. */
function inclusiveDays(startIso, endIso) {
  const [y1, m1, d1] = startIso.slice(0, 10).split("-").map(Number);
  const [y2, m2, d2] = endIso.slice(0, 10).split("-").map(Number);
  return (
    Math.round((Date.UTC(y2, m2 - 1, d2) - Date.UTC(y1, m1 - 1, d1)) / DAY_MS) +
    1
  );
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
          const tasks = await getTasks();
          const byId = new Map(tasks.map((task) => [task.id, task]));
          const pathTasks = criticalPath
            .map((id) => byId.get(id))
            .filter((task) => task && task.date_start && task.date_end);
          if (pathTasks.length > 0) {
            // Длительность пути — промежуток от самого раннего начала
            // до самого позднего конца критических задач (включает лаги).
            const start = pathTasks.reduce((a, b) =>
              a.date_start < b.date_start ? a : b,
            ).date_start;
            const end = pathTasks.reduce((a, b) =>
              a.date_end > b.date_end ? a : b,
            ).date_end;
            criticalDurationDays = inclusiveDays(start, end);
          }
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
