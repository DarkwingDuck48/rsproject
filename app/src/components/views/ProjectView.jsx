import {
  Button,
  Card,
  Descriptions,
  Divider,
  Empty,
  Progress,
  Space,
  Spin,
  Statistic,
  Tag,
  Typography,
} from "antd";
import {
  CalendarOutlined,
  CheckSquareOutlined,
  EditOutlined,
  FieldTimeOutlined,
  MoneyCollectOutlined,
  ProjectOutlined,
  TeamOutlined,
  ThunderboltOutlined,
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
import { formatDate, pluralDays, formatNumber } from "../../lib/format";
import { TASK_STATUS_LABELS } from "../../lib/options";
import EditProjectDialog from "../dialogs/EditProjectDialog";

/** Статусы, которые считаем «завершёнными» для прогресса. */
const DONE_STATUSES = new Set(["Complete", "Closed"]);

/** Процент от a/b, округлённый до целого (b = 0 → 0). */
function percent(a, b) {
  return b > 0 ? Math.round((a / b) * 100) : 0;
}

/**
 * Собирает статистику по дереву задач за один проход:
 * всего/сводных/выполнено, статусы, назначения ресурсов.
 * @param {TaskTreeNode[]} nodes
 * @returns {{
 *   total: number, summaries: number, workItems: number,
 *   done: number,
 *   statuses: Record<string, number>,
 *   allocationSlots: number, tasksWithAllocations: number
 * }}
 */
function collectTaskStats(nodes) {
  const stats = {
    total: 0,
    summaries: 0,
    workItems: 0,
    done: 0,
    statuses: {},
    allocationSlots: 0,
    tasksWithAllocations: 0,
  };
  // Обходим дерево итеративно: вложенность задач не ограничена,
  // и рекурсия могла бы переполнить стек на глубоких деревьях.
  const stack = [...nodes];
  while (stack.length) {
    const node = stack.pop();
    stats.total += 1;
    const task = node.task;
    if (task.is_summary) {
      stats.summaries += 1;
    } else {
      stats.workItems += 1;
      stats.statuses[task.status] = (stats.statuses[task.status] ?? 0) + 1;
      if (DONE_STATUSES.has(task.status)) stats.done += 1;
    }
    stats.allocationSlots += task.allocations_count ?? 0;
    if (task.allocations_count > 0) stats.tasksWithAllocations += 1;
    stack.push(...(node.children ?? []));
  }
  return stats;
}

/**
 * Собирает статистику по пулу ресурсов.
 * @param {ResourceInfo[]} resources
 * @returns {{ count: number, withPeriods: number, periods: number }}
 */
function collectResourceStats(resources) {
  let withPeriods = 0;
  let periods = 0;
  for (const resource of resources) {
    const n = resource.unavailable_periods?.length ?? 0;
    if (n > 0) withPeriods += 1;
    periods += n;
  }
  return { count: resources.length, withPeriods, periods };
}

/**
 * Вкладка «Проект»: просмотр информации о проекте, редактирование через
 * EditProjectDialog и аналитика (задачи, ресурсы, критический путь).
 *
 * Данные — через обёртки `lib/api.js`: `get_project_info`, `get_task_tree`,
 * `get_resources`, `get_tasks`, `get_critical_path`. Вся аналитика считается
 * на фронтенде из этих данных; бэкенд не менялся.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения.
 */
export default function ProjectView({ dataVersion }) {
  /** @type {ProjectInfo|null} Данные текущего проекта. */
  const [project, setProject] = useState(null);
  /** Аналитика по проекту (null, пока проект не загрузился). */
  const [analytics, setAnalytics] = useState(null);
  /** @type {string|null} Ошибка загрузки (raw-сообщение от бэкенда). */
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(false);
  const [editOpen, setEditOpen] = useState(false);

  useEffect(() => {
    void loadProject();
    // loadProject стабильна — запускаем на монтировании и после изменений данных.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dataVersion]);

  /**
   * Загружает информацию о проекте и аналитику через Tauri-команды.
   * При монтировании вьюхи (переключении на вкладку) всегда свежие данные.
   */
  async function loadProject() {
    setLoading(true);
    try {
      const info = await getProjectInfo();
      // Задачи/ресурсы/путь загружаем параллельно — независимые запросы.
      const [tree, resources, criticalPath, tasks] = await Promise.all([
        getTaskTree(),
        getResources(),
        getCriticalPath(),
        getTasks(),
      ]);
      setProject(info);
      setAnalytics({
        tasks: collectTaskStats(tree),
        resources: collectResourceStats(resources),
        path: criticalPathSpan(criticalPath, tasks),
      });
      setError(null);
    } catch (err) {
      setProject(null);
      setAnalytics(null);
      setError(typeof err === "string" ? err : "Неизвестная ошибка");
    } finally {
      setLoading(false);
    }
  }

  /** Пустое состояние: проекта нет, загрузка не удалась или аналитика не готова. */
  if (!project || !analytics) {
    const noProject = !error || isNoProjectError(error);
    return (
      <section className="view">
        <Typography.Title level={3}>Проект</Typography.Title>
        <Card>
          {loading ? (
            <div className="views-loading">
              <Spin />
            </div>
          ) : (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={
                noProject
                  ? "Проект не создан. Создайте или откройте проект через меню «Файл»."
                  : "Не удалось загрузить данные проекта."
              }
            >
              {!noProject && (
                <Typography.Paragraph type="secondary">
                  {error}
                </Typography.Paragraph>
              )}
            </Empty>
          )}
        </Card>
      </section>
    );
  }

  const { tasks, resources, path } = analytics;
  const donePercent = percent(tasks.done, tasks.workItems);
  const allocatedPercent = percent(tasks.tasksWithAllocations, tasks.workItems);
  // Убираем нулевые статусы, чтобы не тащить мусорные теги в карточку.
  const statusRows = Object.entries(tasks.statuses)
    .filter(([, count]) => count > 0)
    .map(([status, count]) => ({ status, count }));

  return (
    <section className="view">
      <Typography.Title level={3}>Проект</Typography.Title>

      {/* Имя проекта — заголовок карточки, действие «Редактировать» — в extra
          (справа): так действие видно сразу, а не внизу
          (паттерн antd: «заголовок + действия»). */}
      <Card
        title={
          <Space size={8}>
            <ProjectOutlined className="project-view__title-icon" />
            <span>{project.name}</span>
          </Space>
        }
        extra={
          <Button
            type="primary"
            icon={<EditOutlined />}
            onClick={() => setEditOpen(true)}
          >
            Редактировать
          </Button>
        }
      >
        {/* Описание показываем только если оно есть — пустой плейсхолдер
            «—» только добавлял шум. */}
        {project.description && (
          <Typography.Paragraph type="secondary">
            {project.description}
          </Typography.Paragraph>
        )}
        {project.description && <Divider />}

        {/* Ключевые факты проекта. Descriptions — нативный компонент antd
            для страниц «сведений». column — адаптивный: на узких окнах
            один столбец, начиная с sm (>576px) — два (ресайзинг, 5.24). */}
        <Descriptions size="middle" column={{ xs: 1, sm: 2 }}>
          <Descriptions.Item label="Дата начала">
            <Space size={6}>
              <CalendarOutlined className="project-view__fact-icon" />
              {formatDate(project.date_start)}
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="Дата окончания">
            <Space size={6}>
              <CalendarOutlined className="project-view__fact-icon" />
              {formatDate(project.date_end)}
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="Длительность">
            <Space size={6}>
              <FieldTimeOutlined className="project-view__fact-icon" />
              {pluralDays(project.duration_days)}
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="Стоимость">
            <Space size={6}>
              <MoneyCollectOutlined className="project-view__fact-icon" />
              {formatNumber(project.project_cost)}
            </Space>
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {/* Аналитика: три карточки в адаптивной сетке (CSS Grid auto-fit —
          на узком окне становится один столбец, ресайзинг 5.24). */}
      <div className="project-view__analytics">
        {/* Задачи */}
        <Card
          size="small"
          title={
            <Space size={6}>
              <CheckSquareOutlined className="project-view__fact-icon" />
              Задачи
            </Space>
          }
        >
          <Statistic title="Всего задач" value={tasks.total} suffix=" шт." />
          {tasks.workItems > 0 && (
            <div className="project-view__analytics-progress">
              <Progress
                percent={donePercent}
                size="small"
                status={donePercent === 100 ? "success" : "normal"}
              />
              <Typography.Text type="secondary">
                Выполнено: {tasks.done} из {tasks.workItems} работ
                {tasks.summaries ? ` (сводных: ${tasks.summaries})` : ""}
              </Typography.Text>
            </div>
          )}
          {statusRows.length > 0 && (
            <Space size={4} wrap>
              {statusRows.map(({ status, count }) => (
                <Tag key={status}>
                  {TASK_STATUS_LABELS[status] ?? status}: {count}
                </Tag>
              ))}
            </Space>
          )}
        </Card>

        {/* Ресурсы */}
        <Card
          size="small"
          title={
            <Space size={6}>
              <TeamOutlined className="project-view__fact-icon" />
              Ресурсы
            </Space>
          }
        >
          <Statistic title="В пуле" value={resources.count} suffix=" шт." />
          {tasks.workItems > 0 && (
            <div className="project-view__analytics-progress">
              <Progress
                percent={allocatedPercent}
                size="small"
                status={allocatedPercent === 100 ? "success" : "normal"}
              />
              <Typography.Text type="secondary">
                Назначено на {tasks.tasksWithAllocations} из {tasks.workItems}{" "}
                работ · слотов занято: {tasks.allocationSlots}
              </Typography.Text>
            </div>
          )}
          <Typography.Text type="secondary">
            Периодов недоступности: {resources.periods}
            {resources.withPeriods
              ? ` у ${resources.withPeriods} ресурсов`
              : ""}
          </Typography.Text>
        </Card>

        {/* Критический путь */}
        <Card
          size="small"
          title={
            <Space size={6}>
              <ThunderboltOutlined className="project-view__fact-icon" />
              Критический путь
            </Space>
          }
        >
          {path ? (
            <>
              <Statistic title="Длительность" value={path.days} suffix=" дн." />
              <div className="project-view__analytics-progress">
                <Typography.Text type="secondary">
                  Задач на пути: {path.count}
                </Typography.Text>
                {path.names.length > 0 && (
                  <Space size={4} wrap>
                    {path.names.map((name, i) => (
                      <Tag key={i} color="red">
                        {name}
                      </Tag>
                    ))}
                    {path.more > 0 && <Tag>+ {path.more}</Tag>}
                  </Space>
                )}
              </div>
            </>
          ) : (
            <Typography.Text type="secondary">
              Путь не рассчитан. Откройте вкладку «Задачи» и нажмите «Рассчитать
              критический путь».
            </Typography.Text>
          )}
        </Card>
      </div>

      <EditProjectDialog
        open={editOpen}
        project={project}
        onClose={() => setEditOpen(false)}
        onSaved={loadProject}
      />
    </section>
  );
}
