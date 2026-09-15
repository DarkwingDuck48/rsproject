import {
  Button,
  Card,
  Empty,
  Space,
  Spin,
  Table,
  Tag,
  Typography,
  message,
} from "antd";
import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  TeamOutlined,
} from "@ant-design/icons";
import { useEffect, useState } from "react";
import {
  deleteTask,
  getTaskTree,
  recalculateCriticalPath,
} from "../../lib/api";
import { isNoProjectError } from "../../lib/errors";
import { formatDate } from "../../lib/format";
import { useHotkey } from "../../hooks/useHotkey";
import { TASK_STATUS_LABELS } from "../../lib/options";
import { useSelection } from "../../context/SelectionContext";
import AssignResourceDialog from "../dialogs/AssignResourceDialog";
import NewTaskDialog from "../dialogs/NewTaskDialog";
import TaskDetailsDialog from "../dialogs/TaskDetailsDialog";

/**
 * Преобразует дерево задач (TaskTreeNode[]) в записи таблицы:
 * каждый узел — TaskInfo + вложенный `children` (для tree-режима Table).
 * @param {TaskTreeNode[]} nodes
 * @returns {Array<Object>}
 */
function toRows(nodes) {
  return nodes.map((node) => ({
    ...node.task,
    children: toRows(node.children),
  }));
}

/**
 * Ищет запись таблицы по ID, включая вложенные `children`.
 * Передаётся в Table через `children`, поэтому поиск по верхнему уровню
 * не находил бы дочерние задачи (баг, исправлен в 5.21).
 * @param {Array<Object>} rows
 * @param {string} id
 * @returns {?Object}
 */
