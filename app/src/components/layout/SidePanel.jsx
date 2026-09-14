import { Empty, Space, Typography, message } from "antd";
import { useEffect, useState } from "react";
import { getProjectInfo, getResources, getTaskTree } from "../../lib/api";
import { isNoProjectError } from "../../lib/errors";
import { formatDate } from "../../lib/format";
import { useSelection } from "../../context/SelectionContext";
import { TABS } from "../../tabs";

/** Пустые состояния списков по вкладкам. */
const EMPTY_HINTS = {
  project: "Создайте или откройте проект через меню «Файл»",
  tasks: "Создайте проект и добавьте задачи",
  resources: "Ресурсов пока нет",
  gantt: "Добавьте задачи во вкладке «Задачи»",
};

/** Максимум элементов, показываемых в списке вкладки. */
const MAX_LIST_ITEMS = 200;

/**
 * Дерево задач → плоский список { id, name, depth, isSummary } для отображения.
 * @param {TaskTreeNode[]} nodes
 * @param {number} depth
 * @param {Array<{id: string, name: string, depth: number, isSummary: boolean}>} out
 * @returns {Array<{id: string, name: string, depth: number, isSummary: boolean}>}
 */
function flattenTaskNames(nodes, depth = 0, out = []) {
  for (const node of nodes) {
    out.push({
      id: node.task.id,
      name: node.task.name,
      isSummary: node.task.is_summary,
      depth,
    });
    flattenTaskNames(node.children, depth + 1, out);
  }
  return out;
}

/**
 * Боковая панель: список элементов текущей вкладки (проект / задачи / ресурсы).
 *
 * Клик по задаче/ресурсу выделяет её в таблице центральной панели (задача
 * 5.21): выделение живёт в общем SelectionContext, поэтому обе стороны
 * (сайдбар и таблица) всегда синхронизированы.
 *
 * @param {Object} props
 * @param {string} props.activeTab   - Текущая активная вкладка (TabKey).
 * @param {number} props.dataVersion - Счётчик изменений данных (для перезагрузки).
 */
export default function SidePanel({ activeTab, dataVersion }) {
  const { selectedTaskId, selectTask, selectedResourceId, selectResource } =
    useSelection();
  const [items, setItems] = useState([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    let cancelled = false;

    const loadTab = async () => {
      try {
        if (activeTab === "project") {
          const info = await getProjectInfo();
          if (cancelled) return;
          if (info) {
            setItems([
              {
                name: info.name,
                sub: `${formatDate(info.date_start)} – ${formatDate(info.date_end)}`,
                depth: 0,
              },
            ]);
          } else {
            setItems([]);
          }
        } else if (activeTab === "tasks" || activeTab === "gantt") {
          const tree = await getTaskTree();
          if (cancelled) return;
          setItems(flattenTaskNames(tree));
        } else if (activeTab === "resources") {
          const resources = await getResources();
          if (cancelled) return;
          setItems(
            resources.map((resource) => ({
              id: resource.id,
              name: resource.name,
              depth: 0,
            })),
          );
        } else {
          setItems([]);
        }
      } catch (err) {
        setItems([]);
        if (!isNoProjectError(err)) {
          message.error(`Не удалось загрузить список: ${err}`);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    void loadTab();
    return () => {
      cancelled = true;
    };
  }, [activeTab, dataVersion]);

  const tab = TABS[activeTab] ?? TABS.project;
  // Вкладки «Задачи» и «Ганта» показывают одно дерево задач → одно выделение.
  const isTaskTab = tab.key === "tasks" || tab.key === "gantt";
  const visible = items.slice(0, MAX_LIST_ITEMS);

  return (
    <aside className="app-side-panel">
      <div className="app-side-panel__header">
        <span className="app-side-panel__header-icon">{tab.icon}</span>
        <span className="app-side-panel__header-title">{tab.label}</span>
      </div>

      <div className="app-side-panel__content">
        {loading ? (
          <Typography.Text type="secondary">Загрузка…</Typography.Text>
        ) : visible.length === 0 ? (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description={EMPTY_HINTS[tab.key]}
          />
        ) : (
          <ul className="side-list">
            {visible.map((item, index) => {
              const selected = isTaskTab
                ? selectedTaskId === item.id
                : selectedResourceId === item.id;
              const className = [
                "side-list__item",
                item.isSummary && "side-list__item--summary",
                item.id && "side-list__item--clickable",
                selected && "side-list__item--selected",
              ]
                .filter(Boolean)
                .join(" ");
              return (
                <li
                  key={`${index}-${item.name}`}
                  className={className}
                  style={{ paddingLeft: 8 + item.depth * 14 }}
                  title={item.name}
                  onClick={() => {
                    // У проекта ID нет — клик по нему ничего не выделяет.
                    if (!item.id) return;
                    if (isTaskTab) selectTask(item.id);
                    else selectResource(item.id);
                  }}
                >
                  <Space size={6}>
                    <span className="side-list__item-name">{item.name}</span>
                    {item.sub && (
                      <span className="side-list__item-sub">{item.sub}</span>
                    )}
                  </Space>
                </li>
              );
            })}
            {items.length > MAX_LIST_ITEMS && (
              <li className="side-list__more">
                … и ещё {items.length - MAX_LIST_ITEMS}
              </li>
            )}
          </ul>
        )}
      </div>
    </aside>
  );
}