function findRow(rows, id) {
  for (const row of rows) {
    if (row.id === id) return row;
    if (Array.isArray(row.children)) {
      const found = findRow(row.children, id);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Вкладка «Задачи»: дерево задач с иерархией и действиями
 * Add / Edit / Delete / Assign Resource.
 *
 * @param {Object} props
 * @param {number} props.dataVersion - Счётчик изменений данных приложения (для перезагрузки).
 * @param {Function} props.onDataChange - Уведомить приложение об изменении данных.
 */
export default function TasksView({ dataVersion, onDataChange }) {
  const { selectedTaskId, selectTask } = useSelection();
  const [rows, setRows] = useState([]);
  const [tree, setTree] = useState([]);
  const [loading, setLoading] = useState(false);
  // Управление диалогами
  const [newOpen, setNewOpen] = useState(false);
  const [detailsTaskId, setDetailsTaskId] = useState(null);
  const [assignTaskId, setAssignTaskId] = useState(null);

  async function load() {
    setLoading(true);
    try {
      const taskTree = await getTaskTree();
      setTree(taskTree);
      setRows(toRows(taskTree));
    } catch (err) {
      setTree([]);
      setRows([]);
      if (!isNoProjectError(err)) {
        message.error(`Не удалось загрузить задачи: ${err}`);
      }
    } finally {
      setLoading(false);
    }
  }

  // Загрузка при монтировании вкладки и после изменения данных
  useEffect(() => {
    void load();
  }, [dataVersion]);

  // 5.21: если выбранной задачи больше нет в данных — сбрасываем выделение
  // (например, задача удалена из другой панели или проект закрыт).
  useEffect(() => {
    if (selectedTaskId && !findRow(rows, selectedTaskId)) {
      selectTask(null);
    }
  }, [rows, selectedTaskId, selectTask]);

  const selectedTask = findRow(rows, selectedTaskId);
  // Деривируем объекты из свежих данных: после `load()` диалоги
  // получают обновлённый список зависимостей/назначений.
  const detailsTask = findRow(rows, detailsTaskId);
  const assignTask = findRow(rows, assignTaskId);

  // Delete удаляет выбранную задачу (задача 5.17). Пока открыт любой диалог —
  // не удаляем ничего «под модалкой».
  const dialogOpen = newOpen || detailsTaskId !== null || assignTaskId !== null;
  useHotkey(
    "Delete",
    () => {
      if (!dialogOpen) void handleDelete();
    },
    { ignoreTyping: true },
  );

  async function handleDelete() {
    if (!selectedTask) return;
    try {
      await deleteTask(selectedTask.id);
      message.success("Задача удалена");
      selectTask(null);
      // Глобальное изменение — пусть статус-бар и другие панели перечитают (5.18).
      onDataChange();
    } catch (err) {
      message.error(`Не удалось удалить задачу: ${err}`);
    }
  }

  async function handleRecalc() {
    try {
      const path = await recalculateCriticalPath();
      message.success(`Критический путь: ${path.length} задач`);
      // Сообщаем приложению: критический путь пересчитан —
      // статус-бар и другие панели перечитают данные.
      onDataChange?.();
    } catch (err) {
      message.error(`Не удалось рассчитать критический путь: ${err}`);
    }
  }

  const columns = [
    {
      title: "Название",
      dataIndex: "name",
      render: (value, record) => (
        <Space size={4}>
          {record.is_summary && <Tag>Сводная</Tag>}
          <span>{value}</span>
        </Space>
      ),
    },
    {
      title: "Начало",
      dataIndex: "date_start",
      render: (value) => formatDate(value),
    },
    {
      title: "Окончание",
      dataIndex: "date_end",
      render: (value) => formatDate(value),
    },
    {
      title: "Длит., дн.",
      dataIndex: "duration_days",
      align: "right",
    },
    {
      title: "Статус",
      dataIndex: "status",
      render: (value) => TASK_STATUS_LABELS[value] ?? value,
    },
    {
      title: "Стоимость",
      dataIndex: "cost",
      align: "right",
      render: (value) => (value ?? 0).toFixed(2),
    },
  ];

  return (
    <section className="view">
      <Typography.Title level={3}>Задачи</Typography.Title>

      <Space className="views-toolbar">
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setNewOpen(true)}
        >
          Добавить
        </Button>
        <Button
          icon={<EditOutlined />}
          disabled={!selectedTask}
          onClick={() => setDetailsTaskId(selectedTask.id)}
        >
          Редактировать
        </Button>
        <Button
          danger
          icon={<DeleteOutlined />}
          disabled={!selectedTask}
          onClick={handleDelete}
        >
          Удалить
        </Button>
        <Button
          icon={<TeamOutlined />}
          disabled={!selectedTask}
          onClick={() => setAssignTaskId(selectedTask.id)}
        >
          Назначить ресурс
        </Button>
        <Button type="dashed" onClick={handleRecalc}>
          Рассчитать критический путь
        </Button>
      </Space>

      {loading ? (
        <div className="views-loading">
          <Spin />
        </div>
      ) : rows.length === 0 ? (
        <Card>
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="Задач нет. Создайте первую задачу."
          />
        </Card>
      ) : (
        <Table
          dataSource={rows}
          rowKey="id"
          columns={columns}
          expandable={{
            childrenColumnName: "children",
            defaultExpandAllRows: true,
          }}
          rowSelection={{
            type: "radio",
            selectedRowKeys: selectedTaskId ? [selectedTaskId] : [],
            onChange: (keys) => selectTask(keys.length ? keys[0] : null),
          }}
          pagination={false}
          // Таблица не сжимает колонки при узком окне: сохраняет естественную
          // ширину и получает горизонтальный скролл (ресайзинг, 5.24).
          scroll={{ x: "max-content" }}
        />
      )}

      {/* Создание/редактирование/назначение меняют данные глобально —
          инкремент dataVersion перезагрузит вьюху и статус-бар (5.18). */}
      <NewTaskDialog
        open={newOpen}
        onClose={() => setNewOpen(false)}
        onCreated={onDataChange}
      />
      <TaskDetailsDialog
        open={detailsTask != null}
        task={detailsTask}
        taskTree={tree}
        onClose={() => setDetailsTaskId(null)}
        onSaved={onDataChange}
      />
      <AssignResourceDialog
        open={assignTask != null}
        task={assignTask}
        onClose={() => setAssignTaskId(null)}
        onSaved={onDataChange}
      />
    </section>
  );
}
